use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn pages_routers(ro: bool) -> Router<Arc<AppState>> {
    let pages_route = Router::new()
        .route("/api/pages/", get(views::list_pages))
        .route("/api/pages/:slug", get(views::get_page_by_slug))
        .route(
            "/api/pages/:slug/fields/:name",
            get(views::get_page_field_by_slug_and_name),
        );
    if !ro {
        pages_route
            .route("/api/pages/", post(views::create_page))
            .route("/api/pages/:slug", patch(views::modify_page_by_slug))
            .route("/api/pages/:slug", delete(views::delete_page_by_slug))
            .route(
                "/api/pages/:slug/fields/",
                post(views::create_page_field_by_slug),
            )
            .route(
                "/api/pages/:slug/fields/:name",
                patch(views::modify_page_field_by_slug_and_name),
            )
            .route(
                "/api/pages/:slug/fields/:name",
                delete(views::delete_page_field_by_slug_and_name),
            )
    } else {
        pages_route
    }
}
