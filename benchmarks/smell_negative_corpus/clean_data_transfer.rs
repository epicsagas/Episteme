// Guards against: Data Class FP.
// A DTO with derive traits and no methods is the intended design — DTOs exist
// to carry data across boundaries and should not contain business logic.

use serde::{Deserialize, Serialize};

/// API response payload for user profile lookups.
///
/// This struct is intentionally a pure data container — it maps 1:1 to the
/// external API contract and has no domain behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: u64,
    pub username: String,
    pub email: String,
    pub display_name: String,
    pub avatar_url: String,
    pub bio: String,
    pub role: String,
    pub is_verified: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Paginated wrapper for list endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub per_page: u32,
    pub has_next: bool,
}

/// Error response returned on API failures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub details: Option<Vec<FieldError>>,
}

/// Field-level validation error detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_response_roundtrips_json() {
        let user = UserResponse {
            id: 42,
            username: "alice".into(),
            email: "alice@example.com".into(),
            display_name: "Alice".into(),
            avatar_url: "https://img.example.com/alice.png".into(),
            bio: "Hello world".into(),
            role: "user".into(),
            is_verified: true,
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-06-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let parsed: UserResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, 42);
        assert_eq!(parsed.username, "alice");
    }

    #[test]
    fn paginated_response_has_next() {
        let page: PaginatedResponse<UserResponse> = PaginatedResponse {
            items: vec![],
            total_count: 100,
            page: 1,
            per_page: 10,
            has_next: true,
        };
        assert!(page.has_next);
    }
}
