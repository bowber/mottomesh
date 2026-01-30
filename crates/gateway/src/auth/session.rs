use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::jwt::Claims;

/// Represents an authenticated session
#[derive(Debug)]
pub struct Session {
    /// Unique session ID
    pub id: String,
    /// User ID from JWT claims
    pub user_id: String,
    /// JWT claims for permission checking
    pub claims: Claims,
    /// Active subscriptions: subscription_id -> subject
    pub subscriptions: HashMap<u64, String>,
    /// Counter for generating subscription IDs
    #[allow(dead_code)]
    next_sub_id: AtomicU64,
}

impl Session {
    pub fn new(claims: Claims) -> Self {
        let id = uuid_v4();
        let user_id = claims.sub.clone();

        Self {
            id,
            user_id,
            claims,
            subscriptions: HashMap::new(),
            next_sub_id: AtomicU64::new(1),
        }
    }

    /// Generate a new subscription ID
    #[allow(dead_code)]
    pub fn next_subscription_id(&self) -> u64 {
        self.next_sub_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Add a subscription
    pub fn add_subscription(&mut self, id: u64, subject: String) {
        self.subscriptions.insert(id, subject);
    }

    /// Remove a subscription
    pub fn remove_subscription(&mut self, id: u64) -> Option<String> {
        self.subscriptions.remove(&id)
    }

    /// Get subject for a subscription ID
    #[allow(dead_code)]
    pub fn get_subscription_subject(&self, id: u64) -> Option<&String> {
        self.subscriptions.get(&id)
    }
}

/// Simple UUID v4 generator (without external dependency)
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    // Mix time with random-ish data (use wrapping to avoid overflow)
    let pid = std::process::id() as u64;
    let random_part: u64 = (now as u64)
        .wrapping_mul(pid.wrapping_add(1))
        .wrapping_add(now as u64);

    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        (now & 0xFFFFFFFF) as u32,
        ((now >> 32) & 0xFFFF) as u16,
        ((now >> 48) & 0x0FFF) as u16,
        ((random_part >> 48) as u16 & 0x3FFF) | 0x8000,
        random_part & 0xFFFFFFFFFFFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_claims() -> Claims {
        Claims {
            sub: "test_user".to_string(),
            exp: 9999999999,
            iat: 1000000000,
            permissions: vec!["publish".to_string(), "subscribe".to_string()],
            allowed_subjects: vec!["messages.*".to_string()],
            deny_subjects: vec![],
        }
    }

    #[test]
    fn test_session_new() {
        let claims = create_test_claims();
        let session = Session::new(claims.clone());

        assert_eq!(session.user_id, "test_user");
        assert!(!session.id.is_empty());
        assert!(session.subscriptions.is_empty());
        assert_eq!(session.claims.sub, claims.sub);
    }

    #[test]
    fn test_session_id_is_unique() {
        let claims1 = create_test_claims();
        let claims2 = create_test_claims();

        let session1 = Session::new(claims1);
        let session2 = Session::new(claims2);

        assert_ne!(session1.id, session2.id);
    }

    #[test]
    fn test_add_subscription() {
        let claims = create_test_claims();
        let mut session = Session::new(claims);

        session.add_subscription(1, "messages.user1".to_string());
        session.add_subscription(2, "messages.user2".to_string());

        assert_eq!(session.subscriptions.len(), 2);
        assert_eq!(
            session.subscriptions.get(&1),
            Some(&"messages.user1".to_string())
        );
        assert_eq!(
            session.subscriptions.get(&2),
            Some(&"messages.user2".to_string())
        );
    }

    #[test]
    fn test_remove_subscription() {
        let claims = create_test_claims();
        let mut session = Session::new(claims);

        session.add_subscription(1, "messages.test".to_string());
        assert_eq!(session.subscriptions.len(), 1);

        let removed = session.remove_subscription(1);
        assert_eq!(removed, Some("messages.test".to_string()));
        assert!(session.subscriptions.is_empty());
    }

    #[test]
    fn test_remove_nonexistent_subscription() {
        let claims = create_test_claims();
        let mut session = Session::new(claims);

        let removed = session.remove_subscription(999);
        assert_eq!(removed, None);
    }

    #[test]
    fn test_get_subscription_subject() {
        let claims = create_test_claims();
        let mut session = Session::new(claims);

        session.add_subscription(42, "events.orders".to_string());

        assert_eq!(
            session.get_subscription_subject(42),
            Some(&"events.orders".to_string())
        );
        assert_eq!(session.get_subscription_subject(999), None);
    }

    #[test]
    fn test_next_subscription_id() {
        let claims = create_test_claims();
        let session = Session::new(claims);

        let id1 = session.next_subscription_id();
        let id2 = session.next_subscription_id();
        let id3 = session.next_subscription_id();

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_next_subscription_id_concurrent() {
        use std::sync::Arc;
        use std::thread;

        let claims = create_test_claims();
        let session = Arc::new(Session::new(claims));

        let mut handles = vec![];
        let mut all_ids = vec![];

        for _ in 0..10 {
            let session_clone = Arc::clone(&session);
            handles.push(thread::spawn(move || {
                let mut ids = vec![];
                for _ in 0..100 {
                    ids.push(session_clone.next_subscription_id());
                }
                ids
            }));
        }

        for handle in handles {
            all_ids.extend(handle.join().unwrap());
        }

        // All IDs should be unique
        all_ids.sort();
        let unique_count = all_ids.len();
        all_ids.dedup();
        assert_eq!(all_ids.len(), unique_count, "Duplicate IDs generated!");
    }

    #[test]
    fn test_add_duplicate_subscription_id() {
        let claims = create_test_claims();
        let mut session = Session::new(claims);

        session.add_subscription(1, "first.subject".to_string());
        session.add_subscription(1, "second.subject".to_string());

        // Second should overwrite first
        assert_eq!(session.subscriptions.len(), 1);
        assert_eq!(
            session.subscriptions.get(&1),
            Some(&"second.subject".to_string())
        );
    }

    #[test]
    fn test_uuid_v4_format() {
        let uuid = uuid_v4();

        // UUID v4 format: xxxxxxxx-xxxx-4xxx-Yxxx-xxxxxxxxxxxx
        // where Y is 8, 9, a, or b
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(8), Some('-'));
        assert_eq!(uuid.chars().nth(13), Some('-'));
        assert_eq!(uuid.chars().nth(14), Some('4')); // Version 4
        assert_eq!(uuid.chars().nth(18), Some('-'));
        assert_eq!(uuid.chars().nth(23), Some('-'));

        // Check variant (should be 8, 9, a, or b)
        let variant_char = uuid.chars().nth(19).unwrap();
        assert!(
            matches!(variant_char, '8' | '9' | 'a' | 'b'),
            "Invalid variant character: {}",
            variant_char
        );
    }

    #[test]
    fn test_session_claims_preserved() {
        let claims = Claims {
            sub: "admin_user".to_string(),
            exp: 1234567890,
            iat: 1234567800,
            permissions: vec![
                "publish".to_string(),
                "subscribe".to_string(),
                "request".to_string(),
            ],
            allowed_subjects: vec![">".to_string()],
            deny_subjects: vec!["admin.>".to_string()],
        };

        let session = Session::new(claims);

        assert_eq!(session.claims.sub, "admin_user");
        assert_eq!(session.claims.permissions.len(), 3);
        assert!(session.claims.permissions.contains(&"publish".to_string()));
        assert_eq!(session.claims.allowed_subjects, vec![">"]);
        assert_eq!(session.claims.deny_subjects, vec!["admin.>"]);
    }
}
