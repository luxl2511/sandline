use axum::{
    http::{header, HeaderValue, Method},
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Use library crate
use dakar_planner_backend::{config, db, jwks, routes, AppState};

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

    // Initialize JWKS cache
    let jwks_url = format!("{}/auth/v1/.well-known/jwks.json", config.supabase_url);
    tracing::info!("Fetching JWKS from {}", jwks_url);
    let jwks_cache = Arc::new(jwks::JwksCache::new(jwks_url).await?);
    tracing::info!("JWKS cache initialized");

    // Spawn background JWKS refresh task (runs every 24 hours)
    jwks_cache.clone().spawn_refresh_task();
    tracing::info!("JWKS background refresh task started");

    // Create app state
    let state = AppState {
        pool,
        jwks_cache,
        supabase_jwt_aud: "authenticated".to_string(),
    };

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
                        if origin_str.ends_with(domain) {
                            return true;
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
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(false);

    // Build router
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .with_state(state);

    // Start server
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
