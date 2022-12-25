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
    let exist = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (SELECT 1 FROM typecho_contents WHERE slug == ?1)
        "#,
    )
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

    if let Ok(r) = sqlx::query(
        r#"
        INSERT INTO typecho_contents (type, title, slug, created, modified, text, authorId, template, status, password, allowComment, allowPing, allowFeed)
        VALUES ("page", ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
    )
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
        return Ok((StatusCode::CREATED,Json(json!({"id": r.last_insert_rowid()}))));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<PagesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_contents
        WHERE type == "page"
        "#,
    )
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
            SELECT typecho_contents.cid,
                json_group_array(json_object(
                    'name', typecho_fields.name,
                    'type', typecho_fields."type",
                    'str_value', typecho_fields.str_value,
                    'int_value', typecho_fields.int_value,
                    'float_value', typecho_fields.float_value
                )) AS fields
            FROM typecho_contents
            JOIN typecho_fields ON typecho_contents.cid == typecho_fields.cid
            WHERE typecho_contents."type" == "post"
            GROUP BY typecho_contents.cid
        )

        SELECT *
        FROM typecho_contents
        WHERE type == "page"
        LEFT OUTER JOIN fields_json ON typecho_contents.cid == fields_json.cid
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by
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
    if let Ok(target_page) = sqlx::query_as::<_, PageWithMeta>(
        r#"
        WITH fields_json AS (
            SELECT typecho_contents.cid,
                json_group_array(json_object(
                    'name', typecho_fields.name,
                    'type', typecho_fields."type",
                    'str_value', typecho_fields.str_value,
                    'int_value', typecho_fields.int_value,
                    'float_value', typecho_fields.float_value
                )) AS fields
            FROM typecho_contents
            JOIN typecho_fields ON typecho_contents.cid == typecho_fields.cid
            WHERE typecho_contents."type" == "post"
            GROUP BY typecho_contents.cid
        )

        SELECT *
        FROM typecho_contents
        LEFT OUTER JOIN fields_json ON typecho_contents.cid == fields_json.cid
        WHERE type == "page" AND slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        return Ok(Json(json!(target_page)));
    }

    Err(FieldError::InvalidParams("slug".to_string()))
}
