
use std::net::TcpListener;

use axum::routing::get;
use axum::Router;

use crate::routes::health_check;

pub async fn run(tcp_listener: TcpListener) -> hyper::Result<()> {
    // build our application with a route
    let app = Router::new().route("/api/v1/health_check", get(health_check));

    axum::Server::from_tcp(tcp_listener)
        .expect("Can't bind tcp listener")
        .serve(app.into_make_service())
        .await
}
