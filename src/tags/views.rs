use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{Tag, TagCreate, TagPostAdd, TagsQuery};
use crate::posts::models::{PostWithMeta, PostsQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(tag_create): ValidatedJson<TagCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
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
        return Ok((StatusCode::CREATED,Json(json!({"id": r.last_insert_rowid()}))));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<TagsQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_metas
        WHERE type == "tag"
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-mid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
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
        WHERE type == "tag"
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        order_by
    );

    match sqlx::query_as::<_, Tag>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(tags) => {
            return Ok(Json(json!({
                "page": q.page,
                "page_size": q.page_size,
                "all_count": all_count,
                "count": tags.len(),
                "results": tags
            })))
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
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

    Err(FieldError::InvalidParams("slug".to_string()))
}

pub async fn add_post_to_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(tag_post_add): ValidatedJson<TagPostAdd>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let mid = match sqlx::query_scalar::<_, i32>(
        r#"
            SELECT mid
            FROM typecho_metas
            WHERE type == "tag" AND slug == ?1
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
    .bind(tag_post_add.slug)
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

pub async fn list_tag_posts_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid = match sqlx::query_scalar::<_, i32>(
        r#"
            SELECT mid
            FROM typecho_metas
            WHERE type == "tag" AND slug == ?1
            "#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(m) => m,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        ""
    } else {
        r#" AND typecho_contents.status == "publish" AND typecho_contents.password IS NULL"#
    };

    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM typecho_contents
        JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
        WHERE type == "post" AND mid == ?1{}
        "#,
        private_sql
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .bind(mid)
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
        LEFT OUTER JOIN typecho_users ON typecho_contents.authorId == typecho_users.uid
        JOIN typecho_relationships ON typecho_contents.cid == typecho_relationships.cid
        WHERE typecho_contents."type" == "post" AND mid == ?1{}
        GROUP BY typecho_contents.cid
        ORDER BY typecho_contents.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql, order_by
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
}
