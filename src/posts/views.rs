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
    let exist_sql = format!(
        r#"
        SELECT EXISTS (SELECT 1 FROM {contents_table} WHERE {contents_table}."slug" == ?1)
        "#,
        contents_table = &state.contents_table,
    );
    let exist = sqlx::query_scalar::<_, bool>(&exist_sql)
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

    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "template", "status", "password", "allowComment", "allowPing", "allowFeed")
        VALUES ('post', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        contents_table = &state.contents_table,
    );
    if let Ok(r) = sqlx::query(&insert_sql)
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
        return Ok((
            StatusCode::CREATED,
            Json(json!({"id": r.last_insert_rowid()})),
        ));
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
        String::from("")
    } else {
        format!(
            r#" AND {contents_table}."status" == 'publish' AND {contents_table}."password" IS NULL"#,
            contents_table = &state.contents_table,
        )
    };

    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}."type" == 'post'{}
        "#,
        private_sql,
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
            
        SELECT {contents_table}.*, "tags", "categories", "fields", {users_table}."screenName", {users_table}."group"
        FROM {contents_table}
        LEFT OUTER JOIN categories_json ON {contents_table}."cid" == categories_json."cid"
        LEFT OUTER JOIN tags_json ON {contents_table}."cid" == tags_json."cid"
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        LEFT OUTER JOIN {users_table} ON {contents_table}."authorId" == {users_table}."uid"
        WHERE {contents_table}."type" == 'post'{}
        GROUP BY {contents_table}."cid"
        ORDER BY {contents_table}.{}
        LIMIT ?1 OFFSET ?2"#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table,
        metas_table = &state.metas_table,
        fields_table = &state.fields_table,
        users_table = &state.users_table
    );

    match sqlx::query_as::<_, PostWithMeta>(&sql)
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

pub async fn get_post_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostQuery>,
) -> Result<Json<Value>, FieldError> {
    let admin = user.group == "editor" || user.group == "administrator";
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        String::from("")
    } else {
        format!(
            r#" AND ({contents_table}."status" == 'publish' OR {contents_table}."status" == 'password' OR {contents_table}."status" == 'hidden')"#,
            contents_table = &state.contents_table,
        )
    };

    let sql = format!(
        r#"
        WITH categories_json AS (
            SELECT {contents_table}.cid,
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
            JOIN {metas_table} ON {relationships_table}.mid == {metas_table}.mid
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
            JOIN {metas_table} ON {relationships_table}.mid == {metas_table}.mid
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

        SELECT {contents_table}.*, "tags", "categories", "fields", {users_table}."screenName", {users_table}."group"
        FROM {contents_table}
        LEFT OUTER JOIN categories_json ON {contents_table}."cid" == categories_json."cid"
        LEFT OUTER JOIN tags_json ON {contents_table}."cid" == tags_json."cid"
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        LEFT OUTER JOIN {users_table} ON {contents_table}.authorId == {users_table}."uid"
        WHERE {contents_table}."type" == 'post' AND {contents_table}."slug" == ?1{}"#,
        private_sql,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table,
        metas_table = &state.metas_table,
        fields_table = &state.fields_table,
        users_table = &state.users_table
    );

    if let Ok(target_post) = sqlx::query_as::<_, PostWithMeta>(&sql)
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
