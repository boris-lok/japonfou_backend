use crate::helpers::spawn_app;
use fake::faker::name::en::Name;
use fake::Fake;
use japonfou::routes::{CreateProductResponse, ListProductsResponse, ProductJson};
use rust_decimal::prelude::ToPrimitive;

#[tokio::test]
async fn create_product_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let name: String = Name().fake();
    let currency = 344_i16;
    let price = 10.0_f64;

    let request = serde_json::json!({
        "name": name,
        "currency": currency,
        "price": price
    });

    let response = app.post("/api/v1/admin/products", &request).await;

    assert_eq!(response.status().as_u16(), 200);
    let new_product: Result<CreateProductResponse, reqwest::Error> = response.json().await;
    assert!(new_product.is_ok());
    let new_product = new_product.unwrap();

    let id = new_product.id;

    let data_from_db = sqlx::query!(
        r#"SELECT name, currency, price from products where id=$1"#,
        id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch product from database");

    assert_eq!(data_from_db.name, name);
    assert_eq!(data_from_db.currency, currency);
    assert!(data_from_db.price.to_f64().unwrap() - price <= f64::EPSILON);
}

#[tokio::test]
async fn create_new_product_return_a_400_when_data_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let test_cases = vec![
        (
            serde_json::json!({
                "name": "",
                "currency": 344,
                "price": 10.0
            }),
            "Name is invalid",
        ),
        (
            serde_json::json!({
                "name": "product",
                "price": 10.0
            }),
            "Missing currency",
        ),
        (
            serde_json::json!({
                "name": "product",
                "currency": 344,
            }),
            "Missing price",
        ),
    ];

    for (body, msg) in test_cases {
        // Act
        let response = app.post("/api/v1/admin/products", &body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn get_product_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_product().await;

    // Act
    let uri = format!("/api/v1/admin/products/{id}");
    let response = app.get(&uri).await;

    assert_eq!(response.status().as_u16(), 200);
    let data: ProductJson = response.json().await.expect("Failed to decode json");

    let data_from_db = sqlx::query_as::<_, ProductJson>(r#"SELECT * FROM products where id=$1; "#)
        .bind(id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved product");

    assert_eq!(data_from_db.id, data.id);
    assert_eq!(data_from_db.name, data.name);
    assert_eq!(data_from_db.currency, data.currency);
    assert!(data_from_db.price.to_f64().unwrap() - data.price.to_f64().unwrap() <= f64::EPSILON);
    assert_eq!(data_from_db.created_at, data.created_at);
    assert_eq!(data_from_db.updated_at, data.updated_at);
    assert_eq!(data_from_db.deleted_at, data.deleted_at);
}

#[tokio::test]
async fn delete_product_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_product().await;

    let request = serde_json::json!({
        "id": id,
    });

    // Act
    let response = app.delete("/api/v1/admin/products", &request).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let data_from_db = sqlx::query!(r#"SELECT deleted_at FROM products where id=$1"#, id)
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved product");

    assert!(data_from_db.deleted_at.is_some());
}

#[tokio::test]
async fn update_product_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_product().await;

    let name: String = Name().fake();
    let currency = 344_i16;
    let price = 5.0_f64;
    let body = serde_json::json!({
        "id": id,
        "name": name,
        "currency": currency,
        "price": price,
    });

    // Act
    let response = app.put("/api/v1/admin/products", &body).await;

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    let data_from_db = sqlx::query!(
        r#"SELECT name, currency, price from products where id=$1; "#,
        id
    )
    .fetch_one(&app.db_pool)
    .await
    .expect("Failed to fetch product from database.");

    assert_eq!(data_from_db.name, name);
    assert_eq!(data_from_db.currency, currency);
    assert!(data_from_db.price.to_f64().unwrap() - price.to_f64().unwrap() <= f64::EPSILON);
}

#[tokio::test]
async fn update_product_return_a_400_when_data_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let id = app.create_a_new_product().await;
    let test_cases = vec![(
        serde_json::json!({
            "id": id,
            "name": "",
        }),
        "product name is empty",
    )];

    for (body, msg) in test_cases {
        let response = app.put("/api/v1/admin/products", &body).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API didn't fail with 400 Bad Request when the payload was {}",
            msg
        );
    }
}

#[tokio::test]
async fn list_products_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let mut expected_ids = vec![];

    for _ in 0..20 {
        expected_ids.push(app.create_a_new_product().await);
    }

    // Act
    let uri = "/api/v1/admin/products";
    let response = app.get(uri).await;

    assert_eq!(response.status().as_u16(), 200);
    let response = response.json::<ListProductsResponse>().await;
    assert!(response.is_ok());
    let data = response.unwrap();
    assert_eq!(data.data.len(), 20);
}

#[tokio::test]
async fn list_products_works_with_filter() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    let mut expected_ids = vec![];

    for _ in 0..20 {
        expected_ids.push(app.create_a_new_product().await);
    }

    // Act
    let uri = "/api/v1/admin/products";
    let response = app.get(uri).await;

    assert_eq!(response.status().as_u16(), 200);
    let response_json = response.json::<ListProductsResponse>().await;
    assert!(response_json.is_ok());
    let data = response_json.unwrap();
    assert_eq!(data.data.len(), 20);

    let data_from_db = sqlx::query_as::<_, ProductJson>(r#"SELECT * FROM products where id=$1; "#)
        .bind(expected_ids[0])
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved products");

    let id = data_from_db.id.to_string();
    let name = data_from_db.name;

    let test_cases = vec![
        format!(r#"{{ "{}": {} }}"#, "id", id),
        format!(r#"{{ "{}": "{}" }}"#, "name", name),
        format!(r#"{{ "{}": {}, "{}": "{}" }}"#, "id", id, "name", name),
    ];

    for keyword in test_cases {
        let keyword = base64::encode(&keyword);
        let uri = format!("/api/v1/admin/products?keyword={}", keyword);

        let response = app.get(&uri).await;
        assert_eq!(response.status().as_u16(), 200);
        let response_json = response.json::<ListProductsResponse>().await;
        assert!(response_json.is_ok());
        let data = response_json.unwrap();
        assert_eq!(data.data.len(), 1);
    }
}

#[tokio::test]
async fn list_products_with_page_and_page_size_works() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;

    for _ in 0..25 {
        let _ = app.create_a_new_product().await;
    }

    let test_cases = vec![(0, 10, 10), (1, 10, 10), (1, 5, 5), (2, 10, 5)];

    // Act
    for (page, page_size, expected) in test_cases {
        let uri = format!(
            "/api/v1/admin/products?page={}&page_size={}",
            page, page_size
        );

        // Assert
        let response = app.get(&uri).await;
        assert_eq!(response.status().as_u16(), 200);
        let response_json = response.json::<ListProductsResponse>().await;
        assert!(response_json.is_ok());
        let data = response_json.unwrap();
        assert_eq!(data.data.len(), expected);
    }
}

#[tokio::test]
async fn list_products_failed_when_keyword_is_invalid() {
    // Arrange
    let app = spawn_app().await;
    let login_body = app.login_body();
    let app = app.login(&login_body).await;
    let keyword = "random_string";
    let uri = format!("/api/v1/admin/products?keyword={}", keyword);

    // Act
    let response = app.get(&uri).await;

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
