use axum::{
    routing::{post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn comments_routers() -> Router<Arc<AppState>> {
    let comments_route = Router::new()
        .route("/api/comments/:slug", post(views::create_comment).get(views::list_content_comments_by_slug));
        comments_route
}
