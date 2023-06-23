use crate::{
    bbo::{
        BboError, BboErrorKind, ClientError, LinkExtractor, NetworkError, BBOBASE, BBOHANDS,
        BBOLOGIN,
    },
    get_bboerrorkind_error,
};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use regex::Regex;
use reqwest::Client;
use time::{Duration, OffsetDateTime};
type Result<T> = std::result::Result<T, ClientError<reqwest::Error>>;

pub struct AsyncBBOClient {
    client: Client,
    username: String,
    password: String,
    hands_links: Vec<String>,
}

impl NetworkError for reqwest::Error {
    fn extract_url(&self) -> &str {
        if let Some(url) = self.url() {
            url.as_str()
        } else {
            "{no_url}"
        }
    }
}

impl ClientError<reqwest::Error> {
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
        write!(f, "unable to connect to {}", url)
    }
}

impl LinkExtractor for AsyncBBOClient {
    fn get_links(&self, webpage: &str, vec: &mut Vec<String>) {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\x22(?P<lin>fetchlin.php\?[\w=&]+)\x22").unwrap();
        }
        vec.extend(
            RE.captures_iter(webpage)
                .map(|matches| matches.name("lin").unwrap().as_str().to_owned()),
        );
    }
}

impl AsyncBBOClient {
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
    pub async fn is_logged(&self) -> Result<bool> {
        info!("checking if logged in");
        if self
            .client
            .get(BBOLOGIN)
            .send()
            .await
            .map_err(|e| ClientError::unknown_error(e))?
            .text()
            .await
            .map_err(|_e| ClientError::IoError {
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "unable to parse response as a String",
                ),
            })?
            .contains("Please login")
        {
            info!("not logged in");
            Ok(false)
        } else {
            info!("logged in");
            Ok(false)
        }
    }

    /// Login procedure
    async fn login(&self) -> Result<()> {
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
                source: std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "unable to parse response as String",
                ),
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

    async fn get_in_interval(
        &mut self,
        mut start_time: OffsetDateTime,
        mut end_time: OffsetDateTime,
    ) -> Result<()> {
        if start_time < end_time {
            debug!(
                "swapped times as start_time is less than end_time:\n\tstart_time: {}\n\t end_time: {}",
                &start_time, &end_time
            );
            std::mem::swap(&mut start_time, &mut end_time)
        }
        let mut vec: Vec<String> = Vec::new();

        loop {
            if (start_time - end_time).whole_days() < 28 {
                break;
            }
            let next_start = start_time - Duration::days(28);
            let _text = self.get_hands_in_interval(start_time, next_start).await?;
            start_time = next_start;
            self.get_links(&String::new(), &mut vec);
        }
        let _text = self.get_hands_in_interval(start_time, end_time).await?;
        self.get_links(&String::new(), &mut vec);

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

    pub async fn download(&self) -> Result<()> {
        for mut hand in self.hands_links.iter().cloned() {
            hand.insert_str(0, BBOBASE);

            info!("downloading from {}", hand);
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
