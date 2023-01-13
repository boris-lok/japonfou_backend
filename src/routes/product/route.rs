use crate::errors::AppError;
use crate::repositories::ProductRepository;
use crate::routes::{Claims, CreateProductRequest, CreateProductResponse, NewProduct};
use anyhow::Context;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
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
