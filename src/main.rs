use std::net::TcpListener;

use japonfou::configuration::get_configuration;
use japonfou::startup::run;
use japonfou::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("japonfou".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Can't get configuration");

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );

    let listener = TcpListener::bind(&address)
        .unwrap_or_else(|_| panic!("Can't bind address {} to TcpListener", &address));

    tracing::info!("listening to TcpListener {:?}", &listener);

    run(configuration, listener).await.unwrap()
}
