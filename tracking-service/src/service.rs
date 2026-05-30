use std::pin::Pin;

use proto_hub::tracking::{AlertRequest, AlertResponse, PriceUpdate, RouteRequest};
use proto_hub::tracking::tracking_service_server::TrackingService;
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel};
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
        request: Request<RouteRequest>,
    ) -> Result<Response<Self::WatchPricesStream>, Status> {
        tracing::info!("[RPC Server] Establishing WatchPrices stream channel");

        let RouteRequest { route_id } = request.into_inner();

        let route = entities::route::Entity::find_by_id(route_id.clone())
            .one(&self.database)
            .await
            .map_err(|_| Status::internal("Database check failed"))?
            .ok_or(Status::not_found("Route not found"))?;

        let route = format!("{}-{}", route.from, route.to);

        let (tx, rx) = tokio::sync::mpsc::channel(128); // В продакшене размер буфера берут с запасом

        tokio::spawn(async move {
            let mut current_price = 500.0;

            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                current_price -= 5.0;

                let update = PriceUpdate {
                    route: route.clone(),
                    current_price,
                };

                if tx.send(Ok(update)).await.is_err() {
                    tracing::info!("[RPC Stream] Stream connection closed by remote peer");
                    break;
                }
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::WatchPricesStream))
    }
}

