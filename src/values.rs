use std::path::Path;
use std::process;
use std::sync::OnceLock;

use crate::locks::Locks;
use crate::options::MasterConfig;
use crate::traits::Saveable;

/// Self identifier of the crate: `scrapyard X.Y.Z (git 123abcd)`
pub static IDENT: OnceLock<String> = OnceLock::new();
/// Holds global master config
pub static MASTER: OnceLock<MasterConfig> = OnceLock::new();
/// Fetch locks to avoid duplicated fetching
pub static mut LOCKS: Locks = Locks::new();

/// Initialise all OnceLocks
pub async fn init(config: Option<&Path>) {
    IDENT
        .set(format!(
            "{} {} (git {})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ))
        .unwrap();

    let path = config
        .unwrap_or(&dirs::config_dir().unwrap().join(env!("CARGO_PKG_NAME")))
        .join("scrapyard.json");
    let master = if path.exists() {
        match MasterConfig::load_json(&path).await {
            Ok(config) => config,
            Err(e) => {
                println!("Could not load {}\nError: {e}", path.to_string_lossy());
                process::exit(0);
            }
        }
    } else {
        let default = MasterConfig::default();
        default.save_json_pretty(&path).await.unwrap();
        default
    };

    MASTER.set(master).unwrap();
}
