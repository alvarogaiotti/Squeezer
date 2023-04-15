use crate::bbo::{LinkExtractor, BBOBASE, BBOHANDS, BBOLOGIN};
use anyhow::{bail, Context, Result};
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use regex::Regex;
use reqwest::Client;
use time::{Duration, OffsetDateTime};

pub struct AsyncBBOClient {
    client: Client,
    username: String,
    password: String,
    hands_links: Vec<String>,
}

impl LinkExtractor for AsyncBBOClient {
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

impl AsyncBBOClient {
    pub fn new(username: String, password: String) -> Self {
        let client = match Client::builder().cookie_store(true).build() {
            Ok(client) => client,
            Err(e) => {
                error!("unable to create cookie storing client: {e}");
                Client::default()
            }
        };
        Self {
            client,
            username,
            password,
            hands_links: Vec::new(),
        }
    }
    /// Check login status
    pub async fn is_logged(&self) -> Result<bool> {
        info!("checking if logged in");
        if self
            .client
            .get(BBOLOGIN)
            .send()
            .await
            .with_context(|| format!("unable to send request, maybe {} not responding", BBOLOGIN))?
            .text()
            .await
            .context("unable to get text from response")?
            .contains("Please login")
        {
            info!("logged in");
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Login procedure
    async fn login(&self) -> Result<()> {
        info!("starting login procedure");
        if self.is_logged().await? {
            return Ok(());
        }
        let params = [
            ("username", self.username.as_str()),
            ("password", self.password.as_str()),
        ];

        info!(
            "sending login request with username: {} and password:{}",
            self.username.as_str(),
            self.password.as_str()
        );

        let response = self
            .client
            .post(BBOLOGIN)
            .form(&params)
            .send()
            .await
            .with_context(|| format!("unable to send login request, maybe {} is down?", BBOLOGIN))?
            .text()
            .await
            .context("unable to extract text")?;
        if response.contains("password incorrect") {
            info!("incorrect username or password");
            bail!("incorrect username or password")
        }
        info!("Succesfully logged in");

        Ok(())
    }

    async fn get_in_interval(
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
        let mut vec: Vec<String> = Vec::new();

        while (start_time - end_time).whole_days() > 28 {
            let next_start = start_time - Duration::days(28);
            let text = self.get_hands_in_interval(start_time, next_start).await?;
            start_time = next_start;
            self.get_links(&text, &mut vec);
        }
        let text = self.get_hands_in_interval(start_time, end_time).await?;
        self.get_links(&text, &mut vec);
        self.hands_links = vec;

        Ok(())
    }

    async fn get_hands_in_interval(
        &self,
        start_time: OffsetDateTime,
        end_time: OffsetDateTime,
    ) -> Result<String, anyhow::Error> {
        let text = self
            .client
            .get(BBOHANDS)
            .query(&[("username", self.username.as_str())])
            .query(&[("start_time", start_time.unix_timestamp())])
            .query(&[("end_time", (end_time).unix_timestamp())])
            .send()
            .await
            .with_context(|| format!("unable to send request to {BBOBASE}, maybe BBO is down?"))?
            .text()
            .await
            .context("unable to parse response text")?;
        Ok(text)
    }

    pub async fn download(&self) -> Result<()> {
        for mut hand in self.hands_links.iter().cloned() {
            hand.insert_str(0, BBOBASE);

            info!("downloading from {}", hand);
            let lin = match self.client.get(&hand).send().await {
                Err(e) => {
                    warn!("unable to download from {}.\nSee: {}", &hand, e);
                    continue;
                }
                Ok(response) => response.text().await.context("unable to read response")?,
            };
        }
        Ok(())
    }
}
