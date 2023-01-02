use axum::{
    routing::{get, post},
    Router,
};
use std::sync::Arc;

use super::views;
use crate::AppState;

pub fn attachments_routers(ro: bool) -> Router<Arc<AppState>> {
    let attachments_route = Router::new().route("/api/attachments/", get(views::list_attachments));
    if !ro {
        attachments_route.route("/api/attachments/", post(views::create_attachment))
    } else {
        attachments_route
    }
}
