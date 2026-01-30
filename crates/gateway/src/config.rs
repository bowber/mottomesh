use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Host to bind to
    pub host: String,
    /// WebSocket port
    pub ws_port: u16,
    /// NATS server URL
    pub nats_url: String,
    /// JWT secret for token validation
    pub jwt_secret: String,
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        Ok(Self {
            host: env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            ws_port: env::var("GATEWAY_WS_PORT")
                .unwrap_or_else(|_| "4434".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string()),
            jwt_secret,
        })
    }

    /// Create a config for testing
    pub fn for_test(ws_port: u16, nats_url: &str, jwt_secret: &str) -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            ws_port,
            nats_url: nats_url.to_string(),
            jwt_secret: jwt_secret.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid port number")]
    InvalidPort,
}
