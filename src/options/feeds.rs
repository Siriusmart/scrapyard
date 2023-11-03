use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;
use subprocess::{Exec, Redirection};
use tokio::{fs, io::AsyncWriteExt, task::spawn_blocking};

use crate::{
    bindings::{ItemizerArg, ItemizerRes, PseudoChannel, PseudoItem},
    take_lock,
    traits::Saveable,
    values::{LOCKS, MASTER},
    PseudoItemCache,
};

use super::fetched::FetchedMeta;

/// Array of feeds to fetch
#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone, Debug)]
pub struct Feeds(#[serde_inline_default(vec![FeedOption::default()])] pub Vec<FeedOption>);

impl Saveable for Feeds {}

impl Feeds {
    /// Validate if all feeds have valid scraper commands
    pub fn validate(&self) {
        self.0.iter().for_each(|channel| channel.validate())
    }

    /// Start auto fetching all feeds by interval, and considering idle sleeping
    pub async fn start_loop(self) {
        self.0.into_iter().for_each(|feed| {
            tokio::spawn(async move {
                let meta_path = MASTER
                    .get()
                    .unwrap()
                    .store
                    .join(&feed.label)
                    .join("meta.json");

                if !fs::try_exists(&meta_path).await.unwrap() {
                    fs::create_dir_all(&meta_path.parent().unwrap())
                        .await
                        .unwrap();
                    FetchedMeta::default().save_json(&meta_path).await.unwrap();
                }

                let feed: Arc<FeedOption> = Arc::new(feed);

                loop {
                    let feed = feed.clone();
                    let meta_path = meta_path.clone();
                    // so panic inside this block wont exit the event loop
                    let _ = tokio::task::spawn(async move {
                        loop {
                            let meta = FetchedMeta::load_json(&meta_path).await.unwrap_or_default();
                            match feed.time_til_outdated(&meta) {
                                Some(secs) => tokio::time::sleep(Duration::from_secs(secs)).await,
                                None => break,
                            }
                        }

                        let meta = FetchedMeta::load_json(&meta_path).await.unwrap_or_default();

                        if feed.idle(&meta) {
                            tokio::time::sleep(Duration::from_secs(feed.interval)).await;
                            return;
                        }

                        let _lock = take_lock!(LOCKS, feed.label.clone());

                        let mut meta = FetchedMeta::load_json(&meta_path).await.unwrap_or_default();
                        if let Err(e) = feed.fetch_items_noreturn(&meta).await {
                            println!("Error fetching feed: {e}");
                        }

                        meta.fetched();
                        meta.save_json(&meta_path).await.unwrap();
                    })
                    .await;
                }
            });
        })
    }

    pub fn to_map(self) -> HashMap<String, FeedOption> {
        HashMap::from_iter(self.0.into_iter().map(|feed| (feed.label.clone(), feed)))
    }
}

/// Specific scraping options for a single feed
#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Clone, Debug)]
pub struct FeedOption {
    /// URL to fetch
    #[serde_inline_default("https://feeds.bbci.co.uk/news/world/rss.xml".to_string())]
    pub origin: String,
    /// Text label of the feed
    #[serde_inline_default("default-feed-name".to_string())]
    pub label: String,
    /// Max feed item length
    #[serde(rename = "max-length")]
    #[serde_inline_default(50)]
    pub max_length: usize,
    /// Max length to fetch per interval
    #[serde(rename = "fetch-length")]
    #[serde_inline_default(10)]
    pub fetch_length: usize,
    /// Interval between fetching
    #[serde_inline_default(3600)] // 1 hour
    pub interval: u64,
    /// Duration of no requests before scraping stops
    #[serde_inline_default(172800)] // 2 days
    #[serde(rename = "idle-limit")]
    pub idle_limit: u64,
    /// To sort items by publish date or not
    #[serde_inline_default(true)]
    pub sort: bool,
    /// Scraper script
    #[serde_inline_default(vec!["/usr/bin/node".to_string(), "/path/to/script.js".to_string()])]
    pub extractor: Vec<String>,
    #[serde_inline_default(true)]
    pub fetch: bool,

    /// Channel details
    #[serde(default)]
    #[serde(flatten)]
    pub channel: PseudoChannel,
}

impl FeedOption {
    /// Check if the extractor command is valid (not empty)
    pub fn validate(&self) {
        if self.extractor.is_empty() {
            panic!("empty extractor")
        }
    }

    pub async fn meta(&self) -> Result<FetchedMeta, Box<dyn Error>> {
        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        FetchedMeta::load_json(&meta_path).await
    }

    /// Fetch rss (xml) string from either remote or cache
    pub async fn lazy_fetch_rss(&self) -> Result<String, Box<dyn Error>> {
        let rss_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.xml");

        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;

        if self.outdated(&meta) {
            self.fetch_items_noreturn(&meta).await?;
            meta.fetched();
            meta.requested();
            meta.save_json(&meta_path).await?;

            return PseudoChannel::load_string(&rss_path).await;
        }

        meta.requested();
        meta.save_json(&meta_path).await?;
        PseudoChannel::load_string(&rss_path).await
    }

    /// Fetch rss (xml) string from remote
    pub async fn force_fetch_rss(&self) -> Result<String, Box<dyn Error>> {
        let rss_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.xml");
        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;
        self.fetch_items_noreturn(&meta).await?;
        meta.fetched();
        meta.requested();
        meta.save_json(&meta_path).await?;

        PseudoChannel::load_string(&rss_path).await
    }

    /// Fetch json string from either remote or cache
    pub async fn lazy_fetch_json(&self) -> Result<String, Box<dyn Error>> {
        let json_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.json");

        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;

        if self.outdated(&meta) {
            self.fetch_items_noreturn(&meta).await?;
            meta.fetched();
            meta.requested();
            meta.save_json(&meta_path).await?;

            return PseudoChannel::load_string(&json_path).await;
        }

        meta.requested();
        meta.save_json(&meta_path).await?;
        PseudoChannel::load_string(&json_path).await
    }

    /// Fetch json string from remote
    pub async fn force_fetch_json(&self) -> Result<String, Box<dyn Error>> {
        let json_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.json");
        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;
        self.fetch_items_noreturn(&meta).await?;
        meta.fetched();
        meta.requested();
        meta.save_json(&meta_path).await?;

        PseudoChannel::load_string(&json_path).await
    }

    /// Fetch a feed either from remote or a cached version
    pub async fn lazy_fetch(&self) -> Result<PseudoChannel, Box<dyn Error>> {
        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;

        if self.outdated(&meta) {
            let fetched = self.fetch_items_return(&meta).await?;
            meta.fetched();
            meta.requested();
            meta.save_json(&meta_path).await?;
            return Ok(self.channel.clone().with_items(fetched));
        }

        meta.requested();
        meta.save_json(&meta_path).await?;

        let json_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.json");
        PseudoChannel::load_json(&json_path).await
    }

    /// Fetch a feed and saves metadata
    pub async fn force_fetch(&self) -> Result<PseudoChannel, Box<dyn Error>> {
        let meta_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("meta.json");
        let mut meta = FetchedMeta::load_json(&meta_path).await?;
        let items = self.fetch_items_return(&meta).await?;
        meta.fetched();
        meta.requested();
        meta.save_json(&meta_path).await?;

        Ok(self.channel.clone().with_items(items))
    }

    /// Check if a feed is outdated
    pub fn outdated(&self, meta: &FetchedMeta) -> bool {
        meta.last_fetch + self.interval < Utc::now().timestamp() as u64
    }

    /// Number of seconds before feed will become outdated
    pub fn time_til_outdated(&self, meta: &FetchedMeta) -> Option<u64> {
        (meta.last_fetch + self.interval).checked_sub(Utc::now().timestamp() as u64)
    }

    /// Check if a feed has passed the idle limit
    pub fn idle(&self, meta: &FetchedMeta) -> bool {
        meta.last_requested + self.idle_limit < Utc::now().timestamp() as u64
    }

    /// Fetch and save cache to files, and return the value
    async fn fetch_items_return(
        &self,
        meta: &FetchedMeta,
    ) -> Result<Vec<PseudoItem>, Box<dyn Error>> {
        let rss_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.xml");
        let json_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.json");

        let mut json = if fs::try_exists(&json_path).await? {
            PseudoItemCache::load_json(&json_path).await.unwrap_or_else(|_| {
                let new_path = json_path.with_file_name(format!("cache-{}.json", chrono::Utc::now().to_rfc3339()));
                println!("Could not load json for {}, continuing with default.\nOld file has been moved to {}", self.label, new_path.to_string_lossy());
                PseudoItemCache::default()
            })
        } else {
            PseudoItemCache::default()
        };

        let mut items = Vec::new();
        let fetch_length = std::cmp::min(
            self.max_length,
            std::cmp::max(
                ((chrono::Utc::now().timestamp() as u64 - meta.last_fetch + 1) / self.interval
                    * self.fetch_length as u64) as usize,
                self.fetch_length,
            ),
        );

        for i in 0..MASTER.get().unwrap().max_retries {
            match self
                .fetch_items_recurse(
                    &mut items,
                    json.0
                        .clone()
                        .into_iter()
                        .map(|item| PseudoItem {
                            content: None,
                            ..item
                        })
                        .collect(),
                    &self.origin,
                    fetch_length as usize,
                )
                .await
            {
                Ok(()) => break,
                Err(e) => println!("Error fetching {} on retry {}: {e}", self.origin, i + 1),
            }

            items.clear()
        }

        items.iter_mut().for_each(|item| {
            if item.timestamp.is_some() {
                return;
            }

            if let Some(pub_date) = &item.pub_date {
                item.timestamp = Some(match DateTime::parse_from_rfc2822(pub_date) {
                    Ok(date) => date.timestamp() as u64,
                    Err(_) => return,
                })
            }
        });
        items.append(&mut json.0);
        if self.sort {
            items.sort_by(|item, other| other.timestamp.cmp(&item.timestamp));
        }

        if items.len() > self.max_length {
            items.drain(self.max_length..);
        }

        json.0 = items.clone();
        json.save_json(&json_path).await?;

        let rss = PseudoChannel {
            items: Some(items.clone()),
            ..self.channel.clone()
        };

        rss.save_rss(&rss_path).await?;

        Ok(items)
    }

    /// Fetch and save cache to files
    async fn fetch_items_noreturn(&self, meta: &FetchedMeta) -> Result<(), Box<dyn Error>> {
        let rss_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.xml");
        let json_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("cache.json");

        let mut json = if fs::try_exists(&json_path).await? {
            PseudoItemCache::load_json(&json_path).await.unwrap_or_else(|_| {
                let new_path = json_path.with_file_name(format!("cache-{}.json", chrono::Utc::now().to_rfc3339()));
                println!("Could not load json for {}, continuing with default.\nOld file has been moved to {}", self.label, new_path.to_string_lossy());
                PseudoItemCache::default()
            })
        } else {
            PseudoItemCache::default()
        };

        let mut items = Vec::new();
        let fetch_length = std::cmp::min(
            self.max_length,
            std::cmp::max(
                ((chrono::Utc::now().timestamp() as u64 - meta.last_fetch + 1) / self.interval
                    * self.fetch_length as u64) as usize,
                self.fetch_length,
            ),
        );

        for i in 0..MASTER.get().unwrap().max_retries {
            match self
                .fetch_items_recurse(
                    &mut items,
                    json.0
                        .clone()
                        .into_iter()
                        .map(|item| PseudoItem {
                            content: None,
                            ..item
                        })
                        .collect(),
                    &self.origin,
                    fetch_length as usize,
                )
                .await
            {
                Ok(()) => break,
                Err(e) => println!("Error fetching {} on retry {}: {e}", self.origin, i + 1),
            }

            items.clear()
        }

        items.iter_mut().for_each(|item| {
            if item.timestamp.is_some() {
                return;
            }

            if let Some(pub_date) = &item.pub_date {
                item.timestamp = Some(match DateTime::parse_from_rfc2822(pub_date) {
                    Ok(date) => date.timestamp() as u64,
                    Err(_) => return,
                })
            }
        });
        items.append(&mut json.0);
        if self.sort {
            items.sort_by(|item, other| other.timestamp.cmp(&item.timestamp));
        }

        if items.len() > self.max_length {
            items.drain(self.max_length..);
        }

        json.0 = items.clone();
        json.save_json(&json_path).await?;

        let rss = PseudoChannel {
            items: Some(items),
            ..self.channel.clone()
        };

        rss.save_rss(&rss_path).await?;

        Ok(())
    }

    /// Private recursive function to fetch items
    #[async_recursion]
    async fn fetch_items_recurse(
        &self,
        items: &mut Vec<PseudoItem>,
        original: Vec<PseudoItem>,
        url: &str,
        fetch_length: usize,
    ) -> Result<(), Box<dyn Error>> {
        let mut preexists = original.clone();
        preexists.append(&mut items.clone());

        let arg = ItemizerArg {
            url: url.to_string(),
            webstr: if self.fetch {
                Some(tokio::select! {
                    res = reqwest::get(url).await?.text() => {
                        res?
                    },
                    _ = tokio::time::sleep(Duration::from_secs(MASTER.get().unwrap().request_timeout)) => {
                        return Err(crate::Error::Timedout.into());
                    }
                })
            } else {
                None
            },
            preexists,
            feed: self.clone(),
            length_left: fetch_length.checked_sub(items.len()).unwrap_or_default() as u32,
        };
        let arg_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("args.json");

        {
            let mut arg_file = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .open(&arg_path)
                .await?;

            arg_file
                .write_all(serde_json::to_vec(&arg)?.as_slice())
                .await?;
        }

        // redirects stdout to a file to avoid the stdio buffer limit
        let label = self.label.clone();
        let extractor = self.extractor.clone();
        let extract = spawn_blocking(move || -> Result<(), serde_json::Error> {
            let stdout_path = MASTER.get().unwrap().store.join(label).join("stdout.txt");
            let stderr_path = stdout_path.with_file_name("stderr.txt");
            let stdout_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(stdout_path)
                .unwrap();
            let stderr_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(stderr_path)
                .unwrap();
            let mut popen = Exec::cmd(extractor.first().unwrap())
                .args(&extractor[1..])
                .arg(arg_path.to_str().unwrap())
                .stdout(Redirection::File(stdout_file))
                .stderr(Redirection::File(stderr_file))
                .popen()
                .unwrap();
            popen.wait().unwrap();
            Ok(())
        });

        tokio::select! {
            res = extract => {
                res??
            },
            _ = tokio::time::sleep(Duration::from_secs(MASTER.get().unwrap().request_timeout)) => {
                return Err(crate::Error::FetchFailed.into());
            }
        };

        let stdout_path = MASTER
            .get()
            .unwrap()
            .store
            .join(&self.label)
            .join("stdout.txt");
        let stdout = fs::read_to_string(stdout_path).await?;
        let res: ItemizerRes = match serde_json::from_str(stdout.as_str()) {
            Ok(res) => res,
            Err(e) => {
                let stderr_path = MASTER
                    .get()
                    .unwrap()
                    .store
                    .join(&self.label)
                    .join("stderr.txt");
                let stderr = fs::read_to_string(stderr_path).await?;
                println!("Could not deserialize scraper output: {e}");
                println!("Scraper stdout:\n{}", stdout);
                println!("Scraper stderr:\n{}", stderr);
                return Err(e.into());
            }
        };
        items.extend(res.items);

        if items.len() >= self.max_length {
            return Ok(());
        }

        if let Some(continuation) = res.continuation {
            self.fetch_items_recurse(items, original, continuation.as_str(), fetch_length)
                .await?
        }

        Ok(())
    }
}
