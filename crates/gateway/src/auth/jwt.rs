use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use tracing::debug;

/// JWT claims structure for mottomesh
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time (Unix timestamp)
    pub exp: usize,
    /// Issued at (Unix timestamp)
    pub iat: usize,
    /// Permissions: ["publish", "subscribe", "request"]
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Allowed subject patterns (supports NATS wildcards * and >)
    #[serde(default)]
    pub allowed_subjects: Vec<String>,
    /// Denied subject patterns (takes precedence over allowed)
    #[serde(default)]
    pub deny_subjects: Vec<String>,
}

pub struct JwtValidator {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtValidator {
    pub fn new(secret: &str) -> Result<Self, JwtError> {
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;

        Ok(Self {
            decoding_key,
            validation,
        })
    }

    pub fn validate(&self, token: &str) -> Result<Claims, JwtError> {
        let token_data =
            decode::<Claims>(token, &self.decoding_key, &self.validation).map_err(|e| {
                debug!("JWT validation failed: {:?}", e);
                JwtError::InvalidToken(e.to_string())
            })?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{EncodingKey, Header, encode};

    fn create_test_token(secret: &str, claims: &Claims) -> String {
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(secret.as_bytes()),
        )
        .unwrap()
    }

    fn valid_claims() -> Claims {
        Claims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            permissions: vec!["publish".to_string(), "subscribe".to_string()],
            allowed_subjects: vec!["messages.*".to_string()],
            deny_subjects: vec![],
        }
    }

    #[test]
    fn test_validate_valid_token() {
        let secret = "test_secret_key_123";
        let validator = JwtValidator::new(secret).unwrap();

        let claims = valid_claims();
        let token = create_test_token(secret, &claims);
        let result = validator.validate(&token);

        assert!(result.is_ok());
        let validated_claims = result.unwrap();
        assert_eq!(validated_claims.sub, "user123");
    }

    #[test]
    fn test_validate_expired_token() {
        let secret = "test_secret_key_123";
        let validator = JwtValidator::new(secret).unwrap();

        let claims = Claims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp() as usize,
            iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp() as usize,
            permissions: vec![],
            allowed_subjects: vec![],
            deny_subjects: vec![],
        };

        let token = create_test_token(secret, &claims);
        let result = validator.validate(&token);

        assert!(result.is_err());
        let JwtError::InvalidToken(msg) = result.unwrap_err();
        assert!(
            msg.contains("ExpiredSignature"),
            "Expected expired error, got: {}",
            msg
        );
    }

    #[test]
    fn test_validate_invalid_signature() {
        let secret = "test_secret_key_123";
        let wrong_secret = "wrong_secret_key_456";
        let validator = JwtValidator::new(secret).unwrap();

        let claims = valid_claims();
        let token = create_test_token(wrong_secret, &claims);
        let result = validator.validate(&token);

        assert!(result.is_err());
        let JwtError::InvalidToken(msg) = result.unwrap_err();
        assert!(
            msg.contains("InvalidSignature"),
            "Expected signature error, got: {}",
            msg
        );
    }

    #[test]
    fn test_validate_malformed_token() {
        let secret = "test_secret_key_123";
        let validator = JwtValidator::new(secret).unwrap();

        let result = validator.validate("not.a.valid.token");
        assert!(result.is_err());

        let result = validator.validate("");
        assert!(result.is_err());

        let result = validator.validate("just_random_string");
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_with_empty_permissions() {
        let secret = "test_secret_key_123";
        let validator = JwtValidator::new(secret).unwrap();

        let claims = Claims {
            sub: "restricted_user".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            permissions: vec![],
            allowed_subjects: vec![],
            deny_subjects: vec![],
        };

        let token = create_test_token(secret, &claims);
        let result = validator.validate(&token);

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert!(validated.permissions.is_empty());
        assert!(validated.allowed_subjects.is_empty());
    }

    #[test]
    fn test_claims_with_wildcards() {
        let secret = "test_secret_key_123";
        let validator = JwtValidator::new(secret).unwrap();

        let claims = Claims {
            sub: "admin".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            permissions: vec![
                "publish".to_string(),
                "subscribe".to_string(),
                "request".to_string(),
            ],
            allowed_subjects: vec![">".to_string()], // Full access
            deny_subjects: vec!["admin.>".to_string()], // Except admin topics
        };

        let token = create_test_token(secret, &claims);
        let result = validator.validate(&token);

        assert!(result.is_ok());
        let validated = result.unwrap();
        assert_eq!(validated.allowed_subjects, vec![">"]);
        assert_eq!(validated.deny_subjects, vec!["admin.>"]);
    }

    #[test]
    fn test_validator_new_with_empty_secret() {
        // Empty secret should still work (though not recommended)
        let result = JwtValidator::new("");
        assert!(result.is_ok());
    }

    #[test]
    fn test_validator_new_with_long_secret() {
        let long_secret = "a".repeat(1000);
        let result = JwtValidator::new(&long_secret);
        assert!(result.is_ok());

        let validator = result.unwrap();
        let claims = Claims {
            sub: "user".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            permissions: vec![],
            allowed_subjects: vec![],
            deny_subjects: vec![],
        };
        let token = create_test_token(&long_secret, &claims);
        assert!(validator.validate(&token).is_ok());
    }
}
