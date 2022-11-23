#[derive(serde::Deserialize)]
pub struct Login {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct LoginResponse {
    pub token: String,
}
