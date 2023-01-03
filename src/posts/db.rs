use super::models::Post;
use crate::users::errors::FieldError;
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
