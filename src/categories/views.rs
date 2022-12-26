use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{CategoriesQuery, Category, CategoryCreate, CategoryPostAdd};
use crate::posts::models::{PostWithMeta, PostsQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(category_create): ValidatedJson<CategoryCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_sql = format!(
        r#"
        SELECT EXISTS (SELECT 1 FROM {metas_table} WHERE {metas_table}."slug" == ?1)
        "#,
        metas_table = &state.metas_table
    );
    let exist = sqlx::query_scalar::<_, bool>(&exist_sql)
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

    let insert_sql = format!(
        r#"
        INSERT INTO {metas_table} ("type", "name", "slug", "description", "parent")
        VALUES ('category', ?1, ?2, ?3, ?4)
        "#,
        metas_table = &state.metas_table
    );
    if let Ok(r) = sqlx::query(&insert_sql)
        .bind(category_create.name)
        .bind(category_create.slug)
        .bind(category_create.description)
        .bind(category_parent)
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

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<CategoriesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category'
        "#,
        metas_table = &state.metas_table
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
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
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category'
        ORDER BY {order_by}
        LIMIT ?1 OFFSET ?2
        "#,
        metas_table = &state.metas_table
    );

    match sqlx::query_as::<_, Category>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(categories) => {
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
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
    let select_sql = format!(
        r#"
        SELECT *
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category' AND {metas_table}."slug" == ?1
        "#,
        metas_table = &state.metas_table
    );
    if let Ok(target_category) = sqlx::query_as::<_, Category>(&select_sql)
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
    let mid_sql = format!(
        r#"
        SELECT mid
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category' AND {metas_table}."slug" == ?1
        "#,
        metas_table = &state.metas_table
    );
    let mid = match sqlx::query_scalar::<_, i32>(&mid_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(m) => m,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid_sql = format!(
        r#"
        SELECT cid
        FROM {contents_table}
        WHERE {contents_table}."type" == 'post' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table
    );
    let cid = match sqlx::query_scalar::<_, i32>(&cid_sql)
        .bind(category_post_add.slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(c) => c,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let exist_sql = format!(
        r#"SELECT EXISTS (SELECT 1 FROM {relationships_table} WHERE {relationships_table}."cid" == ?1 AND {relationships_table}."mid" == ?2)"#,
        relationships_table = &state.relationships_table
    );
    let exist = match sqlx::query_scalar::<_, bool>(&exist_sql)
        .bind(cid)
        .bind(mid)
        .fetch_one(&state.pool)
        .await
    {
        Ok(b) => b,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    if !exist {
        let insert_sql = format!(
            r#"
            INSERT INTO {relationships_table} ("cid", "mid")
            VALUES (?1, ?2)
            "#,
            relationships_table = &state.relationships_table
        );
        if let Ok(_) = sqlx::query(&insert_sql)
            .bind(cid)
            .bind(mid)
            .execute(&state.pool)
            .await
        {
            let update_sql = format!(
                r#"UPDATE {metas_table} SET count=count+1 WHERE {metas_table}."mid" == ?1"#,
                metas_table = &state.metas_table
            );
            let _ = sqlx::query(&update_sql)
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
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid_sql = format!(
        r#"
        SELECT mid
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category' AND {metas_table}."slug" == ?1
        "#,
        metas_table = &state.metas_table
    );
    let mid = match sqlx::query_scalar::<_, i32>(&mid_sql)
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
        String::from("")
    } else {
        format!(
            r#" AND {contents_table}."status" == 'publish' AND {contents_table}."password" IS NULL"#,
            contents_table = &state.contents_table
        )
    };

    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        JOIN {relationships_table} ON {contents_table}."cid" == {relationships_table}."cid"
        WHERE {contents_table}."type" == 'post' AND {relationships_table}."mid" == ?1{}
        "#,
        private_sql,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table
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
            SELECT {contents_table}."cid",
                json_group_array(json_object(
                    'mid', {metas_table}."mid",
                    'slug', {metas_table}."slug",
                    'type', 'category',
                    'name', {metas_table}."name",
                    'description', {metas_table}."description",
                    'count', {metas_table}."count",
                    'order', {metas_table}."order",
                    'parent', {metas_table}."parent"
                )) AS "categories"
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}."cid" == {relationships_table}."cid"
            JOIN {metas_table} ON {relationships_table}."mid" == {metas_table}."mid"
            WHERE {contents_table}."type" == 'post' AND {metas_table}."type" == 'category'
            GROUP BY {contents_table}."cid"
        ), tags_json AS (
            SELECT {contents_table}."cid",
                json_group_array(json_object(
                    'mid', {metas_table}."mid",
                    'slug', {metas_table}."slug",
                    'type', 'tag',
                    'name', {metas_table}."name",
                    'description', {metas_table}."description",
                    'count', {metas_table}."count",
                    'order', {metas_table}."order",
                    'parent', {metas_table}."parent"
                )) AS "tags"
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}."cid" == {relationships_table}."cid"
            JOIN {metas_table} ON {relationships_table}."mid" == {metas_table}."mid"
            WHERE {contents_table}."type" == 'post' AND {metas_table}."type" == 'tag'
            GROUP BY {contents_table}."cid"
        ), fields_json AS (
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
        LEFT OUTER JOIN categories_json ON {contents_table}."cid" == categories_json."cid"
        LEFT OUTER JOIN tags_json ON {contents_table}."cid" == tags_json."cid"
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        LEFT OUTER JOIN {users_table} ON {contents_table}.authorId == {users_table}."uid"
        JOIN {relationships_table} ON {contents_table}.cid == {relationships_table}."cid"
        WHERE {contents_table}."type" == 'post' AND mid == ?1{}
        GROUP BY {contents_table}."cid"
        ORDER BY {contents_table}.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table,
        metas_table = &state.metas_table,
        fields_table = &state.fields_table,
        users_table = &state.users_table
    );
    match sqlx::query_as::<_, PostWithMeta>(&sql)
        .bind(mid)
        .bind(page_size)
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
