use std::sync::Arc;

use anyhow::Context;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use chrono::{Duration, Utc};
use redis::Commands;

use crate::authentication::validate_credentials;
use crate::errors::AppError;
use crate::repositories::UserRepo;
use crate::routes::login::domain::{Claims, LoginResponse};
use crate::routes::Login;
use crate::startup::AppState;
use crate::utils::JWT_SECRET_KEY_INSTANCE;

#[tracing::instrument(skip(app_state, payload, user_repo))]
pub async fn login(
    State(app_state): State<AppState>,
    Extension(user_repo): Extension<Arc<dyn UserRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<Login>, AppError>,
) -> Result<Response, AppError> {
    let credentials = payload.into();

    match validate_credentials(credentials, user_repo).await {
        Ok(user_id) => {
            let user_id = user_id.to_string();
            let exp = Utc::now() + Duration::days(15);
            let mut redis_connection = app_state
                .redis_client
                .get_connection()
                .context("Failed to connect the redis")?;

            redis_connection
                .set(&user_id, exp.timestamp() as usize)
                .context("Failed to set a user to session storage.")?;

            let secret_key = JWT_SECRET_KEY_INSTANCE
                .get()
                .context("Failed to get jwt encoding key")?;
            let claims = Claims {
                sub: user_id,
                exp: exp.timestamp() as usize,
            };

            let token = jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                &claims,
                &secret_key.encoding,
            )
            .context("Failed to encode a json web token")?;

            let response = LoginResponse { token };

            Ok(Json(response).into_response())
        }
        Err(_e) => {
            let body = Json(serde_json::json!({ "error_message":  "login failed"}));
            let status_code = StatusCode::UNAUTHORIZED;
            Ok((status_code, body).into_response())
        }
    }
}
