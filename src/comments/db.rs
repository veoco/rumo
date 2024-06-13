use std::time::SystemTime;

use sea_orm::*;

use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::entity::{comment, comment::Entity as Comment, content};
use crate::AppState;

pub async fn get_comment_by_coid(
    state: &AppState,
    coid: u32,
) -> Result<Option<comment::Model>, FieldError> {
    Comment::find()
        .filter(comment::Column::Coid.eq(coid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("coid".to_string()))
}

pub async fn create_comment_with_params(
    state: &AppState,
    cid: u32,
    author: &str,
    author_id: u32,
    owner_id: u32,
    mail: &str,
    url: Option<String>,
    ip: &str,
    ua: &str,
    text: &str,
    status: &str,
    parent: u32,
) -> Result<comment::ActiveModel, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    comment::ActiveModel {
        cid: Set(cid),
        created: Set(now as u32),
        author: Set(Some(author.to_owned())),
        author_id: Set(author_id),
        owner_id: Set(owner_id),
        mail: Set(Some(mail.to_owned())),
        url: Set(url),
        ip: Set(Some(ip.to_owned())),
        agent: Set(Some(ua.to_owned())),
        text: Set(Some(text.to_owned())),
        status: Set(status.to_owned()),
        parent: Set(parent),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("insert comment failed".to_string()))
}

pub async fn update_content_count_increase_by_cid(
    state: &AppState,
    cid: u32,
) -> Result<content::Model, FieldError> {
    let exist_content = match common_db::get_content_by_cid(state, cid).await? {
        Some(c) => c,
        None => return Err(FieldError::InvalidParams("cid".to_string())),
    };

    let count = exist_content.comments_num;
    let mut c = content::ActiveModel::from(exist_content);
    c.comments_num = Set(count + 1);
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update content failed".to_string()))
}

pub async fn update_content_count_decrease_by_cid(
    state: &AppState,
    cid: u32,
) -> Result<content::Model, FieldError> {
    let exist_content = match common_db::get_content_by_cid(state, cid).await? {
        Some(c) => c,
        None => return Err(FieldError::InvalidParams("cid".to_string())),
    };

    let count = exist_content.comments_num;
    let mut c = content::ActiveModel::from(exist_content);
    c.comments_num = Set(count - 1);
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update content failed".to_string()))
}

pub async fn modify_comment_with_params(
    state: &AppState,
    coid: u32,
    text: &str,
    status: &str,
) -> Result<comment::Model, FieldError> {
    let exist_comment = match get_comment_by_coid(state, coid).await? {
        Some(c) => c,
        None => return Err(FieldError::InvalidParams("coid".to_string())),
    };

    let mut c = comment::ActiveModel::from(exist_comment);
    c.text = Set(Some(text.to_owned()));
    c.status = Set(status.to_owned());
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update comment failed".to_string()))
}

pub async fn delete_comment_by_coid(
    state: &AppState,
    coid: u32,
) -> Result<DeleteResult, FieldError> {
    let exist_comment = match get_comment_by_coid(state, coid).await? {
        Some(c) => c,
        None => return Err(FieldError::InvalidParams("coid".to_string())),
    };

    exist_comment
        .delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete comment failed".to_string()))
}

pub async fn get_comments_count(state: &AppState) -> u64 {
    Comment::find().count(&state.conn).await.unwrap_or(0)
}

pub async fn get_comments_by_list_query(
    state: &AppState,
    page_size: u64,
    page: u64,
    order_by: &str,
) -> Result<Vec<comment::Model>, FieldError> {
    let stmt = Comment::find();

    let stmt = match order_by {
        "-coid" => stmt.order_by_desc(comment::Column::Coid),
        "coid" => stmt.order_by_asc(comment::Column::Coid),
        "-created" => stmt.order_by_desc(comment::Column::Created),
        "created" => stmt.order_by_asc(comment::Column::Created),
        _ => stmt.order_by_desc(comment::Column::Coid),
    };
    let paginator = stmt.paginate(&state.conn, page_size);
    paginator
        .fetch_page(page)
        .await
        .map_err(|_| FieldError::DatabaseFailed("get comments by list query failed".to_string()))
}

pub async fn get_content_comments_count_by_cid_with_private(
    state: &AppState,
    cid: u32,
    private: bool,
) -> u64 {
    let stmt = Comment::find().filter(comment::Column::Cid.eq(cid));

    let stmt = if private {
        stmt
    } else {
        stmt.filter(comment::Column::Status.eq("approved"))
    };

    stmt.count(&state.conn).await.unwrap_or(0)
}

pub async fn get_comments_by_cid_and_list_query_with_private(
    state: &AppState,
    cid: u32,
    private: bool,
    page_size: u64,
    page: u64,
    order_by: &str,
) -> Result<Vec<comment::Model>, FieldError> {
    let stmt = Comment::find().filter(comment::Column::Cid.eq(cid));

    let stmt = if private {
        stmt
    } else {
        stmt.filter(comment::Column::Status.eq("approved"))
    };

    let stmt = match order_by {
        "-coid" => stmt.order_by_desc(comment::Column::Coid),
        "coid" => stmt.order_by_asc(comment::Column::Coid),
        "-created" => stmt.order_by_desc(comment::Column::Created),
        "created" => stmt.order_by_asc(comment::Column::Created),
        _ => stmt.order_by_desc(comment::Column::Coid),
    };
    let paginator = stmt.paginate(&state.conn, page_size);
    paginator
        .fetch_page(page - 1)
        .await
        .map_err(|_| FieldError::DatabaseFailed("get comments by list query failed".to_string()))
}
