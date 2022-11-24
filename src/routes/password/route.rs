use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use secrecy::Secret;

use crate::authentication::{validate_credentials, Credentials};
use crate::errors::{AppError, ChangePasswordError};
use crate::repositories::UserRepo;
use crate::routes::password::domain::ChangePasswordRequest;
use crate::routes::Claims;

pub async fn change_password(
    claims: Claims,
    Extension(user_repo): Extension<Arc<dyn UserRepo + Send + Sync>>,
    WithRejection(Json(payload), _): WithRejection<Json<ChangePasswordRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    if payload.new_password != payload.new_password_check {
        return Err(ChangePasswordError::NewPasswordMissingMatch)?;
    }

    if payload.current_password == payload.new_password {
        return Err(ChangePasswordError::NewPasswordMustBeDifferent)?;
    }

    let user_id = claims.sub;
    let username = user_repo.get_username(user_id.as_str()).await?;
    let credentials = Credentials {
        username,
        password: Secret::new(payload.current_password),
    };

    let user_id = validate_credentials(credentials, user_repo.clone()).await?;

    // TODO: If it return false, it's mean password doesn't change.
    let _ = crate::authentication::change_password(
        user_id,
        Secret::new(payload.new_password),
        user_repo,
    )
    .await?;

    Ok((StatusCode::OK, "".to_string()).into_response())
}
