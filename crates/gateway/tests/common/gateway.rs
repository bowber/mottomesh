use std::sync::Arc;

use mottomesh_gateway::{auth::JwtValidator, bridge::NatsBridge, transport};
use tokio::task::JoinHandle;

use super::jwt::TEST_JWT_SECRET;

/// Test gateway wrapper with shutdown capability
pub struct TestGateway {
    pub port: u16,
    _server_handle: JoinHandle<Result<(), std::io::Error>>,
}

impl TestGateway {
    /// Start a new test gateway
    pub async fn start(nats_url: &str) -> Self {
        Self::start_with_secret(nats_url, TEST_JWT_SECRET).await
    }

    /// Start with a custom JWT secret
    pub async fn start_with_secret(nats_url: &str, jwt_secret: &str) -> Self {
        let jwt_validator =
            Arc::new(JwtValidator::new(jwt_secret).expect("Failed to create JWT validator"));
        let nats_bridge = Arc::new(
            NatsBridge::connect(nats_url)
                .await
                .expect("Failed to connect to NATS"),
        );

        // Start WebSocket server on port 0 (OS assigns free port)
        let (port, server_handle) = transport::websocket::run_server(
            "127.0.0.1".to_string(),
            0,
            jwt_validator,
            nats_bridge,
        )
        .await
        .expect("Failed to start WebSocket server");

        Self {
            port,
            _server_handle: server_handle,
        }
    }

    /// Get the WebSocket URL
    pub fn ws_url(&self) -> String {
        format!("ws://127.0.0.1:{}/ws", self.port)
    }
}
