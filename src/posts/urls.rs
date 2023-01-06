use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn posts_routers(ro: bool) -> Router<Arc<AppState>> {
    let posts_route = Router::new()
        .route("/api/posts/", get(views::list_posts))
        .route("/api/posts/:slug", get(views::get_post_by_slug));
    if !ro {
        posts_route
            .route("/api/posts/", post(views::create_post))
            .route("/api/posts/:slug", patch(views::modify_page_by_slug))
            .route("/api/posts/:slug", delete(views::delete_post_by_slug))
    } else {
        posts_route
    }
}
