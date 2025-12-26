use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::PgPool;

/// Configuration for routing behavior
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    /// Maximum distance (meters) to consider a point "near" a curated track
    pub curated_track_threshold_meters: f64,
    /// Mapbox API access token
    pub mapbox_token: String,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            curated_track_threshold_meters: 100.0,
            mapbox_token: std::env::var("MAPBOX_ACCESS_TOKEN").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MapboxDirectionsResponse {
    routes: Vec<MapboxRoute>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MapboxRoute {
    geometry: MapboxGeometry,
    distance: f64,
    duration: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct MapboxGeometry {
    coordinates: Vec<Vec<f64>>,
}

/// Routes geometry between points using hybrid approach:
/// - On roads: Use Mapbox Directions API
/// - Off-road: Use curated tracks from database
///
/// # Algorithm
/// 1. For each consecutive pair of points (A → B):
///    a. Check if both points are near curated tracks (< threshold)
///    b. If YES: Route using curated tracks (PostGIS nearest-neighbor)
///    c. If NO: Route using Mapbox Directions API (road network)
/// 2. Combine all segments into final MultiLineString
/// 3. Return confidence score based on routing method mix
///
/// # Arguments
/// * `pool` - Database connection pool for curated track queries
/// * `geometry` - GeoJSON MultiLineString with waypoints
/// * `config` - Routing configuration (thresholds, API tokens)
///
/// # Returns
/// Tuple of (routed_geometry, confidence_score)
pub async fn route_geometry(
    pool: &PgPool,
    geometry: &Value,
    config: &RoutingConfig,
) -> Result<(Value, f64)> {
    let coords = geometry["coordinates"]
        .as_array()
        .ok_or_else(|| anyhow!("Invalid geometry: missing coordinates"))?;

    let mut routed_lines = Vec::new();
    let mut total_confidence = 0.0;
    let mut segment_count = 0;

    for line_coords in coords {
        let points: Vec<(f64, f64)> = line_coords
            .as_array()
            .ok_or_else(|| anyhow!("Invalid line coordinates"))?
            .iter()
            .filter_map(|p| {
                let arr = p.as_array()?;
                Some((arr[0].as_f64()?, arr[1].as_f64()?))
            })
            .collect();

        if points.is_empty() {
            continue;
        }

        let mut routed_line = Vec::new();

        // Route each consecutive pair of points
        for i in 0..points.len() {
            let (lng, lat) = points[i];
            routed_line.push(vec![lng, lat]);

            if i < points.len() - 1 {
                let next = points[i + 1];

                // Check if both points are near curated tracks
                let start_near_track =
                    is_near_curated_track(pool, lng, lat, config.curated_track_threshold_meters)
                        .await?;

                let end_near_track = is_near_curated_track(
                    pool,
                    next.0,
                    next.1,
                    config.curated_track_threshold_meters,
                )
                .await?;

                if start_near_track && end_near_track {
                    // OFF-ROAD: Both points near tracks → Use curated track routing
                    tracing::debug!(
                        "Routing via curated tracks: ({}, {}) → ({}, {})",
                        lng,
                        lat,
                        next.0,
                        next.1
                    );

                    let (track_points, confidence) = route_via_curated_tracks(
                        pool,
                        (lng, lat),
                        next,
                        config.curated_track_threshold_meters,
                    )
                    .await?;

                    // Add intermediate points (skip first as it's already added)
                    routed_line.extend(track_points.into_iter().skip(1));
                    total_confidence += confidence;
                } else {
                    // ON-ROAD: At least one point not near tracks → Use Mapbox
                    tracing::debug!(
                        "Routing via Mapbox Directions: ({}, {}) → ({}, {})",
                        lng,
                        lat,
                        next.0,
                        next.1
                    );

                    let (road_points, confidence) =
                        route_via_mapbox((lng, lat), next, &config.mapbox_token).await?;

                    // Add intermediate points (skip first as it's already added)
                    routed_line.extend(road_points.into_iter().skip(1));
                    total_confidence += confidence;
                }

                segment_count += 1;
            }
        }

        if !routed_line.is_empty() {
            routed_lines.push(routed_line);
        }
    }

    let avg_confidence = if segment_count > 0 {
        total_confidence / segment_count as f64
    } else {
        0.0
    };

    Ok((
        serde_json::json!({
            "type": "MultiLineString",
            "coordinates": routed_lines
        }),
        avg_confidence,
    ))
}

/// Check if a point is within threshold distance of any curated track
async fn is_near_curated_track(
    pool: &PgPool,
    lng: f64,
    lat: f64,
    threshold_meters: f64,
) -> Result<bool> {
    let result = sqlx::query_scalar!(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM curated_tracks
            WHERE ST_DWithin(
                geometry::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $3
            )
        ) as "exists!"
        "#,
        lng,
        lat,
        threshold_meters
    )
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Route between two points using curated tracks
/// Returns simplified path along nearest track + confidence score
async fn route_via_curated_tracks(
    pool: &PgPool,
    start: (f64, f64),
    end: (f64, f64),
    max_distance_meters: f64,
) -> Result<(Vec<Vec<f64>>, f64)> {
    // Simplified approach: Find nearest track and snap both points to it
    // Future enhancement: Implement proper A* routing across track network

    let result = sqlx::query!(
        r#"
        WITH start_snap AS (
            SELECT
                id,
                ST_ClosestPoint(
                    geometry,
                    ST_SetSRID(ST_MakePoint($1, $2), 4326)
                ) as snap_point,
                confidence
            FROM curated_tracks
            WHERE ST_DWithin(
                geometry::geography,
                ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography,
                $5
            )
            ORDER BY geometry <-> ST_SetSRID(ST_MakePoint($1, $2), 4326)
            LIMIT 1
        ),
        end_snap AS (
            SELECT
                ST_ClosestPoint(
                    geometry,
                    ST_SetSRID(ST_MakePoint($3, $4), 4326)
                ) as snap_point
            FROM curated_tracks
            WHERE ST_DWithin(
                geometry::geography,
                ST_SetSRID(ST_MakePoint($3, $4), 4326)::geography,
                $5
            )
            ORDER BY geometry <-> ST_SetSRID(ST_MakePoint($3, $4), 4326)
            LIMIT 1
        )
        SELECT
            ST_X(start_snap.snap_point) as start_lng,
            ST_Y(start_snap.snap_point) as start_lat,
            ST_X(end_snap.snap_point) as end_lng,
            ST_Y(end_snap.snap_point) as end_lat,
            start_snap.confidence
        FROM start_snap, end_snap
        "#,
        start.0,
        start.1,
        end.0,
        end.1,
        max_distance_meters
    )
    .fetch_optional(pool)
    .await?;

    if let Some(track) = result {
        // Return simple straight line between snapped points
        // Future: Extract actual track geometry between snap points
        let points = vec![
            vec![track.start_lng.unwrap(), track.start_lat.unwrap()],
            vec![track.end_lng.unwrap(), track.end_lat.unwrap()],
        ];

        let confidence = track.confidence as f64 / 5.0; // Normalize to 0.0-1.0
        Ok((points, confidence))
    } else {
        // Fallback: Direct line if no nearby tracks
        Ok((
            vec![vec![start.0, start.1], vec![end.0, end.1]],
            0.3, // Low confidence for unsnapped
        ))
    }
}

/// Route between two points using Mapbox Directions API
/// Returns road-snapped path + confidence score
async fn route_via_mapbox(
    start: (f64, f64),
    end: (f64, f64),
    token: &str,
) -> Result<(Vec<Vec<f64>>, f64)> {
    if token.is_empty() {
        tracing::warn!("MAPBOX_ACCESS_TOKEN not set, using direct line");
        return Ok((
            vec![vec![start.0, start.1], vec![end.0, end.1]],
            0.5, // Medium confidence for fallback
        ));
    }

    // Mapbox Directions API endpoint
    let url = format!(
        "https://api.mapbox.com/directions/v5/mapbox/driving/{},{};{},{}",
        start.0, start.1, end.0, end.1
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
        .query(&[
            ("access_token", token),
            ("geometries", "geojson"),
            ("overview", "full"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        tracing::error!("Mapbox Directions API error: {}", response.status());
        // Fallback to direct line on API error
        return Ok((vec![vec![start.0, start.1], vec![end.0, end.1]], 0.5));
    }

    let directions: MapboxDirectionsResponse = response.json().await?;

    if let Some(route) = directions.routes.first() {
        let points = route
            .geometry
            .coordinates
            .iter()
            .map(|coord| vec![coord[0], coord[1]])
            .collect();

        // High confidence for successful Mapbox routing
        Ok((points, 0.9))
    } else {
        // No route found, use direct line
        Ok((vec![vec![start.0, start.1], vec![end.0, end.1]], 0.5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_config_default() {
        let config = RoutingConfig::default();
        assert_eq!(config.curated_track_threshold_meters, 100.0);
    }
}
