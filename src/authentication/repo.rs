use std::ops::DerefMut;

use anyhow::Context;
use async_trait::async_trait;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use secrecy::{ExposeSecret, Secret};
use sqlx::Row;

use crate::authentication::domain::Users;
use crate::errors::AuthError;
use crate::utils::PostgresSession;

#[async_trait]
pub trait UserRepo {
    async fn get_store_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error>;

    async fn change_password(
        &self,
        id: uuid::Uuid,
        password: Secret<String>,
    ) -> Result<bool, anyhow::Error>;
}

#[derive(Debug)]
pub struct PostgresUserRepoImpl {
    session: PostgresSession,
}

impl PostgresUserRepoImpl {
    pub fn new(session: PostgresSession) -> Self {
        Self { session }
    }
}

#[async_trait]
impl UserRepo for PostgresUserRepoImpl {
    async fn get_store_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error> {
        let mut conn = self.session.get_session().await;

        let sql = Query::select()
            .columns([Users::Id, Users::PasswordHash])
            .from(Users::Table)
            .and_where(Expr::tbl(Users::Table, Users::Username).eq(username))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&sql)
            .fetch_optional(conn.deref_mut())
            .await
            .context("Failed to perform a query to retrieve stored credentials.")
            .map_err(AuthError::UnexpectedError)?
            .map(|e| {
                let id = e.get::<uuid::Uuid, usize>(0);
                let password = Secret::new(e.get::<String, usize>(1));
                (id, password)
            });

        Ok(res)
    }

    async fn change_password(
        &self,
        id: uuid::Uuid,
        password: Secret<String>,
    ) -> Result<bool, anyhow::Error> {
        let mut conn = self.session.get_session().await;

        let sql = Query::update()
            .table(Users::Table)
            .values([(
                Users::PasswordHash,
                password.expose_secret().to_string().into(),
            )])
            .and_where(Expr::col(Users::Id).eq(id.to_string()))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&sql)
            .execute(conn.deref_mut())
            .await
            .map(|e| e.rows_affected() == 1)?;

        Ok(res)
    }
}
