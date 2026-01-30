use bitcode::{Decode, Encode};

/// Messages sent from client to gateway
#[derive(Debug, Clone, Encode, Decode)]
#[allow(unused_assignments)] // bitcode derive generates field assignments
pub enum ClientMessage {
    /// Authenticate with JWT token
    Auth { token: String },

    /// Subscribe to a subject
    Subscribe { subject: String, id: u64 },

    /// Unsubscribe from a subscription
    Unsubscribe { id: u64 },

    /// Publish a message to a subject
    Publish { subject: String, payload: Vec<u8> },

    /// Request-reply pattern
    Request {
        subject: String,
        payload: Vec<u8>,
        timeout_ms: u32,
        request_id: u64,
    },

    /// Keepalive ping
    Ping,
}

/// Messages sent from gateway to client
#[derive(Debug, Clone, Encode, Decode)]
#[allow(unused_assignments)] // bitcode derive generates field assignments
pub enum ServerMessage {
    /// Authentication successful
    AuthOk { session_id: String },

    /// Authentication failed
    AuthError { reason: String },

    /// Subscription confirmed
    SubscribeOk { id: u64 },

    /// Subscription error
    SubscribeError { id: u64, reason: String },

    /// Message received on subscription
    Message {
        subscription_id: u64,
        subject: String,
        payload: Vec<u8>,
    },

    /// Response to a request
    Response { request_id: u64, payload: Vec<u8> },

    /// Request error
    RequestError { request_id: u64, reason: String },

    /// Generic error
    Error { code: u32, message: String },

    /// Keepalive pong
    Pong,
}

impl ClientMessage {
    /// Check if this message requires authentication
    pub fn requires_auth(&self) -> bool {
        !matches!(self, ClientMessage::Auth { .. } | ClientMessage::Ping)
    }
}

/// Error codes for ServerMessage::Error
#[allow(dead_code)]
pub mod error_codes {
    pub const UNAUTHORIZED: u32 = 401;
    pub const FORBIDDEN: u32 = 403;
    pub const NOT_FOUND: u32 = 404;
    pub const INTERNAL_ERROR: u32 = 500;
    pub const INVALID_MESSAGE: u32 = 400;
}
