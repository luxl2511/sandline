mod config;
mod db;
mod models;
mod routes;

use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,dakar_planner_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::from_env()?;
    tracing::info!("Starting server on {}:{}", config.host, config.port);
    tracing::info!("Allowed CORS origins: {:?}", config.allowed_origins);

    // Initialize database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection established");

    // CORS configuration with wildcard support
    let allowed_origins = config.allowed_origins.clone();

    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::AllowOrigin::predicate(
            move |origin: &HeaderValue, _request_parts| {
                let origin_str = origin.to_str().unwrap_or("");

                // Check exact matches
                if allowed_origins.contains(&origin_str.to_string()) {
                    return true;
                }

                // Check wildcard patterns (e.g., *.vercel.app)
                for pattern in &allowed_origins {
                    if let Some(domain) = pattern.strip_prefix("*.") {
                        // Must end with the domain AND have a dot before it (ensuring subdomain exists)
                        if origin_str.ends_with(domain) && origin_str.len() > domain.len() {
                            let before_domain = &origin_str[..origin_str.len() - domain.len()];
                            if before_domain.ends_with('.') {
                                return true;
                            }
                        }
                    }
                }

                false
            },
        ))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE])
        .allow_credentials(false);

    // Build router
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .with_state(pool);

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    /// Helper function to test CORS origin matching logic
    fn test_origin_matches(allowed_origins: &[&str], origin: &str) -> bool {
        let allowed: Vec<String> = allowed_origins.iter().map(|s| s.to_string()).collect();

        // Check exact matches
        if allowed.contains(&origin.to_string()) {
            return true;
        }

        // Check wildcard patterns
        for pattern in &allowed {
            if let Some(domain) = pattern.strip_prefix("*.") {
                // Must end with the domain AND have a dot before it (ensuring subdomain exists)
                if origin.ends_with(domain) && origin.len() > domain.len() {
                    let before_domain = &origin[..origin.len() - domain.len()];
                    if before_domain.ends_with('.') {
                        return true;
                    }
                }
            }
        }

        false
    }

    #[test]
    fn test_cors_allows_exact_match() {
        let allowed = vec!["https://example.com", "https://app.example.com"];
        assert!(test_origin_matches(&allowed, "https://example.com"));
        assert!(test_origin_matches(&allowed, "https://app.example.com"));
    }

    #[test]
    fn test_cors_blocks_non_matching_origin() {
        let allowed = vec!["https://example.com"];
        assert!(!test_origin_matches(&allowed, "https://evil.com"));
        assert!(!test_origin_matches(
            &allowed,
            "https://example.com.evil.com"
        ));
    }

    #[test]
    fn test_cors_allows_wildcard_match() {
        let allowed = vec!["*.vercel.app"];
        assert!(test_origin_matches(&allowed, "my-app.vercel.app"));
        assert!(test_origin_matches(&allowed, "preview-123.vercel.app"));
        assert!(test_origin_matches(&allowed, "production.vercel.app"));
    }

    #[test]
    fn test_cors_wildcard_requires_subdomain() {
        let allowed = vec!["*.vercel.app"];
        // Direct domain match should fail (no subdomain)
        assert!(!test_origin_matches(&allowed, "vercel.app"));
    }

    #[test]
    fn test_cors_wildcard_domain_boundary() {
        let allowed = vec!["*.vercel.app"];
        // Should not match different TLD
        assert!(!test_origin_matches(&allowed, "vercel.app.com"));
        // Should not match if domain is just a prefix
        assert!(!test_origin_matches(&allowed, "fakevercel.app"));
    }

    #[test]
    fn test_cors_multiple_wildcards() {
        let allowed = vec!["*.vercel.app", "*.netlify.app"];
        assert!(test_origin_matches(&allowed, "my-app.vercel.app"));
        assert!(test_origin_matches(&allowed, "my-app.netlify.app"));
        assert!(!test_origin_matches(&allowed, "my-app.heroku.com"));
    }

    #[test]
    fn test_cors_exact_and_wildcard_mixed() {
        let allowed = vec!["https://example.com", "*.vercel.app"];
        assert!(test_origin_matches(&allowed, "https://example.com"));
        assert!(test_origin_matches(&allowed, "my-app.vercel.app"));
        assert!(!test_origin_matches(&allowed, "https://evil.com"));
    }

    #[test]
    fn test_cors_case_sensitive() {
        let allowed = vec!["https://example.com"];
        // Origins are case-sensitive
        assert!(!test_origin_matches(&allowed, "https://Example.com"));
        assert!(!test_origin_matches(&allowed, "https://EXAMPLE.COM"));
    }
}
