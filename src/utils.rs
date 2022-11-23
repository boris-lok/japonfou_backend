use std::sync::Arc;

use sqlx::pool::PoolConnection;
use sqlx::{PgPool, Postgres};
use tokio::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub struct PostgresSession {
    session: Arc<Mutex<PoolConnection<Postgres>>>,
}

impl PostgresSession {
    pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
        let session = pool.acquire().await?;
        Ok(Self {
            session: Arc::new(Mutex::new(session)),
        })
    }

    pub async fn get_session(&self) -> MutexGuard<PoolConnection<Postgres>> {
        self.session.lock().await
    }
}

pub async fn get_postgres_session(pool: PgPool) -> Result<PoolConnection<Postgres>, sqlx::Error> {
    pool.acquire().await
}
