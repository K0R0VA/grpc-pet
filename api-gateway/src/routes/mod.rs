use axum::Router;
use axum::routing::post;

use crate::routes::alert::handle_create_alert;
use crate::routes::auth::{handle_login, handle_register};
use crate::state::AppState;

mod alert;
mod auth;

pub fn app(state: AppState) -> Router {
    let app = Router::new()
        // При POST запросе на /api/alerts будет вызываться функция handle_create_alert
        .route("/api/alerts", post(handle_create_alert))
        .route("/api/login", post(handle_login))
        .route("/api/register", post(handle_register))
        .with_state(state);
    app
}