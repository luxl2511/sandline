/// Test process_geometry with MultiLineString format used in control points update
use dakar_planner_backend::geometry::routing::route_geometry;
use dakar_planner_backend::geometry::routing::RoutingConfig;
use sqlx::postgres::PgPoolOptions;

#[tokio::test]
async fn test_process_multilinestring_geometry() {
    dotenvy::from_filename(".env.test").ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Create geometry in same format as control points update handler
    let coordinates = vec![
        serde_json::json!([2.5, 35.2]),
        serde_json::json!([2.7, 35.4]),
        serde_json::json!([2.8, 35.5]),
    ];

    let geometry = serde_json::json!({
        "type": "MultiLineString",
        "coordinates": [coordinates]
    });

    println!(
        "Testing geometry: {}",
        serde_json::to_string_pretty(&geometry).unwrap()
    );

    let config = RoutingConfig::default();

    // This should not panic or return an error
    let result = route_geometry(&pool, &geometry, &config).await;

    match result {
        Ok((routed_geom, confidence)) => {
            println!("Success! Confidence: {}", confidence);
            println!(
                "Routed geometry: {}",
                serde_json::to_string_pretty(&routed_geom).unwrap()
            );
        }
        Err(e) => {
            panic!("route_geometry failed: {}", e);
        }
    }
}
