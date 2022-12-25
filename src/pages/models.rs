#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Page {
    pub cid: u32,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    pub text: String,
    pub order: u32,
    pub authorId: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub commentsNum: u32,
    pub allowComment: String,
    pub allowPing: String,
    pub allowFeed: String,
    pub parent: u32,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PagesQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PageCreate {
    #[validate(length(min = 1, max = 150, message = "title length must greater than 1"))]
    pub title: String,
    #[validate(length(min = 1, max = 150, message = "slug length must greater than 1"))]
    pub slug: String,
    pub created: u32,
    pub text: String,
    #[validate(length(min = 1, max = 32, message = "template length must greater than 1"))]
    pub template: Option<String>,
    #[validate(length(min = 1, max = 16, message = "status length must greater than 1"))]
    pub status: String,
    #[validate(length(min = 1, max = 32, message = "password length must greater than 1"))]
    pub password: Option<String>,
    #[validate(length(min = 1, max = 1, message = "allowComment length must equal 1"))]
    pub allowComment: String,
    #[validate(length(min = 1, max = 1, message = "allowPing length must equal 1"))]
    pub allowPing: String,
    #[validate(length(min = 1, max = 1, message = "allowFeed length must equal 1"))]
    pub allowFeed: String,
}
