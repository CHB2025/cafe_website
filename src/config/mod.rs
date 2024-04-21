use std::{path::Path, sync::OnceLock};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use axum_server::tls_rustls::RustlsConfig;
use lettre::{Address, AsyncSmtpTransport, Tokio1Executor};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use tokio::{fs::File, io::AsyncReadExt};

pub use error::ConfigError;

use self::text::{TextConfig, Website};

mod error;
mod text;

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Clone)]
pub struct Config {
    db_pool: Pool<Postgres>,
    mailer: Option<AsyncSmtpTransport<Tokio1Executor>>,
    address: Option<Address>,
    session_key: Key,
    tls_config: Option<RustlsConfig>,

    pub website: Website,
}

pub fn config() -> &'static Config {
    CONFIG.get().expect("Config is not configured")
}

impl Config {
    pub async fn init(path: impl AsRef<Path>) -> Result<(), ConfigError> {
        let mut cfg_file = File::open(path).await?;
        let mut cfg_string = String::new();
        cfg_file.read_to_string(&mut cfg_string).await?;
        let text = toml::from_str::<TextConfig>(&cfg_string)?;

        // Initialize db pool
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&text.database.database_url)
            .await?;

        sqlx::migrate!().run(&pool).await?;

        let session_key = text
            .website
            .session_key
            .as_ref()
            .and_then(|k| Key::try_from(k.as_bytes()).ok())
            .unwrap_or_else(Key::generate);

        let tls_config = if let Some(cfg) = text.ssl {
            Some(RustlsConfig::from_pem_file(cfg.cert.clone(), cfg.key.clone()).await?)
        } else {
            None
        };

        let config = Config {
            db_pool: pool,
            mailer: text
                .email
                .as_ref()
                .map_or(Ok(None), |em| em.mailer().map(Some))?,
            address: text.email.map(|em| em.address()),
            session_key,
            tls_config,

            website: text.website,
        };
        _ = CONFIG.set(config);
        Ok(())
    }

    pub fn url(&self) -> String {
        let mut url = String::new();

        let add_port = if self.tls_config.is_some() {
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

    pub fn mailer(&self) -> Option<&AsyncSmtpTransport<Tokio1Executor>> {
        self.mailer.as_ref()
    }
    pub fn mailing_address(&self) -> Option<&Address> {
        self.address.as_ref()
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }

    pub fn tls_config(&self) -> Option<&RustlsConfig> {
        self.tls_config.as_ref()
    }
}

impl FromRef<Config> for Key {
    fn from_ref(state: &Config) -> Self {
        state.session_key.clone()
    }
}
