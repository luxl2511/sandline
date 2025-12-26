// Library exports for testing and potential future code reuse

pub mod config;
pub mod db;
pub mod geometry;
pub mod jwks;
pub mod middleware;
pub mod models;
pub mod routes;

// Re-export AppState for convenience
pub use crate::middleware::auth::AuthUser;

use std::sync::Arc;

/// Application state shared across all request handlers
#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
    pub jwks_cache: Arc<jwks::JwksCache>,
    pub supabase_jwt_aud: String,
}
