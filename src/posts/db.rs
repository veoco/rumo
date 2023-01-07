use std::time::SystemTime;

use super::models::{Post, PostCreate, PostWithMeta};
use crate::common::errors::FieldError;
use crate::AppState;

pub fn get_with_sql(state: &AppState) -> String {
    format!(
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
        )"#,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table,
        metas_table = &state.metas_table,
        fields_table = &state.fields_table,
    )
}

pub async fn get_post_by_slug(state: &AppState, slug: &str) -> Option<Post> {
    let select_sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."type" == 'post' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    if let Ok(tag) = sqlx::query_as::<_, Post>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Some(tag)
    } else {
        None
    }
}

pub async fn check_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<bool, FieldError> {
    let exist_sql = format!(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM {relationships_table}
            WHERE {relationships_table}."cid" == ?1 AND {relationships_table}."mid" == ?2
        )
        "#,
        relationships_table = &state.relationships_table,
    );
    match sqlx::query_scalar::<_, bool>(&exist_sql)
        .bind(cid)
        .bind(mid)
        .fetch_one(&state.pool)
        .await
    {
        Ok(b) => Ok(b),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn create_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<i64, FieldError> {
    let insert_sql = format!(
        r#"INSERT INTO {relationships_table} ("cid", "mid") VALUES (?1, ?2)"#,
        relationships_table = &state.relationships_table,
    );
    match sqlx::query(&insert_sql)
        .bind(cid)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<i64, FieldError> {
    let delete_sql = format!(
        r#"
        DELETE FROM {relationships_table}
        WHERE {relationships_table}."cid" == ?1 AND {relationships_table}."mid" == ?2"#,
        relationships_table = &state.relationships_table,
    );
    match sqlx::query(&delete_sql)
        .bind(cid)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn create_post_by_post_create_with_uid(
    state: &AppState,
    post_create: &PostCreate,
    uid: u32,
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let allow_comment = match post_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "status", "password", "allowComment", "allowPing", "allowFeed")
        VALUES ('post', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&insert_sql)
        .bind(&post_create.title)
        .bind(&post_create.slug)
        .bind(&post_create.created)
        .bind(now)
        .bind(&post_create.text)
        .bind(uid)
        .bind(&post_create.status)
        .bind(&post_create.password)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_post_by_post_create_with_exist_post(
    state: &AppState,
    post_modify: &PostCreate,
    exist_post: &Post,
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let now = if now > post_modify.created {
        now
    } else {
        post_modify.created
    };

    let allow_comment = match post_modify
        .allowComment
        .unwrap_or(exist_post.allowComment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_modify.allowPing.unwrap_or(exist_post.allowPing == "1") {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_modify.allowFeed.unwrap_or(exist_post.allowFeed == "1") {
        true => "1",
        false => "0",
    };

    let update_sql = format!(
        r#"
        UPDATE {contents_table}
        SET "title" = ?1,
            "slug" = ?2,
            "created" = ?3,
            "modified" = ?4,
            "text" = ?5,
            "status" = ?6,
            "password" = ?7,
            "allowComment" = ?8,
            "allowPing" = ?9,
            "allowFeed" = ?10
        WHERE "cid" == ?11
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&update_sql)
        .bind(&post_modify.title)
        .bind(&post_modify.slug)
        .bind(&post_modify.created)
        .bind(now)
        .bind(&post_modify.text)
        .bind(&post_modify.status)
        .bind(&post_modify.password)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .bind(exist_post.cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_posts_count_by_filter(state: &AppState, filter_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}."type" == 'post'{}
        "#,
        filter_sql,
        contents_table = &state.contents_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_posts_by_filter_and_list_query(
    state: &AppState,
    filter_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<PostWithMeta>, FieldError> {
    let with_sql = get_with_sql(state);
    let sql = format!(
        r#"
        {with_sql}
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
        filter_sql,
        order_by,
        contents_table = &state.contents_table,
        users_table = &state.users_table
    );

    match sqlx::query_as::<_, PostWithMeta>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(posts) => Ok(posts),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_post_by_slug_and_private(
    state: &AppState,
    slug: &str,
    private_sql: &str,
) -> Result<PostWithMeta, FieldError> {
    let with_sql = get_with_sql(state);
    let sql = format!(
        r#"
        {with_sql}
        SELECT {contents_table}.*, "tags", "categories", "fields", {users_table}."screenName", {users_table}."group"
        FROM {contents_table}
        LEFT OUTER JOIN categories_json ON {contents_table}."cid" == categories_json."cid"
        LEFT OUTER JOIN tags_json ON {contents_table}."cid" == tags_json."cid"
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        LEFT OUTER JOIN {users_table} ON {contents_table}.authorId == {users_table}."uid"
        WHERE {contents_table}."type" == 'post' AND {contents_table}."slug" == ?1{}"#,
        private_sql,
        contents_table = &state.contents_table,
        users_table = &state.users_table
    );

    match sqlx::query_as::<_, PostWithMeta>(&sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(post) => Ok(post),
        Err(_) => Err(FieldError::InvalidParams("slug".to_string())),
    }
}
