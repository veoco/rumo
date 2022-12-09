use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn auth_routers() -> Router<Arc<AppState>> {
    let auth_route = Router::new()
        .route("/api/users/", get(views::list_users))
        .route("/api/users/:uid", get(views::get_user_by_id))
        .route("/api/users/token", post(views::login_for_access_token))
        .route("/api/users", post(views::register));
    auth_route
}
