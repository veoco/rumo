use std::time::SystemTime;

use super::models::{FieldCreate, Page, PageCreate, PageWithMeta};
use crate::users::errors::FieldError;
use crate::AppState;

pub fn get_with_sql(state: &AppState) -> String {
    format!(
        r#"
        WITH fields_json AS (
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
        "#,
        contents_table = &state.contents_table,
        fields_table = &state.fields_table,
    )
}

pub async fn get_page_by_slug(state: &AppState, slug: &str) -> Option<Page> {
    let select_sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."type" == 'page' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    if let Ok(page) = sqlx::query_as::<_, Page>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Some(page)
    } else {
        None
    }
}

pub async fn create_page_by_page_create_with_uid(
    state: &AppState,
    page_create: &PageCreate,
    uid: u32,
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let status = match page_create.publish.unwrap_or(true) {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId", "template", "status", "allowComment", "allowPing", "allowFeed")
        VALUES ('page', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&insert_sql)
        .bind(&page_create.title)
        .bind(&page_create.slug)
        .bind(&page_create.created)
        .bind(now)
        .bind(&page_create.text)
        .bind(uid)
        .bind(&page_create.template)
        .bind(status)
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

pub async fn modify_page_by_page_modify_with_exist_page(
    state: &AppState,
    page_modify: &PageCreate,
    exist_page: &Page,
) -> Result<i64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let now = if now > page_modify.created {
        now
    } else {
        page_modify.created
    };

    let status = match page_modify
        .publish
        .unwrap_or(exist_page.status == "publish")
    {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_modify
        .allowComment
        .unwrap_or(exist_page.allowComment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_modify.allowPing.unwrap_or(exist_page.allowPing == "1") {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_modify.allowFeed.unwrap_or(exist_page.allowFeed == "1") {
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
            "template" = ?6,
            "status" = ?7,
            "allowComment" = ?8,
            "allowPing" = ?9,
            "allowFeed" = ?10
        WHERE "cid" == ?11
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query(&update_sql)
        .bind(&page_modify.title)
        .bind(&page_modify.slug)
        .bind(&page_modify.created)
        .bind(now)
        .bind(&page_modify.text)
        .bind(&page_modify.template)
        .bind(status)
        .bind(allow_comment)
        .bind(allow_ping)
        .bind(allow_feed)
        .bind(exist_page.cid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_pages_count_with_private(state: &AppState, private_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}.type == 'page'{}
        "#,
        private_sql,
        contents_table = &state.contents_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_pages_by_list_query_with_private(
    state: &AppState,
    private_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<PageWithMeta>, FieldError> {
    let with_sql = get_with_sql(state);
    let sql = format!(
        r#"
        {with_sql}
        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page'{}
        ORDER BY {}
        LIMIT ?1 OFFSET ?2
        "#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, PageWithMeta>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(pages) => Ok(pages),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_page_with_meta_by_slug(
    state: &AppState,
    slug: &str,
) -> Result<PageWithMeta, FieldError> {
    let with_sql = get_with_sql(state);
    let select_sql = format!(
        r#"
        {with_sql}
        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        WHERE {contents_table}."type" == 'page' AND {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, PageWithMeta>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(page) => Ok(page),
        Err(_) => Err(FieldError::InvalidParams("slug".to_string())),
    }
}

pub async fn create_field_by_cid_with_field_create(
    state: &AppState,
    cid: u32,
    field_create: &FieldCreate,
) -> Result<i64, FieldError> {
    let str_value;
    let int_value;
    let float_value;
    let field_type = match field_create.r#type.as_str() {
        "str" => {
            let value = field_create.str_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams("type and str_value".to_string()));
            }
            str_value = Some(value.unwrap());
            int_value = 0;
            float_value = 0f32;
            "str"
        }
        "int" => {
            let value = field_create.int_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams("type and int_value".to_string()));
            }
            str_value = None;
            int_value = value.unwrap();
            float_value = 0f32;
            "int"
        }
        "float" => {
            let value = field_create.float_value.clone();
            if value.is_none() {
                return Err(FieldError::InvalidParams(
                    "type and float_value".to_string(),
                ));
            }
            str_value = None;
            int_value = 0;
            float_value = value.unwrap();
            "float"
        }
        _ => return Err(FieldError::InvalidParams("type".to_string())),
    };

    let insert_sql = format!(
        r#"
        INSERT INTO {fields_table} ("cid", "name", "type", "str_value", "int_value", "float_value")
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        fields_table = &state.fields_table,
    );
    match sqlx::query(&insert_sql)
        .bind(cid)
        .bind(&field_create.name)
        .bind(field_type)
        .bind(str_value)
        .bind(int_value)
        .bind(float_value)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
