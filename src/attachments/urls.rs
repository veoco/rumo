use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn attachments_routers(ro: bool) -> Router<Arc<AppState>> {
    let attachments_route = Router::new()
        .route("/api/attachments/", get(views::list_attachments))
        .route("/api/attachments/:cid", get(views::get_attachment_by_cid))
        .route(
            "/api/pages/:slug/attachments/",
            get(views::list_content_attachments_by_slug),
        )
        .route(
            "/api/posts/:slug/attachments/",
            get(views::list_content_attachments_by_slug),
        );
    if !ro {
        attachments_route
            .route("/api/attachments/", post(views::create_attachment))
            .route(
                "/api/attachments/:cid",
                patch(views::modify_attachment_by_cid),
            )
            .route(
                "/api/attachments/:cid",
                delete(views::delete_attachment_by_cid),
            )
            .route(
                "/api/pages/:slug/attachments/",
                post(views::add_attachment_to_content_by_cid),
            )
            .route(
                "/api/posts/:slug/attachments/",
                post(views::add_attachment_to_content_by_cid),
            )
            .route(
                "/api/pages/:slug/attachments/:cid",
                delete(views::delete_attachment_from_content_by_cid),
            )
            .route(
                "/api/posts/:slug/attachments/:cid",
                delete(views::delete_attachment_from_content_by_cid),
            )
    } else {
        attachments_route
    }
}
