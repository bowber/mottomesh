use mottomesh_gateway::{Gateway, GatewayConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mottomesh_gateway=info".parse()?),
        )
        .init();

    let config = GatewayConfig::from_env()?;
    let gateway = Gateway::new(config).await?;
    gateway.run_forever().await?;

    Ok(())
}
