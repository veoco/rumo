#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use sqlx::{types::Json, FromRow};
use validator::Validate;

use crate::posts::models::Field;

#[derive(Serialize, Deserialize, FromRow)]
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

#[derive(Serialize, Deserialize, FromRow)]
pub struct PageWithMeta {
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
    pub fields: Option<Json<Vec<Field>>>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct PagesQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
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
    pub publish: Option<bool>,
    pub allowComment: Option<bool>,
    pub allowPing: Option<bool>,
    pub allowFeed: Option<bool>,
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
