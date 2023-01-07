use super::models::Attachment;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn create_attachment_with_params(
    state: &AppState,
    name: &str,
    now: u32,
    text: &str,
    uid: u32,
) -> Result<i64, FieldError> {
    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId")
        VALUES ('attachment', ?1, ?1, ?2, ?2, ?3, ?4)
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&insert_sql)
        .bind(name)
        .bind(now)
        .bind(text)
        .bind(uid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_attachment_by_cid(state: &AppState, cid: u32) -> Option<Attachment> {
    let sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."cid" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, Attachment>(&sql)
        .bind(cid)
        .fetch_one(&state.pool)
        .await
    {
        Ok(comment) => Some(comment),
        Err(_) => None,
    }
}

pub async fn delete_attachment_by_cid(state: &AppState, cid: u32) -> Result<i64, FieldError> {
    let sql = format!(
        r#"
        DELETE FROM {contents_table}
        WHERE {contents_table}."cid" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&sql).bind(cid).execute(&state.pool).await {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_attachments_count_with_private(state: &AppState, private_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}."type" == 'attachment'{}
        "#,
        private_sql,
        contents_table = &state.contents_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_attachments_count_by_list_query(
    state: &AppState,
    private_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<Attachment>, FieldError> {
    let sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."type" == 'attachment'{}
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, Attachment>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(attachements) => Ok(attachements),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
