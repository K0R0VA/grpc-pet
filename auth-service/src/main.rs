use proto_hub::auth::auth_service_server::AuthServiceServer;
use tonic::transport::Server;
use utils::Error;

use crate::service::AuthController;

mod service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt().with_env_filter("tracking_service").init();
    // Адрес, на котором сервер будет слушать входящие HTTP/2 запросы
    let addr = "[::1]:50052".parse().expect("failed parse addr");
    let auth_service = AuthController::initiate().await?;
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto_hub::auth::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("create reflection_service failed");
    tracing::info!("gRPC Сервер авторизации запущен на {}", addr);

    // Запускаем сетевой стек Tonic
    Server::builder()
        .add_service(AuthServiceServer::new(auth_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}