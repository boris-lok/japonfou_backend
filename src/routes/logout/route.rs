use crate::errors::AppError;
use crate::routes::Claims;
use crate::startup::AppState;
use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use redis::Commands;

pub async fn logout(
    claims: Claims,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = claims.sub;

    let mut redis_connection = app_state
        .redis_client
        .get_connection()
        .context("Failed to connect the redis")?;

    redis_connection
        .del(user_id.as_str())
        .context("Failed to clear a login session")
        .map_err(AppError::UnexpectedError)?;

    Ok((StatusCode::OK, "".to_string()).into_response())
}
