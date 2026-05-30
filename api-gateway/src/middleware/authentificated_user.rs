use axum::extract::FromRequestParts;
use utils::{decode_jwt};

use crate::state::AppState;

pub struct AuthenticatedUser {
    pub user_id: i32,
}

impl FromRequestParts<AppState> for AuthenticatedUser
where
{
    type Rejection = (axum::http::StatusCode, String);
    

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection>
     {
        // 1. Достаем заголовок Authorization
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or((axum::http::StatusCode::UNAUTHORIZED, "Missing Authorization Header".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err((axum::http::StatusCode::UNAUTHORIZED, "Invalid Header Format".to_string()));
        }
        let jwt_key = state.jwt_key();

        let token = &auth_header[7..];

        // 2. Валидируем JWT токен (в реальном проекте секрет берут из ENV)
        let token_data = decode_jwt(jwt_key, token)
            .map_err(|_| (axum::http::StatusCode::UNAUTHORIZED, "Invalid or Expired Token".to_string()))?;

        // 3. Если всё ок, возвращаем user_id
        Ok(AuthenticatedUser {
            user_id: token_data.sub,
        })
    }
}

