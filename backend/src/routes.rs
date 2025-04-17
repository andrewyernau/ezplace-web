use axum::Router;
use crate::web::routes_handlers;
use axum::routing::get;

pub fn api_routes() -> Router {
    Router::new()
        .route("/server-stats", get(routes_handlers::get_server_stats))
}