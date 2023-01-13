use std::net::TcpListener;

use argon2::password_hash::SaltString;
use argon2::{Algorithm, Argon2, Params, PasswordHasher, Version};
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;
use secrecy::ExposeSecret;
use serde_json::Value;
use sqlx::types::Uuid;
use sqlx::{Connection, Executor, PgConnection, PgPool};

use japonfou::configuration::{get_configuration, DatabaseSettings};
use japonfou::routes::{LoginResponse, CreateCustomerResponse};
use japonfou::startup::{get_database_connection, run};
use japonfou::utils::{JwtKey, JWT_SECRET_KEY_INSTANCE};

pub struct AuthTestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub jwt_token: String,
    pub redis_client: redis::Client,
}

impl AuthTestApp {
    pub async fn logout(self) -> TestApp {
        let mut header_map = reqwest::header::HeaderMap::new();
        let token = format!("Bearer {}", self.jwt_token);
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
            address: self.address,
            port: self.port,
            api_client: self.api_client,
            db_pool: self.db_pool,
            test_user: self.test_user,
            redis_client: self.redis_client,
        }
    }

    pub async fn create_a_new_customer(&self) -> i64 {
        let name: String = Name().fake();
        let email: String = SafeEmail().fake();
        // TODO: fake phone number is not correct. (e.g. "613-637-8110 x76344")
        let phone = "(853) 12345678".to_string();

        let request = serde_json::json!({
            "name": name,
            "email": email,
            "phone": phone,
        });

        let response = self.post("/api/v1/admin/customers", &request).await;
        assert_eq!(response.status().as_u16(), 200);
        let response: Result<CreateCustomerResponse, reqwest::Error> = response.json().await;
        assert!(response.is_ok());
        let response = response.unwrap();
        response.id
    }

    pub async fn post(&self, uri: &str, body: &Value) -> reqwest::Response {
        send_api_request(
            &self.api_client,
            RequestMethod::Post,
            &self.address,
            uri,
            Some(body),
            Some(&self.jwt_token),
        )
        .await
    }

    pub async fn put(&self, uri: &str, body: &Value) -> reqwest::Response {
        send_api_request(
            &self.api_client,
            RequestMethod::Put,
            &self.address,
            uri,
            Some(body),
            Some(&self.jwt_token),
        )
        .await
    }

    pub async fn delete(&self, uri: &str, body: &Value) -> reqwest::Response {
        send_api_request(
            &self.api_client,
            RequestMethod::Delete,
            &self.address,
            uri,
            Some(body),
            Some(&self.jwt_token),
        )
        .await
    }

    pub async fn get(&self, uri: &str) -> reqwest::Response {
        send_api_request(
            &self.api_client,
            RequestMethod::Get,
            &self.address,
            uri,
            None,
            Some(&self.jwt_token),
        )
        .await
    }
}

pub struct TestApp {
    pub address: String,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub db_pool: PgPool,
    pub test_user: TestUser,
    pub redis_client: redis::Client,
}

impl TestApp {
    pub async fn login(self, body: &Value) -> AuthTestApp {
        let response = self.post("/api/v1/login", body).await;

        assert_eq!(response.status().as_u16(), 200);

        let res = response
            .json::<LoginResponse>()
            .await
            .expect("login failed");

        let token = res.token;

        AuthTestApp {
            jwt_token: token,
            address: self.address,
            port: self.port,
            api_client: self.api_client,
            db_pool: self.db_pool,
            test_user: self.test_user,
            redis_client: self.redis_client,
        }
    }

    pub async fn post(&self, uri: &str, body: &Value) -> reqwest::Response {
        send_api_request(
            &self.api_client,
            RequestMethod::Post,
            &self.address,
            uri,
            Some(body),
            None,
        )
        .await
    }

    pub fn login_body(&self) -> Value {
        serde_json::json!({
            "username": self.test_user.username,
            "password": self.test_user.password,
        })
    }
}

async fn send_api_request(
    client: &reqwest::Client,
    method: RequestMethod,
    address: &str,
    uri: &str,
    body: Option<&Value>,
    token: Option<&str>,
) -> reqwest::Response {
    let mut header_map = reqwest::header::HeaderMap::new();
    if token.is_some() {
        let token = format!("Bearer {}", token.unwrap());
        header_map.append(reqwest::header::AUTHORIZATION, token.parse().unwrap());
    }
    let uri = format!("{}{}", address, uri);
    let builder = match method {
        RequestMethod::Post => client.post(&uri),
        RequestMethod::Get => client.get(&uri),
        RequestMethod::Put => client.put(&uri),
        RequestMethod::Delete => client.delete(&uri),
    };

    let builder = match body {
        None => builder,
        Some(body) => builder.json(body),
    };

    builder
        .headers(header_map)
        .send()
        .await
        .expect("Failed to make a request")
}

pub enum RequestMethod {
    Post,
    Get,
    Put,
    Delete,
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

    let redis_client = redis::Client::open(configuration.redis_uri.expose_secret().as_str())
        .expect("Failed to connect the redis");

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
        redis_client,
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
