use axum::{routing::get, Router};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn posts_routers() -> Router<Arc<AppState>> {
    let posts_route = Router::new()
        .route(
            "/api/posts/",
            get(views::list_posts).post(views::create_post),
        )
        .route("/api/posts/:slug", get(views::get_post_by_slug));
    posts_route
}
