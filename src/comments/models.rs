#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub coid: u32,
    pub cid: u32,
    pub created: u32,
    pub author: Option<String>,
    pub authorId: u32,
    pub ownerId: u32,
    pub mail: Option<String>,
    pub url: Option<String>,
    pub ip: Option<String>,
    pub agent: Option<String>,
    pub text: String,
    pub r#type: String,
    pub status: String,
    pub parent: u32,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CommentCreate {
    #[validate(length(min = 1, max = 150, message = "author can not be longer than 32"))]
    pub author: Option<String>,
    #[validate(email)]
    pub mail: Option<String>,
    #[validate(url)]
    pub url: Option<String>,
    pub text: String,
    #[validate(range(min = 0, message = "parent must greater than 0"))]
    pub parent: Option<u32>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CommentsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
}
