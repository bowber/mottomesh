use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::auth::{JwtValidator, Permission, PermissionChecker, Session};
use crate::bridge::{NatsBridge, NatsMessage, SubscriptionHandle};
use crate::protocol::{ClientMessage, MessageCodec, ServerMessage, messages::error_codes};

/// Handles the logic for a single client connection
/// This is transport-agnostic - works for both WebSocket and WebTransport
pub struct ConnectionHandler {
    jwt_validator: Arc<JwtValidator>,
    nats_bridge: Arc<NatsBridge>,
    session: Option<Session>,
    subscriptions: HashMap<u64, SubscriptionHandle>,
    /// Channel for receiving NATS messages
    nats_rx: mpsc::Receiver<NatsMessage>,
    /// Sender for NATS messages (given to subscription tasks)
    nats_tx: mpsc::Sender<NatsMessage>,
}

impl ConnectionHandler {
    pub fn new(jwt_validator: Arc<JwtValidator>, nats_bridge: Arc<NatsBridge>) -> Self {
        let (nats_tx, nats_rx) = mpsc::channel(256);
        
        Self {
            jwt_validator,
            nats_bridge,
            session: None,
            subscriptions: HashMap::new(),
            nats_rx,
            nats_tx,
        }
    }

    /// Check if the connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.session.is_some()
    }

    /// Get the session ID if authenticated
    #[allow(dead_code)]
    pub fn session_id(&self) -> Option<&str> {
        self.session.as_ref().map(|s| s.id.as_str())
    }

    /// Process an incoming message and return a response
    pub async fn handle_message(&mut self, data: &[u8]) -> Option<ServerMessage> {
        let msg = match MessageCodec::decode_client(data) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to decode client message: {}", e);
                return Some(ServerMessage::Error {
                    code: error_codes::INVALID_MESSAGE,
                    message: "Invalid message format".to_string(),
                });
            }
        };

        // Check authentication for messages that require it
        if msg.requires_auth() && !self.is_authenticated() {
            return Some(ServerMessage::Error {
                code: error_codes::UNAUTHORIZED,
                message: "Not authenticated".to_string(),
            });
        }

        match msg {
            ClientMessage::Auth { token } => self.handle_auth(&token).await,
            ClientMessage::Subscribe { subject, id } => self.handle_subscribe(subject, id).await,
            ClientMessage::Unsubscribe { id } => self.handle_unsubscribe(id).await,
            ClientMessage::Publish { subject, payload } => {
                self.handle_publish(&subject, payload).await
            }
            ClientMessage::Request {
                subject,
                payload,
                timeout_ms,
                request_id,
            } => {
                self.handle_request(&subject, payload, timeout_ms, request_id)
                    .await
            }
            ClientMessage::Ping => Some(ServerMessage::Pong),
        }
    }

    /// Try to receive a NATS message (non-blocking)
    #[allow(dead_code)]
    pub fn try_recv_nats(&mut self) -> Option<NatsMessage> {
        self.nats_rx.try_recv().ok()
    }

    /// Get a reference to the NATS receiver for select!
    pub fn nats_receiver(&mut self) -> &mut mpsc::Receiver<NatsMessage> {
        &mut self.nats_rx
    }

    /// Convert a NATS message to a ServerMessage
    pub fn nats_to_server_message(&self, nats_msg: NatsMessage) -> Option<ServerMessage> {
        let session = self.session.as_ref()?;
        
        // Find the subscription ID for this subject
        for (sub_id, subject) in &session.subscriptions {
            if *subject == nats_msg.subject {
                return Some(ServerMessage::Message {
                    subscription_id: *sub_id,
                    subject: nats_msg.subject,
                    payload: nats_msg.payload,
                });
            }
        }

        // Subject might match via wildcard - find any matching subscription
        for (sub_id, pattern) in &session.subscriptions {
            if subject_matches_pattern(pattern, &nats_msg.subject) {
                return Some(ServerMessage::Message {
                    subscription_id: *sub_id,
                    subject: nats_msg.subject,
                    payload: nats_msg.payload,
                });
            }
        }

        None
    }

    async fn handle_auth(&mut self, token: &str) -> Option<ServerMessage> {
        match self.jwt_validator.validate(token) {
            Ok(claims) => {
                let session = Session::new(claims);
                let session_id = session.id.clone();
                info!("User {} authenticated, session {}", session.user_id, session_id);
                self.session = Some(session);
                Some(ServerMessage::AuthOk { session_id })
            }
            Err(e) => {
                warn!("Authentication failed: {}", e);
                Some(ServerMessage::AuthError {
                    reason: e.to_string(),
                })
            }
        }
    }

    async fn handle_subscribe(&mut self, subject: String, id: u64) -> Option<ServerMessage> {
        let session = self.session.as_mut()?;

        // Check permission
        if !PermissionChecker::can_perform(&session.claims, Permission::Subscribe, &subject) {
            return Some(ServerMessage::SubscribeError {
                id,
                reason: "Permission denied".to_string(),
            });
        }

        // Create NATS subscription
        match self.nats_bridge.subscribe(subject.clone(), self.nats_tx.clone()).await {
            Ok(handle) => {
                session.add_subscription(id, subject.clone());
                self.subscriptions.insert(id, handle);
                debug!("User {} subscribed to {} (id={})", session.user_id, subject, id);
                Some(ServerMessage::SubscribeOk { id })
            }
            Err(e) => {
                error!("Failed to subscribe to {}: {}", subject, e);
                Some(ServerMessage::SubscribeError {
                    id,
                    reason: e.to_string(),
                })
            }
        }
    }

    async fn handle_unsubscribe(&mut self, id: u64) -> Option<ServerMessage> {
        let session = self.session.as_mut()?;

        if let Some(handle) = self.subscriptions.remove(&id) {
            handle.unsubscribe().await;
            session.remove_subscription(id);
            debug!("User {} unsubscribed from id={}", session.user_id, id);
        }

        None // No response needed for unsubscribe
    }

    async fn handle_publish(&mut self, subject: &str, payload: Vec<u8>) -> Option<ServerMessage> {
        let session = self.session.as_ref()?;

        // Check permission
        if !PermissionChecker::can_perform(&session.claims, Permission::Publish, subject) {
            return Some(ServerMessage::Error {
                code: error_codes::FORBIDDEN,
                message: "Permission denied".to_string(),
            });
        }

        match self.nats_bridge.publish(subject, payload).await {
            Ok(_) => {
                debug!("User {} published to {}", session.user_id, subject);
                None // No response needed for publish
            }
            Err(e) => {
                error!("Failed to publish to {}: {}", subject, e);
                Some(ServerMessage::Error {
                    code: error_codes::INTERNAL_ERROR,
                    message: e.to_string(),
                })
            }
        }
    }

    async fn handle_request(
        &mut self,
        subject: &str,
        payload: Vec<u8>,
        timeout_ms: u32,
        request_id: u64,
    ) -> Option<ServerMessage> {
        let session = self.session.as_ref()?;

        // Check permission
        if !PermissionChecker::can_perform(&session.claims, Permission::Request, subject) {
            return Some(ServerMessage::RequestError {
                request_id,
                reason: "Permission denied".to_string(),
            });
        }

        let timeout = Duration::from_millis(timeout_ms as u64);
        match self.nats_bridge.request(subject, payload, timeout).await {
            Ok(response) => Some(ServerMessage::Response {
                request_id,
                payload: response,
            }),
            Err(e) => Some(ServerMessage::RequestError {
                request_id,
                reason: e.to_string(),
            }),
        }
    }

    /// Cleanup when connection closes
    pub async fn cleanup(&mut self) {
        // Unsubscribe from all NATS subscriptions
        for (_, handle) in self.subscriptions.drain() {
            handle.unsubscribe().await;
        }

        if let Some(session) = &self.session {
            info!("Session {} cleaned up", session.id);
        }
    }
}

/// Check if a subject matches a NATS-style pattern
fn subject_matches_pattern(pattern: &str, subject: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('.').collect();
    let subject_parts: Vec<&str> = subject.split('.').collect();

    let mut pi = 0;
    let mut si = 0;

    while pi < pattern_parts.len() && si < subject_parts.len() {
        let p = pattern_parts[pi];

        if p == ">" {
            return true;
        } else if p == "*" || p == subject_parts[si] {
            pi += 1;
            si += 1;
        } else {
            return false;
        }
    }

    pi == pattern_parts.len() && si == subject_parts.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ subject_matches_pattern Tests ============

    #[test]
    fn test_exact_match() {
        assert!(subject_matches_pattern("foo.bar.baz", "foo.bar.baz"));
        assert!(!subject_matches_pattern("foo.bar.baz", "foo.bar.qux"));
        assert!(!subject_matches_pattern("foo.bar", "foo.bar.baz"));
    }

    #[test]
    fn test_single_wildcard() {
        assert!(subject_matches_pattern("foo.*.baz", "foo.bar.baz"));
        assert!(subject_matches_pattern("foo.*.baz", "foo.qux.baz"));
        assert!(!subject_matches_pattern("foo.*.baz", "foo.bar.qux"));
        assert!(!subject_matches_pattern("foo.*.baz", "foo.bar.baz.extra"));
    }

    #[test]
    fn test_multi_wildcard() {
        assert!(subject_matches_pattern("foo.>", "foo.bar"));
        assert!(subject_matches_pattern("foo.>", "foo.bar.baz"));
        assert!(subject_matches_pattern("foo.>", "foo.bar.baz.qux"));
        assert!(!subject_matches_pattern("foo.>", "bar.baz"));
    }

    #[test]
    fn test_full_wildcard() {
        assert!(subject_matches_pattern(">", "foo"));
        assert!(subject_matches_pattern(">", "foo.bar"));
        assert!(subject_matches_pattern(">", "foo.bar.baz"));
    }

    #[test]
    fn test_mixed_wildcards() {
        assert!(subject_matches_pattern("*.bar.>", "foo.bar.baz"));
        assert!(subject_matches_pattern("*.bar.>", "qux.bar.baz.extra"));
        assert!(!subject_matches_pattern("*.bar.>", "foo.qux.baz"));
    }

    #[test]
    fn test_empty_patterns() {
        assert!(subject_matches_pattern("", ""));
        assert!(!subject_matches_pattern("foo", ""));
        assert!(!subject_matches_pattern("", "foo"));
    }

    #[test]
    fn test_single_token() {
        assert!(subject_matches_pattern("foo", "foo"));
        assert!(!subject_matches_pattern("foo", "bar"));
        assert!(subject_matches_pattern("*", "foo"));
        assert!(subject_matches_pattern(">", "foo"));
    }

    // ============ ClientMessage Tests ============

    #[test]
    fn test_requires_auth_auth_message() {
        let msg = ClientMessage::Auth {
            token: "test".to_string(),
        };
        assert!(!msg.requires_auth());
    }

    #[test]
    fn test_requires_auth_ping() {
        let msg = ClientMessage::Ping;
        assert!(!msg.requires_auth());
    }

    #[test]
    fn test_requires_auth_subscribe() {
        let msg = ClientMessage::Subscribe {
            subject: "test".to_string(),
            id: 1,
        };
        assert!(msg.requires_auth());
    }

    #[test]
    fn test_requires_auth_publish() {
        let msg = ClientMessage::Publish {
            subject: "test".to_string(),
            payload: vec![],
        };
        assert!(msg.requires_auth());
    }

    #[test]
    fn test_requires_auth_request() {
        let msg = ClientMessage::Request {
            subject: "test".to_string(),
            payload: vec![],
            timeout_ms: 1000,
            request_id: 1,
        };
        assert!(msg.requires_auth());
    }

    #[test]
    fn test_requires_auth_unsubscribe() {
        let msg = ClientMessage::Unsubscribe { id: 1 };
        assert!(msg.requires_auth());
    }
}
