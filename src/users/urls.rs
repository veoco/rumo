use std::sync::Arc;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use super::views;
use crate::AppState;

pub fn users_routers(ro: bool) -> Router<Arc<AppState>> {
    let users_route = Router::new()
        .route("/api/users/", get(views::list_users))
        .route("/api/users/:uid", get(views::get_user_by_id))
        .route("/api/users/:uid/options/", get(views::list_options))
        .route("/api/users/:uid/options/:name", get(views::get_option_by_uid_and_name));
    if !ro {
        users_route
            .route("/api/users/:uid", patch(views::modify_user_by_id))
            .route("/api/users/:uid", delete(views::delete_user_by_id))
            .route("/api/users/:uid/options/", post(views::create_option_by_option_create))
            .route("/api/users/:uid/options/:name", patch(views::modify_option_by_uid_and_name))
            .route("/api/users/:uid/options/:name", delete(views::delete_option_by_uid_and_name))
            .route("/api/users/token", post(views::login_for_access_token))
            .route("/api/users/", post(views::register))
    } else {
        users_route
    }
}
