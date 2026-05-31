use std::net::SocketAddr;

use axum::{Json, extract::{ConnectInfo, State}};
use proto_hub::auth::AuthRequest;

use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct AuthClaims {
    name: String,
    password: String,
}

pub async fn handle_register(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>, 
    Json(payload): Json<AuthClaims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let request = AuthRequest {
        name: payload.name,
        password: payload.password,
    };
    let grpc_res = state.register(request, addr)
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.message().to_string()))?;
    let inner = grpc_res;
    Ok(Json(serde_json::json!({ "success": inner.success, "token": inner.token, "user_id": inner.user_id })))
}

pub async fn handle_login(
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>, 
    Json(payload): Json<AuthClaims>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let request = AuthRequest {
        name: payload.name,
        password: payload.password,
    };
    let grpc_res = state.login(request, addr)
        .await
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.message().to_string()))?;
    let inner = grpc_res;
    Ok(Json(serde_json::json!({ "success": inner.success, "token": inner.token, "user_id": inner.user_id })))
}