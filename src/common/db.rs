use sqlx::any::AnyKind;

use super::forms::FieldCreate;
use super::utils::get_field_params;
use crate::common::errors::FieldError;
use crate::common::models::{Content, ContentWithMetasUsersFields, Field, Meta};
use crate::users::db as user_db;
use crate::AppState;

pub async fn get_content_by_cid(state: &AppState, cid: i32) -> Option<Content> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "cid" = $1
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(cid)
        .fetch_one(&state.pool)
        .await
    {
        Ok(content) => Some(content),
        Err(_) => None,
    }
}

pub async fn get_content_by_slug(state: &AppState, slug: &str) -> Option<Content> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = $1
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            WHERE "slug" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    if let Ok(content) = sqlx::query_as::<_, Content>(&sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Some(content)
    } else {
        None
    }
}

pub async fn get_contents_count_with_private(
    state: &AppState,
    private_sql: &str,
    content_type: &str,
) -> i32 {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT COUNT(*)
            FROM {contents_table}
            WHERE "type" = '{content_type}'{private_sql}
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            SELECT COUNT(*)
            FROM {contents_table}
            WHERE "type" = '{content_type}'{private_sql}
            "#,
            contents_table = &state.contents_table,
        ),
    };
    let all_count = sqlx::query_scalar::<_, i32>(&sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_content_with_metas_user_from_content(
    state: &AppState,
    content: Content,
) -> Result<ContentWithMetasUsersFields, FieldError> {
    let user = user_db::get_user_by_uid(state, content.authorId).await;
    if user.is_none() {
        return Err(FieldError::DatabaseFailed("Invalid user".to_string()));
    }
    let user = user.unwrap();

    let fields = get_fields_by_cid(state, content.cid).await;
    let metas = get_metas_by_cid(state, content.cid).await;

    let mut categories = vec![];
    let mut tags = vec![];
    for m in metas {
        if m.r#type == "tag" {
            tags.push(m);
        } else {
            categories.push(m);
        }
    }

    let mut content_with = ContentWithMetasUsersFields::from(content);
    content_with.fields = fields;
    content_with.categories = categories;
    content_with.tags = tags;
    content_with.screenName = user.screenName;
    content_with.group = user.group;
    Ok(content_with)
}

pub async fn get_contents_with_metas_user_and_fields_by_mid_list_query_and_private(
    state: &AppState,
    mid: i32,
    private_sql: &str,
    page_size: i32,
    offset: i32,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithMetasUsersFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}.cid = {relationships_table}.cid
            WHERE "type" = '{content_type}' AND "mid" = $1{private_sql}
            GROUP BY {contents_table}.cid
            ORDER BY {contents_table}.{order_by}
            LIMIT $2 OFFSET $3"#,
            contents_table = &state.contents_table,
            relationships_table = &state.relationships_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}.cid = {relationships_table}.cid
            WHERE "type" = '{content_type}' AND "mid" = ?{private_sql}
            GROUP BY {contents_table}.cid
            ORDER BY {contents_table}.{order_by}
            LIMIT ? OFFSET ?"#,
            contents_table = &state.contents_table,
            relationships_table = &state.relationships_table,
        ),
    };
    match sqlx::query_as::<_, Content>(&sql)
        .bind(mid)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(contents) => {
            let mut res = vec![];
            for c in contents {
                let content_with = get_content_with_metas_user_from_content(state, c).await?;
                res.push(content_with);
            }
            Ok(res)
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_content_by_cid(state: &AppState, cid: i32) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {contents_table}
            WHERE "cid" = $1
            "#,
            contents_table = &state.contents_table,
        ),
        _ => format!(
            r#"
            DELETE FROM {contents_table}
            WHERE "cid" = ?
            "#,
            contents_table = &state.contents_table,
        ),
    };
    match sqlx::query(&sql).bind(cid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn check_relationship_by_cid_and_mid(
    state: &AppState,
    cid: i32,
    mid: i32,
) -> Result<bool, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM {relationships_table}
                WHERE "cid" = $1 AND "mid" = $2
            )
            "#,
            relationships_table = &state.relationships_table,
        ),
        _ => format!(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM {relationships_table}
                WHERE "cid" = ? AND "mid" = ?
            )
            "#,
            relationships_table = &state.relationships_table,
        ),
    };
    match sqlx::query_scalar::<_, bool>(&sql)
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
    cid: i32,
    mid: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"INSERT INTO {relationships_table} ("cid", "mid") VALUES ($1, $2)"#,
            relationships_table = &state.relationships_table,
        ),
        _ => format!(
            r#"INSERT INTO {relationships_table} ("cid", "mid") VALUES (?, ?)"#,
            relationships_table = &state.relationships_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(cid)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_relationship_by_cid_and_mid(
    state: &AppState,
    cid: i32,
    mid: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {relationships_table}
            WHERE "cid" = $1 AND "mid" = $2"#,
            relationships_table = &state.relationships_table,
        ),
        _ => format!(
            r#"
            DELETE FROM {relationships_table}
            WHERE "cid" = ? AND "mid" = ?"#,
            relationships_table = &state.relationships_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(cid)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_relationships_by_mid(state: &AppState, mid: i32) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {relationships_table}
            WHERE "mid" = $1
            "#,
            relationships_table = &state.relationships_table
        ),
        _ => format!(
            r#"
            DELETE FROM {relationships_table}
            WHERE "mid" = ?
            "#,
            relationships_table = &state.relationships_table
        ),
    };
    match sqlx::query(&sql).bind(mid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_field_by_cid_and_name(state: &AppState, cid: i32, name: &str) -> Option<Field> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {fields_table}
            WHERE "cid" = $1 AND "name" = $2
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {fields_table}
            WHERE "cid" = ? AND "name" = ?
            "#,
            fields_table = &state.fields_table,
        ),
    };
    if let Ok(field) = sqlx::query_as::<_, Field>(&sql)
        .bind(cid)
        .bind(name)
        .fetch_one(&state.pool)
        .await
    {
        Some(field)
    } else {
        None
    }
}

pub async fn get_fields_by_cid(state: &AppState, cid: i32) -> Vec<Field> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {fields_table}
            WHERE "cid" = $1
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {fields_table}
            WHERE "cid" = ?
            "#,
            fields_table = &state.fields_table,
        ),
    };
    match sqlx::query_as::<_, Field>(&sql)
        .bind(cid)
        .fetch_all(&state.pool)
        .await
    {
        Ok(fields) => fields,
        Err(_) => vec![],
    }
}

pub async fn create_field_by_cid_with_field_create(
    state: &AppState,
    cid: i32,
    field_create: &FieldCreate,
) -> Result<u64, FieldError> {
    let (field_type, str_value, int_value, float_value) = get_field_params(&field_create)?;

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {fields_table} ("cid", "name", "type", "str_value", "int_value", "float_value")
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            INSERT INTO {fields_table} ("cid", "name", "type", "str_value", "int_value", "float_value")
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            fields_table = &state.fields_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(cid)
        .bind(&field_create.name)
        .bind(&field_type)
        .bind(str_value)
        .bind(int_value)
        .bind(float_value)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_field_by_cid_and_name_with_field_create(
    state: &AppState,
    cid: i32,
    name: &str,
    field_create: &FieldCreate,
) -> Result<u64, FieldError> {
    let (field_type, str_value, int_value, float_value) = get_field_params(&field_create)?;

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {fields_table}
            SET "name" = $1,
                "type" = $2,
                "str_value" = $3,
                "int_value" = $4,
                "float_value" = $5
            WHERE "cid" = $6 and "name" = $7
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            UPDATE {fields_table}
            SET "name" = ?,
                "type" = ?,
                "str_value" = ?,
                "int_value" = ?,
                "float_value" = ?
            WHERE "cid" = ? and "name" = ?
            "#,
            fields_table = &state.fields_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(&field_create.name)
        .bind(&field_type)
        .bind(str_value)
        .bind(int_value)
        .bind(float_value)
        .bind(cid)
        .bind(name)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_field_by_cid_and_name(
    state: &AppState,
    cid: i32,
    name: &str,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {fields_table}
            WHERE "cid" = $1 AND "name" = $2
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            DELETE FROM {fields_table}
            WHERE "cid" = ? AND "name" = ?
            "#,
            fields_table = &state.fields_table,
        ),
    };
    match sqlx::query(&sql)
        .bind(cid)
        .bind(name)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_fields_by_cid(state: &AppState, cid: i32) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {fields_table}
            WHERE "cid" = $1
            "#,
            fields_table = &state.fields_table,
        ),
        _ => format!(
            r#"
            DELETE FROM {fields_table}
            WHERE "cid" = ?
            "#,
            fields_table = &state.fields_table,
        ),
    };
    match sqlx::query(&sql).bind(cid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_meta_by_mid(state: &AppState, mid: i32) -> Option<Meta> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "mid" = $1
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "mid" = ?
            "#,
            metas_table = &state.metas_table,
        ),
    };
    if let Ok(meta) = sqlx::query_as::<_, Meta>(&sql)
        .bind(mid)
        .fetch_one(&state.pool)
        .await
    {
        Some(meta)
    } else {
        None
    }
}

pub async fn get_meta_by_slug(state: &AppState, slug: &str, tag: bool) -> Option<Meta> {
    let meta_type = if tag { "tag" } else { "category" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "type" = '{meta_type}' AND "slug" = $1
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "type" = '{meta_type}' AND "slug" = ?
            "#,
            metas_table = &state.metas_table,
        ),
    };
    match sqlx::query_as::<_, Meta>(&sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(meta) => Some(meta),
        Err(_) => None,
    }
}

pub async fn get_metas_by_cid(state: &AppState, cid: i32) -> Vec<Meta> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {metas_table}
            JOIN {relationships_table} ON {metas_table}.mid = {relationships_table}.mid
            WHERE "cid" = $1
            "#,
            metas_table = &state.metas_table,
            relationships_table = &state.relationships_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {metas_table}
            JOIN {relationships_table} ON {metas_table}.mid = {relationships_table}.mid
            WHERE "cid" = ?
            "#,
            metas_table = &state.metas_table,
            relationships_table = &state.relationships_table,
        ),
    };
    match sqlx::query_as::<_, Meta>(&sql)
        .bind(cid)
        .fetch_all(&state.pool)
        .await
    {
        Ok(metas) => metas,
        Err(_) => vec![],
    }
}

pub async fn get_metas_by_list_query(
    state: &AppState,
    page_size: i32,
    offset: i32,
    order_by: &str,
    tag: bool,
) -> Result<Vec<Meta>, FieldError> {
    let meta_type = if tag { "tag" } else { "category" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "type" = '{meta_type}'
            ORDER BY {order_by}
            LIMIT $1 OFFSET $2
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {metas_table}
            WHERE "type" = '{meta_type}'
            ORDER BY {order_by}
            LIMIT ? OFFSET ?
            "#,
            metas_table = &state.metas_table,
        ),
    };
    match sqlx::query_as::<_, Meta>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(metas) => Ok(metas),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_metas_count(state: &AppState, tag: bool) -> i32 {
    let meta_type = if tag { "tag" } else { "category" };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT COUNT(*)
            FROM {metas_table}
            WHERE "type" = '{meta_type}'
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            SELECT COUNT(*)
            FROM {metas_table}
            WHERE "type" = '{meta_type}'
            "#,
            metas_table = &state.metas_table,
        ),
    };
    let all_count = sqlx::query_scalar::<_, i32>(&sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_meta_posts_count_by_mid_with_private(
    state: &AppState,
    mid: i32,
    private_sql: &str,
) -> i32 {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT COUNT(*)
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}.cid = {relationships_table}.cid
            WHERE "type" = 'post' AND "mid" = $1{private_sql}
            "#,
            contents_table = &state.contents_table,
            relationships_table = &state.relationships_table
        ),
        _ => format!(
            r#"
            SELECT COUNT(*)
            FROM {contents_table}
            JOIN {relationships_table} ON {contents_table}.cid = {relationships_table}.cid
            WHERE "type" = 'post' AND "mid" = ?{private_sql}
            "#,
            contents_table = &state.contents_table,
            relationships_table = &state.relationships_table
        ),
    };
    let all_count = sqlx::query_scalar::<_, i32>(&sql)
        .bind(mid)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn update_meta_by_mid_for_increase_count(
    state: &AppState,
    mid: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {metas_table}
            SET "count" = "count" + 1
            WHERE "mid" = $1
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            UPDATE {metas_table}
            SET "count" = "count" + 1
            WHERE "mid" = ?
            "#,
            metas_table = &state.metas_table,
        ),
    };
    match sqlx::query(&sql).bind(mid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_meta_by_mid_for_decrease_count(
    state: &AppState,
    mid: i32,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {metas_table}
            SET "count" = "count" - 1
            WHERE "mid" = $1
            "#,
            metas_table = &state.metas_table,
        ),
        _ => format!(
            r#"
            UPDATE {metas_table}
            SET "count" = "count" - 1
            WHERE "mid" = ?
            "#,
            metas_table = &state.metas_table,
        ),
    };
    match sqlx::query(&sql).bind(mid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn delete_meta_by_mid(state: &AppState, mid: i32) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {metas_table}
            WHERE "mid" = $1
            "#,
            metas_table = &state.metas_table
        ),
        _ => format!(
            r#"
            DELETE FROM {metas_table}
            WHERE "mid" = ?
            "#,
            metas_table = &state.metas_table
        ),
    };
    match sqlx::query(&sql).bind(mid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
