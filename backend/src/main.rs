mod routes;
mod models;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use crate::routes::api_routes;

#[tokio::main]
async fn main() {
    // CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Routes
    let app = Router::new()
        .merge(api_routes())
        .layer(cors);

    // Run
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running on http://{}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}