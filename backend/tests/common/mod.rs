/// Common test utilities for Sandline backend integration tests
///
/// This module provides shared utilities for creating test database pools,
/// test app state, authenticated test users, and helper functions for making
/// HTTP requests to the API.
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use dakar_planner_backend::AppState;
use sqlx::PgPool;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

/// Create test database pool connected to Supabase test project
///
/// Reads DATABASE_URL from environment (.env.test should be loaded before tests)
///
/// # Panics
/// Panics if DATABASE_URL is not set or if connection fails
pub async fn create_test_pool() -> PgPool {
    // Ensure .env.test is loaded
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for tests (check .env.test file)");

    sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

/// Create test app state with real JWKS cache from Supabase test project
///
/// # Panics
/// Panics if SUPABASE_URL is not set or if JWKS fetch fails
pub async fn create_test_app_state() -> AppState {
    use dakar_planner_backend::jwks::JwksCache;

    let pool = create_test_pool().await;

    let supabase_url = std::env::var("SUPABASE_URL")
        .expect("SUPABASE_URL must be set for tests (check .env.test file)");

    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", supabase_url);

    let jwks_cache = Arc::new(
        JwksCache::new(jwks_url)
            .await
            .expect("Failed to fetch JWKS for tests"),
    );

    AppState {
        pool,
        jwks_cache,
        supabase_jwt_aud: std::env::var("SUPABASE_JWT_AUD")
            .unwrap_or_else(|_| "authenticated".to_string()),
    }
}

/// Test user with Supabase credentials
///
/// Represents a test user that exists in the Supabase test project.
/// Use this for authenticated API requests.
pub struct TestUser {
    pub id: Uuid,
    pub email: String,
    pub jwt: String,
}

impl TestUser {
    /// Convert TestUser to AuthUser for use with RlsTransaction
    ///
    /// This creates an AuthUser instance that can be used with RlsTransaction::begin()
    pub fn to_auth_user(&self) -> dakar_planner_backend::middleware::auth::AuthUser {
        use dakar_planner_backend::middleware::auth::{AuthUser, Claims};

        let claims = Claims {
            sub: self.id.to_string(),
            aud: "authenticated".to_string(),
            exp: (chrono::Utc::now().timestamp() + 3600) as i64, // 1 hour from now
            role: "authenticated".to_string(),
            email: Some(self.email.clone()),
        };

        AuthUser {
            id: self.id.to_string(),
            role: "authenticated".to_string(),
            full_claims: claims,
        }
    }

    /// Create test user from email and password
    ///
    /// This attempts to sign in to Supabase with the provided credentials.
    /// The user must already exist in the Supabase test project.
    ///
    /// # Arguments
    /// * `email` - Email of test user (e.g., "test-user-1@example.com")
    /// * `password` - Password of test user
    ///
    /// # Returns
    /// A `TestUser` with valid JWT token for making authenticated requests
    ///
    /// # Panics
    /// Panics if SUPABASE_URL or SUPABASE_ANON_KEY are not set, or if sign-in fails
    pub async fn sign_in(email: &str, password: &str) -> Self {
        // Ensure .env.test is loaded
        dotenvy::from_filename(".env.test").ok();

        let supabase_url =
            std::env::var("SUPABASE_URL").expect("SUPABASE_URL must be set for tests");
        let anon_key =
            std::env::var("SUPABASE_ANON_KEY").expect("SUPABASE_ANON_KEY must be set for tests");

        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}/auth/v1/token?grant_type=password",
                supabase_url
            ))
            .header("apikey", &anon_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "email": email,
                "password": password,
            }))
            .send()
            .await
            .expect("Failed to send sign-in request");

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            panic!(
                "Supabase sign-in failed with status {}: {}. \
                 Make sure test user exists in Supabase test project.",
                status, body
            );
        }

        let json: serde_json::Value = response
            .json()
            .await
            .expect("Failed to parse sign-in response");

        let access_token = json["access_token"]
            .as_str()
            .expect("Missing access_token in response")
            .to_string();

        let user_id = json["user"]["id"]
            .as_str()
            .expect("Missing user id in response");
        let user_id = Uuid::parse_str(user_id).expect("Invalid user UUID");

        Self {
            id: user_id,
            email: email.to_string(),
            jwt: access_token,
        }
    }

    /// Get test user 1 (for owner scenarios)
    ///
    /// Signs in as test-user-1@example.com
    ///
    /// # Panics
    /// Panics if TEST_USER_1_EMAIL or TEST_USER_1_PASSWORD are not set, or if sign-in fails
    pub async fn test_user_1() -> Self {
        let email = std::env::var("TEST_USER_1_EMAIL")
            .unwrap_or_else(|_| "test-user-1@example.com".to_string());
        let password = std::env::var("TEST_USER_1_PASSWORD")
            .expect("TEST_USER_1_PASSWORD must be set for tests");

        Self::sign_in(&email, &password).await
    }

    /// Get test user 2 (for collaborator scenarios)
    ///
    /// Signs in as test-user-2@example.com
    ///
    /// # Panics
    /// Panics if TEST_USER_2_EMAIL or TEST_USER_2_PASSWORD are not set, or if sign-in fails
    pub async fn test_user_2() -> Self {
        let email = std::env::var("TEST_USER_2_EMAIL")
            .unwrap_or_else(|_| "test-user-2@example.com".to_string());
        let password = std::env::var("TEST_USER_2_PASSWORD")
            .expect("TEST_USER_2_PASSWORD must be set for tests");

        Self::sign_in(&email, &password).await
    }
}

/// Helper to send authenticated HTTP request to the API
///
/// # Arguments
/// * `app` - The Axum router
/// * `method` - HTTP method (e.g., "GET", "POST", "PUT", "DELETE")
/// * `path` - Request path (e.g., "/api/routes")
/// * `jwt` - JWT token from TestUser
/// * `body` - Optional JSON body as String
///
/// # Returns
/// Tuple of (StatusCode, response body as String)
///
/// # Example
/// ```rust,ignore
/// let user = TestUser::test_user_1().await;
/// let (status, body) = send_authed_request(
///     app,
///     "POST",
///     "/api/routes",
///     &user.jwt,
///     Some(r#"{"name": "Test Route", "geometry": {...}}"#.to_string()),
/// ).await;
/// assert_eq!(status, StatusCode::OK);
/// ```
pub async fn send_authed_request(
    app: Router,
    method: &str,
    path: &str,
    jwt: &str,
    body: Option<String>,
) -> (StatusCode, String) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("Authorization", format!("Bearer {}", jwt))
        .header("Content-Type", "application/json");

    let req = if let Some(b) = body {
        req.body(Body::from(b)).unwrap()
    } else {
        req.body(Body::empty()).unwrap()
    };

    let response = app.oneshot(req).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    (status, body)
}

/// Helper to send unauthenticated HTTP request to the API
///
/// # Arguments
/// * `app` - The Axum router
/// * `method` - HTTP method (e.g., "GET", "POST")
/// * `path` - Request path (e.g., "/api/routes")
/// * `body` - Optional JSON body as String
///
/// # Returns
/// Tuple of (StatusCode, response body as String)
pub async fn send_request(
    app: Router,
    method: &str,
    path: &str,
    body: Option<String>,
) -> (StatusCode, String) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("Content-Type", "application/json");

    let req = if let Some(b) = body {
        req.body(Body::from(b)).unwrap()
    } else {
        req.body(Body::empty()).unwrap()
    };

    let response = app.oneshot(req).await.unwrap();

    let status = response.status();
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body_bytes.to_vec()).unwrap();

    (status, body)
}

/// Clean up test data for a specific user
///
/// Deletes all routes, route versions, proposals, and editing sessions
/// created by the specified user.
///
/// # Arguments
/// * `pool` - Database pool
/// * `user_id` - UUID of the user whose data should be cleaned up
///
/// # Panics
/// Panics if cleanup queries fail
pub async fn cleanup_test_data(pool: &PgPool, user_id: Uuid) {
    // Delete in order of dependencies:
    // 1. Route versions (references routes)
    // 2. Route point changes (references routes)
    // 3. Route editing sessions (references routes)
    // 4. Routes (owned by user)

    sqlx::query!(
        "DELETE FROM route_versions WHERE route_id IN (SELECT id FROM routes WHERE owner_id = $1)",
        user_id
    )
    .execute(pool)
    .await
    .expect("Failed to cleanup route_versions");

    sqlx::query!(
        "DELETE FROM route_point_changes WHERE route_id IN (SELECT id FROM routes WHERE owner_id = $1)",
        user_id
    )
    .execute(pool)
    .await
    .expect("Failed to cleanup route_point_changes");

    sqlx::query!(
        "DELETE FROM route_editing_sessions WHERE route_id IN (SELECT id FROM routes WHERE owner_id = $1)",
        user_id
    )
    .execute(pool)
    .await
    .expect("Failed to cleanup route_editing_sessions");

    sqlx::query!("DELETE FROM routes WHERE owner_id = $1", user_id)
        .execute(pool)
        .await
        .expect("Failed to cleanup routes");
}

/// Cleanup all proposals created by a user
///
/// Useful for cleaning up after proposal-related tests
pub async fn cleanup_proposals_by_user(pool: &PgPool, user_id: Uuid) {
    sqlx::query!(
        "DELETE FROM route_point_changes WHERE user_id = $1",
        user_id
    )
    .execute(pool)
    .await
    .expect("Failed to cleanup proposals");
}
