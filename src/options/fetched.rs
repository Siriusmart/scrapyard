use serde::{Deserialize, Serialize};
use serde_default::DefaultFromSerde;
use serde_inline_default::serde_inline_default;

use crate::traits::Saveable;

/// Feed metadata
#[serde_inline_default]
#[derive(Serialize, Deserialize, DefaultFromSerde)]
pub struct FetchedMeta {
    /// Last fetched timestamp
    #[serde(rename = "last-fetch")]
    #[serde_inline_default(0)]
    pub last_fetch: u64,
    /// Last user requested fetch timestamp
    #[serde(rename = "last-requested")]
    #[serde_inline_default(chrono::Utc::now().timestamp() as u64)]
    pub last_requested: u64,
}

impl Saveable for FetchedMeta {}

impl FetchedMeta {
    /// Update last fetched to now
    pub fn fetched(&mut self) {
        self.last_fetch = chrono::Utc::now().timestamp() as u64;
    }

    /// Update last requested to now
    pub fn requested(&mut self) {
        self.last_requested = chrono::Utc::now().timestamp() as u64;
    }
}
