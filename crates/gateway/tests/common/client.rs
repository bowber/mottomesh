use std::time::Duration;

use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

use mottomesh_gateway::protocol::{ClientMessage, MessageCodec, ServerMessage};

/// WebSocket test client
pub struct TestClient {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl TestClient {
    /// Connect to a WebSocket URL
    pub async fn connect(url: &str) -> Self {
        let (ws, _) = connect_async(url)
            .await
            .expect("Failed to connect to WebSocket");

        Self { ws }
    }

    /// Send a client message
    pub async fn send(&mut self, msg: ClientMessage) {
        let encoded = MessageCodec::encode_client(&msg);
        self.ws
            .send(Message::Binary(encoded))
            .await
            .expect("Failed to send message");
    }

    /// Receive a server message with timeout
    pub async fn recv(&mut self) -> Option<ServerMessage> {
        self.recv_timeout(Duration::from_secs(5)).await
    }

    /// Receive a server message with custom timeout
    pub async fn recv_timeout(&mut self, timeout: Duration) -> Option<ServerMessage> {
        match tokio::time::timeout(timeout, self.ws.next()).await {
            Ok(Some(Ok(Message::Binary(data)))) => {
                Some(MessageCodec::decode_server(&data).expect("Failed to decode message"))
            }
            Ok(Some(Ok(_))) => {
                // Non-binary message, try again
                Box::pin(self.recv_timeout(timeout)).await
            }
            Ok(Some(Err(e))) => {
                panic!("WebSocket error: {}", e);
            }
            Ok(None) => None,
            Err(_) => {
                // Timeout
                None
            }
        }
    }

    /// Authenticate with the gateway
    pub async fn auth(&mut self, token: &str) -> Result<String, String> {
        self.send(ClientMessage::Auth {
            token: token.to_string(),
        })
        .await;

        match self.recv().await {
            Some(ServerMessage::AuthOk { session_id }) => Ok(session_id),
            Some(ServerMessage::AuthError { reason }) => Err(reason),
            Some(other) => Err(format!("Unexpected response: {:?}", other)),
            None => Err("No response received".to_string()),
        }
    }

    /// Subscribe to a subject
    pub async fn subscribe(&mut self, subject: &str, id: u64) -> Result<u64, String> {
        self.send(ClientMessage::Subscribe {
            subject: subject.to_string(),
            id,
        })
        .await;

        match self.recv().await {
            Some(ServerMessage::SubscribeOk { id }) => Ok(id),
            Some(ServerMessage::SubscribeError { id: _, reason }) => Err(reason),
            Some(other) => Err(format!("Unexpected response: {:?}", other)),
            None => Err("No response received".to_string()),
        }
    }

    /// Publish a message
    pub async fn publish(&mut self, subject: &str, payload: &[u8]) {
        self.send(ClientMessage::Publish {
            subject: subject.to_string(),
            payload: payload.to_vec(),
        })
        .await;
    }

    /// Send a ping
    pub async fn ping(&mut self) {
        self.send(ClientMessage::Ping).await;
    }

    /// Close the connection
    pub async fn close(mut self) {
        let _ = self.ws.close(None).await;
    }
}
