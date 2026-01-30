// Suppress false positive warnings from bitcode derive macros
#![allow(unused_assignments)]

mod auth;
mod bridge;
mod config;
mod protocol;
mod transport;

use std::sync::Arc;

use auth::JwtValidator;
use bridge::NatsBridge;
use config::GatewayConfig;
use tracing::{error, info};

pub struct Gateway {
    config: GatewayConfig,
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
}

impl Gateway {
    pub async fn new(config: GatewayConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let jwt_validator = Arc::new(JwtValidator::new(&config.jwt_secret)?);
        let nats_bridge = Arc::new(NatsBridge::connect(&config.nats_url).await?);

        Ok(Self {
            config,
            jwt_validator,
            nats_bridge,
        })
    }

    pub async fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting gateway...");
        info!(
            "WebSocket endpoint: wss://{}:{}/ws",
            self.config.host, self.config.https_port
        );
        info!(
            "WebTransport endpoint: https://{}:{}/wt",
            self.config.host, self.config.https_port
        );

        let ws_config = self.config.clone();
        let ws_jwt = self.jwt_validator.clone();
        let ws_nats = self.nats_bridge.clone();

        let wt_config = self.config.clone();
        let wt_jwt = self.jwt_validator.clone();
        let wt_nats = self.nats_bridge.clone();

        // Run both transports concurrently
        tokio::select! {
            r = transport::websocket::run_server(ws_config, ws_jwt, ws_nats) => {
                if let Err(e) = r {
                    error!("WebSocket server error: {}", e);
                }
            }
            r = transport::webtransport::run_server(wt_config, wt_jwt, wt_nats) => {
                if let Err(e) = r {
                    error!("WebTransport server error: {}", e);
                }
            }
        }

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mottomesh_gateway=info".parse()?)
                .add_directive("wtransport=info".parse()?),
        )
        .init();

    let config = GatewayConfig::from_env()?;
    let gateway = Gateway::new(config).await?;
    gateway.run().await?;

    Ok(())
}
