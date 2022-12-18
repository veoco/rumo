use axum::extract::State;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{CategoriesQuery, Category};
use crate::users::extractors::ValidatedQuery;
use crate::AppState;

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
    let sql = r#"
        SELECT *
        FROM typecho_metas
        WHERE type == category
        LIMIT ?1 OFFSET ?2
        ORDER BY "#
        .to_string();
    let sql = sql
        + match q.order_by.as_str() {
            "-mid" => "mid DESC",
            "slug" => "slug",
            "-slug" => "slug DESC",
            _ => "mid",
        };

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
