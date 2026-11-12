// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Async BBO client (`async` feature, the default).
//!
//! Mirrors `BlockingBBOClient` (feature `blocking`) but is driven by
//! `reqwest`'s async client. The endpoint constants, error type, login form,
//! session/listing classifiers and hand-link parser live in the crate's private
//! `helpers` module and are shared with the blocking client.

use crate::helpers::{
    BBOLAND, BBOLOGIN, BboError, DownloadFailure, DownloadPolicy, DownloadReport, DownloadedHand,
    FetchedListing, Session, Window, content_disposition_filename, dedup_hand_links,
    ensure_success, hand_filename, hands_url, is_bbolin_content_type, is_lin_document,
    is_login_body, is_retryable_status, login_form, merge_retried, parse_listing,
    retry_after_seconds, retry_delay, session_from_response, split_existing, write_hands,
};
use futures_util::StreamExt;
use log::{info, warn};
use reqwest::Client;
use std::path::Path;
use url::Url;

/// Asynchronous BBO client, backed by `reqwest`.
pub struct AsyncBBOClient {
    client: Client,
    username: String,
    password: String,
    landing_url: Url,
    login_url: Url,
    hands_links: Vec<Url>,
    hidden_tourneys: bool,
    policy: DownloadPolicy,
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
            landing_url: Url::parse(BBOLAND).expect("constant landing URL parses"),
            login_url: Url::parse(BBOLOGIN).expect("constant login URL parses"),
            hands_links: Vec::new(),
            hidden_tourneys: false,
            policy: DownloadPolicy::default(),
        }
    }

    /// Replaces the download pacing and retry policy.
    #[must_use]
    pub fn with_policy(mut self, policy: DownloadPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// The BBO user this client logs in as.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// The hand links collected by the last [`AsyncBBOClient::collect_hand_links`].
    #[must_use]
    pub fn hand_links(&self) -> &[Url] {
        &self.hands_links
    }

    /// Whether the last listing reported that BBO hides some tournament hands.
    #[must_use]
    pub fn hidden_tourneys(&self) -> bool {
        self.hidden_tourneys
    }

    /// Fetches one listing page and returns its request URL, final URL and body.
    async fn fetch_listing(
        &self,
        username: &str,
        window: Window,
    ) -> Result<FetchedListing, BboError> {
        let request_url = hands_url(username, window);
        info!("query: {request_url}");
        let response = self.client.get(request_url.clone()).send().await?;
        ensure_success(&request_url, response.status())?;
        let final_url = response.url().clone();
        let body = response.text().await?;
        Ok(FetchedListing {
            request_url,
            final_url,
            body,
        })
    }

    /// Check login status through the protected index page.
    ///
    /// # Errors
    /// Errors when the site is not reachable or its response is unrecognised.
    pub async fn is_logged(&self) -> Result<bool, BboError> {
        info!("checking if logged in");
        let request_url = self.landing_url.clone();
        let response = self.client.get(request_url.clone()).send().await?;
        ensure_success(&request_url, response.status())?;
        let final_url = response.url().clone();
        let body = response.text().await?;
        match session_from_response(&final_url, &body) {
            Session::LoggedIn => {
                info!("logged in");
                Ok(true)
            }
            Session::LoggedOut => {
                info!("not logged in");
                Ok(false)
            }
            Session::Unknown => Err(BboError::UnexpectedResponse {
                url: final_url,
                reason: "could not determine the BBO session state",
            }),
        }
    }

    /// Login procedure.
    ///
    /// The form matches what the website submits, including the hidden `t` and
    /// `count` fields. The success signal is the final URL and the logged-in
    /// page markers, not a substring of the redirected body.
    ///
    /// # Errors
    /// Errors when the site is not reachable or credentials are rejected.
    pub async fn login(&self) -> Result<(), BboError> {
        info!("starting login procedure");
        if self.is_logged().await? {
            return Ok(());
        }
        let request_url = self.login_url.clone();
        let form = login_form(&self.username, &self.password);
        info!("sending login request for user {}", self.username);

        let response = self
            .client
            .post(request_url.clone())
            .form(&form)
            .send()
            .await?;
        ensure_success(&request_url, response.status())?;
        info!("response status code: {}", response.status());
        let final_url = response.url().clone();
        let body = response.text().await?;
        match session_from_response(&final_url, &body) {
            Session::LoggedIn => Ok(()),
            Session::LoggedOut => {
                warn!("BBO rejected the login");
                Err(BboError::Login)
            }
            Session::Unknown => Err(BboError::UnexpectedResponse {
                url: final_url,
                reason: "no login outcome marker",
            }),
        }
    }

    /// Collects hand links for `username` and the window, in 31-day pages.
    ///
    /// # Errors
    /// Errors when the site is not reachable, the session expired and cannot
    /// be renewed, or BBO rejects the window.
    pub async fn collect_hand_links(
        &mut self,
        username: &str,
        window: Window,
    ) -> Result<(), BboError> {
        info!("trying to get hands links");
        let mut links = Vec::new();
        let mut hidden = false;
        let mut relogged = false;
        for page in window.pages() {
            let listing = loop {
                let fetched = self.fetch_listing(username, page).await?;
                match parse_listing(&fetched.request_url, &fetched.final_url, &fetched.body) {
                    Ok(listing) => break listing,
                    Err(BboError::NotLoggedIn) if !relogged => {
                        warn!("session expired while listing hands; logging in again");
                        self.login().await?;
                        relogged = true;
                    }
                    Err(error) => return Err(error),
                }
            };
            hidden |= listing.hidden_tourneys;
            links.extend(listing.links);
        }
        dedup_hand_links(&mut links);
        self.hands_links = links;
        self.hidden_tourneys = hidden;
        if hidden {
            warn!("BBO hides some tournament hands for security reasons");
        }
        Ok(())
    }

    /// Collects hand links for the logged-in user.
    ///
    /// # Errors
    /// See [`AsyncBBOClient::collect_hand_links`].
    pub async fn own_hands(&mut self, window: Window) -> Result<(), BboError> {
        let username = self.username.clone();
        self.collect_hand_links(&username, window).await
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
    /// documents already fetched before the failure are not returned; use
    /// [`AsyncBBOClient::download_report`] to keep the successful ones.
    pub async fn download(&self) -> Result<Vec<String>, BboError> {
        let DownloadReport {
            hands, failures, ..
        } = self.download_report().await?;
        if let Some(failure) = failures.into_iter().next() {
            return Err(failure.error);
        }
        Ok(hands.into_iter().map(|hand| hand.lin).collect())
    }

    /// Downloads every collected hand and keeps per-hand failures.
    ///
    /// # Errors
    /// Errors when a session refresh is required and fails. Per-hand failures
    /// are returned in the report.
    pub async fn download_report(&self) -> Result<DownloadReport, BboError> {
        self.fetch_links(&self.hands_links).await
    }

    /// Downloads every collected hand into `dir`, skipping files that already
    /// exist and naming each file as BBO suggests.
    ///
    /// # Errors
    /// Errors when the directory cannot be created or a session refresh fails.
    /// Per-hand fetch and write failures are returned in the report.
    pub async fn download_to_dir(&self, dir: impl AsRef<Path>) -> Result<DownloadReport, BboError> {
        let dir = dir.as_ref();
        // LIN documents are small; std::fs keeps the async feature from
        // needing tokio's fs module.
        std::fs::create_dir_all(dir).map_err(|source| BboError::Write {
            path: dir.to_path_buf(),
            source,
        })?;
        let (to_fetch, skipped) = split_existing(dir, &self.hands_links);
        let mut report = self.fetch_links(&to_fetch).await?;
        write_hands(dir, &mut report);
        report.skipped = skipped;
        Ok(report)
    }

    /// Fetches the given links with bounded concurrency, preserving order.
    async fn fetch_links(&self, links: &[Url]) -> Result<DownloadReport, BboError> {
        if links.is_empty() {
            return Ok(DownloadReport::default());
        }
        let mut outcomes = self.fetch_parallel(links).await;
        if outcomes
            .iter()
            .any(|(_, result)| matches!(result, Err(BboError::NotLoggedIn)))
        {
            warn!("session expired while downloading hands; logging in again");
            self.login().await?;
            let retry: Vec<Url> = outcomes
                .iter()
                .filter(|(_, result)| matches!(result, Err(BboError::NotLoggedIn)))
                .map(|(url, _)| url.clone())
                .collect();
            let retried = self.fetch_parallel(&retry).await;
            merge_retried(&mut outcomes, retried);
        }
        let mut report = DownloadReport::default();
        for (url, result) in outcomes {
            match result {
                Ok(hand) => report.hands.push(hand),
                Err(error) => report.failures.push(DownloadFailure { url, error }),
            }
        }
        Ok(report)
    }

    /// Fetches the links in order, with at most `policy.max_in_flight` requests
    /// live at once. `buffered` preserves the input order.
    async fn fetch_parallel(&self, links: &[Url]) -> Vec<(Url, Result<DownloadedHand, BboError>)> {
        let concurrency = self.policy.max_in_flight.clamp(1, 8);
        futures_util::stream::iter(links.iter().cloned())
            .map(|url| async move {
                let result = self.fetch_hand(&url).await;
                (url, result)
            })
            .buffered(concurrency)
            .collect()
            .await
    }

    /// Fetches one hand, retrying transient failures and validating that the
    /// response is a LIN document rather than a login page.
    async fn fetch_hand(&self, url: &Url) -> Result<DownloadedHand, BboError> {
        let mut attempt = 0u32;
        loop {
            let response = match self.client.get(url.clone()).send().await {
                Ok(response) => response,
                Err(_) if attempt < self.policy.max_retries => {
                    tokio::time::sleep(retry_delay(&self.policy, attempt)).await;
                    attempt += 1;
                    continue;
                }
                Err(error) => return Err(BboError::Download(error)),
            };
            if response.status().is_success() {
                let final_url = response.url().clone();
                if final_url.path().ends_with("myhands_login.php") {
                    return Err(BboError::NotLoggedIn);
                }
                let filename = hand_filename(url)
                    .or_else(|| content_disposition_filename(response.headers()))
                    .unwrap_or_else(|| String::from("hand.lin"));
                let is_bbolin = is_bbolin_content_type(response.headers());
                let body = response.text().await.map_err(BboError::Download)?;
                if !is_bbolin && !is_lin_document(&body) {
                    if is_login_body(&body) {
                        return Err(BboError::NotLoggedIn);
                    }
                    return Err(BboError::UnexpectedResponse {
                        url: final_url,
                        reason: "the hand response is not a LIN document",
                    });
                }
                if !self.policy.delay.is_zero() {
                    tokio::time::sleep(self.policy.delay).await;
                }
                return Ok(DownloadedHand {
                    url: url.clone(),
                    filename,
                    lin: body,
                });
            }
            if is_retryable_status(response.status()) && attempt < self.policy.max_retries {
                let delay = retry_after_seconds(response.headers()).map_or_else(
                    || retry_delay(&self.policy, attempt),
                    std::time::Duration::from_secs,
                );
                tokio::time::sleep(delay).await;
                attempt += 1;
                continue;
            }
            return Err(BboError::HttpStatus {
                url: url.clone(),
                status: response.status(),
            });
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
