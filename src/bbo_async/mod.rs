// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::{
    get_bboerrorkind_error,
    prelude::{
        BBOBASE, BBOHANDS, BBOLOGIN, BboError, BboErrorKind, ClientError, LinkExtractor,
        NetworkError,
    },
};
use log::{debug, error, info, warn};
use regex::Regex;
use reqwest::Client;
use std::sync::OnceLock;
use time::{Duration, OffsetDateTime};
type Result<T> = std::result::Result<T, ClientError<reqwest::Error>>;

pub struct AsyncBBOClient {
    client: Client,
    username: String,
    password: String,
    hands_links: Vec<String>,
}

impl NetworkError for reqwest::Error {}

impl ClientError<reqwest::Error> {
    #[inline]
    #[must_use]
    pub fn unknown_error(e: reqwest::Error) -> Self {
        Self::ConnectionError {
            source: BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))),
        }
    }
}

impl From<BboError<reqwest::Error>> for ClientError<reqwest::Error> {
    fn from(value: BboError<reqwest::Error>) -> Self {
        Self::ConnectionError { source: value }
    }
}
#[allow(unused_macros)]
macro_rules! extract_url_reqwest {
    ($e:expr) => {
        if let Some(url) = $e.url() {
            url.to_string()
        } else {
            BBOBASE.to_string()
        }
    };
}
impl From<BboErrorKind<reqwest::Error>> for BboError<reqwest::Error> {
    fn from(value: BboErrorKind<reqwest::Error>) -> Self {
        Self { kind: value }
    }
}

impl std::error::Error for BboError<reqwest::Error> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}
impl std::fmt::Display for BboError<reqwest::Error> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let url = if let Some(error) = get_bboerrorkind_error!(&self.kind) {
            if let Some(url) = error.url() {
                url.as_str()
            } else {
                BBOBASE
            }
        } else {
            BBOBASE
        };
        write!(f, "unable to connect to {url}")
    }
}

impl LinkExtractor for AsyncBBOClient {
    fn get_links(&self, webpage: &str, vec: &mut Vec<String>) {
        #[allow(non_snake_case)]
        let REGEX: OnceLock<Regex> = OnceLock::new();
        let regex =
            REGEX.get_or_init(|| Regex::new(r"\x22(?P<lin>fetchlin.php\?[\w=&]+)\x22").unwrap());
        vec.extend(
            regex
                .captures_iter(webpage)
                .map(|matches| matches.name("lin").unwrap().as_str().to_owned()),
        );
    }
}

/// Decides login status from a login-page body: BBO shows
/// "Please login" to anonymous visitors. Pure so the branch logic is
/// unit-testable without network access (mirrors the sync client).
fn logged_in_from_body(body: &str) -> bool {
    !body.contains("Please login")
}

impl AsyncBBOClient {
    #[inline]
    #[must_use]
    pub fn new(username: String, password: String) -> Self {
        let client = match Client::builder().cookie_store(true).build() {
            Ok(client) => client,
            Err(e) => {
                error!("unable to create cookie storing client: {e}");
                Client::default()
            }
        };
        Self {
            client,
            username,
            password,
            hands_links: Vec::new(),
        }
    }
    /// Check login status
    /// # Errors
    /// Errors when the site is not reachable
    pub async fn is_logged(&self) -> Result<bool> {
        info!("checking if logged in");
        let logged_in = logged_in_from_body(
            &self
                .client
                .get(BBOLOGIN)
                .send()
                .await
                .map_err(ClientError::unknown_error)?
                .text()
                .await
                .map_err(|_e| ClientError::IoError {
                    source: std::io::Error::other("unable to parse response as a String"),
                })?,
        );
        info!(
            "{}",
            if logged_in {
                "logged in"
            } else {
                "not logged in"
            }
        );
        Ok(logged_in)
    }

    /// Login procedure
    /// # Errors
    /// Errors when the site is not reachable or credentials are rejected
    pub async fn login(&self) -> Result<()> {
        info!("starting login procedure");
        if self.is_logged().await? {
            return Ok(());
        }
        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
        ];

        info!(
            "sending login request with username: {} and password:{}",
            self.username.as_str(),
            self.password.as_str()
        );

        let response = self
            .client
            .post(BBOLOGIN)
            .form(&params)
            .send()
            .await
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))),
            })?
            .text()
            .await
            .map_err(|_e| ClientError::IoError {
                source: std::io::Error::other("unable to parse response as String"),
            })?;
        if response.contains("password incorrect") {
            info!("incorrect username or password");
            return Err(ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::LoginError),
            });
        }
        info!("Succesfully logged in");

        Ok(())
    }

    /// Collects hand links for the subscribed interval, in 28-day pages.
    /// # Errors
    /// Errors when the site is not reachable
    pub async fn get_in_interval(
        &mut self,
        mut start_time: OffsetDateTime,
        mut end_time: OffsetDateTime,
    ) -> Result<()> {
        if start_time < end_time {
            debug!(
                "swapped times as start_time is less than end_time:\n\tstart_time: {}\n\t end_time: {}",
                &start_time, &end_time
            );
            std::mem::swap(&mut start_time, &mut end_time);
        }
        let mut vec: Vec<String> = Vec::new();

        loop {
            if (start_time - end_time).whole_days() < 28 {
                break;
            }
            let next_start = start_time - Duration::days(28);
            let text = self.get_hands_in_interval(start_time, next_start).await?;
            start_time = next_start;
            self.get_links(&text, &mut vec);
        }
        let text = self.get_hands_in_interval(start_time, end_time).await?;
        self.get_links(&text, &mut vec);

        self.hands_links = vec;
        Ok(())
    }

    async fn get_hands_in_interval(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<String> {
        let text = self
            .client
            .get(BBOHANDS)
            .query(&[("username", self.username.as_str())])
            .query(&[("start_time", start_time.unix_timestamp())])
            .query(&[("end_time", (end_time).unix_timestamp())])
            .send()
            .await
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::HandsRequestError(Box::new(e))),
            })?
            .text()
            .await
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::HandsRequestError(Box::new(e))),
            })?;
        Ok(text)
    }

    /// # Errors
    /// Errors when the site is not reachable
    pub async fn download(&self) -> Result<()> {
        for mut hand in self.hands_links.iter().cloned() {
            hand.insert_str(0, BBOBASE);

            info!("downloading from {hand}");
            let _lin = match self.client.get(&hand).send().await {
                Err(e) => {
                    warn!("unable to download from {}.\nSee: {}", &hand, e);
                    continue;
                }
                Ok(response) => {
                    response
                        .text()
                        .await
                        .map_err(|e| ClientError::ConnectionError {
                            source: BboError::from(BboErrorKind::DownloadError(Box::new(e))),
                        })?
                }
            };
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn login_status_decision_matches_sync_client() {
        // The exact branch that was buggy (both arms returned false).
        assert!(!logged_in_from_body("Please login to continue"));
        assert!(logged_in_from_body("Welcome back, your tables await"));
        assert!(logged_in_from_body(""));
    }

    #[test]
    fn link_extraction_collects_fetchlin_urls() {
        let client = AsyncBBOClient::new(String::new(), String::new());
        let page = r#"<a href="fetchlin.php?u=user&h=abc123">hand</a> noise <a href="fetchlin.php?u=user&h=def456">hand</a>"#;
        let mut links = Vec::new();
        client.get_links(page, &mut links);
        assert_eq!(
            links,
            vec![
                "fetchlin.php?u=user&h=abc123".to_owned(),
                "fetchlin.php?u=user&h=def456".to_owned(),
            ]
        );
    }

    #[test]
    fn link_extraction_ignores_pages_without_links() {
        let client = AsyncBBOClient::new(String::new(), String::new());
        let mut links = Vec::new();
        client.get_links("", &mut links);
        client.get_links("<html>no hands here</html>", &mut links);
        assert!(links.is_empty());
    }
}
