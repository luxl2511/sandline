use chrono::NaiveDate;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

/// Create a valid GeoJSON LineString geometry for testing
pub fn create_test_geometry() -> serde_json::Value {
    json!({
        "type": "LineString",
        "coordinates": [
            [-5.0, 25.0],
            [-5.1, 25.1],
            [-5.2, 25.2]
        ]
    })
}

/// Create a more complex test geometry with more points
pub fn create_complex_geometry() -> serde_json::Value {
    json!({
        "type": "LineString",
        "coordinates": [
            [-5.0, 25.0],
            [-5.1, 25.1],
            [-5.2, 25.2],
            [-5.3, 25.3],
            [-5.4, 25.4]
        ]
    })
}

/// Insert a sample curated track into the database
pub async fn insert_sample_track(
    pool: &PgPool,
    source: &str,
    surface: Option<&str>,
    confidence: i32,
    region: Option<&str>,
) -> Uuid {
    let geometry = create_test_geometry();
    let geometry_wkt = linestring_to_wkt(&geometry);

    let record = sqlx::query!(
        r#"
        INSERT INTO curated_tracks (geometry, source, surface, confidence, last_verified, region)
        VALUES (ST_GeomFromText($1, 4326), $2, $3, $4, $5, $6)
        RETURNING id
        "#,
        geometry_wkt,
        source,
        surface,
        confidence,
        Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap()),
        region
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert sample track");

    record.id
}

/// Insert multiple sample tracks with different properties
pub async fn insert_sample_tracks(pool: &PgPool) -> Vec<Uuid> {
    vec![
        insert_sample_track(pool, "rally", Some("sand"), 5, Some("Western Sahara")).await,
        insert_sample_track(pool, "rally", Some("gravel"), 4, Some("Morocco")).await,
        insert_sample_track(pool, "curated", Some("sand"), 3, Some("Western Sahara")).await,
        insert_sample_track(pool, "osm", None, 2, Some("Mauritania")).await,
        insert_sample_track(pool, "rally", Some("rock"), 5, None).await,
    ]
}

/// Insert a route with an initial version
pub async fn insert_route_with_version(
    pool: &PgPool,
    name: &str,
    geometry: serde_json::Value,
) -> (Uuid, Uuid) {
    let owner_id = Uuid::new_v4(); // Test owner

    // Start transaction
    let mut tx = pool.begin().await.expect("Failed to start transaction");

    // Insert route
    let route_record = sqlx::query!(
        r#"
        INSERT INTO routes (name, owner_id)
        VALUES ($1, $2)
        RETURNING id
        "#,
        name,
        owner_id
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert route");

    // Insert initial version with JSONB geometry
    let version_record = sqlx::query!(
        r#"
        INSERT INTO route_versions (route_id, geometry)
        VALUES ($1, $2)
        RETURNING id
        "#,
        route_record.id,
        geometry
    )
    .fetch_one(&mut *tx)
    .await
    .expect("Failed to insert route version");

    tx.commit().await.expect("Failed to commit transaction");

    (route_record.id, version_record.id)
}

/// Insert a route proposal
pub async fn insert_proposal(
    pool: &PgPool,
    route_id: Uuid,
    geometry: serde_json::Value,
    comment: &str,
    status: &str,
) -> Uuid {
    let created_by = Uuid::new_v4(); // Test user

    let record = sqlx::query!(
        r#"
        INSERT INTO route_proposals (route_id, geometry, comment, status, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id
        "#,
        route_id,
        geometry,
        comment,
        status,
        created_by
    )
    .fetch_one(pool)
    .await
    .expect("Failed to insert proposal");

    record.id
}

/// Convert GeoJSON LineString to WKT format for PostGIS
fn linestring_to_wkt(geojson: &serde_json::Value) -> String {
    let coords = geojson["coordinates"]
        .as_array()
        .expect("Invalid GeoJSON: missing coordinates array");

    let points: Vec<String> = coords
        .iter()
        .map(|coord| {
            let arr = coord.as_array().expect("Invalid coordinate");
            let lon = arr[0].as_f64().expect("Invalid longitude");
            let lat = arr[1].as_f64().expect("Invalid latitude");
            format!("{} {}", lon, lat)
        })
        .collect();

    format!("LINESTRING({})", points.join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linestring_to_wkt() {
        let geojson = create_test_geometry();
        let wkt = linestring_to_wkt(&geojson);
        assert_eq!(wkt, "LINESTRING(-5 25, -5.1 25.1, -5.2 25.2)");
    }

    #[test]
    fn test_complex_geometry_to_wkt() {
        let geojson = create_complex_geometry();
        let wkt = linestring_to_wkt(&geojson);
        assert_eq!(
            wkt,
            "LINESTRING(-5 25, -5.1 25.1, -5.2 25.2, -5.3 25.3, -5.4 25.4)"
        );
    }
}
