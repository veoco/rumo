use axum::Router;
use sqlx::sqlite::SqlitePool;
use std::env;
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;

mod categories;
mod db;
mod tags;
mod users;
mod posts;
mod pages;
mod comments;
use categories::categories_routers;
use tags::tags_routers;
use posts::posts_routers;
use pages::pages_routers;
use comments::comments_routers;
use users::{users_routers, UserRegister};

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret_key: String,
    pub access_token_expire_secondes: u64,
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
            let pool = SqlitePool::connect(&env::var("DATABASE_URL").expect("DATABASE_URL is required"))
                .await
                .expect("Database connect failed");
            let secret_key = env::var("SECRET_KEY").expect("SECRET_KEY is required");
            let access_token_expire_secondes = 3600 * 24 * 30;

            let table_prefix = env::var("TABLE_PREFIX").unwrap_or("typecho_".to_string());
            let comments_table = format!("{}_comments", table_prefix);
            let contents_table = format!("{}_contents", table_prefix);
            let fields_table = format!("{}_fields", table_prefix);
            let metas_table = format!("{}_metas", table_prefix);
            let options_table = format!("{}_options", table_prefix);
            let relationships_table = format!("{}_relationships", table_prefix);
            let users_table = format!("{}_users", table_prefix);

            let s = AppState {
                pool,
                secret_key,
                access_token_expire_secondes,
                comments_table,
                contents_table,
                fields_table,
                metas_table,
                options_table,
                relationships_table,
                users_table
            };
            s
        }
    };
    state
}

pub async fn app(app_state: Option<AppState>) -> Router {
    let state = Arc::new(get_state(app_state).await);
    let app = Router::new()
        .merge(users_routers())
        .merge(categories_routers())
        .merge(tags_routers())
        .merge(posts_routers())
        .merge(pages_routers())
        .merge(comments_routers())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
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

    db::init_db(&state).await;
    info!("schema created");
    db::init_admin(&state, user_register).await;
    info!("admin user created");
}
