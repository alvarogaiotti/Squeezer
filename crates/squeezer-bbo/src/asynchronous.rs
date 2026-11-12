// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Async BBO client (`async` feature, the default).
//!
//! Mirrors `BlockingBBOClient` (feature `blocking`) but is driven by
//! `reqwest`'s async client. The endpoint constants, error type, and hand-link
//! parser live in the crate's private `helpers` module and are shared with the
//! blocking client.

use crate::helpers::{
    BBOHANDS, BBOLAND, BBOLOGIN, BboError, hand_record_links, interval_pages, logged_in_from_body,
};
use log::{info, warn};
use reqwest::Client;
use time::OffsetDateTime;
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
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<(), BboError> {
        info!("trying to get hands links");
        let mut links = Vec::new();
        for (page_start, page_end) in interval_pages(start_time, end_time) {
            let text = self.get_hands_in_interval(page_start, page_end).await?;
            links.extend(hand_record_links(
                &self.hands_url(page_start, page_end),
                &text,
            ));
        }
        self.hands_links = links;
        Ok(())
    }

    /// Downloads every collected hand record, returning the LIN document of
    /// each in collection order.
    ///
    /// BBO answers a hand request with one complete LIN document, so the batch
    /// is a `Vec` of documents rather than a single multi-board file. Use
    /// `squeezer-io`'s `parse_lin_documents` to turn the batch into boards.
    ///
    /// # Errors
    /// Errors when the site is not reachable or a hand cannot be fetched. LIN
    /// documents already fetched before the failure are not returned.
    pub async fn download(&self) -> Result<Vec<String>, BboError> {
        let mut documents = Vec::with_capacity(self.hands_links.len());
        let mut first_error = None;
        for hand in &self.hands_links {
            info!("downloading from {hand}");
            match self.client.get(hand.clone()).send().await {
                Err(e) => {
                    warn!("unable to download from {hand}.\nSee: {e}");
                    first_error.get_or_insert(BboError::Download(e));
                }
                Ok(response) => {
                    documents.push(response.text().await?);
                }
            }
        }
        match first_error {
            Some(error) => Err(error),
            None => Ok(documents),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn download_returns_an_empty_batch_when_no_links_were_collected() {
        let client = AsyncBBOClient::new(String::from("user"), String::from("password"));
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .expect("current-thread runtime builds");

        let lin = runtime
            .block_on(client.download())
            .expect("no collected links means no request");

        assert!(lin.is_empty());
    }
}
