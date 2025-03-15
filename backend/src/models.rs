use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
}

#[derive(Serialize)]
pub struct ServerStats {
    pub version: String,
    pub online_players: u32,
    pub max_players: u32,
    pub status: String,
}