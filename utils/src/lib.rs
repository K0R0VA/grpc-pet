use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::Error::EnvVar;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Env var key {0} not found")]
    EnvVar(String),
    #[error(transparent)]
    Tonic(#[from] tonic::transport::Error),
    #[error(transparent)]
    Postgres(#[from] sea_orm::DbErr),
    #[error(transparent)]
    Jwt (#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Hex(#[from] hex::FromHexError)
}

pub fn from_env(key: &str) -> Result<String, Error> {
    let value = std::env::var(key).map_err(|_| EnvVar(key.to_string()))?;
    Ok(value)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32,
    exp: usize,
}

pub fn generate_jwt(jwt_key: &[u8], user_id: i32) -> Result<String, Error> {
    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;
    let claims = Claims { sub: user_id, exp: expiration };
    let jwt = encode(&Header::default(), &claims, &EncodingKey::from_secret(jwt_key))?;
    Ok(jwt)
}

pub fn decode_jwt(jwt_key: &[u8], token: &str) -> Result<Claims, Error> {
    let jwt = decode(token, &DecodingKey::from_secret(jwt_key), &Validation::default())?;
    Ok(jwt.claims)
}

#[derive(Clone, Copy)]
pub struct JwtKey {
    pub key: [u8; 64]
}

impl JwtKey {
    pub fn from_env() -> Result<JwtKey, Error> {
        let jwt_key = from_env("JWT_KEY")?;
        let mut key = [0; 64];
        hex::decode_to_slice(jwt_key, &mut key)?;
        Ok(JwtKey { key })
    }
}

#[cfg(test)]
mod test {
    use rand::{Rng, rngs};

    #[test]
    fn generate_jwt_key() {
        let mut bytes = [0; 64];
        Rng::fill(&mut rngs::OsRng, &mut bytes);
        let jwt = hex::encode(bytes);
        println!("{jwt}")
    }
}