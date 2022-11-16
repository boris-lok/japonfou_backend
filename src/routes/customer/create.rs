use crate::routes::customer::{CreateCustomer, NewCustomer};
use crate::startup::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use sqlx::PgPool;

pub async fn create_customer_handler(
    State(app_state): State<AppState>,
    Json(payload): Json<CreateCustomer>,
) -> StatusCode {
    let conn = app_state.db_pool;

    let new_customer = payload.try_into().unwrap();

    let _ = create_customer(&conn, new_customer).await;

    //TODO: return new customer id
    StatusCode::OK
}

async fn create_customer(conn: &PgPool, customer: NewCustomer) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO customers (id, name, email, phone, remark, created_at)
    VALUES ($1, $2, $3, $4, null, now());
    "#,
        //TODO: generate a customer id
        1,
        customer.name,
        customer.email.map(|e| e.0),
        customer.phone.map(|e| e.0),
    )
    .execute(conn)
    .await
    .unwrap();

    Ok(())
}
