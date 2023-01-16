use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    BadArguments(String),
    #[error("decode search parameter failed")]
    DecodeSearchParameterFailed,
    #[error(transparent)]
    Customer(#[from] CustomerError),
    #[error(transparent)]
    Auth(#[from] AuthError),
    #[error(transparent)]
    ChangePassword(#[from] ChangePasswordError),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::BadArguments(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Customer(CustomerError::CustomerIsExist) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            AppError::DecodeSearchParameterFailed => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::JsonExtractorRejection(ref e) => match e {
                JsonRejection::JsonDataError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                JsonRejection::JsonSyntaxError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                JsonRejection::MissingJsonContentType(_) => {
                    (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            },
            AppError::Auth(AuthError::InvalidCredentials(ref e)) => {
                (StatusCode::UNAUTHORIZED, e.to_string())
            }
            AppError::Auth(AuthError::ExpiredCredentials) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AppError::Auth(AuthError::MissingBearer(_)) => {
                (StatusCode::UNAUTHORIZED, self.to_string())
            }
            AppError::ChangePassword(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(serde_json::json!({ "error_message": error_message }));

        (status, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CustomerError {
    #[error("customer is exist.")]
    CustomerIsExist,
}

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Missing bearer header")]
    MissingBearer(#[source] anyhow::Error),
    #[error("Invalid credentials")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error("expired credentials")]
    ExpiredCredentials,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ChangePasswordError {
    #[error("New password miss match")]
    NewPasswordMissingMatch,
    #[error("new password should be difference between old password")]
    NewPasswordMustBeDifferent,
}
