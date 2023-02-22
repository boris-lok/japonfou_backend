use crate::routes::product_id_generator;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::PgRow;
use sqlx::Row;

#[derive(serde::Deserialize, Debug)]
pub struct CreateProductRequest {
    pub name: String,
    pub currency: i16,
    pub price: f64,
}

pub struct ValidCurrency(pub i16);

pub struct ValidProductName(pub String);

pub struct NewProduct {
    pub id: i64,
    pub name: ValidProductName,
    pub currency: ValidCurrency,
    pub price: f64,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ProductJson {
    pub id: String,
    pub name: String,
    pub currency: i16,
    pub price: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateProductResponse {
    pub id: i64,
}

#[derive(serde::Deserialize, Debug)]
pub struct DeleteProductRequest {
    pub id: i64,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateProductRequest {
    pub id: String,
    pub name: Option<String>,
    pub currency: Option<i16>,
    pub price: Option<f64>,
}

pub struct UpdateProduct {
    pub id: i64,
    pub name: Option<ValidProductName>,
    pub currency: Option<ValidCurrency>,
    pub price: Option<f64>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListProductsRequest {
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ListProductsResponse {
    pub data: Vec<ProductJson>,
}

#[derive(serde::Deserialize, Debug, Default)]
pub struct ProductSearchParameters {
    pub id: Option<i64>,
    #[serde(rename(deserialize = "name"))]
    pub partial_name: Option<String>,
}

impl NewProduct {
    pub async fn parse(req: CreateProductRequest) -> Result<Self, String> {
        if req.name.trim().is_empty() {
            return Err("Product name is empty.".to_string());
        }

        let id = async {
            let generator = product_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        Ok(Self {
            id,
            name: ValidProductName(req.name.trim().to_owned()),
            currency: ValidCurrency(req.currency),
            price: req.price,
        })
    }
}

impl UpdateProduct {
    pub async fn parse(req: UpdateProductRequest) -> Result<Self, String> {
        if req.name.is_some() && req.name.as_ref().unwrap().trim().is_empty() {
            return Err("Product name is empty.".to_string());
        }

        let id = req
            .id
            .parse::<i64>()
            .map_err(|_| "Can't parse id to i64.".to_string())?;

        Ok(Self {
            id,
            name: req.name.map(|e| ValidProductName(e.trim().to_owned())),
            currency: req.currency.map(ValidCurrency),
            price: req.price,
        })
    }
}

impl<'r> ::sqlx::FromRow<'r, PgRow> for ProductJson {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.try_get(0)?;
        let name: String = row.try_get(1)?;
        let currency: i16 = row.try_get(2)?;
        let price: Decimal = row.try_get(3)?;
        let created_at: DateTime<Utc> = row.try_get(4)?;
        let updated_at: Option<DateTime<Utc>> = row.try_get(5)?;
        let deleted_at: Option<DateTime<Utc>> = row.try_get(6)?;

        Ok(Self {
            id: id.to_string(),
            name,
            currency,
            price,
            created_at,
            updated_at,
            deleted_at,
        })
    }
}
