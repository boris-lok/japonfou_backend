use crate::helpers::spawn_app;
use redis::Commands;

#[tokio::test]
async fn logout_success() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let mut session = app
        .redis_client
        .get_connection()
        .expect("Failed to make a connection with redis");
    let user_id = app.test_user.id.to_string();

    // Act
    let _ = app.logout().await;

    // Assert
    let exp_from_session: Option<usize> = session
        .get(&user_id)
        .expect("Failed to get user information from session");

    assert!(exp_from_session.is_none());
}

#[tokio::test]
async fn logout_should_be_login_first() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .api_client
        .post(&format!("{}/api/v1/logout", &app.address))
        .send()
        .await
        .expect("Failed to make a request for logout.");

    assert_eq!(response.status().as_u16(), 401);
}
