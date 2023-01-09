use axum::extract::State;
use axum::{routing::get, Router};
use std::sync::Arc;

use crate::users::db as user_db;
use crate::AppState;

pub async fn index(State(state): State<Arc<AppState>>) -> String {
    let tpl = &state.index_page;
    let mut lang = "zh_CN".to_string();
    let mut charset = "UTF-8".to_string();
    let options_str = match user_db::get_options_by_uid(&state, 0).await {
        Ok(user_options) => {
            let mut options = vec![];
            for option in user_options {
                let name = option.name.as_str();
                if [
                    "theme",
                    "theme:default",
                    "plugins",
                    "feedFullText",
                    "xmlrpcMarkdown",
                    "commentsPostIntervalEnable",
                    "commentsPostInterval",
                    "commentsAntiSpam",
                    "routingTable",
                    "actionTable",
                    "panelTable",
                    "secret",
                    "installed",
                    "allowXmlRpc",
                ]
                .contains(&name)
                {
                    if name == "lang" {
                        lang = option.value.clone();
                    } else if name == "charset" {
                        charset = option.value.clone();
                    } else {
                        options.push(option);
                    }
                }
            }
            serde_json::to_string(&options).unwrap_or("".to_string())
        }
        Err(_) => "".to_string(),
    };
    let res = tpl
        .replace("{{ lang }}", &lang)
        .replace("{{ charset }}", &charset)
        .replace("{{ options }}", &options_str);
    res
}

pub fn index_router() -> Router<Arc<AppState>> {
    let index_route = Router::new().route("/", get(index));
    index_route
}
