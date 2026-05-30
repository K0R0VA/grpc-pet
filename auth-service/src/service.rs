
use argon2::{Params, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};
use sea_orm::{ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};
use tonic::{Response, Status};
use utils::{Error, from_env};
use proto_hub::auth::{AuthRequest, AuthResponse, auth_service_server::AuthService};

pub struct AuthController {
    pub database: DatabaseConnection,
    pub secret_key: Vec<u8>
}

impl AuthController {
    pub async fn initiate() -> Result<Self, Error> {
        let connection_url = from_env("DATABASE_URL")?;
        let secret_key = from_env("SECRET_KEY")?;
        let secret_key = hex::decode(&secret_key).expect("secret key decode failed");
        let database = Database::connect(&connection_url).await?;
        Ok(Self {
            database,
            secret_key
        })
    }
}
#[tonic::async_trait]
impl AuthService for AuthController {
    async fn login(&self, request: tonic::Request<AuthRequest>) ->  Result<Response<AuthResponse>, Status> {
        let AuthRequest { name, password } = request.into_inner();
        let entities::user::Model {id, password: hash, ..} = entities::user::Entity::find()
            .filter(entities::user::Column::Name.eq(name))
            .one(&self.database)
            .await
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::unauthenticated("User not found"))?;
        let argon2 = argon2::Argon2::new_with_secret(&self.secret_key, argon2::Algorithm::Argon2d, argon2::Version::V0x10, Params::DEFAULT)
            .map_err(|e| Status::internal(e.to_string()))?;
        let hash = PasswordHash::new(&hash).map_err(|e| Status::internal(e.to_string()))?;
        argon2.verify_password(password.as_bytes(), &hash).map_err(|_| Status::unauthenticated("Incorrent password"))?;
        Ok(Response::new(AuthResponse { success: true, token: "".to_string(), user_id: id }))
    }
    async fn register(&self, request: tonic::Request<proto_hub::auth::AuthRequest>) ->  Result<Response<AuthResponse>, Status> {
        let AuthRequest { name, password } = request.into_inner();
        let argon2 = argon2::Argon2::new_with_secret(&self.secret_key, argon2::Algorithm::Argon2d, argon2::Version::V0x10, Params::DEFAULT)
            .map_err(|e| Status::internal(e.to_string()))?;
        let hash = argon2.hash_password(password.as_bytes(), &SaltString::generate(&mut OsRng))
            .map_err(|e| Status::internal(e.to_string()))?
            .to_string();
        let user = entities::user::ActiveModel {
            name: sea_orm::ActiveValue::Set(name),
            password: sea_orm::ActiveValue::Set(hash),
            role: sea_orm::ActiveValue::Set(entities::user::Role::User),
            ..Default::default()
        };
        let user = user.into_active_model()
            .insert(&self.database)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(AuthResponse { success: true, token: "".to_string(), user_id: user.id }))
    }
}

