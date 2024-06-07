use std::env;
use std::sync::Arc;

use axum::Router;
use tower_http::trace::TraceLayer;

use rumo_service::sea_orm::{Database, DatabaseConnection};

#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection,
    pub secret_key: String,
    pub access_token_expire_secondes: u64,
    pub upload_root: String,
    pub read_only: bool,
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
            };
            s
        }
    };
    state
}

pub async fn setup_app(app_state: Option<AppState>) -> Router {
    let state = Arc::new(get_state(app_state).await);
    let ro = state.read_only;
    let mut router = Router::new();

    let app = router.layer(TraceLayer::new_for_http()).with_state(state);
    app
}
