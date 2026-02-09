use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::IntoResponse,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use tokio::task::JoinHandle;
use tower_http::cors::{Any, CorsLayer};
use tracing::{debug, info, warn};

use crate::auth::JwtValidator;
use crate::bridge::NatsBridge;
use crate::protocol::MessageCodec;

use super::handler::ConnectionHandler;

/// Shared state for WebSocket handlers
#[derive(Clone)]
struct AppState {
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
}

/// Run the WebSocket server
/// Returns the actual bound port and a handle to the server task
pub async fn run_server(
    host: String,
    port: u16,
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
) -> Result<(u16, JoinHandle<Result<(), std::io::Error>>), Box<dyn std::error::Error + Send + Sync>>
{
    let state = AppState {
        jwt_validator,
        nats_bridge,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_handler))
        .with_state(state)
        .layer(cors);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    let actual_addr = listener.local_addr()?;
    let actual_port = actual_addr.port();

    info!("WebSocket server listening on {}", actual_addr);

    let handle = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
    });

    Ok((actual_port, handle))
}

async fn health_handler() -> &'static str {
    "OK"
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    info!("WebSocket connection from {}", addr);
    ws.on_upgrade(move |socket| handle_socket(socket, state, addr))
}

async fn handle_socket(socket: WebSocket, state: AppState, addr: SocketAddr) {
    let mut handler = ConnectionHandler::new(state.jwt_validator, state.nats_bridge);

    let (mut sender, mut receiver) = socket.split();

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Binary(data))) => {
                        if let Some(response) = handler.handle_message(&data).await {
                            let encoded = MessageCodec::encode_server(&response);
                            if sender.send(Message::Binary(encoded.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) => {
                        debug!("WebSocket closed by client {}", addr);
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(_)) => {
                        // Ignore text messages and other types
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error from {}: {}", addr, e);
                        break;
                    }
                    None => {
                        debug!("WebSocket stream ended for {}", addr);
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
                    if sender.send(Message::Binary(encoded.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    handler.cleanup().await;
    info!("WebSocket connection closed for {}", addr);
}
