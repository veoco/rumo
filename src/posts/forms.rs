#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct PostsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<i32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<i32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
    pub own: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PostCreate {
    #[validate(length(min = 1, max = 150, message = "title length must greater than 1"))]
    pub title: String,
    #[validate(length(min = 1, max = 150, message = "slug length must greater than 1"))]
    pub slug: String,
    pub created: i32,
    pub text: String,
    #[validate(length(min = 1, max = 16, message = "status length must greater than 1"))]
    pub status: String,
    #[validate(length(min = 1, max = 32, message = "password length must greater than 1"))]
    pub password: Option<String>,
    pub allowComment: Option<bool>,
    pub allowPing: Option<bool>,
    pub allowFeed: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PostQuery {
    #[validate(length(min = 1, max = 32, message = "password length must greater than 1"))]
    pub password: Option<String>,
    pub private: Option<bool>,
}
