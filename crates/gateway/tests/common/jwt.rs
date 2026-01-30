use jsonwebtoken::{EncodingKey, Header, encode};
use mottomesh_gateway::auth::Claims;
use std::time::{SystemTime, UNIX_EPOCH};

/// Default test JWT secret
pub const TEST_JWT_SECRET: &str = "test-secret-key-for-integration-tests";

/// Create a valid JWT token for testing
pub fn create_valid_token(subject: &str) -> String {
    create_token(
        subject,
        3600,
        vec!["publish".into(), "subscribe".into(), "request".into()],
        vec![">".into()], // `>` matches one or more tokens (i.e., all subjects)
    )
}

/// Create a token with custom claims
pub fn create_token(
    subject: &str,
    expiry_secs: u64,
    permissions: Vec<String>,
    allowed_subjects: Vec<String>,
) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: subject.to_string(),
        exp: now + expiry_secs as usize,
        iat: now,
        permissions,
        allowed_subjects,
        deny_subjects: vec![],
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to create JWT token")
}

/// Create an expired token
pub fn create_expired_token(subject: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let claims = Claims {
        sub: subject.to_string(),
        exp: now - 3600, // Expired 1 hour ago
        iat: now - 7200,
        permissions: vec!["publish".into(), "subscribe".into()],
        allowed_subjects: vec!["*".into()],
        deny_subjects: vec![],
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET.as_bytes()),
    )
    .expect("Failed to create JWT token")
}

/// Create a token with limited permissions
pub fn create_limited_token(subject: &str, allowed_subjects: Vec<String>) -> String {
    create_token(
        subject,
        3600,
        vec!["publish".into(), "subscribe".into()],
        allowed_subjects,
    )
}
