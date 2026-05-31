use redis::{AsyncCommands};
use tonic::{Request, Status, service::Interceptor};
use utils::{Error, from_env};

#[derive(Clone)]
pub struct RateLimiter {
    redis_client: redis::Client,
    max_requests: u32,
    window_secs: u64,
}

impl RateLimiter {
    pub fn new() -> Result<Self, Error> {
        let max_requests = from_env("MAX_REQUESTS")?.parse().expect("failed parse MAX_REQUESTS");
        let window_secs = from_env("WINDOWS_REQUEST_SECS")?.parse().expect("failed parse WINDOWS_REQUEST_SECS");
        let redis_client = redis::Client::open(from_env("REDIS_URL")?)?;
        Ok(Self {
            max_requests,
            redis_client,
            window_secs
        })
    }
    async fn check_limit(&self, client_ip: &str) -> Result<(), Status> {
        // Получаем асинхронное соединение из пула клиента
        let mut con = self.redis_client.get_multiplexed_async_connection().await
            .map_err(|e| Status::internal(format!("Redis connection failed: {}", e)))?;

        // Ключ в Redis будет выглядеть как: ratelimit:192.168.1.50
        let redis_key = format!("ratelimit:{}", client_ip);

        // Атомарно инкрементируем счетчик в Redis
        // Команда INCR возвращает текущее значение после увеличения. Если ключа не было, создаст его со значением 1.
        let current_count: u32 = con.incr(&redis_key, 1).await
            .map_err(|e| Status::internal(e.to_string()))?;

        // Если это первый запрос в текущем окне (счетчик равен 1), выставляем время жизни ключа (TTL)
        if current_count == 1 {
            let _: () = con.expire(&redis_key, self.window_secs as i64).await
                .map_err(|e| Status::internal(e.to_string()))?;
        }

        // Если лимит превышен — блокируем запрос
        if current_count > self.max_requests {
            tracing::info!("[Redis Rate Limiter] ❌ IP {} BLOCKED ({} > {} requests)", client_ip, current_count, self.max_requests);
            return Err(Status::resource_exhausted(format!(
                "Too many requests. Distributed Redis limit exceeded. Allowed: {} req/{}s", 
                self.max_requests, self.window_secs
            )));
        }

        Ok(())
    }
}

impl Interceptor for RateLimiter {
    fn call(&mut self, request: Request<()>) -> Result<Request<()>, Status> {
        // Достаем IP-адрес из метаданных gRPC
        let client_ip = request
            .metadata()
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("127.0.0.1")
            .to_owned();
        let service = self.clone();
        tokio::runtime::Handle::current().block_on(async move {
            service.check_limit(&client_ip).await
        })?;
        Ok(request)
    }
}