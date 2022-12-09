use axum::Router;
use sqlx::sqlite::SqlitePool;
use std::env;
use std::sync::Arc;
use tower_http::trace::TraceLayer;

mod users;
mod db;
use users::auth_routers;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub secret_key: String,
    pub access_token_expire_secondes: u64,
}

async fn get_state(app_state: Option<AppState>)-> AppState{
    let state = match app_state {
        Some(s) => s,
        None => {
            let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
                .await
                .unwrap();
            let secret_key = env::var("SECRET_KEY").unwrap();
            let access_token_expire_secondes = 3600 * 24 * 30;

            let s = AppState {
                pool,
                secret_key,
                access_token_expire_secondes,
            };
            s
        }
    };
    state
}

pub async fn app(app_state: Option<AppState>) -> Router {
    let state = Arc::new(get_state(app_state).await);
    let app = Router::new()
        .merge(auth_routers())
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    app
}

pub async fn init(){
    let state = get_state(None).await;
    db::init_db(state).await;
}
