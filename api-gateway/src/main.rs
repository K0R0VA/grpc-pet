use std::net::SocketAddr;

use crate::state::AppState;


mod state;
mod routes;
mod middleware;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Подключаемся к gRPC-серверу (нашему tracking-service), который мы запустили ранее
    tracing_subscriber::fmt::fmt().init();
    let state = AppState::new()?;
    let app = routes::app(state);

    // 3. Запускаем HTTP-сервер шлюза на порту 3000
    let http_addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(http_addr).await?;
    tracing::info!("[Gateway] HTTP Server API Gateway available on http://{}", http_addr);
    
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await?;

    Ok(())
}
