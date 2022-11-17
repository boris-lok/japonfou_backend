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

pub struct ValidPhone(pub String);

impl ValidPhone {
    pub fn parse(s: String) -> Result<Self, String> {
        //TODO: validate the phone number
        Ok(Self(s))
    }
}

#[derive(serde::Deserialize)]
pub struct CreateCustomer {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
}

pub struct NewCustomer {
    pub name: String,
    pub email: Option<ValidEmail>,
    pub phone: Option<ValidPhone>,
}

impl TryFrom<CreateCustomer> for NewCustomer {
    type Error = String;

    fn try_from(value: CreateCustomer) -> Result<Self, Self::Error> {
        let email = value.email.map(ValidEmail::parse).transpose()?;
        let phone = value.phone.map(ValidPhone::parse).transpose()?;

        Ok(Self {
            name: value.name,
            email,
            phone,
        })
    }
}