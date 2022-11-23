use crate::routes::Login;
use secrecy::Secret;

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

impl From<Login> for Credentials {
    fn from(e: Login) -> Self {
        Self {
            username: e.username,
            password: Secret::new(e.password),
        }
    }
}

#[derive(sea_query::Iden)]
pub enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
}
