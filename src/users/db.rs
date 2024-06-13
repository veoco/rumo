use std::time::SystemTime;

use sea_orm::*;

use super::forms::{OptionCreate, OptionModify, UserModify, UserRegister};
use super::utils::hash;
use crate::common::errors::FieldError;
use crate::entity::{option, option::Entity as UserOption, user, user::Entity as User};
use crate::AppState;

pub async fn get_user_by_mail(
    state: &AppState,
    mail: &str,
) -> Result<Option<user::Model>, FieldError> {
    User::find()
        .filter(user::Column::Mail.eq(mail))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("mail".to_string()))
}

pub async fn get_user_by_uid(
    state: &AppState,
    uid: u32,
) -> Result<Option<user::Model>, FieldError> {
    User::find()
        .filter(user::Column::Uid.eq(uid))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::InvalidParams("uid".to_string()))
}

pub async fn delete_user_by_uid(state: &AppState, uid: u32) -> Result<DeleteResult, FieldError> {
    let u = get_user_by_uid(state, uid).await?;

    if u.is_none() {
        return Err(FieldError::InvalidParams("uid".to_string()));
    }

    let u = u.unwrap();

    u.delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete user failed".to_string()))
}

pub async fn update_user_by_uid_for_activity(
    state: &AppState,
    uid: u32,
    now: u32,
) -> Result<user::Model, FieldError> {
    let mut u: user::ActiveModel = get_user_by_uid(state, uid)
        .await?
        .ok_or(FieldError::InvalidParams("uid".to_string()))
        .map(Into::into)?;

    u.activated = Set(now);
    u.logged = Set(now);
    u.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update user failed".to_string()))
}

pub async fn update_user_by_uid_with_user_modify_for_data_without_password(
    state: &AppState,
    uid: u32,
    user_modify: &UserModify,
) -> Result<user::Model, FieldError> {
    let mut u: user::ActiveModel = get_user_by_uid(state, uid)
        .await?
        .ok_or(FieldError::InvalidParams("uid".to_string()))
        .map(Into::into)?;

    u.name = Set(Some(user_modify.name.to_owned()));
    u.mail = Set(Some(user_modify.mail.to_owned()));
    u.url = Set(Some(user_modify.url.to_owned()));
    u.screen_name = Set(Some(user_modify.screenName.to_owned()));
    u.group = Set(user_modify.group.to_owned());
    u.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update user failed".to_string()))
}

pub async fn update_user_by_uid_for_password(
    state: &AppState,
    uid: u32,
    hashed_password: &str,
) -> Result<user::Model, FieldError> {
    let mut u: user::ActiveModel = get_user_by_uid(state, uid)
        .await?
        .ok_or(FieldError::InvalidParams("uid".to_string()))
        .map(Into::into)?;

    u.password = Set(Some(hashed_password.to_owned()));
    u.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update user failed".to_string()))
}

pub async fn create_user_with_user_register(
    state: &AppState,
    user_register: &UserRegister,
) -> Result<user::ActiveModel, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let hashed_password = hash(&user_register.password);

    user::ActiveModel {
        name: Set(Some(user_register.name.to_owned())),
        mail: Set(Some(user_register.mail.to_owned())),
        url: Set(Some(user_register.url.to_owned())),
        screen_name: Set(Some(user_register.name.to_owned())),
        password: Set(Some(hashed_password.to_owned())),
        created: Set(now),
        group: Set("subscriber".to_owned()),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create user failed".to_string()))
}

pub async fn get_users_count(state: &AppState) -> u64 {
    User::find().count(&state.conn).await.unwrap_or(0)
}

pub async fn get_users_by_list_query(
    state: &AppState,
    page_size: u64,
    page: u64,
    order_by: String,
) -> Result<(Vec<user::Model>, u64), FieldError> {
    let stmt = User::find();
    let stmt = match order_by.as_str() {
        "-uid" => stmt.order_by_desc(user::Column::Uid),
        "uid" => stmt.order_by_asc(user::Column::Uid),
        "-name" => stmt.order_by_desc(user::Column::Name),
        "name" => stmt.order_by_asc(user::Column::Name),
        "-mail" => stmt.order_by_desc(user::Column::Mail),
        "mail" => stmt.order_by_asc(user::Column::Mail),
        _ => stmt.order_by_asc(user::Column::Uid),
    };
    let paginator = stmt.paginate(&state.conn, page_size);
    let num_pages = paginator.num_pages().await.unwrap_or(0);

    paginator
        .fetch_page(page - 1)
        .await
        .map(|p| (p, num_pages))
        .map_err(|_| FieldError::DatabaseFailed("fetch user failed".to_string()))
}

pub async fn get_options_by_uid(
    state: &AppState,
    uid: u32,
) -> Result<Vec<option::Model>, FieldError> {
    UserOption::find()
        .filter(option::Column::User.eq(uid))
        .all(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch option failed".to_string()))
}

pub async fn get_option_by_uid_and_name(
    state: &AppState,
    uid: u32,
    name: &str,
) -> Result<Option<option::Model>, FieldError> {
    UserOption::find()
        .filter(option::Column::User.eq(uid))
        .filter(option::Column::Name.eq(name))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch option failed".to_string()))
}

pub async fn create_option_by_uid_with_option_create(
    state: &AppState,
    uid: u32,
    option_create: &OptionCreate,
) -> Result<InsertResult<option::ActiveModel>, FieldError> {
    let opt = option::Model {
        name: option_create.name.to_owned(),
        user: uid,
        value: Some(option_create.value.to_owned()),
    };

    UserOption::insert(opt.into_active_model()).exec(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create option failed".to_string()))
}

pub async fn modify_option_by_uid_and_name_with_option_modify(
    state: &AppState,
    uid: u32,
    name: &str,
    option_modify: &OptionModify,
) -> Result<option::Model, FieldError> {
    let user_option = get_option_by_uid_and_name(state, uid, name).await?;

    if user_option.is_none() {
        return Err(FieldError::InvalidParams("uid or name".to_string()));
    }
    let user_option = user_option.unwrap();

    let mut o = option::ActiveModel::from(user_option.clone());
    o.value = Set(Some(option_modify.value.to_owned()));
    o.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update option failed".to_string()))
}

pub async fn delete_option_by_uid_and_name(
    state: &AppState,
    uid: u32,
    name: &str,
) -> Result<DeleteResult, FieldError> {
    let user_option = get_option_by_uid_and_name(state, uid, name).await?;

    if user_option.is_none() {
        return Err(FieldError::InvalidParams("uid or name".to_string()));
    }
    let user_option = user_option.unwrap();

    user_option
        .delete(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("delete option failed".to_string()))
}
