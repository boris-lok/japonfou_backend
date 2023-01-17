use crate::errors::AppError;
use crate::repositories::ProductRepository;
use crate::routes::{
    Claims, CreateProductRequest, CreateProductResponse, DeleteProductRequest, ListProductsRequest,
    ListProductsResponse, NewProduct, ProductSearchParameters, UpdateProduct, UpdateProductRequest,
};
use anyhow::Context;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use base64::Engine;
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

#[tracing::instrument(name="delete a product", skip(product_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn delete_product_handler(
    claims: Claims,
    Extension(product_repo): Extension<Arc<dyn ProductRepository + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<DeleteProductRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));

    product_repo
        .delete(payload.id)
        .await
        .context("Failed to delete a product in the database")?;

    Ok(StatusCode::OK)
}

pub async fn update_product_handler(
    claims: Claims,
    Extension(product_repo): Extension<Arc<dyn ProductRepository + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateProductRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));

    let update_product = UpdateProduct::parse(payload)
        .await
        .map_err(AppError::BadArguments)?;

    let need_update = update_product.name.is_some()
        || update_product.currency.is_some()
        || update_product.price.is_some();

    if need_update {
        product_repo
            .update(update_product)
            .await
            .context("Failed to update a product in the database")?;
    }

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "List products", skip(product_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn list_products_handler(
    claims: Claims,
    Extension(product_repo): Extension<Arc<dyn ProductRepository + Sync + Send>>,
    Query(payload): Query<ListProductsRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));

    let search_parameter = if let Some(keyword) = &payload.keyword {
        // TODO: It should be written in pretty way.
        base64::engine::general_purpose::STANDARD
            .decode(keyword)
            .as_ref()
            .map(|e| serde_json::from_slice::<ProductSearchParameters>(e))
            .map_err(|_| AppError::DecodeSearchParameterFailed)?
            .map_err(|_| AppError::DecodeSearchParameterFailed)?
    } else {
        ProductSearchParameters::default()
    };

    let page = payload.page.unwrap_or(0);
    let page_size = payload.page_size.unwrap_or(20);

    let data = product_repo
        .list(search_parameter, page, page_size)
        .await
        .context("Failed to get customers from database")?;

    let response = ListProductsResponse { data };

    Ok(Json(response))
}
