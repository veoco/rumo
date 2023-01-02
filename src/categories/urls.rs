use axum::{
    routing::{get, post},
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
                "/api/categories/:slug/posts/",
                post(views::add_post_to_category),
            )
    } else {
        categories_route
    }
}
