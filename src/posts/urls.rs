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
            .route("/api/posts/:slug", patch(views::modify_post_by_slug))
            .route("/api/posts/:slug", delete(views::delete_post_by_slug))
            .route(
                "/api/posts/:slug/fields/",
                post(views::create_post_field_by_slug),
            )
            .route(
                "/api/posts/:slug/fields/:name",
                get(views::get_post_field_by_slug_and_name),
            )
            .route(
                "/api/posts/:slug/fields/:name",
                patch(views::modify_post_field_by_slug_and_name),
            )
            .route(
                "/api/posts/:slug/fields/:name",
                delete(views::delete_post_field_by_slug_and_name),
            )
    } else {
        posts_route
    }
}
