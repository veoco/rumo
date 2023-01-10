use sqlx::any::AnyKind;

use crate::common::errors::FieldError;
use crate::common::models::Content;
use crate::AppState;

pub async fn create_attachment_with_params(
    state: &AppState,
    name: &str,
    now: i32,
    text: &str,
    uid: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId")
            VALUES ('attachment', $1, $2, $3, $4, $5, $6)
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            INSERT INTO {contents_table} (`type`, `title`, `slug`, `created`, `modified`, `text`, `authorId`)
            VALUES ('attachment', ?, ?, ?, ?, ?, ?)
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId")
            VALUES ('attachment', ?, ?, ?, ?, ?, ?)
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(name)
        .bind(name)
        .bind(now)
        .bind(now)
        .bind(text)
        .bind(uid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_attachment_by_cid_with_params(
    state: &AppState,
    cid: i32,
    name: &str,
    now: i32,
    text: &str,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {contents_table}
            SET "title" = $1,
                "slug" = $2,
                "modified" = $3,
                "text" = $4
            WHERE "cid" = $5
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {contents_table}
            SET `title` = ?,
                `slug` = ?,
                `modified` = ?,
                `text` = ?
            WHERE `cid` = ?
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            UPDATE {contents_table}
            SET "title" = ?,
                "slug" = ?,
                "modified" = ?,
                "text" = ?
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(name)
        .bind(name)
        .bind(now)
        .bind(text)
        .bind(cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_attachments_by_list_query(
    state: &AppState,
    private_sql: &str,
    page_size: i32,
    offset: i32,
    order_by: &str,
) -> Result<Vec<Content>, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = 'attachment'{}
            ORDER BY {}
            LIMIT $1 OFFSET $2"#,
            private_sql,
            order_by,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE `type` = 'attachment'{}
            ORDER BY {}
            LIMIT ? OFFSET ?"#,
            private_sql,
            order_by,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = 'attachment'{}
            ORDER BY {}
            LIMIT ? OFFSET ?"#,
            private_sql,
            order_by,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(attachements) => Ok(attachements),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_attachments_by_parent(
    state: &AppState,
    parent: i32,
) -> Result<Vec<Content>, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = 'attachment' AND "parent" = ?1
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE `type` = 'attachment' AND `parent` = ?
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "type" = 'attachment' AND "parent" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(parent)
        .fetch_all(&state.pool)
        .await
    {
        Ok(attachements) => Ok(attachements),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_attachment_parent_by_cid(
    state: &AppState,
    cid: i32,
    parent: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {contents_table}
            SET "parent" = $1,
            WHERE "cid" = $2
            "#,
            contents_table = &state.contents_table,
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {contents_table}
            SET `parent` = ?
            WHERE `cid` = ?
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            UPDATE {contents_table}
            SET "parent" = ?
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(parent)
        .bind(cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
