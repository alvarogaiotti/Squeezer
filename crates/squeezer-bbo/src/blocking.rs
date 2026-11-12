// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Blocking BBO client (`blocking` feature).
//!
//! Shares the endpoint constants, error type, and hand-link parser with the
//! async client; only the transport differs (a `reqwest::blocking::Client`
//! with a cookie jar). See `AsyncBBOClient` (feature `async`) for the twin.

use crate::helpers::{
    BBOClient, BBOHANDS, BBOLAND, BBOLOGIN, BboError, hand_record_links, logged_in_from_body,
};

#[cfg(not(test))]
use log::{debug, info, warn};
use reqwest::blocking::Client;
#[cfg(test)]
use std::{println as info, println as warn, println as debug}; // Workaround to use println! for logs.
use time::{Duration, OffsetDateTime};
use url::Url;

/// Synchronous BBO client, backed by `reqwest::blocking`.
pub struct BlockingBBOClient {
    client: Client,
    username: String,
    password: String,
    hands_links: Vec<Url>,
}

impl BlockingBBOClient {
    /// Builds a client with its own cookie jar, so `login` persists the
    /// session for subsequent requests.
    ///
    /// # Panics
    ///
    /// Panics if the TLS backend cannot be initialized. There is no
    /// meaningful fallback: without TLS the client cannot reach BBO at all,
    /// so failing at construction is preferable to surfacing the failure on
    /// every request.
    #[inline]
    #[must_use]
    pub fn new(username: String, password: String) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("TLS backend initializes");
        Self {
            client,
            username,
            password,
            hands_links: Vec::new(),
        }
    }

    /// The listing URL for an interval, used both to fetch and to resolve the
    /// relative links in the response.
    fn hands_url(&self, start_time: OffsetDateTime, end_time: OffsetDateTime) -> Url {
        let mut url = Url::parse(BBOHANDS).expect("constant listing URL parses");
        url.query_pairs_mut()
            .append_pair("username", &self.username)
            .append_pair("start_time", &start_time.unix_timestamp().to_string())
            .append_pair("end_time", &end_time.unix_timestamp().to_string());
        url
    }

    fn get_hands_in_interval(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<String, BboError> {
        let url = self.hands_url(start_time, end_time);
        info!("query: {url}");
        let text = self.client.get(url).send()?.text()?;
        info!("{text}");
        Ok(text)
    }
}

impl BBOClient for BlockingBBOClient {
    fn is_logged(&self) -> Result<bool, BboError> {
        info!("checking if logged in");
        let logged_in = logged_in_from_body(&self.client.get(BBOLOGIN).send()?.text()?);
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

    fn login(&self) -> Result<(), BboError> {
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
        info!("sending login request for user {}", self.username);

        let response = self.client.post(BBOLOGIN).form(&params).send()?;
        info!("response status code: {}", response.status());
        if response.text()?.contains("password incorrect") {
            warn!("incorrect username or password");
            return Err(BboError::Login);
        }
        // Land on the hands page so the session is established before the
        // first listing request.
        self.client.get(BBOLAND).send()?;
        Ok(())
    }

    fn get_in_interval(
        &mut self,
        mut start_time: OffsetDateTime,
        mut end_time: OffsetDateTime,
    ) -> Result<(), BboError> {
        if start_time < end_time {
            debug!("swapping start_time and end_time (arguments were reversed)");
            std::mem::swap(&mut start_time, &mut end_time);
        }
        info!("trying to get hands links");
        // BBO caps a listing at 28 days, so walk backwards in 28-day pages.
        let mut links = Vec::new();
        while (start_time - end_time).whole_days() > 28 {
            let next_start = start_time - Duration::days(28);
            let text = self.get_hands_in_interval(start_time, next_start)?;
            links.extend(hand_record_links(
                &self.hands_url(start_time, next_start),
                &text,
            ));
            start_time = next_start;
        }
        let text = self.get_hands_in_interval(start_time, end_time)?;
        links.extend(hand_record_links(
            &self.hands_url(start_time, end_time),
            &text,
        ));
        self.hands_links = links;
        Ok(())
    }
}

impl BlockingBBOClient {
    /// Downloads every collected hand, returning the first failure if any.
    ///
    /// # Errors
    /// Errors when the site is not reachable or a hand cannot be fetched.
    pub fn download(&self) -> Result<(), BboError> {
        let mut first_error = None;
        for hand in &self.hands_links {
            info!("downloading from {hand}");
            match self.client.get(hand.clone()).send() {
                Err(e) => {
                    warn!("unable to download from {hand}.\nSee: {e}");
                    first_error.get_or_insert(BboError::Download(e));
                    continue;
                }
                Ok(response) => {
                    // TODO: persist the returned LIN instead of discarding it.
                    let _lin = response.text()?;
                }
            }
            // BBO is rate-sensitive; be polite between hand downloads.
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod test {
    // This test logs into BBO with hardcoded credentials and downloads real
    // hands. It is kept compiled so it does not rot, but it must be run
    // explicitly (`--ignored`) and never as part of a normal suite.
    #[test]
    #[ignore = "live network test against BBO with hardcoded credentials"]
    fn test_working_connection() {
        use super::*;
        use time::macros::datetime;

        let mut client =
            BlockingBBOClient::new(String::from("thevava"), String::from("ekdallebol"));
        client.login().unwrap();
        client
            .get_in_interval(
                datetime!(2022-04-01 0:00 UTC),
                datetime!(2022-03-03 0:00 UTC),
            )
            .unwrap();
        client.download().unwrap();
    }
}
