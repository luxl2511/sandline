use serde_json::json;
use serial_test::serial;

use crate::common::{db, fixtures, helpers};

#[tokio::test]
#[serial]
async fn test_list_routes_returns_with_latest_geometry() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Insert routes with versions
    let geometry1 = fixtures::create_test_geometry();
    let geometry2 = fixtures::create_complex_geometry();

    fixtures::insert_route_with_version(&pool, "Test Route 1", geometry1.clone()).await;
    fixtures::insert_route_with_version(&pool, "Test Route 2", geometry2.clone()).await;

    let app = helpers::build_test_app(pool.clone());

    // List routes
    let (status, body) = helpers::make_get_request(&app, "/api/routes").await;

    helpers::assert_status_ok(status);

    let routes: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(routes.len(), 2, "Should return both routes");

    // Verify each route has geometry from latest version
    for route in &routes {
        assert!(route.get("id").is_some());
        assert!(route.get("name").is_some());
        assert!(route.get("geometry").is_some());
        helpers::assert_valid_geojson_geometry(&route["geometry"]);
    }
}

#[tokio::test]
#[serial]
async fn test_list_routes_empty_when_no_routes() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) = helpers::make_get_request(&app, "/api/routes").await;

    helpers::assert_status_ok(status);

    let routes: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(routes.len(), 0, "Should return empty array");
}

#[tokio::test]
#[serial]
async fn test_list_routes_ordered_by_created_at_desc() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();

    // Insert routes in sequence
    fixtures::insert_route_with_version(&pool, "Oldest Route", geometry.clone()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    fixtures::insert_route_with_version(&pool, "Middle Route", geometry.clone()).await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    fixtures::insert_route_with_version(&pool, "Newest Route", geometry.clone()).await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) = helpers::make_get_request(&app, "/api/routes").await;

    helpers::assert_status_ok(status);

    let routes: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(routes.len(), 3);

    // Verify newest first
    assert_eq!(routes[0]["name"].as_str().unwrap(), "Newest Route");
    assert_eq!(routes[2]["name"].as_str().unwrap(), "Oldest Route");
}

#[tokio::test]
#[serial]
async fn test_get_route_by_id_success() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _version_id) =
        fixtures::insert_route_with_version(&pool, "My Test Route", geometry.clone()).await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/routes/{}", route_id)).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);
    assert_eq!(route["id"].as_str().unwrap(), route_id.to_string());
    assert_eq!(route["name"].as_str().unwrap(), "My Test Route");
    helpers::assert_valid_geojson_geometry(&route["geometry"]);
}

#[tokio::test]
#[serial]
async fn test_get_route_by_id_not_found() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let fake_id = uuid::Uuid::new_v4();
    let (status, _body) =
        helpers::make_get_request(&app, &format!("/api/routes/{}", fake_id)).await;

    helpers::assert_status_not_found(status);
}

#[tokio::test]
#[serial]
async fn test_create_route_success() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let geometry = fixtures::create_test_geometry();
    let payload = json!({
        "name": "New Adventure Route",
        "geometry": geometry
    });

    let (status, body) = helpers::make_post_request(&app, "/api/routes", payload).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);
    assert!(route.get("id").is_some());
    assert_eq!(route["name"].as_str().unwrap(), "New Adventure Route");
    helpers::assert_valid_geojson_geometry(&route["geometry"]);
}

#[tokio::test]
#[serial]
async fn test_create_route_returns_geometry() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let geometry = fixtures::create_test_geometry();
    let payload = json!({
        "name": "Test Route",
        "geometry": geometry
    });

    let (status, body) = helpers::make_post_request(&app, "/api/routes", payload).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);

    // Verify geometry matches what we sent
    assert_eq!(
        route["geometry"]["type"].as_str().unwrap(),
        geometry["type"].as_str().unwrap()
    );
}

#[tokio::test]
#[serial]
async fn test_update_route_creates_new_version() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let initial_geometry = fixtures::create_test_geometry();
    let (route_id, _version_id) =
        fixtures::insert_route_with_version(&pool, "Evolving Route", initial_geometry).await;

    let app = helpers::build_test_app(pool.clone());

    // Update with new geometry
    let new_geometry = fixtures::create_complex_geometry();
    let payload = json!({
        "geometry": new_geometry
    });

    let (status, body) =
        helpers::make_put_request(&app, &format!("/api/routes/{}", route_id), payload).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);

    // Verify route ID is same
    assert_eq!(route["id"].as_str().unwrap(), route_id.to_string());

    // Verify geometry is the new one
    assert_eq!(
        route["geometry"]["coordinates"].as_array().unwrap().len(),
        5,
        "Should have 5 coordinates from complex geometry"
    );

    // Verify version count in database
    let version_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM route_versions WHERE route_id = $1",
        route_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(
        version_count.count.unwrap(),
        2,
        "Should have 2 versions now"
    );
}

#[tokio::test]
#[serial]
async fn test_update_route_not_found() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let fake_id = uuid::Uuid::new_v4();
    let geometry = fixtures::create_test_geometry();
    let payload = json!({
        "geometry": geometry
    });

    let (status, _body) =
        helpers::make_put_request(&app, &format!("/api/routes/{}", fake_id), payload).await;

    helpers::assert_status_not_found(status);
}

#[tokio::test]
#[serial]
async fn test_update_route_preserves_route_metadata() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let initial_geometry = fixtures::create_test_geometry();
    let (route_id, _version_id) =
        fixtures::insert_route_with_version(&pool, "Original Name", initial_geometry).await;

    // Fetch initial route data
    let initial_route = sqlx::query!(
        "SELECT name, owner_id, created_at FROM routes WHERE id = $1",
        route_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let app = helpers::build_test_app(pool.clone());

    // Update geometry
    let new_geometry = fixtures::create_complex_geometry();
    let payload = json!({
        "geometry": new_geometry
    });

    let (status, body) =
        helpers::make_put_request(&app, &format!("/api/routes/{}", route_id), payload).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);

    // Verify name and owner_id unchanged
    assert_eq!(route["name"].as_str().unwrap(), initial_route.name);
    assert_eq!(
        route["owner_id"].as_str().unwrap(),
        initial_route.owner_id.to_string()
    );
}

#[tokio::test]
#[serial]
async fn test_route_has_multiple_versions() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry1 = fixtures::create_test_geometry();
    let (route_id, _) = fixtures::insert_route_with_version(&pool, "Version Test", geometry1).await;

    let app = helpers::build_test_app(pool.clone());

    // Add second version
    let geometry2 = fixtures::create_complex_geometry();
    let payload2 = json!({ "geometry": geometry2 });
    helpers::make_put_request(&app, &format!("/api/routes/{}", route_id), payload2).await;

    // Add third version
    let geometry3 = fixtures::create_test_geometry();
    let payload3 = json!({ "geometry": geometry3 });
    helpers::make_put_request(&app, &format!("/api/routes/{}", route_id), payload3).await;

    // Verify all versions exist in database
    let versions = sqlx::query!(
        "SELECT id, created_at FROM route_versions WHERE route_id = $1 ORDER BY created_at ASC",
        route_id
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(versions.len(), 3, "Should have 3 versions");

    // Verify timestamps are in ascending order
    assert!(versions[0].created_at < versions[1].created_at);
    assert!(versions[1].created_at < versions[2].created_at);
}

#[tokio::test]
#[serial]
async fn test_create_route_transaction_integrity() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let geometry = fixtures::create_test_geometry();
    let payload = json!({
        "name": "Transaction Test",
        "geometry": geometry
    });

    let (status, body) = helpers::make_post_request(&app, "/api/routes", payload).await;

    helpers::assert_status_ok(status);

    let route: serde_json::Value = helpers::parse_json_response(&body);
    let route_id = uuid::Uuid::parse_str(route["id"].as_str().unwrap()).unwrap();

    // Verify route exists
    let route_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM routes WHERE id = $1",
        route_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(route_count.count.unwrap(), 1);

    // Verify version exists
    let version_count = sqlx::query!(
        "SELECT COUNT(*) as count FROM route_versions WHERE route_id = $1",
        route_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        version_count.count.unwrap(),
        1,
        "Route should have exactly one version"
    );
}
