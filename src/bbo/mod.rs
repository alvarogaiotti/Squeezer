use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
#[cfg(not(test))]
use log::{debug, error, info, warn};
use regex::Regex;
#[cfg(test)]
use std::{println as info, println as warn, println as debug, println as error};
use time::{Duration, OffsetDateTime};
use ureq::Agent; // Workaround to use prinltn! for logs.

pub const BBOLOGIN: &str =
    "https://www.bridgebase.com/myhands/myhands_login.php?t=%2Fmyhands%2Findex.php%3F";
pub const BBOLAND: &str = "https://www.bridgebase.com/myhands/index.php?offset=0";
pub const BBOHANDS: &str = "https://www.bridgebase.com/myhands/hands.php?";
pub const BBOBASE: &str = "https://www.bridgebase.com/myhands/";

pub trait BBOClient {
    fn is_logged(&self) -> Result<bool>;
    fn login(&self) -> Result<()>;
    fn download(&self) -> Result<()>;
    fn get_in_interval(
        &mut self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<()>;
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
    ) -> Result<String, anyhow::Error> {
        let request = self.client.get(BBOHANDS).query_pairs([
            ("username", self.username.as_str()),
            ("start_time", &start_time.unix_timestamp().to_string()),
            ("end_time", &end_time.unix_timestamp().to_string()),
        ]);
        info!("query: {}", request.url());
        let text = request
            .call()
            .with_context(|| format!("unable to send request to {BBOBASE}, maybe BBO is down?"))?
            .into_string()
            .context("unable to parse response text")?;
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
            RE.captures_iter(webpage)
                .map(|matches| matches.name("lin").unwrap().as_str().to_owned()),
        );
    }
}
impl BBOClient for BlockingBBOClient {
    /// Check login status
    fn is_logged(&self) -> Result<bool> {
        info!("checking if logged in");
        if self
            .client
            .get(BBOLOGIN)
            .call()
            .with_context(|| format!("unable to send request, maybe {} not responding", BBOLOGIN))?
            .into_string()
            .context("unable to get text from response")?
            .contains("Please login")
        {
            info!("logged in");
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Login procedure
    fn login(&self) -> Result<()> {
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
            .with_context(|| {
                format!("unable to send login request, maybe {} is down?", BBOLOGIN)
            })?;
        info!("response status code: {}", response.status());
        let text = response.into_string().context("unable to extract text")?;
        if text.contains("password incorrect") {
            info!("incorrect username or password");
            bail!("incorrect username or password")
        }
        let _ = self
            .client
            .get("http://www.bridgebase.com/myhands/hands.php?offset=0")
            .call()?;
        Ok(())
    }

    fn get_in_interval(
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

    fn download(&self) -> Result<()> {
        for mut hand in self.hands_links.iter().cloned() {
            hand.insert_str(0, BBOBASE);

            info!("downloading from {}", hand);
            let lin = match self.client.get(&hand).call() {
                Err(e) => {
                    warn!("unable to download from {}.\nSee: {}", &hand, e);
                    continue;
                }
                Ok(response) => response.into_string().context("unable to read response")?,
            };
            // For now puppy implementation
            println!("{}", lin);
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        Ok(())
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
