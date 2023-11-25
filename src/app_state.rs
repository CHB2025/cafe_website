use std::env;

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Clone)]
pub struct AppState {
    db_pool: Pool<Postgres>,
    session_key: Key,
}

impl AppState {
    pub async fn init() -> Self {
        Self {
            db_pool: db_connection_pool().await,
            session_key: Key::generate(), // Need to store this somehow
        }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.session_key.clone()
    }
}

async fn db_connection_pool() -> Pool<Postgres> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}
