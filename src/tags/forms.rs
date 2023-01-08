#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct TagCreate {
    #[validate(length(min = 1, max = 150, message = "name can not be longer than 150"))]
    pub name: String,
    #[validate(length(min = 1, max = 150, message = "slug can not be longer than 150"))]
    pub slug: String,
    #[validate(length(max = 150, message = "description can not be longer than 150"))]
    pub description: Option<String>,
    #[validate(range(min = 0, message = "parent must greater than 0"))]
    pub parent: Option<i32>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct TagPostAdd {
    #[validate(length(min = 1, max = 150, message = "slug can not be longer than 150"))]
    pub slug: String,
}
