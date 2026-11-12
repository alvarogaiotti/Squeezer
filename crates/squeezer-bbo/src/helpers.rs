// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Shared BBO plumbing: endpoints, errors, the hand-record listing parser and
//! the pure parts of a download.
//!
//! Both the blocking and async clients speak `reqwest`, so the error type is
//! not generic over the transport. Everything here is transport-free so the
//! two clients cannot drift on the parts that BBO actually constrains: the
//! login form, the 31-day listing window, the session markers and the shape of
//! a `fetchlin.php` response.

use std::path::Path;

use scraper::{Html, Selector};
use time::{Duration, OffsetDateTime};
use url::Url;

/// Login endpoint; the `t=` parameter is the page to land on afterwards.
pub const BBOLOGIN: &str =
    "https://www.bridgebase.com/myhands/myhands_login.php?t=%2Fmyhands%2Findex.php%3F";
/// The page that reports whether the cookie jar holds a session.
pub const BBOLAND: &str = "https://www.bridgebase.com/myhands/index.php?offset=0";
/// The endpoint that returns a user's hands for a hand-record window.
pub const BBOHANDS: &str = "https://www.bridgebase.com/myhands/hands.php?";
/// Base used to resolve the relative `fetchlin.php` links in the listing.
pub const BBOBASE: &str = "https://www.bridgebase.com/myhands/";

/// The page the login form redirects to after a successful login.
pub(crate) const LOGIN_TARGET: &str = "/myhands/index.php?";

/// The longest window BBO returns in one listing page.
///
/// A 31-day request works; a 32-day request is answered with HTTP 200 and the
/// body `Invalid input. Please use this page.` The server does not report the
/// limit in a header, so this number is the only contract we have.
pub const MAX_LISTING_DAYS: i64 = 31;

const MAX_LISTING_SECONDS: i64 = MAX_LISTING_DAYS * 24 * 60 * 60;

/// A BBO download or login failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum BboError {
    /// The transport failed (DNS, TLS, timeout, ...).
    Connection(reqwest::Error),
    /// BBO rejected the supplied credentials.
    Login,
    /// A hand page could not be downloaded.
    Download(reqwest::Error),
    /// The cookie jar holds no session, or BBO answered with the login page.
    NotLoggedIn,
    /// BBO answered with a non-success status.
    HttpStatus {
        /// The requested URL.
        url: Url,
        /// The status BBO returned.
        status: reqwest::StatusCode,
    },
    /// The requested window is not one BBO accepts.
    InvalidWindow {
        /// Why the window is invalid.
        reason: &'static str,
    },
    /// BBO answered with a page that is neither a listing nor a login page.
    UnexpectedResponse {
        /// The final URL, after redirects.
        url: Url,
        /// What was missing.
        reason: &'static str,
    },
    /// A downloaded hand could not be written to disk.
    Write {
        /// The path that failed.
        path: std::path::PathBuf,
        /// The filesystem error.
        source: std::io::Error,
    },
}

impl std::fmt::Display for BboError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(_) => write!(f, "unable to connect, check internet connection"),
            Self::Login => write!(f, "BBO rejected the supplied username or password"),
            Self::Download(_) => write!(f, "unable to download hand"),
            Self::NotLoggedIn => write!(f, "not logged in to BBO; call login first"),
            Self::HttpStatus { url, status } => {
                write!(f, "BBO returned HTTP {status} for {url}")
            }
            Self::InvalidWindow { reason } => write!(f, "invalid hand-record window: {reason}"),
            Self::UnexpectedResponse { url, reason } => {
                write!(f, "unexpected BBO response from {url}: {reason}")
            }
            Self::Write { path, .. } => write!(f, "unable to write {}", path.display()),
        }
    }
}

impl std::error::Error for BboError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(error) | Self::Download(error) => Some(error),
            Self::Write { source, .. } => Some(source),
            Self::Login
            | Self::NotLoggedIn
            | Self::HttpStatus { .. }
            | Self::InvalidWindow { .. }
            | Self::UnexpectedResponse { .. } => None,
        }
    }
}

impl From<reqwest::Error> for BboError {
    fn from(value: reqwest::Error) -> Self {
        Self::Connection(value)
    }
}

/// An ordered listing window: `start` is strictly before `end`.
///
/// BBO's `hands.php` always expects `start_time < end_time`; the website's
/// "backward"/"forward" choice is how the *window is built* from one anchor
/// date, not a different parameter order. [`Window::ending_at`] and
/// [`Window::starting_at`] model those two choices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Window {
    start: OffsetDateTime,
    end: OffsetDateTime,
}

impl Window {
    /// Builds a window from two instants.
    ///
    /// # Errors
    /// Errors when `start` is not strictly before `end`.
    pub fn new(start: OffsetDateTime, end: OffsetDateTime) -> Result<Self, BboError> {
        if start >= end {
            return Err(BboError::InvalidWindow {
                reason: "start must be strictly before end",
            });
        }
        Ok(Self { start, end })
    }

    /// Builds a window that ends at `anchor` (the website's "backward" mode).
    ///
    /// # Errors
    /// Errors when `span` is not positive or the result leaves the `time`
    /// crate's supported range.
    pub fn ending_at(anchor: OffsetDateTime, span: Duration) -> Result<Self, BboError> {
        if span <= Duration::ZERO {
            return Err(BboError::InvalidWindow {
                reason: "span must be positive",
            });
        }
        let start = anchor.checked_sub(span).ok_or(BboError::InvalidWindow {
            reason: "window start is outside the supported date range",
        })?;
        Self::new(start, anchor)
    }

    /// Builds a window that starts at `anchor` (the website's "forward" mode).
    ///
    /// # Errors
    /// Errors when `span` is not positive or the result leaves the `time`
    /// crate's supported range.
    pub fn starting_at(anchor: OffsetDateTime, span: Duration) -> Result<Self, BboError> {
        if span <= Duration::ZERO {
            return Err(BboError::InvalidWindow {
                reason: "span must be positive",
            });
        }
        let end = anchor.checked_add(span).ok_or(BboError::InvalidWindow {
            reason: "window end is outside the supported date range",
        })?;
        Self::new(anchor, end)
    }

    /// The inclusive lower bound.
    #[must_use]
    pub const fn start(self) -> OffsetDateTime {
        self.start
    }

    /// The exclusive upper bound.
    #[must_use]
    pub const fn end(self) -> OffsetDateTime {
        self.end
    }

    /// The length of the window.
    #[must_use]
    pub fn duration(self) -> Duration {
        self.end - self.start
    }

    /// Splits the window into pages BBO accepts, each at most 31 days.
    ///
    /// The pages have equal length as far as the seconds allow, so a window
    /// that is a second longer than 31 days becomes two ~15.5-day pages rather
    /// than a 31-day page and a one-second page.
    #[must_use]
    pub fn pages(self) -> Vec<Self> {
        let total_seconds = self.duration().whole_seconds();
        let page_count = ((total_seconds + MAX_LISTING_SECONDS - 1) / MAX_LISTING_SECONDS).max(1);
        let chunk_seconds = total_seconds / page_count;
        let mut pages = Vec::with_capacity(usize::try_from(page_count).unwrap_or(1));
        let mut page_start = self.start;
        for index in 0..page_count {
            let page_end = if index + 1 == page_count {
                self.end
            } else {
                page_start + Duration::seconds(chunk_seconds)
            };
            pages.push(Self {
                start: page_start,
                end: page_end,
            });
            page_start = page_end;
        }
        pages
    }
}

/// How a batch download spaces and retries its requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadPolicy {
    /// How many hand requests the async client may have in flight. The
    /// blocking client is sequential and ignores this.
    pub max_in_flight: usize,
    /// How long each hand task waits after its request. With N requests in
    /// flight this bounds the average rate; it does not stagger request
    /// starts. A value of zero disables the pause.
    pub delay: std::time::Duration,
    /// How many times a transient failure is retried before it is reported.
    pub max_retries: u32,
}

impl Default for DownloadPolicy {
    fn default() -> Self {
        Self {
            max_in_flight: 2,
            delay: std::time::Duration::from_secs(2),
            max_retries: 2,
        }
    }
}

/// One hand that was fetched successfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DownloadedHand {
    /// The `fetchlin.php` URL it came from.
    pub url: Url,
    /// BBO's stable `<id>.lin` name, or its `Content-Disposition` suggestion
    /// when the link has no id.
    pub filename: String,
    /// The complete LIN document.
    pub lin: String,
}

/// One hand that failed.
#[derive(Debug)]
pub struct DownloadFailure {
    /// The `fetchlin.php` URL that failed.
    pub url: Url,
    /// Why it failed.
    pub error: BboError,
}

/// The outcome of a batch download.
#[derive(Debug, Default)]
pub struct DownloadReport {
    /// Hands fetched successfully, in collection order.
    pub hands: Vec<DownloadedHand>,
    /// Hands that failed.
    pub failures: Vec<DownloadFailure>,
    /// Links `download_to_dir` skipped because the target file already
    /// existed.
    pub skipped: Vec<Url>,
}

/// The shape shared by both clients' synchronous surface.
pub trait BBOClient {
    /// # Errors
    /// Errors when the site is not reachable or its response is unrecognised.
    fn is_logged(&self) -> Result<bool, BboError>;
    /// # Errors
    /// Errors when the site is not reachable or credentials are rejected.
    fn login(&self) -> Result<(), BboError>;
    /// # Errors
    /// Errors when the site is not reachable, the session expired and cannot
    /// be renewed, or BBO rejects the window.
    fn collect_hand_links(&mut self, username: &str, window: Window) -> Result<(), BboError>;
    /// The same as [`BBOClient::collect_hand_links`] for the logged-in user.
    ///
    /// # Errors
    /// See [`BBOClient::collect_hand_links`].
    fn own_hands(&mut self, window: Window) -> Result<(), BboError>;
}

/// A listing page fetched from BBO, before classification.
pub(crate) struct FetchedListing {
    pub request_url: Url,
    pub final_url: Url,
    pub body: String,
}

/// What a listing page tells us.
pub(crate) struct Listing {
    pub links: Vec<Url>,
    pub hidden_tourneys: bool,
}

/// The three states a login/index response can be in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Session {
    LoggedIn,
    LoggedOut,
    Unknown,
}

/// The exact form the BBO login page submits.
pub(crate) fn login_form<'a>(username: &'a str, password: &'a str) -> [(&'a str, &'a str); 6] {
    [
        ("t", LOGIN_TARGET),
        ("count", "1"),
        ("username", username),
        ("password", password),
        ("submit", "Login"),
        ("keep", "on"),
    ]
}

/// The listing URL for a user and a single page, used both to fetch and to
/// resolve the relative links in the response.
pub(crate) fn hands_url(username: &str, window: Window) -> Url {
    let mut url = Url::parse(BBOHANDS).expect("constant listing URL parses");
    url.query_pairs_mut()
        .append_pair("username", username)
        .append_pair("start_time", &window.start().unix_timestamp().to_string())
        .append_pair("end_time", &window.end().unix_timestamp().to_string());
    url
}

/// Decides the session state from the final URL and the response body.
///
/// BBO redirects anonymous requests to `myhands_login.php`; the final URL is
/// the reliable signal, with the page text as a fallback. An empty or unknown
/// body is `Unknown`, never "logged in".
pub(crate) fn session_from_response(final_url: &Url, body: &str) -> Session {
    if final_url.path().ends_with("myhands_login.php") || is_login_body(body) {
        Session::LoggedOut
    } else if body.contains("myhands_logout.php")
        || body.contains("name=\"_qf__myhands\"")
        || body.contains("Logged in as")
    {
        Session::LoggedIn
    } else {
        Session::Unknown
    }
}

/// Whether a body is the myhands login page.
pub(crate) fn is_login_body(body: &str) -> bool {
    body.contains("Please login")
}

/// Whether a body is a hand-record listing.
pub(crate) fn is_listing_body(body: &str) -> bool {
    body.contains("Hands played by")
}

/// Whether BBO rejected the window with its `Invalid input` page.
pub(crate) fn is_invalid_input_body(body: &str) -> bool {
    body.contains("Invalid input")
}

/// Classifies a listing response and extracts its hand links.
///
/// A valid listing always carries the `Hands played by ...` header, even when
/// the answer is "No data found"; an expired session redirects to the login
/// page and a window over 31 days gets the `Invalid input` page, both with
/// HTTP 200.
pub(crate) fn parse_listing(
    request_url: &Url,
    final_url: &Url,
    body: &str,
) -> Result<Listing, BboError> {
    if final_url.path().ends_with("myhands_login.php") || is_login_body(body) {
        return Err(BboError::NotLoggedIn);
    }
    if is_invalid_input_body(body) {
        return Err(BboError::InvalidWindow {
            reason: "BBO rejects a hand-record window longer than 31 days",
        });
    }
    if !is_listing_body(body) {
        return Err(BboError::UnexpectedResponse {
            url: final_url.clone(),
            reason: "the response is not a hand-record listing",
        });
    }
    Ok(Listing {
        links: hand_record_links(request_url, body),
        hidden_tourneys: body.contains("Some tourneys not shown"),
    })
}

/// Extracts the absolute, deduplicated hand-record links from a listing page.
///
/// The listing holds relative `fetchlin.php?...` anchors; `page_url` (the URL
/// the listing was fetched from) resolves them into absolute download URLs.
/// Duplicates are removed by `id`, because the same hand can appear in more
/// than one section of the page.
#[must_use]
pub(crate) fn hand_record_links(page_url: &Url, html: &str) -> Vec<Url> {
    // A static selector cannot fail to parse; the literal is covered by tests.
    let selector = Selector::parse("a[href]").expect("static anchor selector parses");
    let mut seen = std::collections::HashSet::new();
    let mut links = Vec::new();
    for anchor in Html::parse_document(html).select(&selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        let Ok(url) = page_url.join(href) else {
            continue;
        };
        if !url.path().ends_with("fetchlin.php") {
            continue;
        }
        let key = hand_id(&url).unwrap_or_else(|| url.as_str().to_string());
        if seen.insert(key) {
            links.push(url);
        }
    }
    links
}

/// Removes duplicate hand links across listing pages, preserving order.
pub(crate) fn dedup_hand_links(links: &mut Vec<Url>) {
    let mut seen = std::collections::HashSet::new();
    links.retain(|url| {
        let key = hand_id(url).unwrap_or_else(|| url.as_str().to_string());
        seen.insert(key)
    });
}

/// The `id` query parameter of a hand-record link, if it is safe as a filename.
pub(crate) fn hand_id(url: &Url) -> Option<String> {
    let id = url
        .query_pairs()
        .find(|(key, _)| key.as_ref() == "id")
        .map(|(_, value)| value.into_owned())?;
    if id.is_empty()
        || !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return None;
    }
    Some(id)
}

/// The fallback filename for a hand link: `<id>.lin`.
pub(crate) fn hand_filename(url: &Url) -> Option<String> {
    hand_id(url).map(|id| format!("{id}.lin"))
}

/// The filename BBO suggests in `Content-Disposition`, if it is safe.
pub(crate) fn content_disposition_filename(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let value = headers
        .get(reqwest::header::CONTENT_DISPOSITION)?
        .to_str()
        .ok()?;
    let filename = value.split(';').find_map(|part| {
        let part = part.trim();
        part.strip_prefix("filename=").map(str::trim)
    })?;
    let filename = filename.trim_matches(|c| c == '"' || c == '\'');
    let filename = filename.rsplit(['/', '\\']).next()?;
    if filename.is_empty()
        || filename == "."
        || filename == ".."
        || !filename
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
    {
        return None;
    }
    Some(filename.to_string())
}

/// Whether a response advertises BBO's LIN MIME type.
pub(crate) fn is_bbolin_content_type(headers: &reqwest::header::HeaderMap) -> bool {
    headers
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.starts_with("application/bbolin"))
}

/// Whether a body looks like a complete LIN document rather than an HTML page.
pub(crate) fn is_lin_document(body: &str) -> bool {
    body.contains("|md|")
        || body.starts_with("pn|")
        || body.starts_with("st||")
        || body.starts_with("vg|")
}

/// Converts a response status into a `BboError` unless it is a success.
pub(crate) fn ensure_success(url: &Url, status: reqwest::StatusCode) -> Result<(), BboError> {
    if status.is_success() {
        Ok(())
    } else {
        Err(BboError::HttpStatus {
            url: url.clone(),
            status,
        })
    }
}

/// Whether a status is worth retrying.
pub(crate) fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 429 | 500 | 502 | 503 | 504)
}

/// The `Retry-After` delay in seconds, when BBO sends one.
///
/// Only the delta-seconds form is parsed; BBO sends that form. An HTTP-date
/// value falls through to the exponential backoff.
pub(crate) fn retry_after_seconds(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get(reqwest::header::RETRY_AFTER)?
        .to_str()
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// The exponential backoff for a retry attempt.
pub(crate) fn retry_delay(policy: &DownloadPolicy, attempt: u32) -> std::time::Duration {
    policy
        .delay
        .checked_mul(2u32.saturating_pow(attempt))
        .unwrap_or(policy.delay)
}

/// Merges retried hand outcomes back into the original ordered outcomes.
pub(crate) fn merge_retried(
    outcomes: &mut [(Url, Result<DownloadedHand, BboError>)],
    retried: Vec<(Url, Result<DownloadedHand, BboError>)>,
) {
    let mut retried: std::collections::HashMap<Url, Result<DownloadedHand, BboError>> =
        retried.into_iter().collect();
    for (url, result) in outcomes {
        if matches!(result, Err(BboError::NotLoggedIn))
            && let Some(new_result) = retried.remove(url)
        {
            *result = new_result;
        }
    }
}

/// Splits links into those whose target file is missing and those already on
/// disk, using the stable `<id>.lin` name.
pub(crate) fn split_existing(dir: &Path, links: &[Url]) -> (Vec<Url>, Vec<Url>) {
    let mut to_fetch = Vec::new();
    let mut skipped = Vec::new();
    for url in links {
        if hand_filename(url).is_some_and(|name| dir.join(name).exists()) {
            skipped.push(url.clone());
        } else {
            to_fetch.push(url.clone());
        }
    }
    (to_fetch, skipped)
}

/// Writes every fetched hand into `dir`, moving write failures into the report.
pub(crate) fn write_hands(dir: &Path, report: &mut DownloadReport) {
    let mut hands = Vec::with_capacity(report.hands.len());
    for hand in std::mem::take(&mut report.hands) {
        let path = dir.join(&hand.filename);
        match std::fs::write(&path, hand.lin.as_bytes()) {
            Ok(()) => hands.push(hand),
            Err(source) => report.failures.push(DownloadFailure {
                url: hand.url,
                error: BboError::Write { path, source },
            }),
        }
    }
    report.hands = hands;
}

#[cfg(test)]
mod test {
    use super::*;
    use reqwest::header::{HeaderMap, HeaderValue};
    use time::macros::datetime;

    const LOGIN_PAGE: &str = include_str!("../tests/fixtures/login.html");
    const INDEX_PAGE: &str = include_str!("../tests/fixtures/index.html");
    const LISTING_EMPTY: &str = include_str!("../tests/fixtures/listing_empty.html");
    const LISTING_TWO: &str = include_str!("../tests/fixtures/listing_two.html");
    const LISTING_HIDDEN: &str = include_str!("../tests/fixtures/listing_hidden.html");
    const INVALID_INPUT: &str = include_str!("../tests/fixtures/invalid_input.html");

    fn listing_url() -> Url {
        Url::parse(BBOHANDS).expect("constant listing URL parses")
    }

    fn login_url() -> Url {
        Url::parse(BBOLOGIN).expect("constant login URL parses")
    }

    #[test]
    fn login_form_matches_the_page_bbo_serves() {
        assert_eq!(
            login_form("testuser", "secret"),
            [
                ("t", "/myhands/index.php?"),
                ("count", "1"),
                ("username", "testuser"),
                ("password", "secret"),
                ("submit", "Login"),
                ("keep", "on"),
            ]
        );
    }

    #[test]
    fn session_markers_classify_the_login_and_index_pages() {
        assert_eq!(
            session_from_response(&login_url(), LOGIN_PAGE),
            Session::LoggedOut
        );
        assert_eq!(
            session_from_response(&listing_url(), INDEX_PAGE),
            Session::LoggedIn
        );
        assert_eq!(session_from_response(&listing_url(), ""), Session::Unknown);
        // A redirect to the login page wins even if the body is stale.
        assert_eq!(
            session_from_response(&login_url(), INDEX_PAGE),
            Session::LoggedOut
        );
    }

    #[test]
    fn listing_pages_are_classified_by_body_marker() {
        let page = parse_listing(&listing_url(), &listing_url(), LISTING_TWO)
            .expect("a two-hand listing parses");
        assert_eq!(page.links.len(), 2);
        assert!(!page.hidden_tourneys);

        let empty = parse_listing(&listing_url(), &listing_url(), LISTING_EMPTY)
            .expect("an empty listing is still a listing");
        assert!(empty.links.is_empty());

        let hidden = parse_listing(&listing_url(), &listing_url(), LISTING_HIDDEN)
            .expect("a listing with hidden tourneys parses");
        assert!(hidden.hidden_tourneys);
    }

    #[test]
    fn listing_classification_distinguishes_expiry_and_invalid_windows() {
        assert!(matches!(
            parse_listing(&listing_url(), &login_url(), LOGIN_PAGE),
            Err(BboError::NotLoggedIn)
        ));
        assert!(matches!(
            parse_listing(&listing_url(), &listing_url(), INVALID_INPUT),
            Err(BboError::InvalidWindow { .. })
        ));
        assert!(matches!(
            parse_listing(&listing_url(), &listing_url(), "<html></html>"),
            Err(BboError::UnexpectedResponse { .. })
        ));
    }

    #[test]
    fn extracts_and_resolves_relative_hand_links() {
        let links = hand_record_links(&listing_url(), LISTING_TWO);
        assert_eq!(
            links,
            vec![
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=4341290925&when_played=1733777178")
                    .expect("expected URL parses"),
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=4341341979&when_played=1733780617")
                    .expect("expected URL parses"),
            ]
        );
    }

    #[test]
    fn duplicate_hand_links_are_collapsed_by_id() {
        let page = r#"
            <a href="fetchlin.php?id=1&when_played=10">lin</a>
            <a href="fetchlin.php?id=1&when_played=10">lin again</a>
            <a href="fetchlin.php?id=2&when_played=20">lin</a>
        "#;
        assert_eq!(hand_record_links(&listing_url(), page).len(), 2);
    }

    #[test]
    fn cross_page_duplicates_are_collapsed_by_id() {
        let first =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=1&when_played=10")
                .expect("valid URL");
        let second =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=2&when_played=20")
                .expect("valid URL");
        let mut links = vec![first.clone(), second, first];
        dedup_hand_links(&mut links);
        assert_eq!(links.len(), 2);
    }

    #[test]
    fn ignores_pages_without_hand_links() {
        assert!(hand_record_links(&listing_url(), "").is_empty());
        assert!(
            hand_record_links(&listing_url(), "<html><a href=\"help.php\">help</a></html>")
                .is_empty()
        );
    }

    #[test]
    fn hand_ids_and_filenames_are_safe() {
        let url = Url::parse(
            "https://www.bridgebase.com/myhands/fetchlin.php?id=4341290925&when_played=1733777178",
        )
        .expect("valid URL");
        assert_eq!(hand_id(&url).as_deref(), Some("4341290925"));
        assert_eq!(hand_filename(&url).as_deref(), Some("4341290925.lin"));

        let unsafe_url = Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=../x")
            .expect("valid URL");
        assert_eq!(hand_id(&unsafe_url), None);
        assert_eq!(hand_filename(&unsafe_url), None);
    }

    #[test]
    fn content_disposition_filename_is_extracted_and_sanitised() {
        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            HeaderValue::from_static("inline; filename=\"4341290925.lin\""),
        );
        assert_eq!(
            content_disposition_filename(&headers).as_deref(),
            Some("4341290925.lin")
        );

        headers.insert(
            reqwest::header::CONTENT_DISPOSITION,
            HeaderValue::from_static("inline; filename=\"../../etc/passwd\""),
        );
        assert_eq!(
            content_disposition_filename(&headers).as_deref(),
            Some("passwd")
        );
    }

    #[test]
    fn lin_documents_are_distinguished_from_html() {
        assert!(is_lin_document("pn|a,b,c,d|st||md|3S6KAH3TKD5JQC59QK,...|"));
        assert!(!is_lin_document(LOGIN_PAGE));

        let mut headers = HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/bbolin"),
        );
        assert!(is_bbolin_content_type(&headers));
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=UTF-8"),
        );
        assert!(!is_bbolin_content_type(&headers));
    }

    #[test]
    fn window_requires_an_ordered_pair() {
        let early = datetime!(2024-12-01 0:00 UTC);
        let late = datetime!(2025-01-01 0:00 UTC);
        assert!(Window::new(early, late).is_ok());
        assert!(Window::new(late, early).is_err());
        assert!(Window::new(early, early).is_err());
    }

    #[test]
    fn window_direction_constructors_model_the_website() {
        let anchor = datetime!(2025-01-01 0:00 UTC);
        let backward = Window::ending_at(anchor, Duration::days(31)).expect("valid window");
        assert_eq!(backward.start(), datetime!(2024-12-01 0:00 UTC));
        assert_eq!(backward.end(), anchor);

        let forward = Window::starting_at(anchor, Duration::days(31)).expect("valid window");
        assert_eq!(forward.start(), anchor);
        assert_eq!(forward.end(), datetime!(2025-02-01 0:00 UTC));
    }

    #[test]
    fn window_pages_are_at_most_31_days_and_split_evenly() {
        let start = datetime!(2024-12-01 0:00 UTC);
        let exact = Window::new(start, start + Duration::days(31)).expect("valid window");
        assert_eq!(exact.pages(), vec![exact]);

        let thirty_two = Window::new(start, start + Duration::days(32)).expect("valid window");
        assert_eq!(
            thirty_two.pages(),
            vec![
                Window::new(start, start + Duration::days(16)).expect("valid page"),
                Window::new(start + Duration::days(16), start + Duration::days(32))
                    .expect("valid page"),
            ]
        );

        let sixty_two = Window::new(start, start + Duration::days(62)).expect("valid window");
        assert_eq!(sixty_two.pages().len(), 2);

        let sixty_three = Window::new(start, start + Duration::days(63)).expect("valid window");
        let pages = sixty_three.pages();
        assert_eq!(pages.len(), 3);
        assert_eq!(
            pages.first().expect("three pages").duration(),
            Duration::days(21)
        );
        assert_eq!(pages.last().expect("three pages").end(), sixty_three.end());
    }

    #[test]
    fn retry_headers_and_statuses_are_recognised() {
        let mut headers = HeaderMap::new();
        headers.insert(reqwest::header::RETRY_AFTER, HeaderValue::from_static("7"));
        assert_eq!(retry_after_seconds(&headers), Some(7));
        assert!(is_retryable_status(reqwest::StatusCode::TOO_MANY_REQUESTS));
        assert!(!is_retryable_status(reqwest::StatusCode::NOT_FOUND));
    }

    #[test]
    fn retried_outcomes_replace_only_the_login_failures() {
        let retried_url =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=1&when_played=10")
                .expect("valid URL");
        let other_url =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=2&when_played=20")
                .expect("valid URL");
        let hand = DownloadedHand {
            url: retried_url.clone(),
            filename: String::from("1.lin"),
            lin: String::from("pn|a,b,c,d|st||md|...|"),
        };
        let mut outcomes = vec![
            (retried_url.clone(), Err(BboError::NotLoggedIn)),
            (
                other_url.clone(),
                Err(BboError::HttpStatus {
                    url: other_url,
                    status: reqwest::StatusCode::BAD_GATEWAY,
                }),
            ),
        ];

        merge_retried(&mut outcomes, vec![(retried_url, Ok(hand))]);

        assert!(outcomes.first().expect("two outcomes").1.is_ok());
        assert!(matches!(
            outcomes.get(1).expect("two outcomes").1,
            Err(BboError::HttpStatus { .. })
        ));
    }

    #[test]
    fn split_existing_skips_files_already_on_disk() {
        let dir = std::env::temp_dir().join(format!(
            "squeezer-bbo-split-existing-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("temp dir is writable");
        std::fs::write(dir.join("1.lin"), "existing").expect("temp file is writable");
        let first =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=1&when_played=10")
                .expect("valid URL");
        let second =
            Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=2&when_played=20")
                .expect("valid URL");

        let (to_fetch, skipped) = split_existing(&dir, &[first.clone(), second.clone()]);

        assert_eq!(to_fetch, vec![second]);
        assert_eq!(skipped, vec![first]);
        std::fs::remove_dir_all(dir).expect("temp dir is removable");
    }

    #[test]
    fn write_hands_writes_documents_and_records_failures() {
        let dir =
            std::env::temp_dir().join(format!("squeezer-bbo-write-report-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir is writable");
        let url = Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=1&when_played=10")
            .expect("valid URL");
        let mut report = DownloadReport {
            hands: vec![DownloadedHand {
                url,
                filename: String::from("1.lin"),
                lin: String::from("pn|a,b,c,d|st||md|...|"),
            }],
            failures: Vec::new(),
            skipped: Vec::new(),
        };

        write_hands(&dir, &mut report);
        assert_eq!(report.hands.len(), 1);
        assert_eq!(
            std::fs::read_to_string(dir.join("1.lin")).expect("file was written"),
            "pn|a,b,c,d|st||md|...|"
        );
        std::fs::remove_dir_all(dir).expect("temp dir is removable");
    }
}
