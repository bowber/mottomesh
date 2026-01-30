use std::time::Duration;

use async_nats::Client;
use bytes::Bytes;
use futures::StreamExt;
use tokio::sync::mpsc;
use tracing::{debug, info};

/// Bridge to NATS messaging system
pub struct NatsBridge {
    client: Client,
}

impl NatsBridge {
    /// Connect to NATS server
    pub async fn connect(url: &str) -> Result<Self, BridgeError> {
        info!("Connecting to NATS at {}", url);
        let client = async_nats::connect(url)
            .await
            .map_err(|e| BridgeError::ConnectionFailed(e.to_string()))?;

        info!("Connected to NATS");
        Ok(Self { client })
    }

    /// Subscribe to a subject and forward messages to the sender
    pub async fn subscribe(
        &self,
        subject: String,
        sender: mpsc::Sender<NatsMessage>,
    ) -> Result<SubscriptionHandle, BridgeError> {
        let subscriber = self
            .client
            .subscribe(subject.clone())
            .await
            .map_err(|e| BridgeError::SubscribeFailed(e.to_string()))?;

        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);

        let subject_clone = subject.clone();
        tokio::spawn(async move {
            let mut subscriber = subscriber;
            loop {
                tokio::select! {
                    msg = subscriber.next() => {
                        match msg {
                            Some(msg) => {
                                let nats_msg = NatsMessage {
                                    subject: msg.subject.to_string(),
                                    payload: msg.payload.to_vec(),
                                };
                                if sender.send(nats_msg).await.is_err() {
                                    debug!("Subscription channel closed for {}", subject_clone);
                                    break;
                                }
                            }
                            None => {
                                debug!("NATS subscription ended for {}", subject_clone);
                                break;
                            }
                        }
                    }
                    _ = cancel_rx.recv() => {
                        debug!("Subscription cancelled for {}", subject_clone);
                        break;
                    }
                }
            }
        });

        Ok(SubscriptionHandle { cancel_tx })
    }

    /// Publish a message to a subject
    pub async fn publish(&self, subject: &str, payload: Vec<u8>) -> Result<(), BridgeError> {
        self.client
            .publish(subject.to_string(), Bytes::from(payload))
            .await
            .map_err(|e| BridgeError::PublishFailed(e.to_string()))?;
        Ok(())
    }

    /// Request-reply pattern
    pub async fn request(
        &self,
        subject: &str,
        payload: Vec<u8>,
        timeout: Duration,
    ) -> Result<Vec<u8>, BridgeError> {
        let response = tokio::time::timeout(
            timeout,
            self.client
                .request(subject.to_string(), Bytes::from(payload)),
        )
        .await
        .map_err(|_| BridgeError::RequestTimeout)?
        .map_err(|e| BridgeError::RequestFailed(e.to_string()))?;

        Ok(response.payload.to_vec())
    }
}

/// Message received from NATS
#[derive(Debug, Clone)]
pub struct NatsMessage {
    pub subject: String,
    pub payload: Vec<u8>,
}

/// Handle to cancel a subscription
pub struct SubscriptionHandle {
    cancel_tx: mpsc::Sender<()>,
}

impl SubscriptionHandle {
    pub async fn unsubscribe(self) {
        let _ = self.cancel_tx.send(()).await;
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    #[error("Failed to connect to NATS: {0}")]
    ConnectionFailed(String),
    #[error("Failed to subscribe: {0}")]
    SubscribeFailed(String),
    #[error("Failed to publish: {0}")]
    PublishFailed(String),
    #[error("Request failed: {0}")]
    RequestFailed(String),
    #[error("Request timed out")]
    RequestTimeout,
}
