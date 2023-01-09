use sqlx::any::AnyKind;
use std::time::SystemTime;

use super::forms::PageCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::models::{Content, ContentWithFields};
use crate::AppState;

pub async fn create_page_by_page_create_with_uid(
    state: &AppState,
    page_create: &PageCreate,
    uid: i32,
) -> Result<u64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let status = match page_create.publish.unwrap_or(true) {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "template", "status", "allowComment", "allowPing", "allowFeed")
            VALUES ('page', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "template", "status", "allowComment", "allowPing", "allowFeed")
            VALUES ('page', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(&page_create.title)
        .bind(&page_create.slug)
        .bind(&page_create.created)
        .bind(now)
        .bind(&page_create.text)
        .bind(uid)
        .bind(&page_create.template)
        .bind(status)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_page_by_page_modify_with_exist_page(
    state: &AppState,
    page_modify: &PageCreate,
    exist_page: &Content,
) -> Result<u64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let now = if now > page_modify.created {
        now
    } else {
        page_modify.created
    };

    let status = match page_modify
        .publish
        .unwrap_or(exist_page.status == "publish")
    {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_modify
        .allowComment
        .unwrap_or(exist_page.allowComment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_modify.allowPing.unwrap_or(exist_page.allowPing == "1") {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_modify.allowFeed.unwrap_or(exist_page.allowFeed == "1") {
        true => "1",
        false => "0",
    };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {contents_table}
            SET "title" = $1,
                "slug" = $2,
                "created" = $3,
                "modified" = $4,
                "text" = $5,
                "template" = $6,
                "status" = $7,
                "allowComment" = $8,
                "allowPing" = $9,
                "allowFeed" = $10
            WHERE "cid" = $11
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            UPDATE {contents_table}
            SET "title" = ?,
                "slug" = ?,
                "created" = ?,
                "modified" = ?,
                "text" = ?,
                "template" = ?,
                "status" = ?,
                "allowComment" = ?,
                "allowPing" = ?,
                "allowFeed" = ?
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(&page_modify.title)
        .bind(&page_modify.slug)
        .bind(&page_modify.created)
        .bind(now)
        .bind(&page_modify.text)
        .bind(&page_modify.template)
        .bind(status)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .bind(exist_page.cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_content_with_fields_by_slug(
    state: &AppState,
    slug: &str,
) -> Result<ContentWithFields, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = $1
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(content) => {
            let fields = common_db::get_fields_by_cid(state, content.cid).await;
            let mut content_with_fields = ContentWithFields::from(content);
            content_with_fields.fields = fields;
            Ok(content_with_fields)
        }
        Err(_) => Err(FieldError::InvalidParams("slug".to_string())),
    }
}

pub async fn get_contents_with_fields_by_list_query_with_private(
    state: &AppState,
    private_sql: &str,
    page_size: i32,
    offset: i32,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = '{content_type}'{private_sql}
            ORDER BY {order_by}
            LIMIT $1 OFFSET $2
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = '{content_type}'{private_sql}
            ORDER BY {order_by}
            LIMIT ? OFFSET ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(contents) => {
            let mut res = vec![];
            for c in contents {
                let fields = common_db::get_fields_by_cid(state, c.cid).await;
                let mut content_with_fields = ContentWithFields::from(c);
                content_with_fields.fields = fields;
                res.push(content_with_fields);
            }
            Ok(res)
        }
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
