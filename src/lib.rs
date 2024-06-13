use std::env;
use std::fs;
use std::sync::Arc;

use axum::Router;
use minijinja::Environment;
use sea_orm::{Database, DatabaseConnection};
use tower_http::trace::TraceLayer;
use tracing::info;

#[macro_use]
extern crate lazy_static;

mod attachments;
mod categories;
mod comments;
mod common;
mod entity;
mod init;
mod pages;
mod posts;
mod preload;
mod tags;
mod users;
use attachments::attachments_routers;
use categories::categories_routers;
use comments::comments_routers;
use pages::pages_routers;
use posts::posts_routers;
use tags::tags_routers;
use users::{forms::UserRegister, users_routers};

lazy_static! {
    pub static ref INDEX_TPL: String = {
        let preload_index = match env::var("PRELOAD_INDEX") {
            Ok(s) => {
                if s == "true" {
                    true
                } else {
                    false
                }
            }
            _ => false,
        };
        let mut index_page = "".to_string();
        let filepath = env::var("INDEX_PAGE").unwrap_or(String::from("./index.html"));
        let filepath = std::path::Path::new(&filepath);
        if preload_index {
            if !filepath.exists() && !filepath.is_file() {
                panic!("INDEX_PAGE is invalid")
            }
            index_page = fs::read_to_string(filepath).expect("INDEX_PAGE is invalid");
        }
        index_page
    };
}

#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub secret_key: String,
    pub access_token_expire_secondes: u64,
    pub upload_root: String,
    pub read_only: bool,
    pub preload_index: bool,
    pub jinja_env: Environment<'static>,
}

async fn get_state(app_state: Option<AppState>) -> AppState {
    let state = match app_state {
        Some(s) => s,
        None => {
            let conn =
                Database::connect(&env::var("DATABASE_URL").expect("DATABASE_URL is required"))
                    .await
                    .expect("Database connect failed");
            let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY is required");

            let access_token_expire_secondes = env::var("TOKEN_EXPIRE")
                .unwrap_or("720".to_string())
                .parse::<u64>()
                .expect("TOKEN_EXPIRE is invalid");

            let preload_index = match env::var("PRELOAD_INDEX") {
                Ok(s) => {
                    if s == "true" {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            let mut jinja_env = Environment::new();
            jinja_env.add_template("index.html", &INDEX_TPL).unwrap();

            let upload_root = env::var("UPLOAD_ROOT").unwrap_or(String::from("."));
            let read_only = match env::var("READ_ONLY") {
                Ok(s) => {
                    if s == "true" {
                        true
                    } else {
                        false
                    }
                }
                _ => false,
            };

            let s = AppState {
                conn,
                secret_key,
                access_token_expire_secondes,
                upload_root,
                read_only,
                preload_index,
                jinja_env,
            };
            s
        }
    };
    state
}

pub async fn app(app_state: Option<AppState>) -> Router {
    let state = Arc::new(get_state(app_state).await);
    let ro = state.read_only;
    let mut router = Router::new()
        .merge(users_routers(ro))
        .merge(categories_routers(ro))
        .merge(tags_routers(ro))
        .merge(posts_routers(ro))
        .merge(pages_routers(ro))
        .merge(comments_routers(ro))
        .merge(attachments_routers(ro));

    if state.preload_index {
        router = router.fallback(preload::index);
    }
    let app = router.layer(TraceLayer::new_for_http()).with_state(state);
    app
}

pub async fn init(name: String, mail: String, password: String) {
    let state = get_state(None).await;
    let user_register = UserRegister {
        name,
        mail,
        password,
        url: "http://127.0.0.1".to_owned(),
    };

    init::init_table(&state).await;
    info!("schema created");
    init::init_options(&state).await;
    info!("options created");
    init::init_admin(&state, user_register).await;
    info!("admin user created");
}
