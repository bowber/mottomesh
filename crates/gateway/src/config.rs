use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Host to bind to
    pub host: String,
    /// HTTPS port (for WebSocket and WebTransport)
    pub https_port: u16,
    /// NATS server URL
    pub nats_url: String,
    /// JWT secret for token validation
    pub jwt_secret: String,
    /// TLS certificate path (optional, generates self-signed if not provided)
    pub tls_cert_path: Option<String>,
    /// TLS key path (optional, generates self-signed if not provided)
    pub tls_key_path: Option<String>,
}

impl GatewayConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        let jwt_secret = env::var("JWT_SECRET")
            .map_err(|_| ConfigError::MissingEnvVar("JWT_SECRET".to_string()))?;

        Ok(Self {
            host: env::var("GATEWAY_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            https_port: env::var("GATEWAY_PORT")
                .unwrap_or_else(|_| "4433".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidPort)?,
            nats_url: env::var("NATS_URL").unwrap_or_else(|_| "localhost:4222".to_string()),
            jwt_secret,
            tls_cert_path: env::var("TLS_CERT_PATH").ok(),
            tls_key_path: env::var("TLS_KEY_PATH").ok(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),
    #[error("Invalid port number")]
    InvalidPort,
}
