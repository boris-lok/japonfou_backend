use std::net::TcpListener;

use japonfou::configuration::get_configuration;
use japonfou::startup::run;

#[tokio::main]
async fn main() {
    let configuration = get_configuration().expect("Can't get configuration");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(&address)
        .unwrap_or_else(|_| panic!("Can't bind address {} to TcpListener", &address));

    println!("listening on {:?}", listener);

    run(configuration, listener).await.unwrap()
}
