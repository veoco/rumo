use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn tags_routers() -> Router<Arc<AppState>> {
    let tags_route = Router::new()
        .route("/api/tags/", get(views::list_tags).post(views::create_tag))
        .route("/api/tags/:slug", get(views::get_tag_by_slug))
        .route(
            "/api/tags/:slug/posts/",
            post(views::add_post_to_tag).get(views::list_tag_posts_by_slug),
        );
    tags_route
}
