#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub sub: String,
    pub exp: u64,
}

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub uid: u32,
    pub name: Option<String>,
    pub password: Option<String>,
    pub mail: Option<String>,
    pub url: Option<String>,
    pub screenName: Option<String>,
    pub created: u32,
    pub activated: u32,
    pub logged: u32,
    pub group: String,
    pub authCode: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct UserLogin {
    #[validate(email)]
    pub mail: String,
    #[validate(length(max = 150, message = "password can not be longer than 150"))]
    pub password: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct UserRegister {
    #[validate(length(min = 1, max = 32, message = "name can not be longer than 32"))]
    pub name: String,
    #[validate(email)]
    pub mail: String,
    #[validate(length(min = 1, max = 150, message = "password can not be longer than 150"))]
    pub password: String,
    #[validate(url)]
    pub url: String,
}
