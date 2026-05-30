use axum::Json;
use axum::extract::State;
use proto_hub::tracking::AlertRequest;
use serde::Deserialize;

use crate::middleware::authentificated_user::AuthenticatedUser;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct CreateAlert {
    route: i32,
    target_price: f32,
}

pub async fn handle_create_alert(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateAlert>, // Автоматически парсим JSON из тела запроса
) -> Result<String, String> {
    println!("[Gateway] HTTP request received for user: {}", user.user_id);

    // Превращаем данные из JSON (HTTP) в структуру Protobuf (gRPC)
    let grpc_request = AlertRequest {
        user_id: user.user_id,
        route: payload.route,
        target_price: payload.target_price,
    };

    // Блокируем мутекс, чтобы сделать запрос через gRPC клиент
    // Вызываем удаленный gRPC метод add_alert на tracking-service по сети!
    match state.alert(grpc_request).await {
        Ok(grpc_response) => {
            // Возвращаем клиенту успешный ответ
            Ok(format!(
                "Success! Alert created. Status: {}, ID: {}",
                grpc_response.success, grpc_response.alert_id
            ))
        }
        Err(status) => {
            // Если gRPC сервер ответил ошибкой, переводим ее в HTTP-текст
            Err(format!("gRPC Error: {}", status.message()))
        }
    }
}