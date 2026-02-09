pub mod auth;
pub mod bridge;
pub mod config;
pub mod protocol;
pub mod transport;

use std::sync::Arc;

use auth::JwtValidator;
use bridge::NatsBridge;
pub use config::GatewayConfig;
use tokio::sync::oneshot;
use tracing::{error, info};

pub struct Gateway {
    config: GatewayConfig,
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
}

impl Gateway {
    pub async fn new(
        config: GatewayConfig,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let jwt_validator = Arc::new(JwtValidator::new(&config.jwt_secret)?);
        let nats_bridge = Arc::new(NatsBridge::connect(&config.nats_url).await?);

        Ok(Self {
            config,
            jwt_validator,
            nats_bridge,
        })
    }

    /// Create a gateway with pre-built components (for testing)
    pub fn with_components(
        config: GatewayConfig,
        jwt_validator: Arc<JwtValidator>,
        nats_bridge: Arc<NatsBridge>,
    ) -> Self {
        Self {
            config,
            jwt_validator,
            nats_bridge,
        }
    }

    /// Run the gateway until shutdown signal is received
    /// Returns the actual WebSocket port the server bound to
    pub async fn run(
        self,
        shutdown: oneshot::Receiver<()>,
    ) -> Result<u16, Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting gateway...");

        let ws_jwt = self.jwt_validator.clone();
        let ws_nats = self.nats_bridge.clone();

        // Run WebSocket server with shutdown support
        let (actual_port, server_handle) = transport::websocket::run_server(
            self.config.host.clone(),
            self.config.ws_port,
            ws_jwt,
            ws_nats,
        )
        .await?;

        info!("WebSocket server listening on port {}", actual_port);

        // Wait for shutdown signal
        tokio::select! {
            _ = shutdown => {
                info!("Shutdown signal received");
            }
            result = server_handle => {
                if let Err(e) = result {
                    error!("Server error: {}", e);
                }
            }
        }

        Ok(actual_port)
    }

    /// Run without shutdown signal (for production use)
    pub async fn run_forever(self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Starting gateway...");
        info!(
            "WebSocket endpoint: ws://{}:{}",
            self.config.host, self.config.ws_port
        );

        let ws_jwt = self.jwt_validator.clone();
        let ws_nats = self.nats_bridge.clone();

        let (actual_port, server_handle) = transport::websocket::run_server(
            self.config.host.clone(),
            self.config.ws_port,
            ws_jwt,
            ws_nats,
        )
        .await?;

        info!("WebSocket server listening on port {}", actual_port);

        server_handle.await??;

        Ok(())
    }
}
