use super::jwt::Claims;

/// Permission types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Permission {
    Publish,
    Subscribe,
    Request,
}

impl Permission {
    /// Parse a permission from a string
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "publish" => Some(Permission::Publish),
            "subscribe" => Some(Permission::Subscribe),
            "request" => Some(Permission::Request),
            _ => None,
        }
    }
}

/// Permission checker for subject patterns
pub struct PermissionChecker;

impl PermissionChecker {
    /// Check if the user has a specific permission
    pub fn has_permission(claims: &Claims, permission: Permission) -> bool {
        let perm_str = match permission {
            Permission::Publish => "publish",
            Permission::Subscribe => "subscribe",
            Permission::Request => "request",
        };
        claims
            .permissions
            .iter()
            .any(|p| p.to_lowercase() == perm_str)
    }

    /// Check if a subject matches any of the allowed patterns
    /// Supports NATS-style wildcards:
    /// - `*` matches a single token
    /// - `>` matches one or more tokens (must be at the end)
    pub fn is_subject_allowed(claims: &Claims, subject: &str) -> bool {
        // First check deny patterns (they take precedence)
        for pattern in &claims.deny_subjects {
            if Self::matches_pattern(pattern, subject) {
                return false;
            }
        }

        // If no allowed patterns specified, allow all (for backward compatibility)
        if claims.allowed_subjects.is_empty() {
            return true;
        }

        // Check allowed patterns
        for pattern in &claims.allowed_subjects {
            if Self::matches_pattern(pattern, subject) {
                return true;
            }
        }

        false
    }

    /// Check if a subject matches a NATS-style pattern
    fn matches_pattern(pattern: &str, subject: &str) -> bool {
        let pattern_parts: Vec<&str> = pattern.split('.').collect();
        let subject_parts: Vec<&str> = subject.split('.').collect();

        let mut pi = 0;
        let mut si = 0;

        while pi < pattern_parts.len() && si < subject_parts.len() {
            let p = pattern_parts[pi];

            if p == ">" {
                // `>` matches the rest of the subject
                return true;
            } else if p == "*" {
                // `*` matches exactly one token
                pi += 1;
                si += 1;
            } else if p == subject_parts[si] {
                // Exact match
                pi += 1;
                si += 1;
            } else {
                return false;
            }
        }

        // Both must be exhausted for a match (unless pattern ended with >)
        pi == pattern_parts.len() && si == subject_parts.len()
    }

    /// Combined check for permission and subject
    pub fn can_perform(claims: &Claims, permission: Permission, subject: &str) -> bool {
        Self::has_permission(claims, permission) && Self::is_subject_allowed(claims, subject)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_claims(permissions: Vec<&str>, allowed: Vec<&str>, denied: Vec<&str>) -> Claims {
        Claims {
            sub: "test".to_string(),
            exp: 9999999999,
            iat: 0,
            permissions: permissions.into_iter().map(String::from).collect(),
            allowed_subjects: allowed.into_iter().map(String::from).collect(),
            deny_subjects: denied.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_exact_match() {
        let claims = create_claims(vec!["subscribe"], vec!["messages"], vec![]);
        assert!(PermissionChecker::is_subject_allowed(&claims, "messages"));
        assert!(!PermissionChecker::is_subject_allowed(&claims, "other"));
    }

    #[test]
    fn test_wildcard_single() {
        let claims = create_claims(vec!["subscribe"], vec!["messages.*"], vec![]);
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user1"
        ));
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user2"
        ));
        assert!(!PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user1.inbox"
        ));
        assert!(!PermissionChecker::is_subject_allowed(&claims, "other"));
    }

    #[test]
    fn test_wildcard_multi() {
        let claims = create_claims(vec!["subscribe"], vec!["messages.>"], vec![]);
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user1"
        ));
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user1.inbox"
        ));
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.a.b.c.d"
        ));
        assert!(!PermissionChecker::is_subject_allowed(&claims, "other"));
    }

    #[test]
    fn test_deny_takes_precedence() {
        let claims = create_claims(
            vec!["subscribe"],
            vec!["messages.>"],
            vec!["messages.admin.*"],
        );
        assert!(PermissionChecker::is_subject_allowed(
            &claims,
            "messages.user1"
        ));
        assert!(!PermissionChecker::is_subject_allowed(
            &claims,
            "messages.admin.secret"
        ));
    }

    #[test]
    fn test_has_permission() {
        let claims = create_claims(vec!["publish", "subscribe"], vec![], vec![]);
        assert!(PermissionChecker::has_permission(
            &claims,
            Permission::Publish
        ));
        assert!(PermissionChecker::has_permission(
            &claims,
            Permission::Subscribe
        ));
        assert!(!PermissionChecker::has_permission(
            &claims,
            Permission::Request
        ));
    }
}
