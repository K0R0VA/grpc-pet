use entities::alert::AlertSubscription;
use proto_hub::tracking::tracking_service_client::TrackingServiceClient;
use proto_hub::tracking::RouteRequest;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use sea_orm::{ColumnTrait, Database, EntityTrait, QueryFilter};
use utils::from_env;
use std::time::Duration;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = from_env("DATABASE_URL")?;
    let database = Database::connect(database_url).await?;
    println!("[Notification Service] Starting background worker...");

    // 1. Инициализируем продюсера Kafka
    let producer: FutureProducer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("message.timeout.ms", "5000")
        .create()?;
    println!("[Notification Service] Connected to Kafka broker");

    // 2. Подключаемся к gRPC-серверу трекинга
    let grpc_addr = "http://[::1]:50051";
    println!("[Notification Service] Connecting to Tracking gRPC Service at {}", grpc_addr);
    let mut tracking_client = TrackingServiceClient::connect(grpc_addr).await?;

    let request = tonic::Request::new(RouteRequest {});

    println!("[Notification Service] Requesting gRPC Server Stream for 'MSK-NYC'...");
    let response = tracking_client.watch_prices(request).await?;
    let mut price_stream = response.into_inner();

    // 4. В бесконечном асинхронном цикле читаем gRPC-стрим
    while let Some(received) = price_stream.next().await {
        match received {
            Ok(price_update) => {
                let alerts = entities::alert::Entity::find()
                    .find_with_linked(AlertSubscription)
                    .filter(entities::alert::Column::TargetPrice.gte(price_update.current_price))
                    .filter(entities::subscription::Column::Enabled.eq(true))
                    .all(&database)
                    .await?;
                for (alert, subscriptions) in alerts {
                    for subscription in subscriptions {
                        let payload = serde_json::json!({
                            "user_id": alert.user_id,
                            "route_id": price_update.route,
                            "price": price_update.current_price,
                            "timestamp": chrono::Utc::now().to_rfc3339()
                        })
                        .to_string();

                        // Формируем запись для Kafka (топик: "price-alerts")
                        let record = FutureRecord {
                            key: Some(&price_update.route),
                            topic: &format!("price-alerts-{:?}", subscription.subscription_type),
                            payload: Some(&payload),
                            headers: None,
                            timestamp: None,
                            partition: None
                        }; // Ключ топика — id маршрута для корректного партиционирования
                        producer.send(record, Duration::from_secs(0)).await.map_err(|(e, _)| e)?;
                    }
                }
            }
            Err(status) => {
                println!("[gRPC Stream Error] Status: {:?}", status);
                // Если gRPC сервер упал или разорвал соединение, выходим из цикла
                break;
            }
        }
    }

    println!("[Notification Service] Stream closed. Shutting down.");
    Ok(())
}