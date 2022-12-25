use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::SystemTime;

use super::models::{PostCreate, PostQuery, PostWithMeta, PostsQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMContributor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    ValidatedJson(post_create): ValidatedJson<PostCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS (SELECT 1 FROM typecho_contents WHERE slug == ?1)
        "#,
    )
    .bind(&post_create.slug)
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
        VALUES ("post", ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
    )
    .bind(post_create.title)
    .bind(post_create.slug)
    .bind(post_create.created)
    .bind(now)
    .bind(post_create.text)
    .bind(user.uid)
    .bind(post_create.template)
    .bind(post_create.status)
    .bind(post_create.password)
    .bind(post_create.allowComment)
    .bind(post_create.allowPing)
    .bind(post_create.allowFeed)
    .execute(&state.pool)
    .await
    {
        return Ok((StatusCode::CREATED,Json(json!({"id": r.last_insert_rowid()}))));
    }
    Err(FieldError::AlreadyExist("slug".to_owned()))
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
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
        WHERE type == "post"{}
        "#,
        private_sql
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
        ), fields_json AS (
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
            
        SELECT typecho_contents.*, tags, categories, fields, typecho_users.screenName, typecho_users."group"
        FROM typecho_contents
        LEFT OUTER JOIN categories_json ON typecho_contents.cid == categories_json.cid
        LEFT OUTER JOIN tags_json ON typecho_contents.cid == tags_json.cid
        LEFT OUTER JOIN fields_json ON typecho_contents.cid == fields_json.cid
        LEFT OUTER JOIN typecho_users ON typecho_contents.authorId == typecho_users.uid
        WHERE typecho_contents."type" == "post"{}
        GROUP BY typecho_contents.cid
        ORDER BY typecho_contents.{}
        LIMIT ?1 OFFSET ?2"#,
        private_sql, order_by
    );

    match sqlx::query_as::<_, PostWithMeta>(&sql)
        .bind(q.page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(posts) => {
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
                "all_count": all_count,
                "count": posts.len(),
                "results": posts
            })))
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_post_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostQuery>,
) -> Result<Json<Value>, FieldError> {
    let admin = user.group == "editor" || user.group == "administrator";

    if let Ok(target_post) = sqlx::query_as::<_, PostWithMeta>(
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
            ), fields_json AS (
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

            SELECT typecho_contents.*, tags, categories, fields, typecho_users.screenName, typecho_users."group"
            FROM typecho_contents
            LEFT OUTER JOIN categories_json ON typecho_contents.cid == categories_json.cid
            LEFT OUTER JOIN tags_json ON typecho_contents.cid == tags_json.cid
            LEFT OUTER JOIN fields_json ON typecho_contents.cid == fields_json.cid
            LEFT OUTER JOIN typecho_users ON typecho_contents.authorId == typecho_users.uid
            WHERE typecho_contents."type" == "post" AND slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        let status = &target_post.status;
        if admin || status == "publish" || status == "hidden" || status == "password" {
            if target_post.password.is_none() {
                return Ok(Json(json!(target_post)));
            }

            let password = target_post.password.clone().unwrap();
            if let Some(query_password) = q.password {
                if password == query_password {
                    return Ok(Json(json!(target_post)));
                }
            } else {
                return Err(FieldError::PasswordRequired);
            }
        } else {
            return Err(FieldError::PermissionDeny);
        }
    }

    Err(FieldError::InvalidParams("slug".to_string()))
}
