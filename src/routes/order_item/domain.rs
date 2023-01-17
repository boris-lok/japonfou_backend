use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use sqlx::postgres::PgRow;
use sqlx::{Error, Row};

use crate::routes::{order_item_id_generator, CustomerJson, ProductJson};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct OrderItemJson {
    pub id: i64,
    pub customer: CustomerJson,
    pub product: ProductJson,
    pub quantity: u32,
    pub status: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct CreateOrderItemsRequest {
    customer_id: i64,
    product_id: i64,
    quantity: u32,
    status: u32,
}

#[derive(Debug)]
pub struct ValidProductId(pub i64);

#[derive(Debug)]
pub struct ValidCustomerId(pub i64);

#[derive(Debug)]
pub struct ValidQuantity(pub u32);

#[derive(Debug)]
pub struct ValidStatus(pub u32);

#[derive(Debug)]
pub struct NewOrderItem {
    pub id: i64,
    pub customer_id: ValidCustomerId,
    pub product_id: ValidProductId,
    pub quantity: ValidQuantity,
    pub status: ValidStatus,
}

impl<'r> ::sqlx::FromRow<'r, PgRow> for OrderItemJson {
    fn from_row(row: &'r PgRow) -> Result<Self, Error> {
        let id: i64 = row.try_get(0)?;
        let quantity: i32 = row.try_get(1)?;
        let status: i32 = row.try_get(2)?;
        let created_at: DateTime<Utc> = row.try_get(3)?;
        let updated_at: Option<DateTime<Utc>> = row.try_get(4)?;
        let deleted_at: Option<DateTime<Utc>> = row.try_get(5)?;

        let customer_id: i64 = row.try_get(6)?;
        let product_id: i64 = row.try_get(7)?;

        let customer_name: String = row.try_get(8)?;
        let customer_created_at: DateTime<Utc> = row.try_get(9)?;

        let product_name: String = row.try_get(10)?;
        let product_currency: i16 = row.try_get(11)?;
        let product_price: Decimal = row.try_get(12)?;
        let product_created_at: DateTime<Utc> = row.try_get(13)?;

        let customer = CustomerJson {
            id: customer_id,
            name: customer_name,
            email: None,
            phone: None,
            remark: None,
            created_at: customer_created_at,
            updated_at: None,
            deleted_at: None,
        };

        let product = ProductJson {
            id: product_id,
            name: product_name,
            currency: product_currency,
            price: product_price,
            created_at: product_created_at,
            updated_at: None,
            deleted_at: None,
        };

        Ok(Self {
            id,
            customer,
            product,
            quantity: quantity as u32,
            status: status as u32,
            created_at,
            updated_at,
            deleted_at,
        })
    }
}

impl NewOrderItem {
    pub async fn parse(req: CreateOrderItemsRequest) -> Result<Self, String> {
        // TODO: check product is exist
        // TODO: check customer is exist

        let id = async {
            let generator = order_item_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        Ok(Self {
            id,
            customer_id: ValidCustomerId(req.customer_id),
            product_id: ValidProductId(req.product_id),
            quantity: ValidQuantity(req.quantity),
            status: ValidStatus(req.status),
        })
    }
}
