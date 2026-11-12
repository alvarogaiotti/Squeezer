// Copyright (C) 2024 Alvaro Gaiotti
// See end of file for license information

//! Shared BBO plumbing: endpoints, errors, and the hand-record link parser.
//!
//! Both the blocking and async clients speak `reqwest`, so the error type is
//! no longer generic over the transport. Link extraction is shared because the
//! listing page markup is identical for both clients.

use scraper::{Html, Selector};
use time::{Duration, OffsetDateTime};
use url::Url;

/// Login endpoint; the `t=` parameter is the page to land on afterwards.
pub const BBOLOGIN: &str =
    "https://www.bridgebase.com/myhands/myhands_login.php?t=%2Fmyhands%2Findex.php%3F";
/// The hand-record listing page.
pub const BBOLAND: &str = "https://www.bridgebase.com/myhands/index.php?offset=0";
/// The endpoint that returns a user's hands for a time interval.
pub const BBOHANDS: &str = "https://www.bridgebase.com/myhands/hands.php?";
/// Base used to resolve the relative `fetchlin.php` links in the listing.
pub const BBOBASE: &str = "https://www.bridgebase.com/myhands/";

/// A BBO download or login failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum BboError {
    /// The transport failed (DNS, TLS, timeout, ...).
    Connection(reqwest::Error),
    /// The response body could not be read.
    Io(std::io::Error),
    /// BBO rejected the supplied credentials.
    Login,
    /// A hand page could not be downloaded.
    Download(reqwest::Error),
    /// The hand-record listing could not be requested.
    HandsRequest(reqwest::Error),
}

impl std::fmt::Display for BboError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connection(_) => write!(f, "unable to connect, check internet connection"),
            Self::Io(_) => write!(f, "unable to read the response body"),
            Self::Login => write!(f, "incorrect username or password"),
            Self::Download(_) => write!(f, "unable to download hand"),
            Self::HandsRequest(_) => write!(f, "incorrect time interval request parameters"),
        }
    }
}

impl std::error::Error for BboError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Connection(error) | Self::Download(error) | Self::HandsRequest(error) => {
                Some(error)
            }
            Self::Io(error) => Some(error),
            Self::Login => None,
        }
    }
}

impl From<reqwest::Error> for BboError {
    fn from(value: reqwest::Error) -> Self {
        Self::Connection(value)
    }
}

impl From<std::io::Error> for BboError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

/// The shape shared by both clients' public surface.
pub trait BBOClient {
    /// # Errors
    /// Errors when the site is not reachable.
    fn is_logged(&self) -> Result<bool, BboError>;
    /// # Errors
    /// Errors when the site is not reachable or credentials are rejected.
    fn login(&self) -> Result<(), BboError>;
    /// # Errors
    /// Errors when the site is not reachable.
    fn get_in_interval(
        &mut self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<(), BboError>;
}

/// Decides login status from a login-page body.
///
/// BBO serves "Please login ..." to anonymous visitors, so a body without that
/// marker means the cookie jar already holds a session. Pure, so the branch is
/// unit-testable without network access.
#[must_use]
pub(crate) fn logged_in_from_body(body: &str) -> bool {
    !body.contains("Please login")
}

/// The longest interval BBO returns in one listing page.
const MAX_PAGE: Duration = Duration::days(28);

/// Splits an interval into listing pages of at most 28 days.
///
/// BBO caps a listing at 28 days, so a wider interval is queried page by page.
/// The arguments may arrive in either order; the pages always run from the
/// earlier instant to the later one. A zero-length interval stays one page,
/// because refusing it here would only move the decision to the caller.
#[must_use]
pub(crate) fn interval_pages(
    start_time: OffsetDateTime,
    end_time: OffsetDateTime,
) -> Vec<(OffsetDateTime, OffsetDateTime)> {
    let (start_time, end_time) = if start_time > end_time {
        log::debug!("interval arguments were reversed; starting from the earlier instant");
        (end_time, start_time)
    } else {
        (start_time, end_time)
    };
    let mut pages = Vec::new();
    let mut page_start = start_time;
    while end_time - page_start > MAX_PAGE {
        let page_end = page_start + MAX_PAGE;
        pages.push((page_start, page_end));
        page_start = page_end;
    }
    pages.push((page_start, end_time));
    pages
}

/// Extracts the absolute hand-record links from a listing page.
///
/// The listing holds relative `fetchlin.php?...` anchors; `page_url` (the URL
/// the listing was fetched from) resolves them into absolute download URLs.
/// Parsing the HTML with `scraper` avoids the brittle regex the clients used
/// to carry, which matched the quoted string anywhere in the page.
#[must_use]
pub(crate) fn hand_record_links(page_url: &Url, html: &str) -> Vec<Url> {
    // A static selector cannot fail to parse; the literal is covered by tests.
    let selector = Selector::parse("a[href]").expect("static anchor selector parses");
    Html::parse_document(html)
        .select(&selector)
        .filter_map(|anchor| anchor.value().attr("href"))
        .filter_map(|href| page_url.join(href).ok())
        .filter(|url| url.path().ends_with("fetchlin.php"))
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    fn listing_url() -> Url {
        Url::parse(BBOHANDS).expect("constant listing URL parses")
    }

    #[test]
    fn extracts_and_resolves_relative_hand_links() {
        let page = r#"
            <html><body>
              <a href="fetchlin.php?id=765880487&when_played=1610464081">lin</a>
              <a href="fetchlin.php?id=2&when_played=3">lin</a>
              <a href="index.php?offset=0">next page</a>
            </body></html>
        "#;
        let links = hand_record_links(&listing_url(), page);
        assert_eq!(
            links,
            vec![
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=765880487&when_played=1610464081")
                    .expect("expected URL parses"),
                Url::parse("https://www.bridgebase.com/myhands/fetchlin.php?id=2&when_played=3")
                    .expect("expected URL parses"),
            ]
        );
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
    fn login_status_decision_matches_site_marker() {
        assert!(!logged_in_from_body("Please login to continue"));
        assert!(logged_in_from_body("Welcome back, your tables await"));
        assert!(logged_in_from_body(""));
    }

    #[test]
    fn interval_pages_split_an_ascending_interval_into_28_day_pages() {
        use time::macros::datetime;

        let start = datetime!(2022-03-03 0:00 UTC);
        let end = datetime!(2022-04-01 0:00 UTC);

        assert_eq!(
            interval_pages(start, end),
            vec![
                (start, datetime!(2022-03-31 0:00 UTC)),
                (datetime!(2022-03-31 0:00 UTC), end),
            ]
        );
    }

    #[test]
    fn interval_pages_accept_either_argument_order() {
        use time::macros::datetime;

        let early = datetime!(2022-03-03 0:00 UTC);
        let late = datetime!(2022-03-03 1:00 UTC);

        assert_eq!(interval_pages(late, early), interval_pages(early, late));
        assert_eq!(interval_pages(early, late), vec![(early, late)]);
    }

    #[test]
    fn interval_pages_keep_an_exact_28_day_interval_in_one_page() {
        use time::macros::datetime;

        let start = datetime!(2022-03-03 0:00 UTC);
        let end = datetime!(2022-03-31 0:00 UTC);

        assert_eq!(interval_pages(start, end), vec![(start, end)]);
    }
}
