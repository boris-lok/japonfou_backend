use crate::routes::customer::{
    customer_id_generator, CreateCustomer, NewCustomer, NewCustomerResponse,
};
use crate::startup::AppState;
use anyhow::Context;
use axum::extract::State;
use axum_extra::extract::WithRejection;

use crate::errors::{AppError, CustomerError};
use crate::routes::{Customers, ValidEmail, ValidPhone};
use axum::Json;
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sqlx::{PgPool, Row};

#[tracing::instrument(name = "Create a new customer", skip(app_state))]
pub async fn create_customer_handler(
    State(app_state): State<AppState>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateCustomer>, AppError>,
) -> Result<Json<NewCustomerResponse>, AppError> {
    let conn = app_state.db_pool;

    let new_customer: NewCustomer = payload.try_into().map_err(CustomerError::BadArguments)?;

    let email = new_customer.email.clone();
    let phone = new_customer.phone.clone();

    if check_if_customer_is_exist(&conn, email, phone)
        .await
        .context("Failed to execute a sql to check if customer is exist")?
    {
        return Err(CustomerError::CustomerIsExist)?;
    }

    let id = create_customer(&conn, new_customer)
        .await
        .context("Failed to insert a new customer in the database")?;

    Ok(Json(NewCustomerResponse(id)))
}

#[tracing::instrument(name = "Save a customer to database", skip(conn, customer))]
async fn create_customer(conn: &PgPool, customer: NewCustomer) -> Result<i64, sqlx::Error> {
    let id = async {
        let generator = customer_id_generator();
        let mut generator = generator.lock().unwrap();
        generator.real_time_generate()
    }
    .await;

    sqlx::query!(
        r#"
    INSERT INTO customers (id, name, email, phone, remark, created_at)
    VALUES ($1, $2, $3, $4, null, now());
    "#,
        id,
        customer.name,
        customer.email.map(|e| e.0),
        customer.phone.map(|e| e.0),
    )
    .execute(conn)
    .await?;

    Ok(id)
}

#[tracing::instrument(
    name = "Check a customer if is exist by email or phone",
    skip(conn, email, phone)
)]
async fn check_if_customer_is_exist(
    conn: &PgPool,
    email: Option<ValidEmail>,
    phone: Option<ValidPhone>,
) -> Result<bool, sqlx::Error> {
    let sql = Query::select()
        .column(Customers::Id)
        .from(Customers::Table)
        .and_where_option(email.map(|e| Expr::tbl(Customers::Table, Customers::Email).eq(e.0)))
        .and_where_option(phone.map(|e| Expr::tbl(Customers::Table, Customers::Phone).eq(e.0)))
        .to_string(PostgresQueryBuilder);

    sqlx::query(&sql)
        .fetch_optional(conn)
        .await
        .map(|row| row.map_or_else(|| false, |e| !e.is_empty()))
}
