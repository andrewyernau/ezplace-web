use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerStats {
    pub online: bool,
    #[serde(default)]
    pub protocol_name: String,
    pub players: Players,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Players {
    pub online: u32,
    pub max: u32,
}

#[derive(Debug, Deserialize)]
pub struct MojangResponse {
    pub id: String,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
}