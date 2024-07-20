// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

use time::OffsetDateTime;
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

#[derive(Debug)]
pub enum ClientError<E: NetworkError> {
    IoError { source: std::io::Error },
    ConnectionError { source: BboError<E> },
}
pub const BBOLOGIN: &str =
    "https://www.bridgebase.com/myhands/myhands_login.php?t=%2Fmyhands%2Findex.php%3F";
pub const BBOLAND: &str = "https://www.bridgebase.com/myhands/index.php?offset=0";
pub const BBOHANDS: &str = "https://www.bridgebase.com/myhands/hands.php?";
pub const BBOBASE: &str = "https://www.bridgebase.com/myhands/";

pub trait NetworkError: std::error::Error {
    fn extract_url(&self) -> &str;
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

#[derive(Debug)]
pub struct BboError<E: NetworkError> {
    pub kind: BboErrorKind<E>,
}
