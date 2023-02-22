use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;
use base64::Engine;

use crate::errors::{AppError, CustomerError};
use crate::repositories::CustomerRepo;
use crate::routes::customer::{CreateCustomerRequest, CreateCustomerResponse, NewCustomer};
use crate::routes::{
    Claims,  CustomerSearchParameters, DeleteCustomerRequest, ListCustomersRequest,
    ListCustomersResponse, UpdateCustomer, UpdateCustomerRequest,
};

#[tracing::instrument(name = "Create a new customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn create_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateCustomerRequest>, AppError>,
) -> Result<Json<CreateCustomerResponse>, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));
    let new_customer = NewCustomer::parse(payload)
        .await
        .map_err(AppError::BadArguments)?;

    let email = &new_customer.email;
    let phone = &new_customer.phone;

    if customer_repo
        .check_if_customer_is_exist(&None, email, phone)
        .await
        .context("Failed to execute a sql to check if customer is exist")?
    {
        return Err(CustomerError::CustomerIsExist)?;
    }

    let id = customer_repo
        .create(new_customer)
        .await
        .context("Failed to insert a new customer in the database")?;

    Ok(Json(CreateCustomerResponse { id }))
}

#[tracing::instrument(name = "Update a customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn update_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateCustomerRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));

    let update_customer = UpdateCustomer::parse(payload).map_err(AppError::BadArguments)?;

    let email = &update_customer.email;
    let phone = &update_customer.phone;

    if (email.is_some() || phone.is_some())
        && customer_repo
            .check_if_customer_is_exist(&Some(update_customer.id), email, phone)
            .await
            .context("Failed to execute a sql to check if customer is exist")?
    {
        return Err(CustomerError::CustomerIsExist)?;
    }

    let need_update = update_customer.name.is_some()
        || update_customer.email.is_some()
        || update_customer.phone.is_some()
        || update_customer.remark.is_some();

    if need_update {
        customer_repo
            .update(update_customer)
            .await
            .context("Failed to update a customer in the database")?;
    }

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Delete a customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn delete_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Send + Sync>>,
    WithRejection(Json(payload), _): WithRejection<Json<DeleteCustomerRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));
    customer_repo
        .delete(payload.id)
        .await
        .context("Failed to delete a customer in the database")?;

    Ok(StatusCode::OK)
}

#[tracing::instrument(name = "Get a customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn get_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Send + Sync>>,
    Path(params): Path<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));
    let customer_id = params.get("id").and_then(|e| e.parse::<i64>().ok());

    if customer_id.is_none() {
        return Err(AppError::BadArguments(
            "There is no customer id in the query string".to_string(),
        ))?;
    }

    let customer_json = customer_repo
        .get(customer_id.unwrap())
        .await
        .map(|e| e.map(Json))
        .context("Failed to get a customer from database")?;

    Ok(match customer_json {
        None => StatusCode::OK.into_response(),
        Some(json) => json.into_response(),
    })
}

#[tracing::instrument(name = "List customers", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn list_customers_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Send + Sync>>,
    Query(payload): Query<ListCustomersRequest>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));

    let search_parameter = if let Some(keyword) = &payload.keyword {
        // TODO: It should be written in pretty way.
        base64::engine::general_purpose::STANDARD
            .decode(keyword)
            .as_ref()
            .map(|e| serde_json::from_slice::<CustomerSearchParameters>(e))
            .map_err(|_| AppError::DecodeSearchParameterFailed)?
            .map_err(|_| AppError::DecodeSearchParameterFailed)?
    } else {
        CustomerSearchParameters::default()
    };

    let page = payload.page.unwrap_or(0);
    let page_size = payload.page_size.unwrap_or(20);

    let data = customer_repo
        .list(search_parameter, page, page_size)
        .await
        .context("Failed to get customers from database")?;

    let response = ListCustomersResponse { data };

    Ok(Json(response))
}
