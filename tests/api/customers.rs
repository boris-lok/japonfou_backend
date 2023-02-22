use base64::Engine;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::Fake;

use japonfou::routes::{CreateCustomerResponse, CustomerJson, ListCustomersResponse};

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
    let response: Result<CreateCustomerResponse, reqwest::Error> = response.json().await;
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
            "The API didn't fail with 400 Bad Request when the payload was {msg}",
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
            "The API didn't fail with 400 Bad Request when the payload was {msg}",
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
        "id": id.to_string(),
        "name": name,
        "email": email,
        "phone": phone,
    });

    let response = app.put("/api/v1/admin/customers", &update_request).await;

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
            "The API didn't fail with 400 Bad Request when the payload was {msg}",
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
        "id": b_id.to_string(),
        "email": data_from_db.email
    });

    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 409);

    let request = serde_json::json!({
        "id": b_id.to_string(),
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
        "id": id.to_string(),
        "email": data_from_db.email
    });

    let response = app.put("/api/v1/admin/customers", &request).await;
    assert_eq!(response.status().as_u16(), 200);

    let request = serde_json::json!({
        "id": id.to_string(),
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

    let data_from_db = sqlx::query!(r#"SELECT deleted_at FROM customers where id=$1"#, id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved customer");

    assert!(data_from_db.deleted_at.is_some());
}

#[tokio::test]
async fn get_customer_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_customer().await;

    // Act
    let uri = format!("/api/v1/admin/customers/{id}");
    let response = app.get(&uri).await;

    assert_eq!(response.status().as_u16(), 200);

    let data: CustomerJson = response.json().await.expect("Failed to decode json");

    let data_from_db =
        sqlx::query_as::<_, CustomerJson>(r#"SELECT * FROM customers where id=$1; "#)
            .bind(id)
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved customer");

    assert_eq!(data_from_db.id, data.id);
    assert_eq!(data_from_db.name, data.name);
    assert_eq!(data_from_db.email, data.email);
    assert_eq!(data_from_db.phone, data.phone);
    assert_eq!(data_from_db.remark, data.remark);
    assert_eq!(data_from_db.created_at, data.created_at);
    assert_eq!(data_from_db.updated_at, data.updated_at);
    assert_eq!(data_from_db.deleted_at, data.deleted_at);
}

#[tokio::test]
async fn get_customer_return_a_400_when_id_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_customer().await;

    // Act
    let uri = format!("/api/v1/admin/customers/haha_{id}");
    let response = app.get(&uri).await;

    assert_eq!(response.status().as_u16(), 400);
}

#[tokio::test]
async fn list_customers_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let mut expected_ids = vec![];

    for _ in 0..20 {
        expected_ids.push(app.create_a_new_customer().await);
    }

    // Act
    let uri = "/api/v1/admin/customers";
    let response = app.get(uri).await;

    assert_eq!(response.status().as_u16(), 200);
    let response = response.json::<ListCustomersResponse>().await;
    assert!(response.is_ok());
    let data = response.unwrap();
    assert_eq!(data.data.len(), 20);
}

#[tokio::test]
async fn list_customers_works_with_filter() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let mut expected_ids = vec![];

    for _ in 0..20 {
        expected_ids.push(app.create_a_new_customer().await);
    }

    // Act
    let uri = "/api/v1/admin/customers";
    let response = app.get(uri).await;

    assert_eq!(response.status().as_u16(), 200);
    let response_json = response.json::<ListCustomersResponse>().await;
    assert!(response_json.is_ok());
    let data = response_json.unwrap();
    assert_eq!(data.data.len(), 20);

    let data_from_db =
        sqlx::query_as::<_, CustomerJson>(r#"SELECT * FROM customers where id=$1; "#)
            .bind(expected_ids[0])
            .fetch_one(&app.db_pool)
            .await
            .expect("Failed to fetch saved customer");

    let id = data_from_db.id.to_string();
    let name = data_from_db.name;
    let email = data_from_db.email.unwrap();
    let phone = data_from_db.phone.unwrap();

    let test_cases = vec![
        format!(r#"{{ "{}": {} }}"#, "id", id),
        format!(r#"{{ "{}": "{}" }}"#, "name", name),
        format!(r#"{{ "{}": "{}" }}"#, "email", email),
        // TODO phone should be duplicate, because the phone is hard code
        // format!(r#"{{ "{}": "{}" }}"#, "phone", phone),
        format!(
            r#"{{ "{}": {}, "{}": "{}", "{}": "{}", "{}": "{}" }}"#,
            "id", id, "name", name, "email", email, "phone", phone
        ),
    ];

    for keyword in test_cases {
        let keyword = base64::engine::general_purpose::STANDARD.encode(&keyword);
        let uri = format!("/api/v1/admin/customers?keyword={keyword}");

        let response = app.get(&uri).await;
        assert_eq!(response.status().as_u16(), 200);
        let response_json = response.json::<ListCustomersResponse>().await;
        assert!(response_json.is_ok());
        let data = response_json.unwrap();
        assert_eq!(data.data.len(), 1);
    }
}

#[tokio::test]
async fn list_customers_with_page_and_page_size_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    for _ in 0..25 {
        let _ = app.create_a_new_customer().await;
    }

    let test_cases = vec![(0, 10, 10), (1, 10, 10), (1, 5, 5), (2, 10, 5)];

    // Act
    for (page, page_size, expected) in test_cases {
        let uri = format!("/api/v1/admin/customers?page={page}&page_size={page_size}");

        // Assert
        let response = app.get(&uri).await;
        assert_eq!(response.status().as_u16(), 200);
        let response_json = response.json::<ListCustomersResponse>().await;
        assert!(response_json.is_ok());
        let data = response_json.unwrap();
        assert_eq!(data.data.len(), expected);
    }
}

#[tokio::test]
async fn list_customers_failed_when_keyword_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let keyword = "random_string";
    let uri = format!("/api/v1/admin/customers?keyword={keyword}");

    // Act
    let response = app.get(&uri).await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
