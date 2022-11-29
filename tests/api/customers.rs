use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;

use japonfou::routes::NewCustomerResponse;

use crate::helpers::spawn_app;

#[tokio::test]
async fn create_customer_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
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
    let response = app.post("/api/v1/admin/customers", &request).await;

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
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
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
        let response = app.post("/api/v1/admin/customers", &body).await;

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
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
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
        let response = app.post("/api/v1/admin/customers", &body).await;

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
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
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
    let response = app.post("/api/v1/admin/customers", &request).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let response = app.post("/api/v1/admin/customers", &request).await;

    assert_eq!(response.status().as_u16(), 409);
}

#[tokio::test]
async fn update_customer_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_customer().await;

    // Act 2 - Update a customer
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let phone = "(853) 87654321".to_string();

    let update_request = serde_json::json!({
        "id": id,
        "name": name,
        "email": email,
        "phone": phone,
    });

    let response = app
        .put("/api/v1/admin/customers", &update_request)
        .await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

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
async fn update_customer_return_a_400_when_data_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let id = app.create_a_new_customer().await;

    // Act 2 - Update a customer
    let test_case = vec![
        (
            serde_json::json!({
                "id": id,
                "name": "boris",
                "email": "123456789",
                "phone": "123456789",
            }),
            "Email is invalid",
        ),
        (
            serde_json::json!({
                "id": id,
                "name": "boris",
                "email": "boris.lok@outlook.com",
                "phone": "adfadfa",
            }),
            "Phone is invalid",
        ),
    ];

    for (body, msg) in test_case {
        let response = app.put("/api/v1/admin/customers", &body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn update_customer_return_409_when_data_conflict_with_existing_email_or_phone() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let a_id = app.create_a_new_customer().await;
    let b_id = app.create_a_new_customer().await;

    let data_from_db = sqlx::query!(
        r#"SELECT name, email, phone FROM customers where id=$1"#,
        a_id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved customer");

    // Act - Use `A` information to update `B`
    let request = serde_json::json!({
        "id": b_id,
        "email": data_from_db.email
    });

    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 409);

    let request = serde_json::json!({
        "id": b_id,
        "phone": data_from_db.phone
    });
    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 409);
}

#[tokio::test]
async fn update_customer_works_when_using_the_same_email_or_phone() {
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let id = app.create_a_new_customer().await;

    let data_from_db = sqlx::query!(
        r#"SELECT name, email, phone FROM customers where id=$1"#,
        id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch saved customer");

    // Act - Use `A` information to update `A`
    let request = serde_json::json!({
        "id": id,
        "email": data_from_db.email
    });

    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 200);

    let request = serde_json::json!({
        "id": id,
        "phone": data_from_db.phone
    });
    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn delete_customer_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_customer().await;
    let request = serde_json::json!({
        "id": id,
    });

    // Act
    let response = app.delete("/api/v1/admin/customers", &request).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let data_from_db = sqlx::query!(
        r#"SELECT deleted_at FROM customers where id=$1"#,
        id
    )
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved customer");

    assert!(data_from_db.deleted_at.is_some());
}