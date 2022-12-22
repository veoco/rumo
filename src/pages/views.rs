use axum::extract::{Path, State};
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::SystemTime;

use super::models::{Page, PageCreate, PagesQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    PMEditor(user): PMEditor,
    ValidatedJson(page_create): ValidatedJson<PageCreate>,
) -> Result<Json<Value>, FieldError> {
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
        return Ok(Json(json!({"id": r.last_insert_rowid()})));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<PagesQuery>,
) -> Json<Value> {
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

    let offset = (q.page - 1) * q.page_size;
    let order_by = match q.order_by.as_str() {
        "-cid" => "cid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        _ => "cid",
    };
    let sql = format!(
        r#"
        SELECT *
        FROM typecho_contents
        WHERE type == "page"
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by
    );

    if let Ok(pages) = sqlx::query_as::<_, Page>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        return Json(json!({
            "page": q.page,
            "page_size": q.page_size,
            "all_count": all_count,
            "count": pages.len(),
            "results": pages
        }));
    }
    Json(json!({
        "page": q.page,
        "page_size": q.page_size,
        "all_count": all_count,
        "count": 0,
        "results": []
    }))
}

pub async fn get_page_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    if let Ok(target_page) = sqlx::query_as::<_, Page>(
        r#"
            SELECT *
            FROM typecho_contents
            WHERE type == "page" AND slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        return Ok(Json(json!(target_page)));
    }

    Err(FieldError::PermissionDeny)
}
