use std::time::SystemTime;

use super::models::{Page, PageCreate, PageWithMeta};
use crate::users::errors::FieldError;
use crate::AppState;

pub fn get_with_sql(state: &AppState) -> String {
    format!(
        r#"
        WITH fields_json AS (
            SELECT {contents_table}."cid",
                json_group_array(json_object(
                    'name', {fields_table}."name",
                    'type', {fields_table}."type",
                    'str_value', {fields_table}."str_value",
                    'int_value', {fields_table}."int_value",
                    'float_value', {fields_table}."float_value"
                )) AS "fields"
            FROM {contents_table}
            JOIN {fields_table} ON {contents_table}."cid" == {fields_table}."cid"
            WHERE {contents_table}."type" == 'post'
            GROUP BY {contents_table}."cid"
        )
        "#,
        contents_table = &state.contents_table,
        fields_table = &state.fields_table,
    )
}

pub async fn get_page_by_slug(state: &AppState, slug: &str) -> Option<Page> {
    let select_sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."type" == 'page' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    if let Ok(tag) = sqlx::query_as::<_, Page>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Some(tag)
    } else {
        None
    }
}

pub async fn create_page_by_page_create_with_uid(
    state: &AppState,
    page_create: &PageCreate,
    uid: u32,
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;

    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "template", "status", "password", "allowComment", "allowPing", "allowFeed")
        VALUES ('page', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&insert_sql)
        .bind(&page_create.title)
        .bind(&page_create.slug)
        .bind(&page_create.created)
        .bind(now)
        .bind(&page_create.text)
        .bind(uid)
        .bind(&page_create.template)
        .bind(&page_create.status)
        .bind(&page_create.password)
        .bind(&page_create.allowComment)
        .bind(&page_create.allowPing)
        .bind(&page_create.allowFeed)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_pages_count_with_private(state: &AppState, private_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}.type == 'page'{}
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

pub async fn get_pages_by_list_query_with_private(
    state: &AppState,
    private_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<PageWithMeta>, FieldError> {
    let with_sql = get_with_sql(state);
    let sql = format!(
        r#"
        {with_sql}
        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page'{}
        ORDER BY {}
        LIMIT ?1 OFFSET ?2
        "#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, PageWithMeta>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(pages) => Ok(pages),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_page_with_meta_by_slug(state: &AppState, slug: &str)-> Result<PageWithMeta, FieldError>{
    let with_sql = get_with_sql(state);
    let select_sql = format!(
        r#"
        {with_sql}
        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, PageWithMeta>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(page) => Ok(page),
        Err(_) => Err(FieldError::InvalidParams("slug".to_string())),
    }
}
