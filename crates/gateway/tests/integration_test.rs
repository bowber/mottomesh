//! Integration tests for the Mottomesh Gateway
//!
//! These tests spin up a real NATS container and test the full WebSocket flow.

mod common;

use std::time::Duration;

use common::{
    client::TestClient,
    gateway::TestGateway,
    jwt::{create_expired_token, create_limited_token, create_valid_token},
    nats::{get_nats, test_subject},
};
use futures::StreamExt;
use mottomesh_gateway::protocol::{ClientMessage, ServerMessage, error_codes};

// ============================================================================
// Auth Flow Tests
// ============================================================================

#[tokio::test]
async fn test_auth_success() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    let token = create_valid_token("user-1");
    let result = client.auth(&token).await;

    assert!(result.is_ok(), "Auth should succeed: {:?}", result);
    let session_id = result.unwrap();
    assert!(!session_id.is_empty(), "Session ID should not be empty");

    client.close().await;
}

#[tokio::test]
async fn test_auth_invalid_token() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    let result = client.auth("invalid-token-12345").await;

    assert!(result.is_err(), "Auth should fail with invalid token");
    let error = result.unwrap_err();
    assert!(
        error.contains("invalid") || error.contains("Invalid") || error.contains("error"),
        "Error should mention invalid: {}",
        error
    );

    client.close().await;
}

#[tokio::test]
async fn test_auth_expired_token() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    let token = create_expired_token("user-expired");
    let result = client.auth(&token).await;

    assert!(result.is_err(), "Auth should fail with expired token");
    let error = result.unwrap_err();
    assert!(
        error.to_lowercase().contains("expir") || error.to_lowercase().contains("invalid"),
        "Error should mention expiration: {}",
        error
    );

    client.close().await;
}

#[tokio::test]
async fn test_unauthenticated_subscribe() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Try to subscribe without authenticating first
    client
        .send(ClientMessage::Subscribe {
            subject: "test.topic".to_string(),
            id: 1,
        })
        .await;

    let response = client.recv().await;

    match response {
        Some(ServerMessage::Error { code, .. }) => {
            assert_eq!(
                code,
                error_codes::UNAUTHORIZED,
                "Should get unauthorized error"
            );
        }
        Some(ServerMessage::SubscribeError { reason, .. }) => {
            assert!(
                reason.to_lowercase().contains("auth")
                    || reason.to_lowercase().contains("unauthorized"),
                "Error should mention auth: {}",
                reason
            );
        }
        other => panic!("Expected error response, got: {:?}", other),
    }

    client.close().await;
}

// ============================================================================
// Subscribe/Publish Flow Tests
// ============================================================================

#[tokio::test]
async fn test_subscribe_success() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate first
    let token = create_valid_token("user-sub");
    client.auth(&token).await.expect("Auth should succeed");

    // Subscribe
    let subject = test_subject("test_subscribe_success", "events");
    let result = client.subscribe(&subject, 1).await;

    assert!(result.is_ok(), "Subscribe should succeed: {:?}", result);
    assert_eq!(result.unwrap(), 1, "Should return the subscription ID");

    client.close().await;
}

#[tokio::test]
async fn test_subscribe_receive_message() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate and subscribe
    let token = create_valid_token("user-recv");
    client.auth(&token).await.expect("Auth should succeed");

    let subject = test_subject("test_subscribe_receive", "messages");
    client
        .subscribe(&subject, 42)
        .await
        .expect("Subscribe should succeed");

    // Give subscription time to propagate
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish from NATS directly
    let payload = b"Hello from NATS!";
    nats.publish(&subject, payload).await;

    // Client should receive the message
    let response = client.recv().await;

    match response {
        Some(ServerMessage::Message {
            subscription_id,
            subject: msg_subject,
            payload: msg_payload,
        }) => {
            assert_eq!(subscription_id, 42, "Subscription ID should match");
            assert_eq!(msg_subject, subject, "Subject should match");
            assert_eq!(msg_payload, payload, "Payload should match");
        }
        other => panic!("Expected Message, got: {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn test_subscribe_wildcard() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate
    let token = create_valid_token("user-wildcard");
    client.auth(&token).await.expect("Auth should succeed");

    // Subscribe with wildcard
    let wildcard_subject = test_subject("test_wildcard", "*");
    client
        .subscribe(&wildcard_subject, 1)
        .await
        .expect("Subscribe should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish to a specific subject that matches the wildcard
    let specific_subject = test_subject("test_wildcard", "events");
    nats.publish(&specific_subject, b"Wildcard test").await;

    // Should receive the message
    let response = client.recv().await;

    match response {
        Some(ServerMessage::Message {
            subscription_id,
            subject,
            payload,
        }) => {
            assert_eq!(subscription_id, 1);
            assert_eq!(subject, specific_subject);
            assert_eq!(payload, b"Wildcard test");
        }
        other => panic!("Expected Message, got: {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn test_unsubscribe() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate and subscribe
    let token = create_valid_token("user-unsub");
    client.auth(&token).await.expect("Auth should succeed");

    let subject = test_subject("test_unsubscribe", "events");
    client
        .subscribe(&subject, 1)
        .await
        .expect("Subscribe should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Unsubscribe
    client.send(ClientMessage::Unsubscribe { id: 1 }).await;

    // Give time for unsubscribe to process
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Publish a message - client should NOT receive it
    nats.publish(&subject, b"Should not receive").await;

    // Use a short timeout since we expect no message
    let response = client.recv_timeout(Duration::from_millis(500)).await;

    assert!(
        response.is_none(),
        "Should not receive message after unsubscribe, got: {:?}",
        response
    );

    client.close().await;
}

#[tokio::test]
async fn test_publish_to_nats() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate
    let token = create_valid_token("user-pub");
    client.auth(&token).await.expect("Auth should succeed");

    // Subscribe on NATS side
    let subject = test_subject("test_publish", "outgoing");
    let mut nats_sub = nats.subscribe(&subject).await;

    // Publish from client
    let payload = b"Hello from client!";
    client.publish(&subject, payload).await;

    // NATS should receive the message
    let msg = tokio::time::timeout(Duration::from_secs(5), nats_sub.next())
        .await
        .expect("Timeout waiting for NATS message")
        .expect("Should receive message on NATS");

    assert_eq!(msg.payload.as_ref(), payload);

    client.close().await;
}

// ============================================================================
// Request/Reply Tests
// ============================================================================

#[tokio::test]
async fn test_request_reply() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate
    let token = create_valid_token("user-request");
    client.auth(&token).await.expect("Auth should succeed");

    let subject = test_subject("test_request_reply", "rpc");

    // Set up a responder on NATS
    let mut responder = nats.subscribe(&subject).await;
    let nats_client = nats.client().clone();

    tokio::spawn(async move {
        if let Some(msg) = responder.next().await
            && let Some(reply) = msg.reply
        {
            let response = format!("Echo: {}", String::from_utf8_lossy(&msg.payload));
            nats_client
                .publish(reply, response.into_bytes().into())
                .await
                .expect("Failed to send reply");
        }
    });

    // Give responder time to subscribe
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send request from client
    client
        .send(ClientMessage::Request {
            subject: subject.clone(),
            payload: b"Hello".to_vec(),
            timeout_ms: 5000,
            request_id: 123,
        })
        .await;

    // Should receive response
    let response = client.recv().await;

    match response {
        Some(ServerMessage::Response {
            request_id,
            payload,
        }) => {
            assert_eq!(request_id, 123, "Request ID should match");
            assert_eq!(
                String::from_utf8_lossy(&payload),
                "Echo: Hello",
                "Response payload should match"
            );
        }
        other => panic!("Expected Response, got: {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn test_request_timeout() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate
    let token = create_valid_token("user-timeout");
    client.auth(&token).await.expect("Auth should succeed");

    // Request to a subject with no responder
    let subject = test_subject("test_request_timeout", "no-responder");

    client
        .send(ClientMessage::Request {
            subject: subject.clone(),
            payload: b"Hello?".to_vec(),
            timeout_ms: 500, // Short timeout
            request_id: 456,
        })
        .await;

    // Should receive timeout error
    let response = client.recv_timeout(Duration::from_secs(2)).await;

    match response {
        Some(ServerMessage::RequestError { request_id, reason }) => {
            assert_eq!(request_id, 456, "Request ID should match");
            assert!(
                reason.to_lowercase().contains("timeout")
                    || reason.to_lowercase().contains("no response")
                    || reason.to_lowercase().contains("no responders"),
                "Error should mention timeout or no responders: {}",
                reason
            );
        }
        other => panic!("Expected RequestError, got: {:?}", other),
    }

    client.close().await;
}

// ============================================================================
// Connection Tests
// ============================================================================

#[tokio::test]
async fn test_ping_pong() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Ping should work without authentication
    client.ping().await;

    let response = client.recv().await;

    match response {
        Some(ServerMessage::Pong) => {
            // Success!
        }
        other => panic!("Expected Pong, got: {:?}", other),
    }

    client.close().await;
}

#[tokio::test]
async fn test_multiple_clients() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;

    // Connect multiple clients
    let mut client1 = TestClient::connect(&gateway.ws_url()).await;
    let mut client2 = TestClient::connect(&gateway.ws_url()).await;
    let mut client3 = TestClient::connect(&gateway.ws_url()).await;

    // All should be able to authenticate
    let token1 = create_valid_token("user-1");
    let token2 = create_valid_token("user-2");
    let token3 = create_valid_token("user-3");

    let (r1, r2, r3) = tokio::join!(
        client1.auth(&token1),
        client2.auth(&token2),
        client3.auth(&token3)
    );

    assert!(r1.is_ok(), "Client 1 auth failed: {:?}", r1);
    assert!(r2.is_ok(), "Client 2 auth failed: {:?}", r2);
    assert!(r3.is_ok(), "Client 3 auth failed: {:?}", r3);

    // All should have different session IDs
    let s1 = r1.unwrap();
    let s2 = r2.unwrap();
    let s3 = r3.unwrap();

    assert_ne!(s1, s2, "Session IDs should be unique");
    assert_ne!(s2, s3, "Session IDs should be unique");
    assert_ne!(s1, s3, "Session IDs should be unique");

    client1.close().await;
    client2.close().await;
    client3.close().await;
}

#[tokio::test]
async fn test_client_disconnect_cleanup() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Authenticate and subscribe
    let token = create_valid_token("user-disconnect");
    client.auth(&token).await.expect("Auth should succeed");

    let subject = test_subject("test_disconnect", "events");
    client
        .subscribe(&subject, 1)
        .await
        .expect("Subscribe should succeed");

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Close the client
    client.close().await;

    // Give time for cleanup
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Publish a message - should not cause any errors
    // (subscription should be cleaned up on disconnect)
    nats.publish(&subject, b"After disconnect").await;

    // If we get here without panicking, cleanup worked
    // The gateway should have removed the subscription when the client disconnected
}

// ============================================================================
// Permission Tests
// ============================================================================

#[tokio::test]
async fn test_subscribe_with_limited_permissions() {
    let nats = get_nats().await;
    let gateway = TestGateway::start(nats.url()).await;
    let mut client = TestClient::connect(&gateway.ws_url()).await;

    // Create token that only allows specific subjects
    let allowed_subject = test_subject("test_limited", "allowed");
    let token = create_limited_token("user-limited", vec![allowed_subject.clone()]);
    client.auth(&token).await.expect("Auth should succeed");

    // Subscribe to allowed subject should work
    let result = client.subscribe(&allowed_subject, 1).await;
    assert!(
        result.is_ok(),
        "Subscribe to allowed subject should succeed: {:?}",
        result
    );

    // Subscribe to disallowed subject should fail
    let denied_subject = test_subject("test_limited", "denied");
    client
        .send(ClientMessage::Subscribe {
            subject: denied_subject.clone(),
            id: 2,
        })
        .await;

    let response = client.recv().await;

    match response {
        Some(ServerMessage::SubscribeError { id, reason }) => {
            assert_eq!(id, 2);
            assert!(
                reason.to_lowercase().contains("permission")
                    || reason.to_lowercase().contains("denied")
                    || reason.to_lowercase().contains("forbidden")
                    || reason.to_lowercase().contains("allowed"),
                "Error should mention permissions: {}",
                reason
            );
        }
        Some(ServerMessage::Error { code, message }) => {
            assert_eq!(
                code,
                error_codes::FORBIDDEN,
                "Should get forbidden error: {}",
                message
            );
        }
        other => panic!("Expected error for denied subject, got: {:?}", other),
    }

    client.close().await;
}
