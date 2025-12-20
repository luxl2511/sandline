mod config;
mod db;
mod models;
mod routes;

use axum::{
    Router,
    http::{header, Method},
};
use tower_http::cors::{Any, CorsLayer};
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

    // Initialize database pool
    let pool = db::create_pool(&config.database_url).await?;
    tracing::info!("Database connection established");

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE])
        .allow_headers([header::CONTENT_TYPE]);

    // Build router
    let app = Router::new()
        .nest("/api", routes::api_routes())
        .layer(cors)
        .with_state(pool);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port))
        .await?;

    tracing::info!("Server listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
