use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::traits::Saveable;

/// Main config file
#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde, Debug)]
pub struct MasterConfig {
    /// Where cache and metadata of feeds are stored
    #[serde_inline_default(PathBuf::from("/full/path/to/dir"))]
    pub store: PathBuf,
    /// Maximum number of retries before giving up scraping a feed
    #[serde(rename = "max-retries")]
    #[serde_inline_default(3)]
    pub max_retries: u16,
    /// Number of seconds before request is considered timed out
    #[serde(rename = "request-timeout")]
    #[serde_inline_default(20)]
    pub request_timeout: u64,
    /// Number of seconds before scraper script is considered timed out
    #[serde(rename = "script-timeout")]
    #[serde_inline_default(20)]
    pub script_timeout: u64,
}

impl Saveable for MasterConfig {}
