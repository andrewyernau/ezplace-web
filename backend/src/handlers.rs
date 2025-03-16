use axum::{
    extract::Form,
    response::{Html, IntoResponse},
};

use crate::models::{LoginRequest, MojangResponse, ServerStats, Players};
use reqwest::header::USER_AGENT;

use std::fmt;


#[derive(Debug)]
pub enum ServerError {
    ReqwestError(reqwest::Error),
    ParseError(String),
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ServerError::ReqwestError(e) => write!(f, "Request error: {}", e),
            ServerError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl From<reqwest::Error> for ServerError {
    fn from(err: reqwest::Error) -> Self {
        ServerError::ReqwestError(err)
    }
}

// Function to fetch Minecraft server status
async fn fetch_server_status(host: &str) -> Result<ServerStats, ServerError> {
    let url = format!("https://api.mcsrvstat.us/2/{}", host);
    
    //client with headers
    let client = reqwest::Client::new();
    
    // User-Agent header
    let response = client.get(&url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await?;
    let _status = response.status();

    let response_text = response.text().await?;
    
    if response_text.is_empty() {
        return Err(ServerError::ParseError("Empty response from server".to_string()));
    }
    
    match serde_json::from_str::<serde_json::Value>(&response_text) {
        Ok(json) => {
            // Extract the relevant fields
            let online = json.get("online")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
                
            let protocol_name = json.get("protocol_name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");
                
            let players_online = json.get("players")
                .and_then(|p| p.get("online"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
                
            let players_max = json.get("players")
                .and_then(|p| p.get("max"))
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            
            Ok(ServerStats {
                online,
                protocol_name: protocol_name.to_string(),
                players: Players {
                    online: players_online,
                    max: players_max
                },
            })
        },
        Err(e) => {
            println!("JSON parsing error: {}", e);
            Err(ServerError::ParseError(format!("Failed to parse JSON: {}", e)))
        }
    }
}


// Return the login form HTML
pub async fn get_login_form() -> Html<String> {
    let html = r#"
    <span class='close'>&times;</span>
    <h2>Login with your Minecraft username</h2>
    <form id='login-form' hx-post='/api/session' hx-target='#auth-section' hx-swap='outerHTML'>
        <div class='form-group'>
            <label for='minecraft-username'>Minecraft Username</label>
            <input type='text' id='minecraft-username' name='username' required>
        </div>
        <button type='submit' class='submit-btn'>Login</button>
    </form>
    "#;
    
    Html(html.to_string())
}

// Handle login requests and return user profile HTML
pub async fn handle_login(Form(payload): Form<LoginRequest>) -> impl IntoResponse {
    if payload.username.trim().is_empty() {
        return Html(format!(r#"
            <div id='auth-section'>
                <div class='error-message'>Username cannot be empty</div>
                <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
            </div>
        "#));
    }

    let mojang_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", payload.username);
    let user_uuid = match reqwest::get(&mojang_url).await {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<MojangResponse>().await {
                    Ok(data) => data.id,
                    Err(_) => {
                        return Html(format!(r#"
                            <div id='auth-section'>
                                <div class='error-message'>User not found</div>
                                <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                            </div>
                        "#));
                    }
                }
            } else {
                return Html(format!(r#"
                    <div id='auth-section'>
                        <div class='error-message'>User not found</div>
                        <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                    </div>
                "#));
            }
        }
        Err(_) => {
            return Html(format!(r#"
                <div id='auth-section'>
                    <div class='error-message'>Error connecting to Mojang API</div>
                    <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
                </div>
            "#));
        }
    };

    let avatar_url = format!("https://crafatar.com/avatars/{}?size=50&overlay", user_uuid);

    Html(format!(r#"
        <div id='auth-section'>
            <div class="user-profile">
                <img src='{}' alt='User Avatar' class='user-avatar'>
                <span class='username'>{}</span>
                <div class='dropdown-content'>
                    <a href='#' hx-get='/api/logout' hx-target='#auth-section' hx-swap='outerHTML'>Logout</a>
                </div>
            </div>
        </div>
    "#, avatar_url, payload.username))
}

// Get server statistics HTML
pub async fn get_server_stats() -> Html<String> {
    let host = "play.ezplace.net";

    match fetch_server_status(host).await {
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
                status.protocol_name,
                status.players.online,
                status.players.max,
                status_class,
                status_text
            );

            Html(html)
        }
        Err(err) => {
            eprintln!("Error fetching server status: {}", err);
            let html = format!(
                r#"
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
                "#
            );

            Html(html)
        }
    }
}

// Handle logout request
pub async fn logout() -> Html<String> {
    Html(r#"
    <div id='auth-section'>
        <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
    </div>
    "#.to_string())
}