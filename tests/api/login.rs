use crate::helpers::spawn_app;

use japonfou::routes::{Claims, LoginResponse};
use japonfou::utils::JWT_SECRET_KEY_INSTANCE;
use jsonwebtoken::{Algorithm, Validation};

#[tokio::test]
async fn login_failed() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });
    let uri = format!("{}/api/v1/login", app.address);

    let response = app
        .api_client
        .post(&uri)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute a request.");

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn login_success() {
    let app = spawn_app().await;

    let body = serde_json::json!({
        "username": &app.test_user.username,
        "password": &app.test_user.password,
    });
    let uri = format!("{}/api/v1/login", app.address);

    let response = app
        .api_client
        .post(&uri)
        .json(&body)
        .send()
        .await
        .expect("Failed to execute a request.");

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
}
