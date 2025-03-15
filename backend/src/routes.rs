use axum::{
    routing::{get, post},
    Router,
};

use crate::handlers::{get_login_form, handle_login, get_server_stats, logout};

pub fn api_routes() -> Router {
    Router::new()
        .route("/api/login-form", get(get_login_form))
        .route("/api/session", post(handle_login))
        .route("/api/server-stats", get(get_server_stats))
        .route("/api/logout", get(logout))
}