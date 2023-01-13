use crate::helpers::spawn_app;
use fake::faker::name::en::Name;
use fake::Fake;
use japonfou::routes::CreateProductResponse;

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
    assert_eq!(data_from_db.price, price);
}
