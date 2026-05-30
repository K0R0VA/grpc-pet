use std::pin::Pin;

use proto_hub::price_receiver::price_receiver_service_server::{PriceReceiverService};
use proto_hub::price_receiver::{PriceUpdateRequest, PriceUpdateResponse};
use sea_orm::ActiveValue::{Set, Unchanged};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection};
use tokio_stream::StreamExt;
use tonic::{Request, Response, Status};
use utils::{Error, from_env};

pub struct PriceReceiver {
    database: DatabaseConnection,
}

impl PriceReceiver {
    pub async fn initiate() -> Result<Self, Error> {
        let connection_url = from_env("DATABASE_URL")?;
        let database = Database::connect(&connection_url).await?;
        Ok(Self {
            database
        })
    }
}

#[tonic::async_trait]
impl PriceReceiverService for PriceReceiver {
    type ReceivePricesStream = Pin<Box<dyn tokio_stream::Stream<Item = Result<PriceUpdateResponse, Status>> + Send>>;

    async fn receive_prices(
        &self,
        request: Request<tonic::Streaming<PriceUpdateRequest>>,
    ) -> Result<Response<Self::ReceivePricesStream>, Status> {
        let mut stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(128); // В продакшене размер буфера берут с запасом
        let database = self.database.clone();
        tokio::spawn(async move {
            while let Some(r) = stream.next().await {
                let PriceUpdateRequest { route_id, new_price } = match r {
                    Ok(update) => update,
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                };
                let model = entities::route::ActiveModel {
                    id: Unchanged(route_id),
                    current_price: Set(new_price),
                    ..Default::default()
                };
                let result = match model.update(&database).await {
                    Ok(_) => tx.send(Ok(PriceUpdateResponse {})).await,
                    Err(e) => {
                        let _ = tx.send(Err(Status::internal(e.to_string()))).await;
                        break;
                    }
                };
                if result.is_err() {
                    tracing::info!("[RPC Stream] Stream connection closed by remote peer");
                }

            }
        });
        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::ReceivePricesStream))
    }
}