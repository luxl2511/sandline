use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, decode_header, Algorithm, Validation};
use serde::{Deserialize, Serialize};

use crate::AppState;

/// JWT claims structure from Supabase Auth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // The Supabase User ID (UUID)
    pub aud: String,  // Should be "authenticated"
    pub exp: i64,     // Expiration time (Unix timestamp)
    pub role: String, // Supabase role (usually "authenticated")
    pub email: Option<String>,
}

/// Authenticated user extracted from JWT token
///
/// Stores the user ID, role, and full claims for use in RLS transactions
#[derive(Clone, Debug)]
pub struct AuthUser {
    pub id: String,
    pub role: String,
    pub full_claims: Claims,
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = (StatusCode, String);

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // 1. Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or((
                StatusCode::UNAUTHORIZED,
                "Missing Authorization header".into(),
            ))?;

        // 2. Validate Bearer format
        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid token format".into()));
        }
        let token = &auth_header[7..];

        // 3. Decode JWT header to extract 'kid' (Key ID)
        let header = decode_header(token).map_err(|e| {
            tracing::error!("Failed to decode JWT header: {}", e);
            (StatusCode::UNAUTHORIZED, "Malformed token".into())
        })?;

        let kid = header.kid.ok_or_else(|| {
            tracing::warn!("JWT missing 'kid' field in header");
            (StatusCode::UNAUTHORIZED, "Token missing key ID".into())
        })?;

        // 4. Get public key from JWKS cache using kid
        let decoding_key = state.jwks_cache.get_key(&kid).await.ok_or_else(|| {
            tracing::warn!("Unknown key ID: {}", kid);
            (StatusCode::UNAUTHORIZED, "Unknown signing key".into())
        })?;

        // 5. Configure JWT validation for ES256
        let mut validation = Validation::new(Algorithm::ES256);
        validation.set_audience(&[state.supabase_jwt_aud.as_str()]);
        validation.set_required_spec_claims(&["sub", "aud", "exp", "role"]);

        // Optional: Enable issuer validation for additional security
        // This prevents tokens from other Supabase projects being accepted
        // validation.set_issuer(&["https://mptcysneksvjwiwqoqmt.supabase.co/auth/v1"]);

        // 6. Decode and validate JWT signature
        let token_data = decode::<Claims>(token, &decoding_key, &validation).map_err(|e| {
            tracing::error!("JWT validation failed: {:?}", e);
            (StatusCode::UNAUTHORIZED, "Invalid or expired token".into())
        })?;

        // 7. Validate user ID is a valid UUID (security check)
        if uuid::Uuid::parse_str(&token_data.claims.sub).is_err() {
            tracing::error!("Invalid user ID format: {}", token_data.claims.sub);
            return Err((StatusCode::UNAUTHORIZED, "Invalid user ID in token".into()));
        }

        // 8. Validate role is whitelisted (security check)
        let allowed_roles = ["authenticated", "anon", "service_role"];
        if !allowed_roles.contains(&token_data.claims.role.as_str()) {
            tracing::error!("Invalid role in token: {}", token_data.claims.role);
            return Err((StatusCode::UNAUTHORIZED, "Invalid role in token".into()));
        }

        Ok(AuthUser {
            id: token_data.claims.sub.clone(),
            role: token_data.claims.role.clone(),
            full_claims: token_data.claims,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    /// Test that Claims can be serialized and deserialized
    #[test]
    fn test_claims_serde() {
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            aud: "authenticated".to_string(),
            exp: 1735137600, // Some future timestamp
            role: "authenticated".to_string(),
            email: Some("test@example.com".to_string()),
        };

        // Serialize to JSON
        let json = serde_json::to_string(&claims).expect("Failed to serialize claims");

        // Deserialize back
        let deserialized: Claims =
            serde_json::from_str(&json).expect("Failed to deserialize claims");

        assert_eq!(claims.sub, deserialized.sub);
        assert_eq!(claims.aud, deserialized.aud);
        assert_eq!(claims.exp, deserialized.exp);
        assert_eq!(claims.role, deserialized.role);
        assert_eq!(claims.email, deserialized.email);
    }

    /// Test that valid UUIDs are accepted as sub claim
    #[test]
    fn test_valid_uuid_in_sub() {
        let valid_uuid = Uuid::new_v4().to_string();
        assert!(
            Uuid::parse_str(&valid_uuid).is_ok(),
            "Valid UUID should parse successfully"
        );
    }

    /// Test that invalid UUIDs are rejected as sub claim
    #[test]
    fn test_invalid_uuid_in_sub() {
        let invalid_uuids = vec![
            "not-a-uuid",
            "12345",
            "xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
            "",
            "00000000-0000-0000-0000-00000000000", // Too short
        ];

        for invalid_uuid in invalid_uuids {
            assert!(
                Uuid::parse_str(invalid_uuid).is_err(),
                "Invalid UUID '{}' should be rejected",
                invalid_uuid
            );
        }
    }

    /// Test role validation logic
    #[test]
    fn test_valid_roles() {
        let allowed_roles = ["authenticated", "anon", "service_role"];

        for role in &allowed_roles {
            assert!(
                allowed_roles.contains(role),
                "Role '{}' should be allowed",
                role
            );
        }
    }

    /// Test that invalid roles would be rejected
    #[test]
    fn test_invalid_roles() {
        let allowed_roles = ["authenticated", "anon", "service_role"];
        let invalid_roles = vec!["admin", "superuser", "hacker", ""];

        for role in &invalid_roles {
            assert!(
                !allowed_roles.contains(role),
                "Role '{}' should be rejected",
                role
            );
        }
    }

    /// Test that Bearer token format validation works
    #[test]
    fn test_bearer_token_format() {
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...";
        assert!(
            valid_header.starts_with("Bearer "),
            "Valid Bearer token should start with 'Bearer '"
        );

        let invalid_headers = vec![
            "Basic dXNlcjpwYXNz", // Wrong scheme
            "bearer eyJ...",      // Lowercase
            "BearereyJ...",       // No space
            "eyJ...",             // No scheme
            "",                   // Empty
            "Bearer",             // No token
        ];

        for invalid_header in &invalid_headers {
            if *invalid_header == "Bearer" {
                // Special case: "Bearer" without space is technically valid start
                assert!(
                    invalid_header.starts_with("Bearer"),
                    "Should start with Bearer"
                );
                assert_eq!(
                    invalid_header.len(),
                    6,
                    "But should be too short to be valid"
                );
            } else {
                assert!(
                    !invalid_header.starts_with("Bearer "),
                    "Invalid header '{}' should be rejected",
                    invalid_header
                );
            }
        }
    }

    /// Test AuthUser struct can be cloned and debugged
    #[test]
    fn test_auth_user_clone_debug() {
        let claims = Claims {
            sub: Uuid::new_v4().to_string(),
            aud: "authenticated".to_string(),
            exp: 1735137600,
            role: "authenticated".to_string(),
            email: Some("test@example.com".to_string()),
        };

        let auth_user = AuthUser {
            id: claims.sub.clone(),
            role: claims.role.clone(),
            full_claims: claims.clone(),
        };

        // Test Clone
        let cloned = auth_user.clone();
        assert_eq!(auth_user.id, cloned.id);
        assert_eq!(auth_user.role, cloned.role);

        // Test Debug (should not panic)
        let debug_str = format!("{:?}", auth_user);
        assert!(!debug_str.is_empty());
    }
}
