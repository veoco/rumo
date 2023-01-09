use sqlx::any::AnyKind;
use std::time::SystemTime;

use super::forms::PostCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::models::{Content, ContentWithMetasUsersFields};
use crate::AppState;

pub async fn create_post_by_post_create_with_uid(
    state: &AppState,
    post_create: &PostCreate,
    uid: i32,
) -> Result<u64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let allow_comment = match post_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "status", "password", "allowComment", "allowPing", "allowFeed")
            VALUES ('post', $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            INSERT INTO {contents_table} (`type`, `title`, `slug`, `created`, `modified`, `text`, `authorId`, `status`, `password`, `allowComment`, `allowPing`, `allowFeed`)
            VALUES ('post', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "status", "password", "allowComment", "allowPing", "allowFeed")
            VALUES ('post', ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(&post_create.title)
        .bind(&post_create.slug)
        .bind(&post_create.created)
        .bind(now)
        .bind(&post_create.text)
        .bind(uid)
        .bind(&post_create.status)
        .bind(&post_create.password)
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

pub async fn modify_post_by_post_create_with_exist_post(
    state: &AppState,
    post_modify: &PostCreate,
    exist_post: &Content,
) -> Result<u64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let now = if now > post_modify.created {
        now
    } else {
        post_modify.created
    };

    let allow_comment = match post_modify
        .allowComment
        .unwrap_or(exist_post.allowComment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_modify.allowPing.unwrap_or(exist_post.allowPing == "1") {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_modify.allowFeed.unwrap_or(exist_post.allowFeed == "1") {
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
                "status" = $6,
                "password" = $7,
                "allowComment" = $8,
                "allowPing" = $9,
                "allowFeed" = $10
            WHERE "cid" = $11
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {contents_table}
            SET `title` = ?,
                `slug` = ?,
                `created` = ?,
                `modified` = ?,
                `text` = ?,
                `status` = ?,
                `password` = ?,
                `allowComment` = ?,
                `allowPing` = ?,
                `allowFeed` = ?
            WHERE `cid` = ?
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
                "status" = ?,
                "password" = ?,
                "allowComment" = ?,
                "allowPing" = ?,
                "allowFeed" = ?
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(&post_modify.title)
        .bind(&post_modify.slug)
        .bind(&post_modify.created)
        .bind(now)
        .bind(&post_modify.text)
        .bind(&post_modify.status)
        .bind(&post_modify.password)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .bind(exist_post.cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_contents_with_metas_user_and_fields_by_filter_and_list_query(
    state: &AppState,
    filter_sql: &str,
    page_size: i32,
    offset: i32,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithMetasUsersFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = '{content_type}'{filter_sql}
            GROUP BY "cid"
            ORDER BY {order_by}
            LIMIT $1 OFFSET $2"#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE `type` = '{content_type}'{filter_sql}
            GROUP BY `cid`
            ORDER BY {order_by}
            LIMIT ? OFFSET ?"#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = '{content_type}'{filter_sql}
            GROUP BY "cid"
            ORDER BY {order_by}
            LIMIT ? OFFSET ?"#,
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
                let content_with =
                    common_db::get_content_with_metas_user_from_content(state, c).await?;
                res.push(content_with);
            }
            Ok(res)
        }
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_content_with_metas_user_fields_by_slug_and_private(
    state: &AppState,
    slug: &str,
    private_sql: &str,
) -> Result<ContentWithMetasUsersFields, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = $1{private_sql}"#,
            contents_table = &state.contents_table
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE `slug` = ?{private_sql}"#,
            contents_table = &state.contents_table
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = ?{private_sql}"#,
            contents_table = &state.contents_table
        ),
    };

    match sqlx::query_as::<_, Content>(&sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(c) => {
            let content_with =
                common_db::get_content_with_metas_user_from_content(state, c).await?;
            Ok(content_with)
        }
        Err(_) => Err(FieldError::InvalidParams("slug".to_string())),
    }
}
