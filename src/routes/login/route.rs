use std::sync::Arc;

use anyhow::Context;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::State, Extension, Json};
use axum_extra::extract::WithRejection;
use chrono::{Duration, Utc};
use secrecy::ExposeSecret;

use crate::authentication::{validate_credentials, UserRepo};
use crate::errors::AppError;
use crate::routes::login::domain::{Claims, LoginResponse};
use crate::routes::Login;
use crate::startup::AppState;

#[tracing::instrument(skip(app_state, payload, user_repo))]
pub async fn login(
    State(app_state): State<AppState>,
    Extension(user_repo): Extension<Arc<dyn UserRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<Login>, AppError>,
) -> Result<Response, AppError> {
    let credentials = payload.into();

    match validate_credentials(credentials, user_repo).await {
        Ok(user_id) => {
            let exp = Utc::now() + Duration::days(15);
            let claims = Claims {
                sub: user_id.to_string(),
                exp: exp.timestamp() as usize,
            };

            let token = jsonwebtoken::encode(
                &jsonwebtoken::Header::default(),
                &claims,
                &jsonwebtoken::EncodingKey::from_secret(
                    app_state.jwt_secret_key.expose_secret().as_bytes(),
                ),
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
