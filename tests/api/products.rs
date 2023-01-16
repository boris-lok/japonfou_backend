use crate::helpers::spawn_app;
use fake::faker::name::en::Name;
use fake::Fake;
use japonfou::routes::{CreateProductResponse, ProductJson};
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
