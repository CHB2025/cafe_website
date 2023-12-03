use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    db_pool: Pool<Postgres>,
    session_key: Key,
    config: Config,
}

impl AppState {
    pub async fn init(config: Config) -> Self {
        Self {
            db_pool: db_connection_pool(&config.database.database_url).await,
            session_key: Key::generate(), // Need to store this somehow
            config,
        }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }

    pub fn config(&self) -> &Config {
        &self.config
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.session_key.clone()
    }
}

async fn db_connection_pool(url: &str) -> Pool<Postgres> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(url)
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
