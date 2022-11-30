use crate::routes::customer_id_generator;
use crate::utils::get_phone_number_regex;

use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct ValidEmail(pub String);

impl ValidEmail {
    pub fn parse(s: String) -> Result<Self, String> {
        if validator::validate_email(&s) {
            Ok(Self(s))
        } else {
            Err(format!("{s} is not a valid email"))
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValidPhone(pub String);

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

#[derive(serde::Deserialize, Debug)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct UpdateCustomerRequest {
    pub id: i64,
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

impl NewCustomer {
    pub async fn parse(customer: CreateCustomerRequest) -> Result<Self, String> {
        let id = async {
            let generator = customer_id_generator();
            let mut generator = generator.lock().unwrap();
            generator.real_time_generate()
        }
        .await;

        let email = customer.email.map(ValidEmail::parse).transpose()?;
        let phone = customer.phone.map(ValidPhone::parse).transpose()?;

        if email.is_none() && phone.is_none() {
            return Err("Email and phone are missing.".to_string());
        }

        Ok(Self {
            id,
            name: customer.name,
            email,
            phone,
            remark: customer.remark,
        })
    }
}

pub struct UpdateCustomer {
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<ValidEmail>,
    pub phone: Option<ValidPhone>,
    pub remark: Option<String>,
}

impl UpdateCustomer {
    pub fn parse(customer: UpdateCustomerRequest) -> Result<Self, String> {
        let email = customer.email.map(ValidEmail::parse).transpose()?;
        let phone = customer.phone.map(ValidPhone::parse).transpose()?;

        Ok(Self {
            id: customer.id,
            name: customer.name,
            email,
            phone,
            remark: customer.remark,
        })
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NewCustomerResponse {
    pub id: i64,
}

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow, Debug)]
pub struct CustomerJson {
    pub id: i64,
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

#[derive(serde::Serialize, Debug)]
pub struct ListCustomersResponse {
    pub(crate) data: Vec<CustomerJson>,
}

#[derive(serde::Deserialize, Default)]
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
