use schema_sdk::codec::{Decode, Encode};
pub use schema_sdk::{ClientEnvelope, ClientMessage, ServerEnvelope, ServerMessage};

pub struct MessageCodec;

impl MessageCodec {
    #[allow(dead_code)]
    pub fn encode_client(msg: &ClientMessage) -> Vec<u8> {
        ClientEnvelope {
            message: msg.clone(),
        }
        .to_bytes()
    }

    #[allow(dead_code)]
    pub fn decode_server(data: &[u8]) -> Result<ServerMessage, CodecError> {
        ServerEnvelope::from_bytes(data)
            .map(|envelope| envelope.message)
            .map_err(|e| CodecError::DecodeError(e.to_string()))
    }

    pub fn encode_server(msg: &ServerMessage) -> Vec<u8> {
        ServerEnvelope {
            message: msg.clone(),
        }
        .to_bytes()
    }

    pub fn decode_client(data: &[u8]) -> Result<ClientMessage, CodecError> {
        ClientEnvelope::from_bytes(data)
            .map(|envelope| envelope.message)
            .map_err(|e| CodecError::DecodeError(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Failed to decode message: {0}")]
    DecodeError(String),
}

pub mod error_codes {
    pub const UNAUTHORIZED: u32 = 401;
    pub const FORBIDDEN: u32 = 403;
    pub const NOT_FOUND: u32 = 404;
    pub const INTERNAL_ERROR: u32 = 500;
    pub const INVALID_MESSAGE: u32 = 400;
}
