use crate::routes::customer_id_generator;
use crate::utils::get_phone_number_regex;

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
pub struct CreateCustomer {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub remark: Option<String>,
}

pub struct NewCustomer {
    pub id: i64,
    pub name: String,
    pub email: Option<ValidEmail>,
    pub phone: Option<ValidPhone>,
    pub remark: Option<String>,
}

impl NewCustomer {
    pub async fn parse(customer: CreateCustomer) -> Result<Self, String> {
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct NewCustomerResponse {
    pub id: i64,
}
