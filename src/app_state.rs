use std::env;

use anyhow::Result;
use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

#[derive(Clone, Debug)]
pub struct AppState {
    db_pool: Pool<ConnectionManager<PgConnection>>,
}

impl AppState {
    pub fn init() -> Self {
        Self {
            db_pool: db_connection_pool(),
        }
    }

    pub fn db_connection(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>> {
        Ok(self.db_pool.get()?)
    }
}

fn db_connection_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Could not build connection pool")
}
