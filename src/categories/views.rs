use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{CategoriesQuery, Category, CategoryCreate, CategoryPostAdd};
use crate::posts::models::{Post, PostWithMeta, PostsQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(category_create): ValidatedJson<CategoryCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
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
        return Ok((StatusCode::CREATED,Json(json!({"id": r.last_insert_rowid()}))));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<CategoriesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_metas
        WHERE type == "category"
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let offset = (q.page - 1) * q.page_size;
    let order_by = match q.order_by.as_str() {
        "mid" => "mid",
        "-mid" => "mid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
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

    match sqlx::query_as::<_, Category>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(categories) => {
            return Ok(Json(json!({
                "page": q.page,
                "page_size": q.page_size,
                "all_count": all_count,
                "count": categories.len(),
                "results": categories
            })))
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_category_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    if let Ok(target_category) = sqlx::query_as::<_, Category>(
        r#"
            SELECT *
            FROM typecho_metas
            WHERE type == "category" AND slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        return Ok(Json(json!(target_category)));
    }

    Err(FieldError::InvalidParams("slug".to_string()))
}

pub async fn add_post_to_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(category_post_add): ValidatedJson<CategoryPostAdd>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let mid = match sqlx::query_scalar::<_, i32>(
        r#"
            SELECT mid
            FROM typecho_metas
            WHERE type == "category" AND slug == ?1
            "#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(m) => m,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match sqlx::query_scalar::<_, i32>(
        r#"
            SELECT cid
            FROM typecho_contents
            WHERE type == "post" AND slug == ?1
            "#,
    )
    .bind(category_post_add.slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(c) => c,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let exist = match sqlx::query_scalar::<_, bool>(
        r#"SELECT EXISTS (SELECT 1 FROM typecho_relationships WHERE cid == ?1 AND mid == ?2)"#,
    )
    .bind(cid)
    .bind(mid)
    .fetch_one(&state.pool)
    .await
    {
        Ok(b) => b,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    if !exist {
        if let Ok(_) =
            sqlx::query(r#"INSERT INTO typecho_relationships (cid, mid) VALUES (?1, ?2)"#)
                .bind(cid)
                .bind(mid)
                .execute(&state.pool)
                .await
        {
            let _ = sqlx::query(r#"UPDATE typecho_metas SET count=count+1 WHERE mid == ?1"#)
                .bind(mid)
                .execute(&state.pool)
                .await;

            return Ok((StatusCode::CREATED, Json(json!({"msg": "ok"}))));
        }
    }

    return Err(FieldError::AlreadyExist("slug".to_string()));
}

pub async fn list_category_posts_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid = match sqlx::query_scalar::<_, i32>(
        r#"
            SELECT mid
            FROM typecho_metas
            WHERE type == "category" AND slug == ?1
            "#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(m) => m,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_contents
        JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
        WHERE type == "post" AND mid == ?1
        "#,
    )
    .bind(mid)
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let offset = (q.page - 1) * q.page_size;
    let order_by = match q.order_by.as_str() {
        "cid" => "cid",
        "-cid" => "cid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    if q.with_meta.unwrap_or(false) {
        let sql = format!(
            r#"
            WITH categories_json AS (
                SELECT typecho_contents.cid,
                    json_group_array(json_object(
                        'mid', typecho_metas.mid,
                        'slug', typecho_metas.slug,
                        'type', 'category',
                        'name', typecho_metas.name,
                        'description', typecho_metas.description,
                        'count', typecho_metas."count",
                        'order', typecho_metas."order",
                        'parent', typecho_metas.parent
                    )) AS categories
                FROM typecho_contents
                JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
                JOIN typecho_metas ON typecho_relationships.mid == typecho_metas.mid
                WHERE typecho_contents."type" == "post" AND typecho_metas."type" == "category"
                GROUP BY typecho_contents.cid
            ), tags_json AS (
                SELECT typecho_contents.cid,
                    json_group_array(json_object(
                        'mid', typecho_metas.mid,
                        'slug', typecho_metas.slug,
                        'type', 'tag',
                        'name', typecho_metas.name,
                        'description', typecho_metas.description,
                        'count', typecho_metas."count",
                        'order', typecho_metas."order",
                        'parent', typecho_metas.parent
                    )) AS tags
                FROM typecho_contents
                JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
                JOIN typecho_metas ON typecho_relationships.mid == typecho_metas.mid
                WHERE typecho_contents."type" == "post" AND typecho_metas."type" == "tag"
                GROUP BY typecho_contents.cid
            )
            
            SELECT *
            FROM typecho_contents
            LEFT OUTER JOIN categories_json ON typecho_contents.cid == categories_json.cid
            LEFT OUTER JOIN tags_json ON typecho_contents.cid == tags_json.cid
            JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
            WHERE typecho_contents."type" == "post" AND mid == ?1
            GROUP BY typecho_contents.cid
            ORDER BY typecho_contents.{}
            LIMIT ?2 OFFSET ?3"#,
            order_by
        );

        match sqlx::query_as::<_, PostWithMeta>(&sql)
            .bind(mid)
            .bind(q.page_size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
        {
            Ok(posts) => {
                return Ok(Json(json!({
                    "page": q.page,
                    "page_size": q.page_size,
                    "all_count": all_count,
                    "count": posts.len(),
                    "results": posts
                })))
            }
            Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
        }
    } else {
        let sql = format!(
            r#"
        SELECT *
        FROM typecho_contents
        JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
        WHERE type == "post" AND mid == ?1
        ORDER BY {}
        LIMIT ?2 OFFSET ?3"#,
            order_by
        );

        match sqlx::query_as::<_, Post>(&sql)
            .bind(mid)
            .bind(q.page_size)
            .bind(offset)
            .fetch_all(&state.pool)
            .await
        {
            Ok(posts) => {
                return Ok(Json(json!({
                    "page": q.page,
                    "page_size": q.page_size,
                    "all_count": all_count,
                    "count": posts.len(),
                    "results": posts
                })))
            }
            Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
        }
    }
}
