use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, Image};

/// Custom Postgres image with PostGIS extension
pub struct PostgresWithPostGIS;

impl Default for PostgresWithPostGIS {
    fn default() -> Self {
        Self
    }
}

impl Image for PostgresWithPostGIS {
    fn name(&self) -> &str {
        "postgis/postgis"
    }

    fn tag(&self) -> &str {
        "15-3.4"
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
        vec![WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        )]
    }

    fn env_vars(
        &self,
    ) -> impl IntoIterator<
        Item = (
            impl Into<std::borrow::Cow<'_, str>>,
            impl Into<std::borrow::Cow<'_, str>>,
        ),
    > {
        vec![
            ("POSTGRES_DB", "test"),
            ("POSTGRES_USER", "postgres"),
            ("POSTGRES_PASSWORD", "postgres"),
        ]
    }

    fn expose_ports(&self) -> &[testcontainers::core::ContainerPort] {
        &[testcontainers::core::ContainerPort::Tcp(5432)]
    }
}

/// Set up a test database with PostGIS extension and run migrations
pub async fn setup_test_db() -> (PgPool, ContainerAsync<PostgresWithPostGIS>) {
    // Start PostgreSQL container with PostGIS
    let postgres_image = PostgresWithPostGIS::default();
    let container = postgres_image
        .start()
        .await
        .expect("Failed to start PostgreSQL container");

    // Get connection details
    let host = container.get_host().await.expect("Failed to get host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("Failed to get port");

    let database_url = format!("postgres://postgres:postgres@{}:{}/test", host, port);

    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    // Enable PostGIS extension
    sqlx::query("CREATE EXTENSION IF NOT EXISTS postgis")
        .execute(&pool)
        .await
        .expect("Failed to create PostGIS extension");

    sqlx::query("CREATE EXTENSION IF NOT EXISTS \"uuid-ossp\"")
        .execute(&pool)
        .await
        .expect("Failed to create uuid-ossp extension");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    (pool, container)
}

/// Clean all test data from the database (for test isolation)
pub async fn clean_database(pool: &PgPool) {
    sqlx::query("TRUNCATE route_proposals, route_versions, routes, curated_tracks CASCADE")
        .execute(pool)
        .await
        .expect("Failed to clean database");
}
