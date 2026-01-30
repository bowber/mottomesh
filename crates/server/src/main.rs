use std::io::Read;

use flate2::{Compression, bufread::GzDecoder, read::GzEncoder};
use futures::StreamExt;
use mottomesh::{CustomError, TestData};
use tracing::{error, info};

fn compress(data: &[u8]) -> Result<Vec<u8>, CustomError> {
    let mut ret_vec = Vec::new();
    let mut gz = GzEncoder::new(data, Compression::fast());
    gz.read_to_end(&mut ret_vec)?;
    Ok(ret_vec)
}

fn decompress(bytes: &[u8]) -> Result<Vec<u8>, CustomError> {
    let mut gz = GzDecoder::new(bytes);
    let mut b = Vec::new();
    gz.read_to_end(&mut b)?;
    Ok(b)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("mottomesh_server=info".parse()?)
        )
        .init();

    info!("Starting mottomesh server...");

    // Connect to the NATS server
    let client = async_nats::connect("localhost:4222").await?;
    info!("Connected to NATS");

    // Subscribe to the "messages" subject
    let mut subscriber = client.subscribe("messages").await?;

    // Publish messages to the "messages" subject
    for i in 0..3 {
        let name = format!("Test {}", i);
        let data = TestData::new(i, name.as_str());
        let compressed = compress(&data.encode()?)?;
        client.publish("messages", compressed.into()).await?;
        info!("Tx: Published message {}", name);
    }

    // Receive and process messages
    while let Some(message) = subscriber.next().await {
        let decompressed = match decompress(&message.payload) {
            Ok(data) => data,
            Err(e) => {
                error!(
                    "Rx Failed to decompress {:?} bytes message: {:#?}",
                    message.length,
                    e.message()
                );
                continue;
            }
        };
        match TestData::decode(&decompressed) {
            Ok(data) => info!(
                "Rx: Received {:?} bytes message: {:?}",
                message.length,
                data.name()
            ),
            Err(e) => error!(
                "Rx Failed to decode {:?} bytes message: {:#?}",
                message.length,
                e.message()
            ),
        }
    }

    Ok(())
}
