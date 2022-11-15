//! tests/api/health_check.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn health_check_works() {
    // Arrange
    let app = spawn_app().await;
    let uri = format!("{}/api/v1/health_check", app.address);
    dbg!(&uri);

    // Act
    let response = app
        .api_client
        .get(&uri)
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
}
