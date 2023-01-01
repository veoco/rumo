use axum::{routing::get, Router};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn attachments_routers() -> Router<Arc<AppState>> {
    let attachments_route = Router::new().route(
        "/api/attachments/",
        get(views::list_attachments).post(views::create_attachments),
    );
    attachments_route
}
