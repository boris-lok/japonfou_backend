use std::sync::Arc;

use anyhow::Context;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;

use crate::errors::{AppError, CustomerError};
use crate::repositories::CustomerRepo;
use crate::routes::customer::{CreateCustomerRequest, NewCustomer, NewCustomerResponse};
use crate::routes::{Claims, UpdateCustomer, UpdateCustomerRequest};

#[tracing::instrument(name = "Create a new customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn create_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateCustomerRequest>, AppError>,
) -> Result<Json<NewCustomerResponse>, AppError> {
    tracing::Span::current().record("user_id", &tracing::field::display(&claims.sub));
    let new_customer = NewCustomer::parse(payload)
        .await
        .map_err(CustomerError::BadArguments)?;

    let email = &new_customer.email;
    let phone = &new_customer.phone;

    if customer_repo
        .check_if_customer_is_exist(email, phone)
        .await
        .context("Failed to execute a sql to check if customer is exist")?
    {
        return Err(CustomerError::CustomerIsExist)?;
    }

    let id = customer_repo
        .create(new_customer)
        .await
        .context("Failed to insert a new customer in the database")?;

    Ok(Json(NewCustomerResponse { id }))
}

#[tracing::instrument(name = "Update a customer", skip(customer_repo, claims), fields(user_id=tracing::field::Empty))]
pub async fn update_customer_handler(
    claims: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateCustomerRequest>, AppError>,
) -> Result<impl IntoResponse, AppError> {
    tracing::Span::current().record("user_id", tracing::field::display(&claims.sub));

    let update_customer = UpdateCustomer::parse(payload).map_err(CustomerError::BadArguments)?;

    let email = &update_customer.email;
    let phone = &update_customer.phone;

    if (email.is_some() || phone.is_some())
        && customer_repo
            .check_if_customer_is_exist(email, phone)
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
