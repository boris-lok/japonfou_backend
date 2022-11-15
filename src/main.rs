use std::net::TcpListener;

use axum::Router;
use axum::routing::get;

use japonfou::configuration::get_configuration;
use japonfou::routes::health_check;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/health_check", get(health_check));

    let configuration = get_configuration().expect("Can't get configuration");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(&address)
        .expect(format!("Can't bind address {} to TcpListener", &address).as_str());

    println!("listening on {:?}", listener);

    axum::Server::from_tcp(listener)
        .expect("Axum can't bind tcp listener")
        .serve(app.into_make_service())
        .await
        .unwrap()
}
