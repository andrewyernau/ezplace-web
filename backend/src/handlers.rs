use axum::{
    extract::{Form, Path},
    response::{Html, IntoResponse},
};
use serde_json::json;

use crate::models::{LoginRequest, ServerStats};

use reqwest::Error;
use serde::Deserialize;

// Return the login form HTML
pub async fn get_login_form() -> Html<String> {
    let html = r#'
    <span class="close">&times;</span>
    <h2>Login with your Minecraft username</h2>
    <form id="login-form" hx-post="http://localhost:8000/api/session" hx-target="#auth-section" hx-swap="outerHTML">
        <div class="form-group">
            <label for="minecraft-username">Minecraft Username</label>
            <input type="text" id="minecraft-username" name="username" required>
        </div>
        <button type="submit" class="submit-btn">Login</button>
    </form>
    '#;
    
    Html(html.to_string())
}

// Handle login requests and return user profile HTML
pub async fn handle_login(Form(payload): Form<LoginRequest>) -> impl IntoResponse {
    if payload.username.trim().is_empty() {
        // Return error message in HTML
        return Html(format!(r#'
        <div id="auth-section">
            <div class="error-message">Username cannot be empty</div>
            <button id="login-btn" class="login-btn" hx-get="http://localhost:8000/api/login-form" hx-target="#login-modal-content" hx-trigger="click" onclick="document.getElementById('login-modal').style.display='block'">Login</button>
        </div>
        '#));
    }

    // Generate avatar URL for the user
    let avatar_url = format!("https://crafatar.com/avatars/{}?size=50&overlay", payload.username);
    
    // Return user profile HTML
    Html(format!(r#'
    <div id="auth-section">
        <div class="user-profile">
            <img src="{}" alt="User Avatar" class="user-avatar">
            <span class="username">{}</span>
            <div class="dropdown-content">
                <a href="#" hx-get="http://localhost:8000/api/logout" hx-target="#auth-section" hx-swap="outerHTML">Logout</a>
            </div>
        </div>
    </div>
    '#, avatar_url, payload.username))
}

// Get server statistics HTML
pub async fn get_server_stats() -> Html<String> {
    let host = "play.ezplace.net";

    match get_server_status(host).await {
        Ok(status) => {
            let status_class = if status.online { "status-online" } else { "status-offline" };
            let status_text = if status.online { "Online" } else { "Offline" };

            let html = format!(
                r#"
                <div class="stats-placeholder">
                    <div class="stat">
                        <h3>Version</h3>
                        <p>{}</p>
                    </div>
                    <div class="stat">
                        <h3>Players Online</h3>
                        <p>{}/{}</p>
                    </div>
                    <div class="stat">
                        <h3>Status</h3>
                        <p class="{}">{}</p>
                    </div>
                </div>
                "#,
                status.version,
                status.players_online,
                status.max_players,
                status_class,
                status_text
            );

            Html(html)
        }
        Err(_) => {
            Html(r#"
            <div class="stats-placeholder">
                <div class="stat">
                    <h3>Version</h3>
                    <p>Unknown</p>
                </div>
                <div class="stat">
                    <h3>Players Online</h3>
                    <p>0/0</p>
                </div>
                <div class="stat">
                    <h3>Status</h3>
                    <p class="status-offline">Offline</p>
                </div>
            </div>
            "#.to_string())
        }
    }
}

// Handle logout request
pub async fn logout() -> Html<String> {
    Html(r#'
    <div id="auth-section">
        <button id="login-btn" class="login-btn" hx-get="http://localhost:8000/api/login-form" hx-target="#login-modal-content" hx-trigger="click" onclick="document.getElementById('login-modal').style.display='block'">Login</button>
    </div>
    '#.to_string())
}