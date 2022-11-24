use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;

use japonfou::routes::NewCustomerResponse;

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_customer_works() {
    // Arrange
    let app = spawn_app().await.login().await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    // let phone: String = PhoneNumber().fake();
    // TODO: fake phone number is not correct. (e.g. "613-637-8110 x76344")
    let phone = "(853) 12345678".to_string();

    let request = serde_json::json!({
        "name": name,
        "email": email,
        "phone": phone,
    });

    // Act
    let response = app
        .auth_post_json("/api/v1/admin/customers", &request)
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);
    let response: Result<NewCustomerResponse, reqwest::Error> = response.json().await;
    assert!(response.is_ok());
    let response = response.unwrap();

    let id = response.id;

    let data_from_db = sqlx::query!(
        r#"SELECT name, email, phone FROM customers where id=$1"#,
        id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved customer");

    assert_eq!(data_from_db.email, Some(email));
    assert_eq!(data_from_db.name, name);
    assert_eq!(data_from_db.phone, Some(phone));
}

#[tokio::test]
async fn create_new_customer_return_a_400_when_data_is_invalid() {
    // Arrange
    let app = spawn_app().await.login().await;
    let test_case = vec![
        (
            serde_json::json!({
                "name": "boris",
                "email": "123456789",
                "phone": "123456789",
            }),
            "Email is invalid",
        ),
        (
            serde_json::json!({
                "name": "boris",
                "email": "boris.lok@outlook.com",
                "phone": "adfadfa",
            }),
            "Phone is invalid",
        ),
    ];

    for (body, msg) in test_case {
        // Act
        let response = app.auth_post_json("/api/v1/admin/customers", &body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn create_new_customer_return_a_400_when_data_is_missing() {
    // Arrange
    let app = spawn_app().await.login().await;
    let test_case = vec![
        (
            serde_json::json!({
                "email": "123456789",
                "phone": "123456789",
            }),
            "Name is missing",
        ),
        (
            serde_json::json!({
                "name": "boris",
            }),
            "Email and phone are missing",
        ),
    ];

    for (body, msg) in test_case {
        // Act
        let response = app.auth_post_json("/api/v1/admin/customers", &body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn create_new_customer_return_a_400_when_customer_is_duplicate() {
    // Arrange
    let app = spawn_app().await.login().await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    // let phone: String = PhoneNumber().fake();
    let phone = "(853) 12345678".to_string();

    let request = serde_json::json!({
        "name": name,
        "email": email,
        "phone": phone,
    });

    // Act
    let response = app
        .auth_post_json("/api/v1/admin/customers", &request)
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response = app
        .auth_post_json("/api/v1/admin/customers", &request)
        .await;

    assert_eq!(response.status().as_u16(), 409);
}
