use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error(transparent)]
    Customer(#[from] CustomerError),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Customer(CustomerError::BadArguments(msg)) => (StatusCode::BAD_REQUEST, msg),
            AppError::UnexpectedError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            AppError::JsonExtractorRejection(ref e) => match e {
                JsonRejection::JsonDataError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                JsonRejection::JsonSyntaxError(_) => (StatusCode::BAD_REQUEST, self.to_string()),
                JsonRejection::MissingJsonContentType(_) => {
                    (StatusCode::UNSUPPORTED_MEDIA_TYPE, self.to_string())
                }
                _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            },
        };

        let body = Json(serde_json::json!({ "error_message": error_message }));

        (status, body).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CustomerError {
    #[error("{0}")]
    BadArguments(String),
}
