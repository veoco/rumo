use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn categories_routers() -> Router<Arc<AppState>> {
    let categories_route = Router::new().route("/api/categories/", get(views::list_categories));
    categories_route
}
