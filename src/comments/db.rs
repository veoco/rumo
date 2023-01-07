use std::time::SystemTime;

use super::models::Comment;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn get_comment_by_coid(state: &AppState, coid: u32) -> Option<Comment> {
    let sql = format!(
        r#"
        SELECT *
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment' AND {comments_table}."coid" == ?1
        "#,
        comments_table = &state.comments_table
    );
    match sqlx::query_as::<_, Comment>(&sql)
        .bind(coid)
        .fetch_one(&state.pool)
        .await
    {
        Ok(comment) => Some(comment),
        Err(_) => None,
    }
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
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let insert_sql = format!(
        r#"
        INSERT INTO {comments_table} ("type", "cid", "created", "author", "authorId", "ownerId", "mail", "url", "ip", "agent", "text", "status", "parent")
        VALUES ('comment', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        comments_table = &state.comments_table
    );
    match sqlx::query(&insert_sql)
        .bind(cid)
        .bind(now as u32)
        .bind(author)
        .bind(author_id)
        .bind(owner_id)
        .bind(mail)
        .bind(url)
        .bind(ip)
        .bind(ua)
        .bind(text)
        .bind(status)
        .bind(parent)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_content_count_increase_by_cid(
    state: &AppState,
    cid: u32,
) -> Result<i64, FieldError> {
    let sql = format!(
        r#"
        UPDATE {contents_table}
        SET commentsNum=commentsNum+1
        WHERE "coid" == ?3
        "#,
        contents_table = &state.contents_table
    );
    match sqlx::query(&sql)
        .bind(cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_content_count_decrease_by_cid(
    state: &AppState,
    cid: u32,
) -> Result<i64, FieldError> {
    let sql = format!(
        r#"
        UPDATE {contents_table}
        SET commentsNum=commentsNum-1
        WHERE "coid" == ?3
        "#,
        contents_table = &state.contents_table
    );
    match sqlx::query(&sql)
        .bind(cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_comment_with_params(
    state: &AppState,
    coid: u32,
    text: &str,
    status: &str,
) -> Result<i64, FieldError> {
    let sql = format!(
        r#"
        UPDATE {comments_table}
        SET "text" = ?1, "status" = ?2
        WHERE "coid" == ?3
        "#,
        comments_table = &state.comments_table
    );
    match sqlx::query(&sql)
        .bind(text)
        .bind(status)
        .bind(coid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_comment_by_coid(
    state: &AppState,
    coid: u32,
) -> Result<i64, FieldError> {
    let sql = format!(
        r#"
        DELETE FROM {comments_table}
        WHERE "coid" == ?1
        "#,
        comments_table = &state.comments_table
    );
    match sqlx::query(&sql)
        .bind(coid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_comments_count(state: &AppState) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment'
        "#,
        comments_table = &state.comments_table
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_comments_by_list_query(
    state: &AppState,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<Comment>, FieldError> {
    let sql = format!(
        r#"            
        SELECT *
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment'
        ORDER BY {comments_table}.{}
        LIMIT ?1 OFFSET ?2"#,
        order_by,
        comments_table = &state.comments_table
    );
    match sqlx::query_as::<_, Comment>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(comments) => Ok(comments),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_comments_count_with_private(state: &AppState, private_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment' AND {comments_table}."cid" == ?1{}
        "#,
        private_sql,
        comments_table = &state.comments_table
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_comments_by_cid_and_list_query_with_private(
    state: &AppState,
    cid: u32,
    private_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<Comment>, FieldError> {
    let sql = format!(
        r#"            
        SELECT *
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment' AND {comments_table}."cid" == ?1{}
        ORDER BY {comments_table}.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql,
        order_by,
        comments_table = &state.comments_table
    );
    match sqlx::query_as::<_, Comment>(&sql)
        .bind(cid)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(comments) => Ok(comments),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
