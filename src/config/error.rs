use std::{error, fmt, io};

#[derive(Debug)]
pub struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl error::Error for ConfigError {}

impl From<toml::de::Error> for ConfigError {
    fn from(value: toml::de::Error) -> Self {
        Self(value.message().to_owned())
    }
}

impl From<io::Error> for ConfigError {
    fn from(value: io::Error) -> Self {
        Self(format!("Unable to read config file: {}", value.kind()))
    }
}

impl From<sqlx::Error> for ConfigError {
    fn from(value: sqlx::Error) -> Self {
        Self(format!("Unable to connect to the database: {}", value))
    }
}

impl From<sqlx::migrate::MigrateError> for ConfigError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self(format!("Unable to run database migrations: {}", value))
    }
}

impl From<lettre::transport::smtp::Error> for ConfigError {
    fn from(value: lettre::transport::smtp::Error) -> Self {
        Self(format!("Unable to set up mailer: {}", value))
    }
}
