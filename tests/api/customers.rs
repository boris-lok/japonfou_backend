use crate::helpers::spawn_app;
use fake::faker::internet::en::SafeEmail;
use fake::faker::name::en::Name;
use fake::faker::phone_number::en::PhoneNumber;
use fake::Fake;

#[tokio::test]
async fn create_customer_works() {
    // Arrange
    let app = spawn_app().await;
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let phone: String = PhoneNumber().fake();

    let request = serde_json::json!({
        "name": name,
        "email": email,
        "phone": phone,
    });
    let uri = format!("{}/api/v1/customers", app.address);

    // Act
    let response = app
        .api_client
        .post(&uri)
        .json(&request)
        .send()
        .await
        .expect("Failed to execute a request.");

    // Assert
    assert_eq!(response.status().as_u16(), 200);

    // let data_from_db = sqlx::query!("SELECT name, email, phone FROM customers")
    //     .fetch_one(&app.db_pool)
    //     .await
    //     .expect("Failed to fetch saved customer");
    //
    // assert_eq!(data_from_db.email, Some(email));
    // assert_eq!(data_from_db.name, name);
    // assert_eq!(data_from_db.phone, Some(phone));
}
