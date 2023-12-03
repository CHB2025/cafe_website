use std::path::{Path, PathBuf};

use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

pub use error::ConfigError;

mod error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub website: Website,
    pub database: Database,
    pub ssl: Option<Ssl>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Website {
    pub base_url: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_port() -> u16 {
    3000
}

#[derive(Debug, Deserialize, Clone)]
pub struct Database {
    pub database_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Ssl {
    pub cert: PathBuf,
    pub key: PathBuf,
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut cfg_file = File::open(path).await?;
        let mut cfg_string = String::new();
        cfg_file.read_to_string(&mut cfg_string).await?;
        Ok(toml::from_str::<Config>(&cfg_string)?)
    }
}
