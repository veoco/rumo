#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize)]
pub struct TokenData {
    pub sub: String,
    pub exp: u64,
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

#[derive(Serialize, Deserialize, Validate)]
pub struct UserModify {
    #[validate(length(min = 1, max = 32, message = "name can not be longer than 32"))]
    pub name: String,
    #[validate(length(min = 1, max = 32, message = "screenName can not be longer than 32"))]
    pub screenName: String,
    #[validate(email)]
    pub mail: String,
    #[validate(length(min = 1, max = 150, message = "password can not be longer than 150"))]
    pub password: Option<String>,
    #[validate(url)]
    pub url: String,
    #[validate(length(min = 6, max = 13, message = "group name invalid"))]
    pub group: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct UsersQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<i32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<i32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OptionCreate {
    #[validate(length(min = 1, max = 32, message = "name length must greater than 1"))]
    pub name: String,
    #[validate(length(min = 1, message = "value length must greater than 1"))]
    pub value: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct OptionModify {
    #[validate(length(min = 1, message = "value length must greater than 1"))]
    pub value: String,
}
