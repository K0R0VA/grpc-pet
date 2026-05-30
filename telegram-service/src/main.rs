use entities::subscription::SubscriptionType;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use teloxide::prelude::*;
use teloxide::types::ChatId;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("[Telegram Service] Initializing telegram notification worker...");

    // 1. Инициализируем бота. Токен он автоматически возьмет из переменной среды TELOXIDE_TOKEN
    let bot = Bot::from_env();
    println!("[Telegram Service] Telegram Bot API client initialized.");

    // 2. Настраиваем потребителя (Consumer) Кафки для топика телеграма
    let consumer: StreamConsumer = ClientConfig::new()
        .set("bootstrap.servers", "localhost:9092")
        .set("group.id", "telegram-delivery-group") // Своя выделенная группа
        .set("enable.auto.commit", "true")
        .set("auto.offset.reset", "earliest")
        .create()?;
    let topic = format!("price-alerts-{:?}", SubscriptionType::Telegram);
    // Подписываемся строго на топик для Телеграма
    consumer.subscribe(&[&topic])?;
    println!("[Telegram Service] Subscribed to Kafka topic: 'notifications.telegram'. Listening...");

    // 3. Бесконечный цикл чтения очереди Кафки
    loop {
        match consumer.recv().await {
            Err(e) => {
                println!("[Kafka Error] Error receiving from Kafka: {:?}", e);
            }
            Ok(message) => {
                if let Some(payload) = message.payload_view::<str>() {
                    match payload {
                        Ok(json_str) => {
                            // Парсим JSON с данными о билете
                            let parsed: serde_json::Value = serde_json::from_str(json_str)?;
                            
                            // Извлекаем параметры. В вашей системе user_id должен быть равен Telegram Chat ID
                            let user_id_str = parsed["user_id"].as_str().unwrap_or("0");
                            let route = parsed["route_id"].as_str().unwrap_or("UNKNOWN");
                            let price = parsed["price"].as_f64().unwrap_or(0.0);

                            if let Ok(chat_id_num) = user_id_str.parse::<i64>() {
                                let chat_id = ChatId(chat_id_num);
                                
                                // Формируем красивый текст сообщения для пользователя
                                let message_text = format!(
                                    "✈️ *Скидка на авиабилет!*\n\n\
                                     📍 Маршрут: *{}*\n\
                                     💰 Новая цена: *${:.2}*\n\n\
                                     Успейте купить, пока цена не выросла!",
                                    route, price
                                );

                                println!("[Telegram Service] Sending notification to Chat ID: {}", chat_id_num);

                                // Отправляем сообщение через API Телеграма (с поддержкой Markdown форматирования)
                                if let Err(err) = bot.send_message(chat_id, message_text)
                                    .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                                    .await 
                                {
                                    println!("[Telegram API Error] Failed to send message: {:?}", err);
                                } else {
                                    println!(" -> [Telegram Sent] Message delivered to user {}", chat_id_num);
                                }
                            } else {
                                println!("[Error] user_id '{}' is not a valid Telegram Chat ID integer", user_id_str);
                            }
                        }
                        Err(e) => println!("[Parser Error] Invalid UTF-8 JSON: {:?}", e),
                    }
                }
            }
        }
    }
}