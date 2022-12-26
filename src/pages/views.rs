use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::SystemTime;

use super::models::{PageCreate, PageWithMeta, PagesQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    PMEditor(user): PMEditor,
    ValidatedJson(page_create): ValidatedJson<PageCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_sql = format!(
        r#"
        SELECT EXISTS (SELECT 1 FROM {contents_table} WHERE {contents_table}."slug" == ?1)
        "#,
        contents_table = &state.contents_table,
    );
    let exist = sqlx::query_scalar::<_, bool>(&exist_sql)
        .bind(&page_create.slug)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(false);

    if exist {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

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
    if let Ok(r) = sqlx::query(&insert_sql)
        .bind(page_create.title)
        .bind(page_create.slug)
        .bind(page_create.created)
        .bind(now)
        .bind(page_create.text)
        .bind(user.uid)
        .bind(page_create.template)
        .bind(page_create.status)
        .bind(page_create.password)
        .bind(page_create.allowComment)
        .bind(page_create.allowPing)
        .bind(page_create.allowFeed)
        .execute(&state.pool)
        .await
    {
        return Ok((
            StatusCode::CREATED,
            Json(json!({"id": r.last_insert_rowid()})),
        ));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<PagesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}.type == 'page'
        "#,
        contents_table = &state.contents_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "cid" => "cid",
        "-cid" => "cid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let sql = format!(
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

        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page'
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by,
        contents_table = &state.contents_table,
        fields_table = &state.fields_table,
    );
    match sqlx::query_as::<_, PageWithMeta>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(pages) => {
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
                "all_count": all_count,
                "count": pages.len(),
                "results": pages
            })))
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_page_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let select_sql = format!(
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

        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page' AND {contents_table}."slug" == ?1"#,
        contents_table = &state.contents_table,
        fields_table = &state.fields_table,
    );
    if let Ok(target_page) = sqlx::query_as::<_, PageWithMeta>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        return Ok(Json(json!(target_page)));
    }

    Err(FieldError::InvalidParams("slug".to_string()))
}
