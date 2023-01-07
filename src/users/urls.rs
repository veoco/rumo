use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn users_routers(ro: bool) -> Router<Arc<AppState>> {
    let users_route = Router::new()
        .route("/api/users/", get(views::list_users))
        .route("/api/users/:uid", get(views::get_user_by_id));
    if !ro {
        users_route
            .route("/api/users/:uid", patch(views::modify_user_by_id))
            .route("/api/users/:uid", delete(views::delete_user_by_id))
            .route("/api/users/token", post(views::login_for_access_token))
            .route("/api/users", post(views::register))
    } else {
        users_route
    }
}
