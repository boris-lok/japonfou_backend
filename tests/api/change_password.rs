use uuid::Uuid;

use crate::helpers::spawn_app;

#[tokio::test]
async fn you_must_logged_in_to_change_password() {
    // Arrange
    let app = spawn_app().await;
    let new_password = Uuid::new_v4().to_string();
    let request_body = serde_json::json!({
       "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });

    // Act
    let response = app
        .post_json("/api/v1/admin/change_password", &request_body)
        .await;

    assert_eq!(response.status().as_u16(), 401);
}

#[tokio::test]
async fn new_password_fields_must_match() {
    // Arrange
    let app = spawn_app().await.login().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();
    let request_body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &another_new_password,
    });

    // Act
    let response = app
        .auth_post_json("/api/v1/admin/change_password", &request_body)
        .await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn current_password_must_be_valid() {
    // Arrange
    let app = spawn_app().await.login().await;
    let wrong_password = Uuid::new_v4().to_string();
    let new_password = Uuid::new_v4().to_string();
    let request_body = serde_json::json!({
        "current_password": &wrong_password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });

    // Act
    let response = app
        .auth_post_json("/api/v1/admin/change_password", &request_body)
        .await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn change_password_works() {
    // Arrange
    let app = spawn_app().await.login().await;
    let new_password = Uuid::new_v4().to_string();
    let request_body = serde_json::json!({
        "current_password": &app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
    });

    // Act 1 - Change password
    let response = app
        .auth_post_json("/api/v1/admin/change_password", &request_body)
        .await;

    assert_eq!(response.status().as_u16(), 200);

    // Act 2 - Logout
    let app = app.logout().await;

    // Act 3 - use new password to login
    let _ = app.login().await;
}
