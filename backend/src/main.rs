mod routes;
mod models;
mod handlers;

use axum::{
    Router,
    routing::get,
    response::Html,
    http::{StatusCode, Uri},
};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;

use crate::routes::api_routes;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend_dir = PathBuf::from("../frontend");
    
    if !frontend_dir.exists() {
        eprintln!("Error: Frontend directory not found at {:?}", frontend_dir);
        std::process::exit(1);
    }

    let _static_service = ServeDir::new(frontend_dir.clone());

    let app = Router::new()
        .nest("/api", api_routes())
        .route("/", get(serve_index_html))
        .nest_service("/css", ServeDir::new(frontend_dir.join("css")))
        .nest_service("/js", ServeDir::new(frontend_dir.join("js")))
        .nest_service("/assets", ServeDir::new(frontend_dir.join("assets")))
        .fallback(handle_404)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn serve_index_html() -> Result<Html<String>, (StatusCode, String)> {
    let path = PathBuf::from("../frontend/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(html) => Ok(Html(html)),
        Err(err) => {
            eprintln!("Error reading index.html: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to load index.html".to_string()))
        }
    }
}

async fn handle_404(uri: Uri) -> Result<Html<String>, (StatusCode, String)> {
    println!("404 for path: {}", uri);
    serve_index_html().await
}