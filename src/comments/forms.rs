use serde::{Deserialize, Serialize};
use validator::Validate;

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
pub struct CommentModify {
    pub text: String,
    #[validate(length(min = 1, max = 16, message = "status can not be longer than 16"))]
    pub status: String,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct CommentsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u64>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u64>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
}
