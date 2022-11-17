use crate::routes::customer::{
    customer_id_generator, CreateCustomer, NewCustomer, NewCustomerResponse,
};
use crate::startup::AppState;
use anyhow::Context;
use axum::extract::State;

use crate::errors::{AppError, CustomerError};
use axum::Json;
use sqlx::PgPool;

pub async fn create_customer_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateCustomer>,
) -> Result<Json<NewCustomerResponse>, AppError> {
    let conn = app_state.db_pool;

    let new_customer = payload
        .try_into()
        .map_err(|e| CustomerError::BadArguments(e))?;

    let id = create_customer(&conn, new_customer)
        .await
        .context("Failed to insert a new customer in the database")?;

    Ok(Json(NewCustomerResponse(id)))
}

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
