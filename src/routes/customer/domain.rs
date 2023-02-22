use chrono::{DateTime, Utc};
use sqlx::postgres::PgRow;
use sqlx::Row;

use crate::routes::customer_id_generator;
use crate::utils::get_phone_number_regex;

#[derive(Clone, Debug)]
pub struct ValidEmail(pub String);

#[derive(Clone, Debug)]
pub struct ValidPhone(pub String);

#[derive(serde::Deserialize, Debug)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateCustomerRequest {
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct DeleteCustomerRequest {
    pub id: i64,
}

pub struct NewCustomer {
    pub id: i64,
    pub name: String,
    pub email: Option<ValidEmail>,
    pub phone: Option<ValidPhone>,
    pub remark: Option<String>,
}

pub struct UpdateCustomer {
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<ValidEmail>,
    pub phone: Option<ValidPhone>,
    pub remark: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct CreateCustomerResponse {
    pub id: i64,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct RawCustomer {
    pub id: i64,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct CustomerJson {
    pub id: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(serde::Deserialize, Debug)]
pub struct ListCustomersRequest {
    pub keyword: Option<String>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct ListCustomersResponse {
    pub data: Vec<CustomerJson>,
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct CustomerSearchParameters {
    pub id: Option<i64>,
    #[serde(rename(deserialize = "name"))]
    pub partial_name: Option<String>,
    #[serde(rename(deserialize = "email"))]
    pub partial_email: Option<String>,
    #[serde(rename(deserialize = "phone"))]
    pub partial_phone: Option<String>,
    #[serde(rename(deserialize = "remark"))]
    pub partial_remark: Option<String>,
}

impl ValidEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if validator::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid email"))
        }
    }
}

impl ValidPhone {
    pub fn parse(s: String) -> Result<Self, String> {
        dbg!(&s);
        let re = get_phone_number_regex();
        if re.is_match(&s) {
            Ok(Self(s))
        } else {
            Err(format!(
                "{s} is not a valid phone number. Please follow this format (853) 12345678"
            ))
        }
    }
}

impl NewCustomer {
    pub async fn parse(customer: CreateCustomerRequest) -> Result<Self, String> {
        let email = customer.email.map(ValidEmail::parse).transpose()?;
        let phone = customer.phone.map(ValidPhone::parse).transpose()?;

        if email.is_none() && phone.is_none() {
            return Err("Email and phone are missing.".to_string());
        }

        let id = async {
            let generator = customer_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        Ok(Self {
            id,
            name: customer.name,
            email,
            phone,
            remark: customer.remark,
        })
    }
}

impl UpdateCustomer {
    pub fn parse(customer: UpdateCustomerRequest) -> Result<Self, String> {
        let email = customer.email.map(ValidEmail::parse).transpose()?;
        let phone = customer.phone.map(ValidPhone::parse).transpose()?;
        let id = customer
            .id
            .parse::<i64>()
            .map_err(|_| "Can't parse the id to i64.".to_string())?;

        Ok(Self {
            id,
            name: customer.name,
            email,
            phone,
            remark: customer.remark,
        })
    }
}

impl<'r> ::sqlx::FromRow<'r, PgRow> for CustomerJson {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        let id: i64 = row.try_get(0)?;
        let name: String = row.try_get(1)?;
        let email: Option<String> = row.try_get(2)?;
        let phone: Option<String> = row.try_get(3)?;
        let remark: Option<String> = row.try_get(4)?;
        let created_at: DateTime<Utc> = row.try_get(5)?;
        let updated_at: Option<DateTime<Utc>> = row.try_get(6)?;
        let deleted_at: Option<DateTime<Utc>> = row.try_get(7)?;

        Ok(Self {
            id: id.to_string(),
            name,
            email,
            phone,
            remark,
            created_at,
            updated_at,
            deleted_at,
        })
    }
}
