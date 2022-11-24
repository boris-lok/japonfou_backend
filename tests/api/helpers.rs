use std::net::TcpListener;

use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use japonfou::configuration::{get_configuration, DatabaseSettings};
use japonfou::routes::LoginResponse;
use japonfou::startup::{get_database_connection, run};
use japonfou::utils::{JwtKey, JWT_SECRET_KEY_INSTANCE};

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub jwt_token: Option<String>,
}

impl TestApp {
    pub async fn login(self) -> TestApp {
        let request_body = serde_json::json!({
            "username": self.test_user.username,
            "password": self.test_user.password,
        });

        let response = self.post_json("/api/v1/login", &request_body).await;

        assert_eq!(response.status().as_u16(), 200);

        let res = response
            .json::<LoginResponse>()
            .await
            .expect("login failed");

        let token = res.token;

        TestApp {
            jwt_token: Some(token),
            ..self
        }
    }

    pub async fn logout(self) -> TestApp {
        let mut header_map = reqwest::header::HeaderMap::new();
        let token = format!("Bearer {}", self.jwt_token.as_ref().unwrap());
        header_map.append(reqwest::header::AUTHORIZATION, token.parse().unwrap());
        let response = self
            .api_client
            .post(&format!("{}/api/v1/logout", self.address))
            .headers(header_map)
            .send()
            .await
            .expect("Failed to make a request");

        assert_eq!(response.status().as_u16(), 200);

        TestApp {
            jwt_token: None,
            ..self
        }
    }

    pub async fn auth_post_json(&self, uri: &str, body: &Value) -> reqwest::Response {
        if self.jwt_token.is_some() {
            self.base_post_json(uri, body, true).await
        } else {
            panic!("Failed to make a request before login");
        }
    }

    pub async fn post_json(&self, uri: &str, body: &Value) -> reqwest::Response {
        self.base_post_json(uri, body, false).await
    }

    async fn base_post_json(&self, uri: &str, body: &Value, with_token: bool) -> reqwest::Response {
        let mut header_map = reqwest::header::HeaderMap::new();
        if with_token {
            let token = format!("Bearer {}", self.jwt_token.as_ref().unwrap());
            header_map.append(reqwest::header::AUTHORIZATION, token.parse().unwrap());
        }
        self.api_client
            .post(&format!("{}{}", self.address, uri))
            .json(body)
            .headers(header_map)
            .send()
            .await
            .expect("Failed to make a request")
    }
}

pub struct TestUser {
    pub id: Uuid,
    pub username: String,
    pub password: String,
}

impl TestUser {
    pub fn generate() -> Self {
        Self {
            id: Uuid::new_v4(),
            username: Uuid::new_v4().to_string(),
            password: Uuid::new_v4().to_string(),
        }
    }

    async fn store(&self, pool: &PgPool) {
        let salt = SaltString::generate(&mut rand::thread_rng());
        // Match parameters of the default password
        let password_hash = Argon2::new(
            Algorithm::Argon2id,
            Version::V0x13,
            Params::new(15000, 2, 1, None).unwrap(),
        )
        .hash_password(self.password.as_bytes(), &salt)
        .unwrap()
        .to_string();

        sqlx::query!(
            "INSERT INTO users (id, username, password_hash) VALUES ($1, $2, $3);",
            self.id,
            self.username,
            password_hash,
        )
        .execute(pool)
        .await
        .expect("Failed to create test users.");
    }
}

pub async fn spawn_app() -> TestApp {
    let configuration = {
        let mut c = get_configuration().expect("Failed to read configuration");
        // Use a random OS port
        c.application.port = 0;
        // Use a different database for each test case.
        c.database.database_name = Uuid::new_v4().to_string();
        c
    };

    // Configure the test database
    configure_database(&configuration.database).await;
    let db_pool = get_database_connection(&configuration.database).await;

    let address = format!("127.0.0.1:{}", configuration.application.port);
    let listener = TcpListener::bind(&address).expect("Can't bind tcp listener");
    let application_port = listener.local_addr().unwrap().port();

    let _ = JWT_SECRET_KEY_INSTANCE
        .get_or_init(|| JwtKey::new(configuration.jwt.secret_key.as_bytes()));

    let _ = tokio::spawn(run(configuration, listener));

    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .cookie_store(true)
        .build()
        .unwrap();

    let app = TestApp {
        address: format!("http://127.0.0.1:{}", application_port),
        port: application_port,
        api_client: client,
        db_pool,
        test_user: TestUser::generate(),
        jwt_token: None,
    };

    app.test_user.store(&app.db_pool).await;

    app
}

async fn configure_database(config: &DatabaseSettings) -> sqlx::PgPool {
    let mut conn = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect postgres");

    conn.execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
