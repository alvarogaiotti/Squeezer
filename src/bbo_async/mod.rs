// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Async BBO client (`bbo_async` feature, the default).
//!
//! Mirrors [`crate::bbo`] but is driven by `reqwest`'s async client. The
//! endpoint constants, error type, and hand-link parser live in
//! [`crate::bbohelpers`] and are shared with the blocking client.

use crate::bbohelpers::{
    BBOHANDS, BBOLAND, BBOLOGIN, BboError, hand_record_links, logged_in_from_body,
};
use log::{debug, info, warn};
use reqwest::Client;
use time::{Duration, OffsetDateTime};
use url::Url;

/// Asynchronous BBO client, backed by `reqwest`.
pub struct AsyncBBOClient {
    client: Client,
    username: String,
    password: String,
    hands_links: Vec<Url>,
}

impl AsyncBBOClient {
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

    async fn get_hands_in_interval(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<String, BboError> {
        let url = self.hands_url(start_time, end_time);
        info!("query: {url}");
        Ok(self.client.get(url).send().await?.text().await?)
    }
}

impl AsyncBBOClient {
    /// Check login status.
    ///
    /// # Errors
    /// Errors when the site is not reachable.
    pub async fn is_logged(&self) -> Result<bool, BboError> {
        info!("checking if logged in");
        let logged_in = logged_in_from_body(&self.client.get(BBOLOGIN).send().await?.text().await?);
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

    /// Login procedure.
    ///
    /// # Errors
    /// Errors when the site is not reachable or credentials are rejected.
    pub async fn login(&self) -> Result<(), BboError> {
        info!("starting login procedure");
        if self.is_logged().await? {
            return Ok(());
        }
        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
            ("submit", "Login"),
            ("keep", "on"),
        ];
        info!("sending login request for user {}", self.username);

        let response = self.client.post(BBOLOGIN).form(&params).send().await?;
        info!("response status code: {}", response.status());
        if response.text().await?.contains("password incorrect") {
            warn!("incorrect username or password");
            return Err(BboError::Login);
        }
        // Land on the hands page so the session is established before the
        // first listing request.
        self.client.get(BBOLAND).send().await?;
        Ok(())
    }

    /// Collects hand links for the interval, in 28-day pages.
    ///
    /// # Errors
    /// Errors when the site is not reachable.
    pub async fn get_in_interval(
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
            let text = self.get_hands_in_interval(start_time, next_start).await?;
            links.extend(hand_record_links(
                &self.hands_url(start_time, next_start),
                &text,
            ));
            start_time = next_start;
        }
        let text = self.get_hands_in_interval(start_time, end_time).await?;
        links.extend(hand_record_links(
            &self.hands_url(start_time, end_time),
            &text,
        ));
        self.hands_links = links;
        Ok(())
    }

    /// Downloads every collected hand, returning the first failure if any.
    ///
    /// # Errors
    /// Errors when the site is not reachable or a hand cannot be fetched.
    pub async fn download(&self) -> Result<(), BboError> {
        let mut first_error = None;
        for hand in &self.hands_links {
            info!("downloading from {hand}");
            match self.client.get(hand.clone()).send().await {
                Err(e) => {
                    warn!("unable to download from {hand}.\nSee: {e}");
                    first_error.get_or_insert(BboError::Download(e));
                }
                Ok(response) => {
                    // TODO: persist the returned LIN instead of discarding it.
                    let _lin = response.text().await?;
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(()),
        }
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
    fn link_extraction_collects_hand_record_urls() {
        let page = r#"<a href="fetchlin.php?u=user&h=abc123">hand</a> noise <a href="fetchlin.php?u=user&h=def456">hand</a>"#;
        let base = Url::parse(BBOHANDS).expect("constant listing URL parses");
        let links = hand_record_links(&base, page);
        assert_eq!(
            links,
            vec![
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?u=user&h=abc123")
                    .expect("expected URL parses"),
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?u=user&h=def456")
                    .expect("expected URL parses"),
            ]
        );
    }

    #[test]
    fn link_extraction_ignores_pages_without_links() {
        let base = Url::parse(BBOHANDS).expect("constant listing URL parses");
        assert!(hand_record_links(&base, "").is_empty());
        assert!(hand_record_links(&base, "<html>no hands here</html>").is_empty());
    }
}
