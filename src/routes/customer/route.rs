use std::sync::Arc;

use anyhow::Context;
use axum::{Extension, Json};
use axum_extra::extract::WithRejection;

use crate::errors::{AppError, CustomerError};
use crate::repositories::CustomerRepo;
use crate::routes::customer::{CreateCustomer, NewCustomer, NewCustomerResponse};
use crate::routes::Claims;

#[tracing::instrument(name = "Create a new customer", skip(customer_repo))]
pub async fn create_customer_handler(
    _: Claims,
    Extension(customer_repo): Extension<Arc<dyn CustomerRepo + Sync + Send>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateCustomer>, AppError>,
) -> Result<Json<NewCustomerResponse>, AppError> {
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
