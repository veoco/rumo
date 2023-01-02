use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn pages_routers(ro: bool) -> Router<Arc<AppState>> {
    let pages_route = Router::new()
        .route("/api/pages/", get(views::list_pages))
        .route("/api/pages/:slug", get(views::get_page_by_slug));
    if !ro {
        pages_route.route("/api/pages/", post(views::create_page))
    } else {
        pages_route
    }
}
