use axum::routing::get;
use axum::Router;
use japonfou::routes::health_check;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/health_check", get(health_check));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    println!("listening on {}", &addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
