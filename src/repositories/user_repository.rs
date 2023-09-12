use anyhow::Context;
use async_trait::async_trait;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use secrecy::{ExposeSecret, Secret};
use sqlx::Row;

use crate::errors::{AppError, AuthError};
use crate::utils::PostgresSession;

#[derive(sea_query::Iden)]
pub(crate) enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
}

#[async_trait]
pub trait UserRepo {
    async fn get_store_credentials(
        &self,
        username: &str,
    ) -> Result<Option<(uuid::Uuid, Secret<String>)>, anyhow::Error>;

    async fn get_username(&self, user_id: &str) -> Result<String, anyhow::Error>;

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

        let query = Query::select()
            .columns([Users::Id, Users::PasswordHash])
            .from(Users::Table)
            .and_where(Expr::col((Users::Table, Users::Username)).eq(username))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&query)
            .fetch_optional(conn.as_mut())
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

    async fn get_username(&self, user_id: &str) -> Result<String, anyhow::Error> {
        let mut conn = self.session.get_session().await;

        let query = Query::select()
            .column(Users::Username)
            .from(Users::Table)
            .and_where(Expr::col((Users::Table, Users::Id)).eq(user_id))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&query)
            .fetch_one(conn.as_mut())
            .await
            .context("Failed to perform a query to retrieve a username")
            .map_err(AppError::UnexpectedError)
            .map(|e| e.get::<String, usize>(0))?;

        Ok(res)
    }

    async fn change_password(
        &self,
        id: uuid::Uuid,
        password: Secret<String>,
    ) -> Result<bool, anyhow::Error> {
        let mut conn = self.session.get_session().await;

        let query = Query::update()
            .table(Users::Table)
            .values([(
                Users::PasswordHash,
                password.expose_secret().to_string().into(),
            )])
            .and_where(Expr::col(Users::Id).eq(id.to_string()))
            .to_string(PostgresQueryBuilder);

        let res = sqlx::query(&query)
            .execute(conn.as_mut())
            .await
            .map(|e| e.rows_affected() == 1)?;

        Ok(res)
    }
}
