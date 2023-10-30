use std::{error::Error, path::Path};

use rss::Channel;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{fs, io::AsyncWriteExt};

/// An convenience trait to quickly load and save files
#[async_trait::async_trait]
pub trait Saveable
where
    Self: Sized + Serialize + DeserializeOwned,
{
    /// Load string content from a file
    async fn load_string(path: &Path) -> Result<String, Box<dyn Error>> {
        Ok(fs::read_to_string(path).await?)
    }

    /// Loads and deserializes json content from a file
    async fn load_json(path: &Path) -> Result<Self, Box<dyn Error>> {
        Ok(serde_json::from_str(
            Self::load_string(path).await?.as_str(),
        )?)
    }

    /// Loads and deserializes rss compatible xml from a file
    async fn load_rss(path: &Path) -> Result<Channel, Box<dyn Error>> {
        Ok(Channel::read_from(
            Self::load_string(path).await?.as_bytes(),
        )?)
    }

    /// Saves a string to file
    async fn save_string(path: &Path, s: String) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }

        Ok(fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(path)
            .await?
            .write_all(s.as_bytes())
            .await?)
    }

    /// Serializes and save as json
    async fn save_json(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        Ok(Self::save_string(path, serde_json::to_string(self)?).await?)
    }

    /// Serializes and save as pretty json
    async fn save_json_pretty(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        Ok(Self::save_string(path, serde_json::to_string_pretty(self)?).await?)
    }

    /// Serializes and save as rss compatible xml
    async fn save_rss(self, path: &Path) -> Result<(), Box<dyn Error>> {
        Ok(Self::save_string(path, self.get_rss().to_string()).await?)
    }

    /// Get rss content from self (if relevant)
    fn get_rss(self) -> Channel {
        panic!("this is not an rss saveable")
    }
}
