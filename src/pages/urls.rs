use axum::{routing::get, Router};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn pages_routers() -> Router<Arc<AppState>> {
    let pages_route = Router::new()
        .route(
            "/api/pages/",
            get(views::list_pages).post(views::create_page),
        )
        .route("/api/pages/:slug", get(views::get_page_by_slug));
    pages_route
}
