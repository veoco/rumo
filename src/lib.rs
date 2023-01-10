use axum::Router;
use sqlx::AnyPool;
use std::env;
use std::sync::Arc;
use tokio::fs;
use tower_http::trace::TraceLayer;
use tracing::info;

mod attachments;
mod categories;
mod comments;
mod common;
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
use preload::index_router;
use tags::tags_routers;
use users::{models::UserRegister, users_routers};

#[derive(Clone)]
pub struct AppState {
    pub pool: AnyPool,
    pub secret_key: String,
    pub access_token_expire_secondes: u64,
    pub upload_root: String,
    pub read_only: bool,
    pub preload_index: bool,
    pub index_page: String,

    pub comments_table: String,
    pub contents_table: String,
    pub fields_table: String,
    pub metas_table: String,
    pub options_table: String,
    pub relationships_table: String,
    pub users_table: String,
}

async fn get_state(app_state: Option<AppState>) -> AppState {
    let state = match app_state {
        Some(s) => s,
        None => {
            let pool =
                AnyPool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL is required"))
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

            let mut index_page = "".to_string();
            let filepath = env::var("INDEX_PAGE").unwrap_or(String::from("./index.html"));
            let filepath = std::path::Path::new(&filepath);
            if preload_index {
                if !filepath.exists() && !filepath.is_file() {
                    panic!("INDEX_PAGE is invalid")
                }
                index_page = fs::read_to_string(filepath)
                    .await
                    .expect("INDEX_PAGE is invalid");
            }

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

            let table_prefix = env::var("TABLE_PREFIX").unwrap_or("typecho_".to_string());
            let comments_table = format!("{}comments", table_prefix);
            let contents_table = format!("{}contents", table_prefix);
            let fields_table = format!("{}fields", table_prefix);
            let metas_table = format!("{}metas", table_prefix);
            let options_table = format!("{}options", table_prefix);
            let relationships_table = format!("{}relationships", table_prefix);
            let users_table = format!("{}users", table_prefix);

            let s = AppState {
                pool,
                secret_key,
                access_token_expire_secondes,
                upload_root,
                read_only,
                preload_index,
                index_page,

                comments_table,
                contents_table,
                fields_table,
                metas_table,
                options_table,
                relationships_table,
                users_table,
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
        router = router.merge(index_router());
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
