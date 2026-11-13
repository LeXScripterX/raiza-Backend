use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    /// Prefijo completo de la API: `/api`, o `/<CONTEXT_PATH>/api` si se define.
    /// En Java el WAR se desplegaba en `/YuweLongo-Backend`, así que las URLs eran
    /// `/YuweLongo-Backend/api/...`. Con CONTEXT_PATH=/YuweLongo-Backend el frontend no cambia.
    pub api_prefix: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "mysql://userYuwe:123456@localhost:3306/yuwelongo".to_string());
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "fQw8vDNz3pL!eR$Yx7uT@9sZqG2nH#jM".to_string());
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .ok()
            .and_then(|h| h.parse().ok())
            .unwrap_or(1);

        let context_path = env::var("CONTEXT_PATH").unwrap_or_default();
        let context_path = context_path.trim().trim_matches('/');
        let api_prefix = if context_path.is_empty() {
            "/api".to_string()
        } else {
            format!("/{context_path}/api")
        };

        Self {
            database_url,
            host,
            port,
            jwt_secret,
            jwt_expiration_hours,
            api_prefix,
        }
    }
}
