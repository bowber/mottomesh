use std::sync::Arc;
use std::time::Duration;

use tracing::{debug, error, info, warn};
use wtransport::{
    Endpoint, Identity, ServerConfig,
    endpoint::IncomingSession,
};

use crate::auth::JwtValidator;
use crate::bridge::NatsBridge;
use crate::config::GatewayConfig;
use crate::protocol::MessageCodec;

use super::handler::ConnectionHandler;

/// Run the WebTransport server
pub async fn run_server(
    config: GatewayConfig,
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Generate or load TLS certificate
    let identity = match (&config.tls_cert_path, &config.tls_key_path) {
        (Some(cert_path), Some(key_path)) => {
            info!("Loading TLS certificate from {} and {}", cert_path, key_path);
            Identity::load_pemfiles(cert_path, key_path).await?
        }
        _ => {
            info!("Generating self-signed certificate for development");
            Identity::self_signed(["localhost", "127.0.0.1", "::1"])?
        }
    };

    let server_config = ServerConfig::builder()
        .with_bind_default(config.https_port)
        .with_identity(identity)
        .keep_alive_interval(Some(Duration::from_secs(15)))
        .build();

    let server = Endpoint::server(server_config)?;
    
    info!("WebTransport server listening on port {}", config.https_port);

    loop {
        let incoming = server.accept().await;
        
        let jwt = jwt_validator.clone();
        let nats = nats_bridge.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handle_incoming(incoming, jwt, nats).await {
                error!("WebTransport connection error: {}", e);
            }
        });
    }
}

async fn handle_incoming(
    incoming: IncomingSession,
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let session_request = incoming.await?;
    
    info!(
        "WebTransport connection from {} to {}",
        session_request.authority(),
        session_request.path()
    );

    let connection = session_request.accept().await?;
    let stable_id = connection.stable_id();
    
    info!("WebTransport session established: {}", stable_id);

    let mut handler = ConnectionHandler::new(jwt_validator, nats_bridge);

    loop {
        tokio::select! {
            // Handle incoming bidirectional streams
            stream = connection.accept_bi() => {
                match stream {
                    Ok((mut send, mut recv)) => {
                        // Read message from stream
                        let mut buf = vec![0u8; 65536];
                        match recv.read(&mut buf).await {
                            Ok(Some(n)) => {
                                if let Some(response) = handler.handle_message(&buf[..n]).await {
                                    let encoded = MessageCodec::encode_server(&response);
                                    if let Err(e) = send.write_all(&encoded).await {
                                        warn!("Failed to send response: {}", e);
                                    }
                                }
                            }
                            Ok(None) => {
                                debug!("Stream closed");
                            }
                            Err(e) => {
                                warn!("Error reading from stream: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Error accepting stream: {}", e);
                        break;
                    }
                }
            }
            
            // Handle datagrams (unreliable, low-latency messages)
            datagram = connection.receive_datagram() => {
                match datagram {
                    Ok(data) => {
                        if let Some(response) = handler.handle_message(&data).await {
                            let encoded = MessageCodec::encode_server(&response);
                            if let Err(e) = connection.send_datagram(encoded) {
                                warn!("Failed to send datagram response: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        // Connection errors here mean the session is done
                        debug!("Datagram/connection error: {}", e);
                        break;
                    }
                }
            }
            
            // Handle NATS messages to forward to client
            nats_msg = handler.nats_receiver().recv() => {
                if let Some(nats_msg) = nats_msg
                    && let Some(server_msg) = handler.nats_to_server_message(nats_msg)
                {
                    let encoded = MessageCodec::encode_server(&server_msg);
                    // Use datagram for subscription messages (faster, no head-of-line blocking)
                    if connection.send_datagram(encoded.clone()).is_err() {
                        // Fall back to reliable stream if datagram fails
                        match connection.open_uni().await {
                            Ok(opening) => {
                                // Await the opening stream to get the actual SendStream
                                if let Ok(mut send) = opening.await {
                                    let _ = send.write_all(&encoded).await;
                                }
                            }
                            Err(_) => {
                                // Connection is likely closed
                                break;
                            }
                        }
                    }
                }
            }
            
            // Check if connection is closed
            _ = connection.closed() => {
                info!("WebTransport connection closed: {}", stable_id);
                break;
            }
        }
    }

    handler.cleanup().await;
    Ok(())
}
