use axum::{
    routing::{get, post},
    Router,   
};
use crate::handlers::{get_login_form, handle_login, get_server_stats, logout};

pub fn api_routes() -> Router {
    Router::new()
        .route("/login-form", get(get_login_form))
        .route("/session", post(handle_login))
        .route("/server-stats", get(get_server_stats))
        .route("/logout", get(logout))
}