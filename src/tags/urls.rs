use axum::{
    routing::{get, post, delete},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn tags_routers(ro: bool) -> Router<Arc<AppState>> {
    let tags_route = Router::new()
        .route("/api/tags/", get(views::list_tags))
        .route("/api/tags/:slug", get(views::get_tag_by_slug))
        .route("/api/tags/:slug/posts/", get(views::list_tag_posts_by_slug));
    if !ro {
        tags_route
            .route("/api/tags/", post(views::create_tag))
            .route("/api/tags/:slug/posts/", post(views::add_post_to_tag))
            .route("/api/tags/:slug/posts/:post_slug", delete(views::delete_post_from_tag))
    } else {
        tags_route
    }
}
