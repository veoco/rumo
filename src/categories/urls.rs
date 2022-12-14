use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn categories_routers(ro: bool) -> Router<Arc<AppState>> {
    let categories_route = Router::new()
        .route("/api/categories/", get(views::list_categories))
        .route("/api/categories/:slug", get(views::get_category_by_slug))
        .route(
            "/api/categories/:slug/posts/",
            get(views::list_category_posts_by_slug),
        );

    if !ro {
        categories_route
            .route("/api/categories/", post(views::create_category))
            .route(
                "/api/categories/:slug",
                patch(views::modify_category_by_slug),
            )
            .route(
                "/api/categories/:slug",
                delete(views::delete_category_by_slug),
            )
            .route(
                "/api/categories/:slug/posts/",
                post(views::add_post_to_category),
            )
            .route(
                "/api/categories/:slug/posts/:post_slug",
                delete(views::delete_post_from_category),
            )
    } else {
        categories_route
    }
}
