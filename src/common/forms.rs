#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct ListQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct ListQueryWithPrivate {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct FieldCreate {
    #[validate(length(min = 1, max = 150, message = "name length must greater than 1"))]
    pub name: String,
    #[validate(length(min = 1, max = 8, message = "type length must greater than 1"))]
    pub r#type: String,
    #[validate(length(min = 1, message = "str_value length must greater than 1"))]
    pub str_value: Option<String>,
    pub int_value: Option<i32>,
    pub float_value: Option<f32>,
}
