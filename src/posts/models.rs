#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::categories::models::Category;
use crate::tags::models::Tag;

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct Post {
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

#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub struct PostWithMeta {
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
    pub categories: sqlx::types::Json<Vec<Category>>,
    pub tags: sqlx::types::Json<Vec<Tag>>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PostsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: u32,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: u32,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: String,
    pub with_meta: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PostCreate {
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
