use std::sync::Arc;

use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::OnceCell;
use regex::Regex;
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

pub struct JwtKey {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl JwtKey {
    pub fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

pub static JWT_SECRET_KEY_INSTANCE: OnceCell<JwtKey> = OnceCell::new();

pub fn get_phone_number_regex() -> &'static Regex {
    static INSTANCE: OnceCell<Regex> = OnceCell::new();
    INSTANCE.get_or_init(|| {
        Regex::new(r#"^[\\+]?[(]?[0-9]{3}[)]?[-\s\\.]?[0-9]{3}[-\s\\.]?[0-9]{4,6}$"#).unwrap()
    })
}
