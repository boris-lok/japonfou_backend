use axum::http::Request;
use std::net::TcpListener;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestId, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use tracing::Level;
use uuid::Uuid;

use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{create_customer_handler, health_check};

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
}

#[derive(Clone)]
struct MakeRequestUuid;

impl MakeRequestId for MakeRequestUuid {
    fn make_request_id<B>(&mut self, _: &Request<B>) -> Option<RequestId> {
        let request_id = Uuid::new_v4().to_string();

        Some(RequestId::new(request_id.parse().unwrap()))
    }
}

pub async fn run(config: Settings, listener: TcpListener) -> hyper::Result<()> {
    let state = AppState {
        db_pool: get_database_connection(&config.database).await,
    };

    // build our application with a route
    let app = Router::new()
        .route("/api/v1/health_check", get(health_check))
        .route("/api/v1/customers", post(create_customer_handler))
        .layer(
            ServiceBuilder::new()
                .set_x_request_id(MakeRequestUuid)
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(
                            DefaultMakeSpan::new()
                                .include_headers(true)
                                .level(Level::INFO),
                        )
                        .on_response(DefaultOnResponse::new().include_headers(true)),
                ),
        )
        .with_state(state);

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
