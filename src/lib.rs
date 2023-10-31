//! Automatic web scraper and RSS generator library
//!
//! ## Quickstart
//!
//! Get started by creating an event loop.
//!
//! ```
//! #[tokio::main]
//! async fn main() {
//!     // initialise values
//!     scrapyard::init(None).await;
//!     
//!     // load feeds from a config file
//!     // or create a default config file
//!     let feeds_path = PathBuf::from("feeds.json");
//!     let feeds = Feeds::load_json(&feeds_path).await
//!     .unwrap_or_else(|| {
//!         let default = Feeds::default();
//!         default.save_json();
//!         default
//!     });
//!     
//!     // start the event loop, this will not block
//!     feeds.start_loop().await;
//!     
//!     // as long as the program is running
//!     // the feeds will be updated regularly
//!     HttpServer::new(|| {})
//!         .bind(("0.0.0.0", 8080)).unwrap()
//!         .run().await.unwrap();
//! }
//! ```
//!
//! ## Configuration
//!
//! By default, config files can be found in `~/.config/scrapyard` (Linux),
//! `/Users/[Username]/Library/Application/Support/scrapyard` (Mac) or
//! `C:\Users\[Username]\AppData\Roaming\scrapyard` (Windows).
//!
//! To change the config directory location, specify the path:
//!
//! ```
//! let config_path = PathBuf::from("/my/special/path");
//! scrapyard::init(Some(config_path)).await;
//! ```
//!
//! Here are all the options in the main configuration file `scrapyard.json`.
//!
//! ```json
//! {
//!     "store": String, // i.e. /home/user/.local/share/scrapyard/
//!     "max-retries": Number, // number of retries before giving up
//!     "request-timeout": Number, // number of seconds before giving up request
//!     "script-timeout": Number, // number of seconds before giving up on the extractor script
//! }
//! ```
//!
//! ### Adding feeds
//!
//! To add feeds, edit `feeds.json`.
//!
//! ```json
//! {
//!     "origin": String, // origin of the feed
//!     "label": String, // text id of the feed
//!     "max-length": Number, // maximum number of items allowed in the feed
//!     "fetch-length": Number, // maximum number of items allowed to be fetched each interval
//!     "interval": Number, // number of seconds between fetching,
//!     "idle-limit": Number, // number of seconds without requests to that feed before fetching stops
//!     "sort": Boolean, // to sort by publish date or not
//!     "extractor": [String], // all command line args to run the extractor, i.e. ["node", "extractor.js"]
//!
//!     "title": String, // displayed feed title
//!     "link": String, // displayed feed source url
//!     "description": String, // displayed feed description
//! }
//! ```
//!
//! You can also include additional fields in [PseudoChannel](https://docs.rs/scrapyard/latest/struct.PseudoChannel.html) to
//! overwrite default empty values.
//!
//! ### Getting feeds
//!
//! Referencing functions under [FeedOption](https://docs.rs/scrapyard/latest/struct.FeedOption.html), there are 2 types of fetch functions.
//!
//! **Force fetching** always request for a new copy of the feed, ignoring the fetch interval. **Lazy
//! fetching** only fetched a new copy when the existing copy is out of date. This is particularly
//! relevant when used without the auto-fetch loop.
//!
//! ### Extractor scripts
//!
//! The extractor scripts must accept 1 command line argument (in JSON) and prints out 1 JSON
//! response to stdout, normal `console.log()` in JS will do. You get the idea.
//!
//! Command line input:
//!
//! ```json
//! {
//!     "url": String, // origin of the info fetched
//!     "webstr": String, // response from the url
//!     "preexists": [ PseudoItem ], // don't output these again to avoid duplication
//!     "lengthLeft": Number // maximum length before the fetch-length quota is met
//! }
//! ```
//!
//! Expected output:
//!
//! ```json
//! {
//!     "items": [PseudoItem], // list of items extracted
//!     "continuation": String? // optionally continue fetching in the next url
//! }
//! ```

mod bindings;
pub use bindings::*;
mod options;
pub use options::*;
mod traits;
pub use traits::*;
mod locks;
pub use values::*;
mod values;
pub use errors::*;
mod errors;
