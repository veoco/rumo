use sea_orm::*;

use super::forms::FieldCreate;
use super::utils::get_field_params;
use crate::common::errors::FieldError;
use crate::common::models::ContentWithMetasUsersFields;
use crate::entity::{
    content, content::Entity as Content, field, field::Entity as ContentField, meta,
    meta::Entity as Meta, relationship, relationship::Entity as Relationship, user,
};
use crate::AppState;

pub async fn get_content_by_cid(
    state: &AppState,
    cid: u32,
) -> Result<Option<content::Model>, FieldError> {
    Content::find()
        .filter(content::Column::Cid.eq(cid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("cid".to_string()))
}

pub async fn get_content_by_slug(
    state: &AppState,
    slug: &str,
) -> Result<Option<content::Model>, FieldError> {
    Content::find()
        .filter(content::Column::Slug.eq(slug))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("slug".to_string()))
}

pub async fn get_contents_count_with_private(
    state: &AppState,
    private: bool,
    own: bool,
    author: &user::Model,
    content_type: &str,
) -> u64 {
    let stmt = Content::find().filter(content::Column::Type.eq(content_type));
    let stmt = if own {
        stmt.filter(content::Column::AuthorId.eq(author.uid))
    } else {
        stmt
    };
    let stmt = if private {
        stmt
    } else {
        stmt.filter(content::Column::Status.eq("publish"))
    };
    stmt.count(&state.conn).await.unwrap_or(0)
}

pub async fn get_contents_with_metas_user_and_fields_by_mid_list_query_and_private(
    state: &AppState,
    mid: u32,
    private: bool,
    author: &user::Model,
    page_size: u64,
    page: u64,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithMetasUsersFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let stmt = Content::find().left_join(Meta).filter(
        meta::Column::Mid
            .eq(mid)
            .and(content::Column::Type.eq(content_type)),
    );

    let stmt = if private {
        stmt
    } else {
        stmt.filter(content::Column::Status.eq("publish"))
    };

    let stmt = match order_by {
        "-cid" => stmt.order_by_desc(content::Column::Cid),
        "cid" => stmt.order_by_asc(content::Column::Cid),
        "-slug" => stmt.order_by_desc(content::Column::Slug),
        "slug" => stmt.order_by_asc(content::Column::Slug),
        _ => stmt.order_by_desc(content::Column::Cid),
    };

    let paginator = stmt.paginate(&state.conn, page_size);

    let contents = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch contents failed".to_string()))?;

    let metas = contents
        .load_many_to_many(meta::Entity, relationship::Entity, &state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch metas failed".to_string()))?;

    let fields = contents
        .load_many(ContentField, &state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch fields failed".to_string()))?;

    let mut res = vec![];
    for ((content, meta_list), field_list) in contents
        .into_iter()
        .zip(metas.into_iter())
        .zip(fields.into_iter())
    {
        let mut c = ContentWithMetasUsersFields::from(content);
        c.screen_name = author.screen_name.clone();
        c.group = author.group.clone();

        let mut tags = vec![];
        let mut categories = vec![];
        for m in meta_list {
            if m.r#type == "tag" {
                tags.push(m);
            } else {
                categories.push(m);
            }
        }
        c.tags = tags;
        c.categories = categories;
        c.fields = field_list;
        res.push(c);
    }
    Ok(res)
}

pub async fn delete_content_by_cid(state: &AppState, cid: u32) -> Result<DeleteResult, FieldError> {
    let c: content::ActiveModel = get_content_by_cid(state, cid)
        .await?
        .ok_or(FieldError::InvalidParams("cid".to_string()))
        .map(Into::into)?;

    c.delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete content failed".to_string()))
}

pub async fn check_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<bool, FieldError> {
    Relationship::find()
        .filter(
            relationship::Column::Cid
                .eq(cid)
                .and(relationship::Column::Mid.eq(mid)),
        )
        .count(&state.conn)
        .await
        .map(|r| r > 0)
        .map_err(|_| FieldError::DatabaseFailed("check relationship failed".to_string()))
}

pub async fn create_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<relationship::Model, FieldError> {
    relationship::ActiveModel {
        cid: Set(cid),
        mid: Set(mid),
        ..Default::default()
    }
    .insert(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create relationship failed".to_string()))
}

pub async fn delete_relationship_by_cid_and_mid(
    state: &AppState,
    cid: u32,
    mid: u32,
) -> Result<DeleteResult, FieldError> {
    let r: relationship::ActiveModel = Relationship::find()
        .filter(
            relationship::Column::Cid
                .eq(cid)
                .and(relationship::Column::Mid.eq(mid)),
        )
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete relationship failed".to_string()))?
        .ok_or(FieldError::InvalidParams(
            "relationship not found".to_string(),
        ))
        .map(Into::into)?;

    r.delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete relationship failed".to_string()))
}

pub async fn delete_relationships_by_mid(
    state: &AppState,
    mid: u32,
) -> Result<DeleteResult, FieldError> {
    Relationship::delete_many()
        .filter(relationship::Column::Mid.eq(mid))
        .exec(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete relationships failed".to_string()))
}

pub async fn get_field_by_cid_and_name(
    state: &AppState,
    cid: u32,
    name: &str,
) -> Result<Option<field::Model>, FieldError> {
    ContentField::find()
        .filter(field::Column::Cid.eq(cid).and(field::Column::Name.eq(name)))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch field failed".to_string()))
}

pub async fn create_field_by_cid_with_field_create(
    state: &AppState,
    cid: u32,
    field_create: &FieldCreate,
) -> Result<field::Model, FieldError> {
    let (field_type, str_value, int_value, float_value) = get_field_params(&field_create)?;

    field::ActiveModel {
        cid: Set(cid),
        name: Set(field_create.name.to_owned()),
        r#type: Set(field_type),
        str_value: Set(str_value),
        int_value: Set(int_value),
        float_value: Set(float_value),
        ..Default::default()
    }
    .insert(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create field failed".to_string()))
}

pub async fn modify_field_by_cid_and_name_with_field_create(
    state: &AppState,
    cid: u32,
    name: &str,
    field_create: &FieldCreate,
) -> Result<field::Model, FieldError> {
    let (field_type, str_value, int_value, float_value) = get_field_params(&field_create)?;

    let exist_field = get_field_by_cid_and_name(state, cid as u32, name).await?;
    if exist_field.is_none() {
        return Err(FieldError::InvalidParams("name".to_owned()));
    }
    let exist_field = exist_field.unwrap();

    let mut f = field::ActiveModel::from(exist_field);
    f.r#type = Set(field_type);
    f.str_value = Set(str_value);
    f.int_value = Set(int_value);
    f.float_value = Set(float_value);
    f.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("modify field failed".to_string()))
}

pub async fn delete_field_by_cid_and_name(
    state: &AppState,
    cid: u32,
    name: &str,
) -> Result<DeleteResult, FieldError> {
    let f = get_field_by_cid_and_name(state, cid, name).await?;

    if f.is_none() {
        return Err(FieldError::InvalidParams("name".to_owned()));
    }
    let f = f.unwrap();

    f.delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed(format!("delete field {} failed", name)))
}

pub async fn delete_fields_by_cid(state: &AppState, cid: u32) -> Result<DeleteResult, FieldError> {
    ContentField::delete_many()
        .filter(field::Column::Cid.eq(cid))
        .exec(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete fields failed".to_string()))
}

pub async fn get_meta_by_mid(
    state: &AppState,
    mid: u32,
) -> Result<Option<meta::Model>, FieldError> {
    Meta::find()
        .filter(meta::Column::Mid.eq(mid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch meta failed".to_string()))
}

pub async fn get_meta_by_slug(
    state: &AppState,
    slug: &str,
    tag: bool,
) -> Result<Option<meta::Model>, FieldError> {
    let meta_type = if tag { "tag" } else { "category" };

    Meta::find()
        .filter(
            meta::Column::Slug
                .eq(slug)
                .and(meta::Column::Type.eq(meta_type)),
        )
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch meta failed".to_string()))
}

pub async fn get_metas_by_list_query(
    state: &AppState,
    page_size: u64,
    page: u64,
    order_by: &str,
    tag: bool,
) -> Result<Vec<meta::Model>, FieldError> {
    let meta_type = if tag { "tag" } else { "category" };

    let stmt = Meta::find().filter(meta::Column::Type.eq(meta_type));

    let stmt = match order_by {
        "-mid" => stmt.order_by_desc(meta::Column::Mid),
        "mid" => stmt.order_by_asc(meta::Column::Mid),
        "-slug" => stmt.order_by_desc(meta::Column::Slug),
        "slug" => stmt.order_by_asc(meta::Column::Slug),
        _ => stmt.order_by_desc(meta::Column::Mid),
    };
    let paginator = stmt.paginate(&state.conn, page_size);

    paginator
        .fetch_page(page - 1)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch metas failed".to_string()))
}

pub async fn get_metas_count(state: &AppState, tag: bool) -> u64 {
    let meta_type = if tag { "tag" } else { "category" };

    Meta::find()
        .filter(meta::Column::Type.eq(meta_type))
        .count(&state.conn)
        .await
        .unwrap_or(0)
}

pub async fn get_meta_posts_count_by_mid_with_private(
    state: &AppState,
    mid: u32,
    private: bool,
) -> u64 {
    let stmt = Content::find()
        .left_join(Meta)
        .filter(meta::Column::Mid.eq(mid));

    if private {
        stmt.count(&state.conn).await.unwrap_or(0)
    } else {
        stmt.filter(content::Column::Status.eq("publish"))
            .count(&state.conn)
            .await
            .unwrap_or(0)
    }
}

pub async fn update_meta_by_mid_for_increase_count(
    state: &AppState,
    mid: u32,
) -> Result<meta::Model, FieldError> {
    let exist_meta = Meta::find()
        .filter(meta::Column::Mid.eq(mid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("mid".to_owned()))?;
    if exist_meta.is_none() {
        return Err(FieldError::InvalidParams("mid".to_owned()));
    }
    let exist_meta = exist_meta.unwrap();
    let count = exist_meta.count;

    let mut m = meta::ActiveModel::from(exist_meta);
    m.count = Set(count + 1);
    m.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update meta count failed".to_string()))
}

pub async fn update_meta_by_mid_for_decrease_count(
    state: &AppState,
    mid: u32,
) -> Result<meta::Model, FieldError> {
    let exist_meta = Meta::find()
        .filter(meta::Column::Mid.eq(mid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("mid".to_owned()))?;
    if exist_meta.is_none() {
        return Err(FieldError::InvalidParams("mid".to_owned()));
    }
    let exist_meta = exist_meta.unwrap();
    let count = exist_meta.count;

    let mut m = meta::ActiveModel::from(exist_meta);
    m.count = Set(count - 1);
    m.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update meta count failed".to_string()))
}

pub async fn delete_meta_by_mid(state: &AppState, mid: u32) -> Result<DeleteResult, FieldError> {
    let m = Meta::find()
        .filter(meta::Column::Mid.eq(mid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete meta failed".to_string()))?;
    if m.is_none() {
        return Err(FieldError::InvalidParams("mid".to_owned()));
    }
    let m = m.unwrap();
    m.delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete meta failed".to_string()))
}
