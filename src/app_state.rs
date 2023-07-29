use std::env;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Clone, Debug)]
pub struct AppState {
    db_pool: Pool<Postgres>,
}

impl AppState {
    pub async fn init() -> Self {
        Self {
            db_pool: db_connection_pool().await,
        }
    }

    pub fn pool(&self) -> &Pool<Postgres> {
        &self.db_pool
    }
}

async fn db_connection_pool() -> Pool<Postgres> {
    dotenvy::dotenv().ok();
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
