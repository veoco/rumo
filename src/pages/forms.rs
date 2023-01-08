#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct PageCreate {
    #[validate(length(min = 1, max = 150, message = "title length must greater than 1"))]
    pub title: String,
    #[validate(length(min = 1, max = 150, message = "slug length must greater than 1"))]
    pub slug: String,
    pub created: i32,
    pub text: String,
    #[validate(length(min = 1, max = 32, message = "template length must greater than 1"))]
    pub template: Option<String>,
    pub publish: Option<bool>,
    pub allowComment: Option<bool>,
    pub allowPing: Option<bool>,
    pub allowFeed: Option<bool>,
}
