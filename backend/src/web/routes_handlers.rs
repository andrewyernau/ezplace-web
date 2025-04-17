// web/routes_handlers.rs
use axum::{
    extract::Form,
    response::{Html, IntoResponse},
};
use tower_cookies::{Cookie, Cookies};

use crate::models::{LoginRequest, MojangResponse, ServerStats, Players};
use crate::web::AUTH_TOKEN;
use crate::{Error, Result};
use reqwest::header::USER_AGENT;

// Function to fetch Minecraft server status
async fn fetch_server_status(host: &str) -> Result<ServerStats> {
    let url = format!("https://api.mcsrvstat.us/2/{}", host);
    
    //client with headers
    let client = reqwest::Client::new();
    
    // User-Agent header
    let response = client.get(&url)
        .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .send()
        .await
        .map_err(|e| Error::ServerError(format!("Request error: {}", e)))?;

    let response_text = response.text().await
        .map_err(|e| Error::ServerError(format!("Response text error: {}", e)))?;
    
    if response_text.is_empty() {
        return Err(Error::ServerError("Empty response from server".to_string()));
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
            Err(Error::ServerError(format!("Failed to parse JSON: {}", e)))
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

// Get server statistics HTML
pub async fn get_server_stats() -> Result<Html<String>> {
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

            Ok(Html(html))
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

            Ok(Html(html))
        }
    }
}

// Update the logout function to clear the auth cookie
pub async fn logout(cookies: Cookies) -> Html<String> {
    cookies.remove(Cookie::new(AUTH_TOKEN, ""));
    
    Html(r#"
    <div id='auth-section'>
        <button id='login-btn' class='login-btn' hx-get='/api/login-form' hx-target='#login-modal-content' hx-trigger='click' onclick="document.getElementById('login-modal').style.display='block'">Login</button>
    </div>
    "#.to_string())
}