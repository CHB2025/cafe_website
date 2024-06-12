use std::path::PathBuf;

use lettre::{
    transport::smtp::authentication::Credentials, Address, AsyncSmtpTransport, Tokio1Executor,
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct TextConfig {
    pub website: Website,
    pub admin: Admin,
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
pub struct Admin {
    pub name: String,
    pub email: String,
    pub phone: String,
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
