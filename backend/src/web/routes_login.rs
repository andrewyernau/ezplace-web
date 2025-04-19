// web/routes_login.rs
use crate::web::{self, routes_handlers};
use crate::Result;
use crate::ctx::Ctx;

use axum::routing::{get, post};
use axum::Router;
use axum::extract::Form;
use axum::response::Html;
use tower_cookies::{Cookie, Cookies};
use crate::models::LoginRequest;

pub fn routes() -> Router {
    Router::new()
        .route("/login-form", get(routes_handlers::get_login_form))
        .route("/session", post(handle_login))
        .route("/logout", get(routes_handlers::logout))
        .route("/auth-status", get(check_auth_status))
}

async fn handle_login(
    cookies: Cookies,
    Form(payload): Form<LoginRequest>,
) -> Result<Html<String>> {
    println!("->> {:<12} - handle_login", "HANDLER");

    if payload.username.trim().is_empty() {
        return Ok(Html(format!(r#"
            <div id='auth-section'>
                <div class='error-message'>Username cannot be empty</div>
                <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
            </div>
        "#)));
    }

    let mojang_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", payload.username);
    let user_uuid = match reqwest::get(&mojang_url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<crate::models::MojangResponse>().await {
                    Ok(data) => data.id,
                    Err(_) => {
                        return Ok(Html(format!(r#"
                            <div id='auth-section'>
                                <div class='error-message'>User not found</div>
                                <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                            </div>
                        "#)));
                    }
                }
            } else {
                return Ok(Html(format!(r#"
                    <div id='auth-section'>
                        <div class='error-message'>User not found</div>
                        <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                    </div>
                "#)));
            }
        }
        Err(_) => {
            return Ok(Html(format!(r#"
                <div id='auth-section'>
                    <div class='error-message'>Error connecting to Mojang API</div>
                    <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                </div>
            "#)));
        }
    };

    // Generate auth token with UUID, username, expiration, and signature
    let auth_token = format!("minecraft-{}.{}.exp.sign", user_uuid, payload.username);
    let mut cookie = Cookie::new(web::AUTH_TOKEN, auth_token);
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookies.add(cookie);

    let avatar_url = format!("https://crafatar.com/avatars/{}?size=50&overlay", user_uuid);

    Ok(Html(format!(r#"
        <div id='auth-section'>
            <div class="user-profile">
                <img src='{}' alt='User Avatar' class='user-avatar'>
                <span class='username'>{}</span>
                <div class='dropdown-content'>
                    <a href='#' hx-get='/api/logout' hx-target='#auth-section' hx-swap='outerHTML'>Logout</a>
                </div>
            </div>
        </div>
    "#, avatar_url, payload.username)))
}

pub async fn check_auth_status(
    ctx: Result<Ctx>
) -> Html<String> {
    println!("->> {:<12} - check_auth_status", "HANDLER");
    
    match ctx {
        Ok(ctx) => {
            let user_id = ctx.user_id();
            let username = ctx.username();
            let avatar_url = format!("https://crafatar.com/avatars/{}?size=50&overlay", user_id);
            
            Html(format!(r#"
                <div class="user-profile">
                    <img src='{}' alt='User Avatar' class='user-avatar'>
                    <span class='username'>{}</span>
                    <div class='dropdown-content'>
                        <a href='#' hx-get='/api/logout' hx-target='#auth-section' hx-swap='outerHTML'>Logout</a>
                    </div>
                </div>
            "#, avatar_url, username))
        },
        Err(_) => Html(r#"
            <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
        "#.to_string())
    }
}