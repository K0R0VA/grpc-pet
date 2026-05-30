use proto_hub::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;
use utils::Error;

use crate::service::AuthController;

mod service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt().with_env_filter("tracking_service").init();
    // Адрес, на котором сервер будет слушать входящие HTTP/2 запросы
    let addr = "[::1]:50051".parse().expect("failed parse addr");
    let tracking_service = AuthController::initiate().await?;

    tracing::info!("gRPC Сервер авторизации запущен на {}", addr);

    // Запускаем сетевой стек Tonic
    Server::builder()
        .add_service(AuthServiceServer::new(tracking_service))
        .serve(addr)
        .await?;

    Ok(())
}