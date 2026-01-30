use bytes::Bytes;

use super::messages::{ClientMessage, ServerMessage};

/// Codec for encoding/decoding protocol messages
pub struct MessageCodec;

impl MessageCodec {
    /// Encode a server message to bytes
    pub fn encode_server(msg: &ServerMessage) -> Bytes {
        Bytes::from(bitcode::encode(msg))
    }

    /// Decode a client message from bytes
    pub fn decode_client(data: &[u8]) -> Result<ClientMessage, CodecError> {
        bitcode::decode(data).map_err(|e| CodecError::DecodeError(e.to_string()))
    }

    /// Encode a client message to bytes (for testing)
    #[allow(dead_code)]
    pub fn encode_client(msg: &ClientMessage) -> Bytes {
        Bytes::from(bitcode::encode(msg))
    }

    /// Decode a server message from bytes (for testing)
    #[allow(dead_code)]
    pub fn decode_server(data: &[u8]) -> Result<ServerMessage, CodecError> {
        bitcode::decode(data).map_err(|e| CodecError::DecodeError(e.to_string()))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CodecError {
    #[error("Failed to decode message: {0}")]
    DecodeError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============ ClientMessage Tests ============

    #[test]
    fn test_roundtrip_client_auth() {
        let msg = ClientMessage::Auth {
            token: "my.jwt.token".to_string(),
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Auth { token } => assert_eq!(token, "my.jwt.token"),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_client_message() {
        let msg = ClientMessage::Subscribe {
            subject: "test.subject".to_string(),
            id: 42,
        };

        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();

        match decoded {
            ClientMessage::Subscribe { subject, id } => {
                assert_eq!(subject, "test.subject");
                assert_eq!(id, 42);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_client_unsubscribe() {
        let msg = ClientMessage::Unsubscribe { id: 123 };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Unsubscribe { id } => assert_eq!(id, 123),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_client_publish() {
        let msg = ClientMessage::Publish {
            subject: "events.user.created".to_string(),
            payload: vec![0x01, 0x02, 0x03, 0x04, 0x05],
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Publish { subject, payload } => {
                assert_eq!(subject, "events.user.created");
                assert_eq!(payload, vec![0x01, 0x02, 0x03, 0x04, 0x05]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_client_request() {
        let msg = ClientMessage::Request {
            subject: "api.user.get".to_string(),
            payload: vec![1, 2, 3],
            timeout_ms: 5000,
            request_id: 999,
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Request {
                subject,
                payload,
                timeout_ms,
                request_id,
            } => {
                assert_eq!(subject, "api.user.get");
                assert_eq!(payload, vec![1, 2, 3]);
                assert_eq!(timeout_ms, 5000);
                assert_eq!(request_id, 999);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_client_ping() {
        let msg = ClientMessage::Ping;
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        assert!(matches!(decoded, ClientMessage::Ping));
    }

    // ============ ServerMessage Tests ============

    #[test]
    fn test_roundtrip_server_message() {
        let msg = ServerMessage::Message {
            subscription_id: 1,
            subject: "test".to_string(),
            payload: vec![1, 2, 3],
        };

        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();

        match decoded {
            ServerMessage::Message {
                subscription_id,
                subject,
                payload,
            } => {
                assert_eq!(subscription_id, 1);
                assert_eq!(subject, "test");
                assert_eq!(payload, vec![1, 2, 3]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_auth_ok() {
        let msg = ServerMessage::AuthOk {
            session_id: "session-abc-123".to_string(),
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::AuthOk { session_id } => {
                assert_eq!(session_id, "session-abc-123");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_auth_error() {
        let msg = ServerMessage::AuthError {
            reason: "Invalid token".to_string(),
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::AuthError { reason } => {
                assert_eq!(reason, "Invalid token");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_subscribe_ok() {
        let msg = ServerMessage::SubscribeOk { id: 42 };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::SubscribeOk { id } => assert_eq!(id, 42),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_subscribe_error() {
        let msg = ServerMessage::SubscribeError {
            id: 42,
            reason: "Permission denied".to_string(),
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::SubscribeError { id, reason } => {
                assert_eq!(id, 42);
                assert_eq!(reason, "Permission denied");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_response() {
        let msg = ServerMessage::Response {
            request_id: 100,
            payload: vec![0xDE, 0xAD, 0xBE, 0xEF],
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::Response {
                request_id,
                payload,
            } => {
                assert_eq!(request_id, 100);
                assert_eq!(payload, vec![0xDE, 0xAD, 0xBE, 0xEF]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_request_error() {
        let msg = ServerMessage::RequestError {
            request_id: 100,
            reason: "Timeout".to_string(),
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::RequestError { request_id, reason } => {
                assert_eq!(request_id, 100);
                assert_eq!(reason, "Timeout");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_error() {
        let msg = ServerMessage::Error {
            code: 500,
            message: "Internal server error".to_string(),
        };
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        match decoded {
            ServerMessage::Error { code, message } => {
                assert_eq!(code, 500);
                assert_eq!(message, "Internal server error");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_roundtrip_server_pong() {
        let msg = ServerMessage::Pong;
        let encoded = MessageCodec::encode_server(&msg);
        let decoded = MessageCodec::decode_server(&encoded).unwrap();
        assert!(matches!(decoded, ServerMessage::Pong));
    }

    // ============ Edge Case Tests ============

    #[test]
    fn test_empty_payload() {
        let msg = ClientMessage::Publish {
            subject: "test".to_string(),
            payload: vec![],
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Publish { payload, .. } => assert!(payload.is_empty()),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_large_payload() {
        let large_payload: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
        let msg = ClientMessage::Publish {
            subject: "large.message".to_string(),
            payload: large_payload.clone(),
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Publish { payload, .. } => {
                assert_eq!(payload.len(), 10000);
                assert_eq!(payload, large_payload);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_unicode_subject() {
        let msg = ClientMessage::Subscribe {
            subject: "日本語.テスト.🎉".to_string(),
            id: 1,
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Subscribe { subject, .. } => {
                assert_eq!(subject, "日本語.テスト.🎉");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_decode_invalid_data() {
        let invalid_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = MessageCodec::decode_client(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_empty_data() {
        let result = MessageCodec::decode_client(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_u64_values() {
        let msg = ClientMessage::Request {
            subject: "test".to_string(),
            payload: vec![],
            timeout_ms: u32::MAX,
            request_id: u64::MAX,
        };
        let encoded = MessageCodec::encode_client(&msg);
        let decoded = MessageCodec::decode_client(&encoded).unwrap();
        match decoded {
            ClientMessage::Request {
                timeout_ms,
                request_id,
                ..
            } => {
                assert_eq!(timeout_ms, u32::MAX);
                assert_eq!(request_id, u64::MAX);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
