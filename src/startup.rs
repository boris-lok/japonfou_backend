use std::net::TcpListener;
use std::sync::Arc;

use axum::http::Request;
use axum::routing::{delete, get, post, put};
use axum::{Extension, Router};
use secrecy::{ExposeSecret, Secret};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestId, RequestId};
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer};
use tower_http::ServiceBuilderExt;
use tracing::Level;
use uuid::Uuid;

use crate::configuration::{DatabaseSettings, Settings};
use crate::repositories::{
    CustomerRepo, PostgresCustomerRepoImpl, PostgresProductRepoImpl, PostgresUserRepoImpl,
    ProductRepository, UserRepo,
};
use crate::routes::{
    change_password, create_customer_handler, create_product_handler, delete_customer_handler,
    get_customer_handler, get_product_handler, health_check, list_customers_handler, login, logout,
    update_customer_handler,
};
use crate::utils::PostgresSession;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub redis_client: Arc<redis::Client>,
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
    let client = redis::Client::open(config.redis_uri.expose_secret().as_str())
        .expect("Failed to connect the redis");

    let state = AppState {
        db_pool: get_database_connection(&config.database).await,
        redis_client: Arc::new(client),
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

    let product_repo = PostgresSession::new(state.db_pool.clone())
        .await
        .map(PostgresProductRepoImpl::new)
        .map(Arc::new)
        .expect("Failed to create product repository")
        as Arc<dyn ProductRepository + Send + Sync>;

    let customer_routes = Router::new()
        .route("/customers/:id", get(get_customer_handler))
        .route("/customers", get(list_customers_handler))
        .route("/customers", post(create_customer_handler))
        .route("/customers", put(update_customer_handler))
        .route("/customers", delete(delete_customer_handler));

    let product_routes = Router::new()
        .route("/products", post(create_product_handler))
        .route("/products/:id", get(get_product_handler));

    let change_password_route = Router::new().route("/change_password", post(change_password));

    let admin_routes = Router::new()
        .merge(customer_routes)
        .merge(product_routes)
        .merge(change_password_route);

    let authorization_routes = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout));

    // build our application with a route
    let app = Router::new()
        .route("/api/v1/health_check", get(health_check))
        .nest("/api/:version/admin", admin_routes)
        .nest("/api/:version", authorization_routes)
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
        .layer(Extension(product_repo))
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
