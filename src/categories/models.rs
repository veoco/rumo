#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub mid: u32,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub count: u32,
    pub order: u32,
    pub parent: u32,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CategoriesQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: u32,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: u32,
    #[validate(length(min = 1, max = 13, message = "order_by lenght must greater than 1"))]
    pub order_by: String,
}
