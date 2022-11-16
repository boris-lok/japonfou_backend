use std::net::TcpListener;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{create_customer_handler, health_check};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

pub async fn run(config: Settings, listener: TcpListener) -> hyper::Result<()> {
    let state = AppState {
        db_pool: get_database_connection(&config.database).await,
    };

    // build our application with a route
    let app = Router::with_state(state)
        .route("/api/v1/health_check", get(health_check))
        .route("/api/v1/customers", post(create_customer_handler));

    axum::Server::from_tcp(listener)
        .expect("Can't bind tcp listener")
        .serve(app.into_make_service())
        .await
}

pub async fn get_database_connection(config: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(config.with_db())
}
