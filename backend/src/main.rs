mod routes;
mod models;
mod handlers;

use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::api_routes;

