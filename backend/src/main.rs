mod routes;
mod models;
mod ctx;
mod error;
mod web;
mod log;
mod model;

use crate::ctx::Ctx;
use crate::log::log_request;
use crate::model::ModelController;
use crate::web::routes_handlers;
use axum::{
    Router,
    routing::get,
    response::Html,
    http::{StatusCode, Uri, Method},
    middleware,
    response::{IntoResponse, Response},
};
use std::{net::SocketAddr, path::PathBuf};
use tower_cookies::CookieManagerLayer;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use uuid::Uuid;

pub use self::error::{Error, Result, ClientError};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize ModelController
    let mc = ModelController::new().await?;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let frontend_dir = PathBuf::from("../frontend");
    
    if !frontend_dir.exists() {
        eprintln!("Error: Frontend directory not found at {:?}", frontend_dir);
        std::process::exit(1);
    }

    // API routes with auth requirement
    let api_routes_auth = web::routes_tickets::routes(mc.clone())
        .route_layer(middleware::from_fn(web::mw_auth::mw_require_auth));

    // API routes without auth requirement
    let api_routes = Router::new()
        .route("/server-stats", get(routes_handlers::get_server_stats))
        .route("/read-more-form", get(routes_handlers::get_read_more_content))
        .route("/terms", get(routes_handlers::get_terms_content))
        .route("/cookies", get(routes_handlers::get_cookies_content))
        .route("/contact", get(routes_handlers::get_contact_content))
        .route("/community", get(routes_handlers::get_community_content))
        .merge(web::routes_login::routes())
        .merge(api_routes_auth);

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/", get(serve_index_html))
        .nest_service("/css", ServeDir::new(frontend_dir.join("css")))
        .nest_service("/js", ServeDir::new(frontend_dir.join("js")))
        .nest_service("/assets", ServeDir::new(frontend_dir.join("assets")))
        .fallback(handle_404)
        .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 5173));
    println!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap(); 

    Ok(())
}

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    println!("->> {:<12} - main_response_mapper", "RES_MAPPER");
    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // -- If client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = serde_json::json!({
                "error": {
                    "type": client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });

            println!("    ->> client_error_body: {client_error_body}");

            // Build the new response from the client_error_body
            (StatusCode::from_u16(status_code.as_u16()).unwrap(), axum::Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.map(|(_, client_err)| client_err);
    let _ = log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    error_response.unwrap_or(res)
}

async fn serve_index_html(ctx: Result<Ctx>) -> Result<Html<String>> {
    let path = PathBuf::from("../frontend/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(mut html) => {
            // Replace the authentication section according to the context
            match ctx {
                Ok(ctx) => {
                    let user_id = ctx.user_id();
                    let username = ctx.username();
                    let avatar_url = format!("https://crafatar.com/avatars/{}?size=50&overlay", user_id);
                    
                    println!("->> {:<12} - serve_index_html - Authenticated as {}", "HANDLER", username);
                    
                    let auth_html = format!(r#"<div id='auth-section'>
                        <div class='user-profile'>
                            <img src='{}' alt='User Avatar' class='user-avatar'>
                            <span class='username'>{}</span>
                            <div class='dropdown-content'>
                                <a href='#' id="logout-txt" hx-get='/api/logout' hx-target='#auth-section' hx-swap='outerHTML'>Logout</a>
                            </div>
                        </div>
                    </div>"#, avatar_url, username);
                    
                    // Replace the authentication section in the HTML
                    html = html.replace(r#"<div id='auth-section'>
                <button id='login-btn' 
                        class='login-btn'
                        hx-get='/api/login-form'
                        hx-target='#login-modal-content'
                        hx-trigger='click' 
                        onclick="document.getElementById('login-modal').style.display='block'">
                            Login
                </button>
            </div>"#, &auth_html);
                },
                Err(err) => {
                    println!("->> {:<12} - serve_index_html - Not authenticated: {:?}", "HANDLER", err);
                    // No need to modify HTML for unauthenticated users
                }
            }
            
            Ok(Html(html))
        },
        Err(err) => {
            eprintln!("Error reading index.html: {}", err);
            Err(Error::ServerError(format!("Failed to load index.html: {}", err)))
        }
    }
}

async fn handle_404(uri: Uri) -> Result<Html<String>> {
    println!("404 for path: {}", uri);
    serve_index_html(Err(Error::ServerError("Not Found".into()))).await
}