use super::models::Attachment;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn create_attachment_with_params(
    state: &AppState,
    name: &str,
    now: i32,
    text: &str,
    uid: i32,
) -> Result<u64, FieldError> {
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
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_attachments_count_by_list_query(
    state: &AppState,
    private_sql: &str,
    page_size: i32,
    offset: i32,
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
