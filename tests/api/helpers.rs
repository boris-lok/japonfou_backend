use std::net::TcpListener;

use japonfou::configuration::get_configuration;
use japonfou::startup::run;

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");

        c.application.port = 0;

        c
    };

    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Can't bind tcp listener");
    let application_port = listener.local_addr().unwrap().port();
    let _ = tokio::spawn(run(listener));

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        port: application_port,
        api_client: client,
    }
}
