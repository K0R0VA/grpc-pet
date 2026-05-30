use std::pin::Pin;
use std::time::Duration;

use chrono::{Utc};
use entities::subscription::SubscriptionType;
use proto_hub::tracking::{AlertRequest, AlertResponse, PriceUpdate, RouteRequest, SubscriptionRequest, SubscriptionResponse};
use proto_hub::tracking::tracking_service_server::TrackingService;
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};
use tonic::{Request, Response, Status};
use utils::{Error, from_env};

pub struct Tracker {
    pub database: DatabaseConnection
}

impl Tracker {
    pub async fn initiate() -> Result<Self, Error> {
        let connection_url = from_env("DATABASE_URL")?;
        let database = Database::connect(&connection_url).await?;
        Ok(Self {
            database
        })
    }
}

#[tonic::async_trait]
impl TrackingService for Tracker {
    type WatchPricesStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<PriceUpdate, Status>> + Send>>;
    async fn set_subscription(&self, request: tonic::Request<proto_hub::tracking::SubscriptionRequest>) ->  Result<tonic::Response<proto_hub::tracking::SubscriptionResponse>, tonic::Status> {
        let SubscriptionRequest {subscription_type, user_id, enabled} = request.into_inner();
        let subscription_type = match subscription_type {
            1 => SubscriptionType::Telegram,
            2 => SubscriptionType::Email,
            3 => SubscriptionType::Sms,
            _ => return Err(Status::unknown("subscription_type"))
        };
        let subscription = entities::subscription::ActiveModel {
            subscription_type: sea_orm::ActiveValue::Set(subscription_type),
            user_id: sea_orm::ActiveValue::Set(user_id),
            enabled: sea_orm::ActiveValue::Set(enabled),
        };
        subscription
            .into_active_model()
            .insert(&self.database).await
            .map_err(|e| Status::internal(e.to_string()))?;
        tracing::info!("[RPC Server] Processing AddAlert for user: {}", user_id);
        Ok(Response::new(SubscriptionResponse {}))
    }
    async fn add_alert(
        &self,
        request: Request<AlertRequest>,
    ) -> Result<Response<AlertResponse>, Status> {
        let AlertRequest { user_id, route, target_price } = request.into_inner();
        let alert = entities::alert::ActiveModel {
            route_id: sea_orm::ActiveValue::Set(route),
            user_id: sea_orm::ActiveValue::Set(user_id),
            target_price: sea_orm::ActiveValue::Set(target_price),
            ..Default::default()
        };
        let alert = alert
            .into_active_model()
            .insert(&self.database).await
            .map_err(|e| Status::internal(e.to_string()))?;
        tracing::info!("[RPC Server] Processing AddAlert for user: {}", user_id);
        let response = AlertResponse {
            success: true,
            alert_id: alert.id,
        };
        Ok(Response::new(response))
    }

    async fn watch_prices(
        &self,
        _request: Request<RouteRequest>,
    ) -> Result<Response<Self::WatchPricesStream>, Status> {
        tracing::info!("[RPC Server] Establishing WatchPrices stream channel");
        let (tx, rx) = tokio::sync::mpsc::channel(128); // В продакшене размер буфера берут с запасом
        let database = self.database.clone();
        tokio::spawn(async move {
            loop {
                let now = Utc::now() - chrono::Duration::seconds(5);
                let routes = entities::route::Entity::find()
                    // updated_at > now - 5s
                    .filter(entities::route::Column::UpdatedAt.gt(now))
                    .all(&database)
                    .await
                    .map_err(|_| Status::internal("Database check failed"));
                let routes = match routes {
                    Ok(routes) => routes,
                    Err(e) => {
                        tracing::info!("[RPC Stream] Database connection receive error {e}");
                        break;
                    }
                };

                for entities::route::Model {id, from, to, current_price, ..} in routes {
                    let route = format!("{from}-{to}");
                    let update = PriceUpdate {
                        route_id: id,
                        route,
                        current_price,
                    };

                    if tx.send(Ok(update)).await.is_err() {
                        tracing::info!("[RPC Stream] Stream connection closed by remote peer");
                        break;
                    }
                }
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::WatchPricesStream))
    }
}

