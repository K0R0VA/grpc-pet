use std::net::SocketAddr;

use proto_hub::auth::{AuthRequest, AuthResponse};
use proto_hub::auth::auth_service_client::AuthServiceClient;
use proto_hub::tracking::{AlertRequest, AlertResponse};
use proto_hub::tracking::tracking_service_client::TrackingServiceClient;
use tonic::metadata::MetadataValue;
use utils::{Error, JwtKey};


#[derive(Clone)]
pub struct AppState {
    tracking_client: TrackingServiceClient<tonic::transport::Channel>,
    auth_client: AuthServiceClient<tonic::transport::Channel>,
    key: JwtKey,
}

impl AppState {
    pub fn new() -> Result<Self, Error> {
        let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
            .concurrency_limit(1024) 
            .connect_lazy();        
        let tracking_client = TrackingServiceClient::new(channel);
        let channel = tonic::transport::Channel::from_static("http://[::1]:50051")
            .concurrency_limit(1024) 
            .connect_lazy();        
        let auth_client = AuthServiceClient::new(channel);
        let jwt_key = JwtKey::from_env()?;
        Ok(Self {tracking_client, auth_client, key: jwt_key})
    }
    pub fn jwt_key(&self) -> &[u8] {
        &self.key.key
    }
    pub async fn alert(mut self, request: AlertRequest) -> Result<AlertResponse, tonic::Status> {
        let request = tonic::Request::new(request);
        let response = self.tracking_client.add_alert(request).await?;
        Ok(response.into_inner())
    }
    pub async fn login(mut self, request: AuthRequest, socket_addr: SocketAddr) -> Result<AuthResponse, tonic::Status> {
        let mut request = tonic::Request::new(request);
        let value = MetadataValue::from_bytes(socket_addr.to_string().as_bytes());
        request.metadata_mut().insert_bin("x-forwarded-for", value);
        let response = self.auth_client.login(request).await?;
        Ok(response.into_inner())
    }
    pub async fn register(mut self, request: AuthRequest, socket_addr: SocketAddr) -> Result<AuthResponse, tonic::Status> {
        let mut request = tonic::Request::new(request);
        let value = MetadataValue::from_bytes(socket_addr.to_string().as_bytes());
        request.metadata_mut().insert_bin("x-forwarded-for", value);
        let response = self.auth_client.register(request).await?;
        Ok(response.into_inner())
    }
}