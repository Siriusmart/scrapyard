use serde::{Deserialize, Serialize};

use crate::PseudoItem;

/// Json arguments passed to the scraper script
#[derive(Serialize, Deserialize)]
pub struct ItemizerArg {
    /// URL of origin
    pub url: String,
    /// String returned from origin
    pub webstr: String,
    /// Items to ignore when extracting
    pub preexists: Vec<PseudoItem>,
    /// Items left to scrap
    #[serde(rename = "lengthLeft")]
    pub length_left: u32,
}

/// Json response expected from the scraper script
#[derive(Serialize, Deserialize)]
pub struct ItemizerRes {
    /// Next URL to fetch (in case length wasn't enough)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
    /// Parsed items
    pub items: Vec<PseudoItem>,
}
