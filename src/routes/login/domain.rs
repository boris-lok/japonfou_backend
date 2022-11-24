use anyhow::Context;
use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::headers::authorization::Bearer;
use axum::headers::Authorization;
use axum::http::request::Parts;
use axum::{RequestPartsExt, TypedHeader};
use chrono::Utc;
use redis::Commands;

use crate::errors::{AppError, AuthError};
use crate::startup::AppState;
use crate::utils::JWT_SECRET_KEY_INSTANCE;

#[derive(serde::Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[async_trait]
impl<S> FromRequestParts<S> for Claims
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let now = Utc::now().timestamp() as usize;
        let state = AppState::from_ref(state);

        let mut redis_connection = state
            .redis_client
            .get_connection()
            .context("Failed to connect redis")
            .map_err(AppError::UnexpectedError)?;

        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .context("Missing bearer header")
            .map_err(AuthError::MissingBearer)?;

        let decoding_key = JWT_SECRET_KEY_INSTANCE
            .get()
            .context("Failed to get the jwt decoding key")
            .map_err(AuthError::UnexpectedError)?;

        let token_data = jsonwebtoken::decode::<Claims>(
            bearer.token(),
            &decoding_key.decoding,
            &jsonwebtoken::Validation::default(),
        )
        .context("Failed to decode jwt")
        .map_err(AuthError::InvalidCredentials)?;

        let exp_from_session: Option<usize> = redis_connection
            .get(&token_data.claims.sub)
            .context("Failed to get expired date by token")
            .map_err(AuthError::InvalidCredentials)?;

        if exp_from_session.is_none() || exp_from_session.unwrap() < now {
            return Err(AuthError::ExpiredCredentials)?;
        }

        Ok(token_data.claims)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
