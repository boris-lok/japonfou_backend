use chrono::Utc;
use jsonwebtoken::{Algorithm, Validation};
use redis::Commands;

use japonfou::routes::{Claims, LoginResponse};
use japonfou::utils::JWT_SECRET_KEY_INSTANCE;

use crate::helpers::spawn_app;

#[tokio::test]
async fn login_failed() {
    // Arrange
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });

    // Act
    let response = app.post("/api/v1/login", &body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn login_success() {
    // Arrange
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });

    // Act
    let response = app.post("/api/v1/login", &body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let data: LoginResponse = response
        .json()
        .await
        .expect("Failed to parse json to `LoginResponse`");

    let token = data.token;
    let decoding_key = JWT_SECRET_KEY_INSTANCE.get().unwrap();
    let claims = jsonwebtoken::decode::<Claims>(
        &token,
        &decoding_key.decoding,
        &Validation::new(Algorithm::HS256),
    )
    .expect("Failed to decode token to claims");

    assert_eq!(claims.claims.sub, app.test_user.id.to_string());

    let mut session = app
        .redis_client
        .get_connection()
        .expect("Failed to connect the redis");

    let exp_from_session: Option<usize> = session
        .get(&claims.claims.sub)
        .expect("Failed to get the expired timestamp");

    assert!(exp_from_session.is_some());

    let exp_from_session = exp_from_session.unwrap();
    assert!(exp_from_session > Utc::now().timestamp() as usize);
}
