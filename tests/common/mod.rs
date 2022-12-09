use axum::Router;
use sqlx::sqlite::SqlitePool;
use std::env;

use ters::{app, AppState};

pub async fn setup_state() -> AppState {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let secret_key = env::var("SECRET_KEY").unwrap();
    let access_token_expire_secondes = 3600 * 24 * 30;

    AppState {
        pool,
        secret_key,
        access_token_expire_secondes,
    }
}

pub async fn setup_app(state: AppState) -> Router {
    app(Some(state)).await
}
