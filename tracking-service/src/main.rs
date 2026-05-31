use proto_hub::tracking::tracking_service_server::TrackingServiceServer;
use tonic::transport::Server;
use utils::Error;
use crate::service::Tracker ;

mod service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt().with_env_filter("tracking_service").init();
    // Адрес, на котором сервер будет слушать входящие HTTP/2 запросы
    let addr = "[::1]:50051".parse().expect("failed parse addr");
    let tracking_service = Tracker::initiate().await?;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto_hub::tracking::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("create reflection_service failed");
    tracing::info!("gRPC Сервер Трекинга запущен на {}", addr);

    // Запускаем сетевой стек Tonic
    Server::builder()
        .add_service(TrackingServiceServer::new(tracking_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;

    Ok(())
}