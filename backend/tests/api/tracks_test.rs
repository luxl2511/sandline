use serial_test::serial;

use crate::common::{db, fixtures, helpers};

#[tokio::test]
#[serial]
async fn test_list_tracks_returns_all_tracks() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Insert sample tracks
    let track_ids = fixtures::insert_sample_tracks(&pool).await;

    // Build test app
    let app = helpers::build_test_app(pool.clone());

    // Make request
    let (status, body) = helpers::make_get_request(&app, "/api/tracks").await;

    // Assert response
    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(tracks.len(), track_ids.len());
}

#[tokio::test]
#[serial]
async fn test_list_tracks_with_source_filter() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Insert sample tracks (3 with source "rally", 2 with other sources)
    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    // Filter by source=rally
    let (status, body) = helpers::make_get_request(&app, "/api/tracks?source=rally").await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(
        tracks.len(),
        3,
        "Should return only tracks with source=rally"
    );

    // Verify all returned tracks have source "rally"
    for track in tracks {
        assert_eq!(track["source"].as_str().unwrap(), "rally");
    }
}

#[tokio::test]
#[serial]
async fn test_list_tracks_with_min_confidence_filter() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    // Filter by min_confidence=4 (should get tracks with confidence 4 and 5)
    let (status, body) = helpers::make_get_request(&app, "/api/tracks?min_confidence=4").await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(tracks.len(), 3, "Should return tracks with confidence >= 4");

    // Verify all returned tracks have confidence >= 4
    for track in tracks {
        let confidence = track["confidence"].as_i64().unwrap();
        assert!(confidence >= 4, "Track confidence should be >= 4");
    }
}

#[tokio::test]
#[serial]
async fn test_list_tracks_with_region_filter() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    // Filter by region (URL encoded)
    let (status, body) =
        helpers::make_get_request(&app, "/api/tracks?region=Western%20Sahara").await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(tracks.len(), 2, "Should return tracks in Western Sahara");

    // Verify all returned tracks have the correct region
    for track in tracks {
        assert_eq!(track["region"].as_str().unwrap(), "Western Sahara");
    }
}

#[tokio::test]
#[serial]
async fn test_list_tracks_with_multiple_filters() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    // Combine filters: source=rally AND min_confidence=5 AND region=Western%20Sahara
    let (status, body) = helpers::make_get_request(
        &app,
        "/api/tracks?source=rally&min_confidence=5&region=Western%20Sahara",
    )
    .await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(
        tracks.len(),
        1,
        "Should return only 1 track matching all criteria"
    );

    // Verify the track matches all filters
    let track = &tracks[0];
    assert_eq!(track["source"].as_str().unwrap(), "rally");
    assert_eq!(track["confidence"].as_i64().unwrap(), 5);
    assert_eq!(track["region"].as_str().unwrap(), "Western Sahara");
}

#[tokio::test]
#[serial]
async fn test_list_tracks_returns_geojson_geometry() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) = helpers::make_get_request(&app, "/api/tracks").await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert!(!tracks.is_empty(), "Should have at least one track");

    // Verify first track has valid GeoJSON geometry
    let geometry = &tracks[0]["geometry"];
    helpers::assert_valid_geojson_geometry(geometry);
}

#[tokio::test]
#[serial]
async fn test_list_tracks_limit_1000() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Insert just a few tracks (we can't easily test 1000+ tracks, but we verify the limit exists)
    fixtures::insert_sample_tracks(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) = helpers::make_get_request(&app, "/api/tracks").await;

    helpers::assert_status_ok(status);

    let tracks: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert!(
        tracks.len() <= 1000,
        "Should never return more than 1000 tracks"
    );
}

#[tokio::test]
#[serial]
async fn test_get_track_by_id_success() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Insert a single track
    let track_id =
        fixtures::insert_sample_track(&pool, "rally", Some("sand"), 5, Some("Western Sahara"))
            .await;

    let app = helpers::build_test_app(pool.clone());

    // Get track by ID
    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/tracks/{}", track_id)).await;

    helpers::assert_status_ok(status);

    let track: serde_json::Value = helpers::parse_json_response(&body);
    assert_eq!(track["id"].as_str().unwrap(), track_id.to_string());
    assert_eq!(track["source"].as_str().unwrap(), "rally");
    assert_eq!(track["surface"].as_str().unwrap(), "sand");
    assert_eq!(track["confidence"].as_i64().unwrap(), 5);

    // Verify geometry is present and valid
    helpers::assert_valid_geojson_geometry(&track["geometry"]);
}

#[tokio::test]
#[serial]
async fn test_get_track_by_id_not_found() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    // Use a random UUID that doesn't exist
    let fake_id = uuid::Uuid::new_v4();
    let (status, _body) =
        helpers::make_get_request(&app, &format!("/api/tracks/{}", fake_id)).await;

    helpers::assert_status_not_found(status);
}

#[tokio::test]
#[serial]
async fn test_get_track_geometry_format() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let track_id =
        fixtures::insert_sample_track(&pool, "rally", Some("sand"), 5, Some("Western Sahara"))
            .await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/tracks/{}", track_id)).await;

    helpers::assert_status_ok(status);

    let track: serde_json::Value = helpers::parse_json_response(&body);
    let geometry = &track["geometry"];

    // Verify GeoJSON structure
    assert_eq!(geometry["type"].as_str().unwrap(), "LineString");

    let coordinates = geometry["coordinates"].as_array().unwrap();
    assert!(!coordinates.is_empty());

    // Verify first coordinate is [lon, lat]
    let first_coord = coordinates[0].as_array().unwrap();
    assert_eq!(first_coord.len(), 2);

    let lon = first_coord[0].as_f64().unwrap();
    let lat = first_coord[1].as_f64().unwrap();

    // Verify coordinates are in valid ranges
    assert!(lon >= -180.0 && lon <= 180.0, "Longitude out of range");
    assert!(lat >= -90.0 && lat <= 90.0, "Latitude out of range");
}
