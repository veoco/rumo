use axum::extract::{Path, State};
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{Tag, TagCreate, TagsQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(tag_create): ValidatedJson<TagCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (SELECT 1 FROM typecho_metas WHERE slug == ?1)
        "#,
    )
    .bind(&tag_create.slug)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if exist {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let tag_parent = match tag_create.parent {
        Some(p) => p,
        _ => 0,
    };
    if let Ok(r) = sqlx::query(
        r#"
        INSERT INTO typecho_metas (type, name, slug, description, parent) VALUES ("tag", ?1, ?2, ?3, ?4)"#,
    )
    .bind(tag_create.name)
    .bind(tag_create.slug)
    .bind(tag_create.description)
    .bind(tag_parent)
    .execute(&state.pool)
    .await
    {
        return Ok(Json(json!({"id": r.last_insert_rowid()})));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<TagsQuery>,
) -> Json<Value> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_metas
        WHERE type == tag
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let offset = (q.page - 1) * q.page_size;
    let order_by = match q.order_by.as_str() {
        "-mid" => "mid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        _ => "mid",
    };
    let sql = format!(
        r#"
        SELECT *
        FROM typecho_metas
        WHERE type == "tag"
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by
    );

    if let Ok(tags) = sqlx::query_as::<_, Tag>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        return Json(json!({
            "page": q.page,
            "page_size": q.page_size,
            "all_count": all_count,
            "count": tags.len(),
            "results": tags
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

pub async fn get_tag_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    if let Ok(target_tag) = sqlx::query_as::<_, Tag>(
        r#"
            SELECT *
            FROM typecho_metas
            WHERE type == "tag" AND slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        return Ok(Json(json!(target_tag)));
    }

    Err(FieldError::PermissionDeny)
}
