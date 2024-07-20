// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use crate::prelude::{
    BBOClient, BboError, BboErrorKind, ClientError, LinkExtractor, NetworkError, BBOBASE, BBOHANDS,
    BBOLOGIN,
};
use std::cell::OnceLock;

#[cfg(not(test))]
use log::{debug, info, warn};
use regex::Regex;
#[cfg(test)]
use std::{println as info, println as warn, println as debug}; // Workaround to use println! for logs.
use time::{Duration, OffsetDateTime};
use ureq::Agent;

impl From<BboError<ureq::Error>> for ClientError<ureq::Error> {
    fn from(value: BboError<ureq::Error>) -> Self {
        Self::ConnectionError { source: value }
    }
}

impl<E: NetworkError> From<std::io::Error> for ClientError<E> {
    fn from(value: std::io::Error) -> Self {
        Self::IoError { source: value }
    }
}

pub struct BlockingBBOClient {
    client: Agent,
    username: String,
    password: String,
    hands_links: Vec<String>,
}

impl NetworkError for ureq::Error {
    fn extract_url(&self) -> &str {
        match self {
            ureq::Error::Status(_, r) => r.get_url(),
            ureq::Error::Transport(t) => {
                if let Some(url) = t.url() {
                    url.as_str()
                } else {
                    "{no_url}"
                }
            }
        }
    }
}

macro_rules! extract_url_ureq {
    ($e:expr) => {
        match $e {
            ureq::Error::Transport(t) => {
                if let Some(url) = t.url() {
                    url.as_str().to_owned()
                } else {
                    BBOBASE.to_owned()
                }
            }
            ureq::Error::Status(_, response) => response.get_url().to_owned(),
        }
    };
}

impl std::fmt::Display for BboError<ureq::Error> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unable to connect")
    }
}
impl std::error::Error for BboError<ureq::Error> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.kind)
    }
}

impl From<BboErrorKind<ureq::Error>> for BboError<ureq::Error> {
    fn from(value: BboErrorKind<ureq::Error>) -> Self {
        Self { kind: value }
    }
}

impl BlockingBBOClient {
    pub fn new(username: String, password: String) -> Self {
        let client = Agent::new();
        Self {
            client,
            username,
            password,
            hands_links: Vec::new(),
        }
    }
    fn get_hands_in_interval(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<String, ClientError<ureq::Error>> {
        let request = self.client.get(BBOHANDS).query_pairs([
            ("username", self.username.as_str()),
            ("start_time", &start_time.unix_timestamp().to_string()),
            ("end_time", &end_time.unix_timestamp().to_string()),
        ]);
        info!("query: {}", request.url());
        let text = request
            .call()
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))),
            })?
            .into_string()
            .map_err(|e| ClientError::IoError { source: e })?;
        info!("{}", text);
        Ok(text)
    }
}
impl LinkExtractor for BlockingBBOClient {
    fn get_links(&self, webpage: &str, vec: &mut Vec<String>) {
        let REGEX: OnceLock<Regex> = OnceLock::new();
        let regex =
            REGEX.get_or_init(|| Regex::new(r"\x22(?P<lin>fetchlin.php\?[\w=&]+)\x22").unwrap());
        vec.extend(
            regex
                .captures_iter(&webpage)
                .map(|matches| matches.name("lin").unwrap().as_str().to_owned()),
        );
    }
}
impl BBOClient<ureq::Error> for BlockingBBOClient {
    /// Check login status
    fn is_logged(&self) -> Result<bool, ClientError<ureq::Error>> {
        info!("checking if logged in");
        if self
            .client
            .get(BBOLOGIN)
            .call()
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))),
            })?
            .into_string()
            .map_err(|e| ClientError::IoError { source: e })?
            .contains("Please login")
        {
            info!("not logged in");
            Ok(false)
        } else {
            info!("logged in");
            Ok(true)
        }
    }

    /// Login procedure
    fn login(&self) -> Result<(), ClientError<ureq::Error>> {
        info!("starting login procedure");
        if self.is_logged()? {
            return Ok(());
        }
        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
            ("submit", "Login"),
            ("keep", "on"),
        ];

        info!(
            "sending login request with username: {} and password:{}",
            self.username.as_str(),
            self.password.as_str()
        );

        let response = self
            .client
            .post(BBOLOGIN)
            .send_form(&params)
            .map_err(|e| BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))))?;
        info!("response status code: {}", response.status());
        let text = response
            .into_string()
            .map_err(|e| ClientError::IoError { source: e })?;
        if text.contains("password incorrect") {
            warn!("incorrect username or password");
            return Err(ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::LoginError),
            });
        }
        let _ = self
            .client
            .get("http://www.bridgebase.com/myhands/hands.php?offset=0")
            .call()
            .map_err(|e| ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::UnknownConnectionError(Box::new(e))),
            })?;
        Ok(())
    }

    fn get_in_interval(
        &mut self,
        mut start_time: OffsetDateTime,
        mut end_time: OffsetDateTime,
    ) -> Result<(), ClientError<ureq::Error>> {
        if start_time < end_time {
            debug!(
                "swapped times as start_time is less than end_time:\n\tstart_time: {}\n\t end_time: {}",
                &start_time, &end_time
            );
            std::mem::swap(&mut start_time, &mut end_time)
        }
        info!("trying to get hands links");
        let mut vec: Vec<String> = Vec::new();

        while (start_time - end_time).whole_days() > 28 {
            info!("Time delta: {}", start_time - end_time);
            let next_start = start_time - Duration::days(28);

            let text = self.get_hands_in_interval(start_time, next_start)?;
            start_time = next_start;
            self.get_links(&text, &mut vec);
        }
        info!("Time delta after reduction: {}", start_time - end_time);
        let text = self.get_hands_in_interval(start_time, end_time)?;
        self.get_links(&text, &mut vec);
        self.hands_links = vec;

        Ok(())
    }

    fn download(&self) -> Result<(), ClientError<ureq::Error>> {
        let mut queue = Vec::new();
        for mut hand in self.hands_links.iter().cloned() {
            hand.insert_str(0, BBOBASE);

            info!("downloading from {}", hand);
            let lin = match self.client.get(&hand).call() {
                Err(e) => {
                    warn!("unable to download from {}.\nSee: {}", &hand, e);
                    queue.push(e);
                    continue;
                }
                Ok(response) => response
                    .into_string()
                    .map_err(|e| ClientError::IoError { source: e })?,
            };
            // TODO:
            // For now puppy implementation
            todo!();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        if queue.is_empty() {
            Ok(())
        } else {
            Err(ClientError::ConnectionError {
                source: BboError::from(BboErrorKind::DownloadError(Box::new(
                    queue.into_iter().last().unwrap(),
                ))),
            })
        }
    }
}

#[cfg(test)]
#[test]
fn test_working_connection() {
    use time::macros::datetime;

    let mut client = BlockingBBOClient::new(String::from("thevava"), String::from("ekdallebol"));
    client.login().unwrap();
    client
        .get_in_interval(
            datetime!(2022-04-01 0:00 UTC),
            datetime!(2022-03-03 0:00 UTC),
        )
        .unwrap();
    client.download().unwrap();
}
