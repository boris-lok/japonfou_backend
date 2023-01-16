use crate::errors::AppError;
use crate::repositories::ProductRepository;
use crate::routes::{Claims, CreateProductRequest, CreateProductResponse, NewProduct, ProductJson};
use anyhow::Context;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use std::collections::HashMap;
use std::sync::Arc;

#[tracing::instrument(name="create a new product", skip(product_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn create_product_handler(
    claims: Claims,
    Extension(product_repo): Extension<Arc<dyn ProductRepository + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateProductRequest>, AppError>,
) -> Result<Json<CreateProductResponse>, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));
    let new_product = NewProduct::parse(payload)
        .await
        .map_err(AppError::BadArguments)?;

    // TODO: need some checking?
    let id = product_repo
        .create(new_product)
        .await
        .context("Failed to insert a new product in the database")?;

    Ok(Json(CreateProductResponse { id }))
}

#[tracing::instrument(name="get a product", skip(product_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn get_product_handler(
    claims: Claims,
    Extension(product_repo): Extension<Arc<dyn ProductRepository + Sync + Send>>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));
    let product_id = params.get("id").and_then(|e| e.parse::<i64>().ok());

    if product_id.is_none() {
        return Err(AppError::BadArguments(
            "There is no product id in the query string".to_string(),
        ));
    }

    let product_json = product_repo
        .get(product_id.unwrap())
        .await
        .map(|e| e.map(Json))
        .context("Failed to get a product from database")?;

    Ok(match product_json {
        None => StatusCode::OK.into_response(),
        Some(json) => json.into_response(),
    })
}
