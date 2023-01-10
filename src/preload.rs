use axum::extract::State;
use axum::{routing::get, Router};
use std::collections::HashMap;
use std::sync::Arc;

use crate::common::errors::FieldError;
use crate::users::db as user_db;
use crate::AppState;

pub async fn index(State(state): State<Arc<AppState>>) -> Result<String, FieldError> {
    let template = state.jinja_env.get_template("index.html").unwrap();
    let mut context = HashMap::new();

    let mut options_map = HashMap::new();
    let options = user_db::get_options_by_uid(&state, 0).await?;
    for option in options {
        options_map.insert(option.name, option.value);
    }
    context.insert(String::from("options"), options_map);

    let output = template
        .render(context)
        .map_err(|e| FieldError::DatabaseFailed(e.to_string()))?;
    Ok(output)
}

pub fn index_router() -> Router<Arc<AppState>> {
    let index_route = Router::new().route("/", get(index));
    index_route
}
