use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn comments_routers(ro: bool) -> Router<Arc<AppState>> {
    let comments_route = Router::new()
        .route("/api/comments/", get(views::list_comments))
        .route(
            "/api/pages/:slug/comments/",
            get(views::list_page_comments_by_slug),
        )
        .route(
            "/api/posts/:slug/comments/",
            get(views::list_post_comments_by_slug),
        );

    if !ro {
        comments_route
            .route(
                "/api/pages/:slug/comments/",
                post(views::create_page_comment),
            )
            .route(
                "/api/posts/:slug/comments/",
                post(views::create_post_comment),
            )
    } else {
        comments_route
    }
}
