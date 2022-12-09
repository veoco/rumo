use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views::{list_users, login_for_access_token, register};
use crate::AppState;

pub fn auth_routers() -> Router<Arc<AppState>> {
    let auth_route = Router::new()
        .route("/api/users/", get(list_users))
        .route("/api/users/token", post(login_for_access_token))
        .route("/api/users", post(register));
    auth_route
}
