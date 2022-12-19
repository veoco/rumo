use axum::extract::State;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{CategoriesQuery, Category, CategoryCreate};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(category_create): ValidatedJson<CategoryCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (SELECT 1 FROM typecho_metas WHERE slug == ?1)
        "#,
    )
    .bind(&category_create.slug)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(false);

    if exist {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let category_parent = match category_create.parent {
        Some(p) => p,
        _ => 0,
    };
    if let Ok(r) = sqlx::query(
        r#"
        INSERT INTO typecho_metas (type, name, slug, description, parent) VALUES ("category", ?1, ?2, ?3, ?4)"#,
    )
    .bind(category_create.name)
    .bind(category_create.slug)
    .bind(category_create.description)
    .bind(category_parent)
    .execute(&state.pool)
    .await
    {
        return Ok(Json(json!({"id": r.last_insert_rowid()})));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<CategoriesQuery>,
) -> Json<Value> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_metas
        WHERE type == category
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
        WHERE type == "category"
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by
    );

    if let Ok(categories) = sqlx::query_as::<_, Category>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        return Json(json!({
            "page": q.page,
            "page_size": q.page_size,
            "all_count": all_count,
            "count": categories.len(),
            "results": categories
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
