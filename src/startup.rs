use std::net::TcpListener;
use std::sync::Arc;

use axum::http::Request;
use axum::routing::get;
use axum::routing::post;
use axum::{Extension, Router};
use secrecy::Secret;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestId, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use tracing::Level;
use uuid::Uuid;

use crate::configuration::{DatabaseSettings, Settings};
use crate::repositories::{CustomerRepo, PostgresCustomerRepoImpl, PostgresUserRepoImpl, UserRepo};
use crate::routes::{change_password, create_customer_handler, health_check, login};
use crate::utils::PostgresSession;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub jwt_secret_key: Secret<String>,
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
        jwt_secret_key: Secret::new(config.jwt.secret_key),
    };

    let customer_repo = PostgresSession::new(state.db_pool.clone())
        .await
        .map(PostgresCustomerRepoImpl::new)
        .map(Arc::new)
        .expect("Failed to create a customer repository.")
        as Arc<dyn CustomerRepo + Send + Sync>;

    let user_repo = PostgresSession::new(state.db_pool.clone())
        .await
        .map(PostgresUserRepoImpl::new)
        .map(Arc::new)
        .expect("Failed to create a user repository")
        as Arc<dyn UserRepo + Send + Sync>;

    // build our application with a route
    let app = Router::new()
        .route("/api/v1/health_check", get(health_check))
        .route("/api/v1/admin/customers", post(create_customer_handler))
        .route("/api/v1/login", post(login))
        .route("/api/v1/admin/change_password", post(change_password))
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
        .layer(Extension(customer_repo))
        .layer(Extension(user_repo))
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
