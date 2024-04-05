use std::path::{Path, PathBuf};

use lettre::{
    transport::smtp::authentication::Credentials, Address, AsyncSmtpTransport, Tokio1Executor,
};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

pub use error::ConfigError;

mod error;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub website: Website,
    pub database: Database,
    pub ssl: Option<Ssl>,
    pub email: Option<Email>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Website {
    pub base_url: String,
    #[serde(default = "default_port")]
    pub port: u16,
    pub session_key: Option<String>,
    pub otel_endpoint: Option<String>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Email {
    server: String,
    address: Address,
    password: String,
}

impl Email {
    pub fn mailer(
        &self,
    ) -> Result<AsyncSmtpTransport<Tokio1Executor>, lettre::transport::smtp::Error> {
        let builder = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.server)?.credentials(
            Credentials::new(self.address.user().to_owned(), self.password.clone()),
        );
        Ok(builder.build())
    }

    pub fn address(&self) -> Address {
        self.address.clone()
    }
}

impl Config {
    pub async fn load(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let mut cfg_file = File::open(path).await?;
        let mut cfg_string = String::new();
        cfg_file.read_to_string(&mut cfg_string).await?;
        Ok(toml::from_str::<Config>(&cfg_string)?)
    }

    pub fn url(&self) -> String {
        let mut url = String::new();

        let add_port = if self.ssl.is_some() {
            url += "https://";
            self.website.port != 443
        } else {
            url += "http://";
            self.website.port != 80
        };

        url += &self.website.base_url;
        if add_port {
            url.push(':');
            url += &format!("{}", self.website.port);
        }
        url
    }
}
