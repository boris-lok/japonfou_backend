use crate::routes::product_id_generator;
use chrono::{DateTime, Utc};

#[derive(serde::Deserialize, Debug)]
pub struct CreateProductRequest {
    pub name: String,
    pub currency: i16,
    pub price: f64,
}

pub struct ValidCurrency(pub i16);

pub struct NewProduct {
    pub id: i64,
    pub name: String,
    pub currency: ValidCurrency,
    pub price: f64,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct ProductJson {
    pub id: i64,
    pub name: String,
    pub currency: i16,
    pub price: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateProductResponse {
    pub id: i64,
}

impl NewProduct {
    pub async fn parse(req: CreateProductRequest) -> Result<Self, String> {
        let id = async {
            let generator = product_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        Ok(Self {
            id,
            name: req.name,
            currency: ValidCurrency(req.currency),
            price: req.price,
        })
    }
}
