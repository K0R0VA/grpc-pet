use proto_hub::{price_receiver::price_receiver_service_server::PriceReceiverServiceServer};
use tonic::transport::Server;
use utils::Error;
use crate::service::PriceReceiver;

mod service;

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt::fmt().with_env_filter("tracking_service").init();
    // Адрес, на котором сервер будет слушать входящие HTTP/2 запросы
    let addr = "[::1]:50053".parse().expect("failed parse addr");
    let price_receiver = PriceReceiver::initiate().await?;

    tracing::info!("gRPC Сервер обновления цен запущен на {}", addr);

    // Запускаем сетевой стек Tonic
    Server::builder()
        .add_service(PriceReceiverServiceServer::new(price_receiver))
        .serve(addr)
        .await?;

    Ok(())
}