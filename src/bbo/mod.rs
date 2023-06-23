use lazy_static::lazy_static;
#[cfg(not(test))]
use log::{debug, info, warn};
use regex::Regex;
#[cfg(test)]
use std::{println as info, println as warn, println as debug}; // Workaround to use println! for logs.
use time::{Duration, OffsetDateTime};
use ureq::Agent;
#[macro_export]
macro_rules! get_bboerrorkind_error {
    ($e: expr) => {
        match $e {
            BboErrorKind::UnknownConnectionError(error) => Some(error),
            BboErrorKind::LoginError => None,
            BboErrorKind::DownloadError(error) => Some(error),
            BboErrorKind::HandsRequestError(error) => Some(error),
        }
    };
}

pub const BBOLOGIN: &str =
    "https://www.bridgebase.com/myhands/myhands_login.php?t=%2Fmyhands%2Findex.php%3F";
pub const BBOLAND: &str = "https://www.bridgebase.com/myhands/index.php?offset=0";
pub const BBOHANDS: &str = "https://www.bridgebase.com/myhands/hands.php?";
pub const BBOBASE: &str = "https://www.bridgebase.com/myhands/";

#[derive(Debug)]
pub enum ClientError<E: NetworkError> {
    IoError { source: std::io::Error },
    ConnectionError { source: BboError<E> },
}

impl From<BboError<ureq::Error>> for ClientError<ureq::Error> {
    fn from(value: BboError<ureq::Error>) -> Self {
        Self::ConnectionError { source: value }
    }
}

pub trait BBOClient<E: NetworkError> {
    fn is_logged(&self) -> Result<bool, ClientError<E>>;
    fn login(&self) -> Result<(), ClientError<E>>;
    fn download(&self) -> Result<(), ClientError<E>>;
    fn get_in_interval(
        &mut self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<(), ClientError<E>>;
}

pub(crate) trait LinkExtractor {
    fn get_links(&self, text: &str, matches: &mut Vec<String>);
}

pub struct BlockingBBOClient {
    client: Agent,
    username: String,
    password: String,
    hands_links: Vec<String>,
}

#[derive(Debug)]
pub struct BboError<E: NetworkError> {
    pub kind: BboErrorKind<E>,
}

pub trait NetworkError: std::error::Error {
    fn extract_url(&self) -> &str;
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

#[derive(Debug)]
#[non_exhaustive]
pub enum BboErrorKind<E: NetworkError> {
    UnknownConnectionError(Box<E>),
    LoginError,
    DownloadError(Box<E>),
    HandsRequestError(Box<E>),
}

impl<E: NetworkError + 'static> std::error::Error for BboErrorKind<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BboErrorKind::UnknownConnectionError(error) => Some(error),
            BboErrorKind::DownloadError(error) => Some(error),
            BboErrorKind::HandsRequestError(error) => Some(error),
            BboErrorKind::LoginError => None,
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

impl<E: NetworkError> std::fmt::Display for BboErrorKind<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BboErrorKind::LoginError => write!(f, "incorrect username or password"),
            BboErrorKind::DownloadError(e) => {
                write!(f, "unable to download hand from {}", e.extract_url())
            }
            BboErrorKind::UnknownConnectionError(e) => {
                write!(
                    f,
                    "unable to connect to {}, check internet connection",
                    e.extract_url()
                )
            }
            BboErrorKind::HandsRequestError(e) => {
                write!(
                    f,
                    "incorrect time interval request parameters, tried to get from: {}",
                    e.extract_url()
                )
            }
        }
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
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\x22(?P<lin>fetchlin.php\?[\w=&]+)\x22").unwrap();
        }
        vec.extend(
            RE.captures_iter(&webpage)
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
            // For now puppy implementation
            println!("{}", lin);
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
