use serde_json::json;
use serial_test::serial;

use crate::common::{db, fixtures, helpers};

#[tokio::test]
#[serial]
async fn test_list_proposals_for_route() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Create a route
    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    // Create proposals for the route
    let proposal_geometry1 = fixtures::create_test_geometry();
    let proposal_geometry2 = fixtures::create_complex_geometry();

    fixtures::insert_proposal(
        &pool,
        route_id,
        proposal_geometry1,
        "First proposal",
        "pending",
    )
    .await;
    fixtures::insert_proposal(
        &pool,
        route_id,
        proposal_geometry2,
        "Second proposal",
        "pending",
    )
    .await;

    let app = helpers::build_test_app(pool.clone());

    // List proposals for the route
    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/routes/{}/proposals", route_id)).await;

    helpers::assert_status_ok(status);

    let proposals: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(proposals.len(), 2, "Should return 2 proposals");

    // Verify each proposal has required fields
    for proposal in &proposals {
        assert!(proposal.get("id").is_some());
        assert_eq!(proposal["route_id"].as_str().unwrap(), route_id.to_string());
        assert!(proposal.get("geometry").is_some());
        assert!(proposal.get("comment").is_some());
        assert_eq!(proposal["status"].as_str().unwrap(), "pending");
    }
}

#[tokio::test]
#[serial]
async fn test_list_proposals_ordered_by_created_at_desc() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    // Create proposals in sequence
    fixtures::insert_proposal(&pool, route_id, geometry.clone(), "Oldest", "pending").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    fixtures::insert_proposal(&pool, route_id, geometry.clone(), "Middle", "pending").await;
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    fixtures::insert_proposal(&pool, route_id, geometry.clone(), "Newest", "pending").await;

    let app = helpers::build_test_app(pool.clone());

    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/routes/{}/proposals", route_id)).await;

    helpers::assert_status_ok(status);

    let proposals: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(proposals.len(), 3);

    // Verify newest first
    assert_eq!(proposals[0]["comment"].as_str().unwrap(), "Newest");
    assert_eq!(proposals[2]["comment"].as_str().unwrap(), "Oldest");
}

#[tokio::test]
#[serial]
async fn test_list_proposals_empty_for_nonexistent_route() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let fake_route_id = uuid::Uuid::new_v4();
    let (status, body) =
        helpers::make_get_request(&app, &format!("/api/routes/{}/proposals", fake_route_id)).await;

    helpers::assert_status_ok(status);

    let proposals: Vec<serde_json::Value> = helpers::parse_json_response(&body);
    assert_eq!(
        proposals.len(),
        0,
        "Should return empty array for nonexistent route"
    );
}

#[tokio::test]
#[serial]
async fn test_create_proposal_success() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    // Create a route
    let geometry = fixtures::create_test_geometry();
    let (route_id, _) = fixtures::insert_route_with_version(&pool, "Test Route", geometry).await;

    let app = helpers::build_test_app(pool.clone());

    // Create a proposal
    let proposal_geometry = fixtures::create_complex_geometry();
    let payload = json!({
        "route_id": route_id,
        "geometry": proposal_geometry,
        "comment": "This is a better route!"
    });

    let (status, body) = helpers::make_post_request(&app, "/api/proposals", payload).await;

    helpers::assert_status_ok(status);

    let proposal: serde_json::Value = helpers::parse_json_response(&body);
    assert!(proposal.get("id").is_some());
    assert_eq!(proposal["route_id"].as_str().unwrap(), route_id.to_string());
    assert_eq!(
        proposal["comment"].as_str().unwrap(),
        "This is a better route!"
    );
    assert_eq!(
        proposal["status"].as_str().unwrap(),
        "pending",
        "Default status should be pending"
    );
    helpers::assert_valid_geojson_geometry(&proposal["geometry"]);
}

#[tokio::test]
#[serial]
async fn test_create_proposal_for_nonexistent_route() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let fake_route_id = uuid::Uuid::new_v4();
    let geometry = fixtures::create_test_geometry();
    let payload = json!({
        "route_id": fake_route_id,
        "geometry": geometry,
        "comment": "Proposal for nonexistent route"
    });

    let (status, _body) = helpers::make_post_request(&app, "/api/proposals", payload).await;

    // Should fail due to foreign key constraint
    helpers::assert_status_internal_server_error(status);
}

#[tokio::test]
#[serial]
async fn test_update_proposal_status_to_accepted() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    let proposal_id =
        fixtures::insert_proposal(&pool, route_id, geometry, "Test proposal", "pending").await;

    let app = helpers::build_test_app(pool.clone());

    // Update status to accepted
    let payload = json!({
        "status": "accepted"
    });

    let (status, body) =
        helpers::make_patch_request(&app, &format!("/api/proposals/{}", proposal_id), payload)
            .await;

    helpers::assert_status_ok(status);

    let proposal: serde_json::Value = helpers::parse_json_response(&body);
    assert_eq!(proposal["id"].as_str().unwrap(), proposal_id.to_string());
    assert_eq!(proposal["status"].as_str().unwrap(), "accepted");
}

#[tokio::test]
#[serial]
async fn test_update_proposal_status_to_rejected() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    let proposal_id =
        fixtures::insert_proposal(&pool, route_id, geometry, "Test proposal", "pending").await;

    let app = helpers::build_test_app(pool.clone());

    // Update status to rejected
    let payload = json!({
        "status": "rejected"
    });

    let (status, body) =
        helpers::make_patch_request(&app, &format!("/api/proposals/{}", proposal_id), payload)
            .await;

    helpers::assert_status_ok(status);

    let proposal: serde_json::Value = helpers::parse_json_response(&body);
    assert_eq!(proposal["status"].as_str().unwrap(), "rejected");
}

#[tokio::test]
#[serial]
async fn test_update_proposal_status_not_found() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let app = helpers::build_test_app(pool.clone());

    let fake_id = uuid::Uuid::new_v4();
    let payload = json!({
        "status": "accepted"
    });

    let (status, _body) =
        helpers::make_patch_request(&app, &format!("/api/proposals/{}", fake_id), payload).await;

    helpers::assert_status_not_found(status);
}

#[tokio::test]
#[serial]
async fn test_update_proposal_status_invalid_status() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    let proposal_id =
        fixtures::insert_proposal(&pool, route_id, geometry, "Test proposal", "pending").await;

    let app = helpers::build_test_app(pool.clone());

    // Try to set an invalid status
    let payload = json!({
        "status": "invalid_status"
    });

    let (status, _body) =
        helpers::make_patch_request(&app, &format!("/api/proposals/{}", proposal_id), payload)
            .await;

    // Should fail due to CHECK constraint
    helpers::assert_status_internal_server_error(status);
}

#[tokio::test]
#[serial]
async fn test_updated_at_changes_on_status_update() {
    let (pool, _container) = db::setup_test_db().await;
    db::clean_database(&pool).await;

    let geometry = fixtures::create_test_geometry();
    let (route_id, _) =
        fixtures::insert_route_with_version(&pool, "Test Route", geometry.clone()).await;

    let proposal_id =
        fixtures::insert_proposal(&pool, route_id, geometry, "Test proposal", "pending").await;

    // Get initial updated_at
    let initial_proposal = sqlx::query!(
        "SELECT updated_at FROM route_proposals WHERE id = $1",
        proposal_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Wait a bit to ensure timestamp difference
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let app = helpers::build_test_app(pool.clone());

    // Update status
    let payload = json!({
        "status": "accepted"
    });

    let (status, _body) =
        helpers::make_patch_request(&app, &format!("/api/proposals/{}", proposal_id), payload)
            .await;

    helpers::assert_status_ok(status);

    // Get updated updated_at
    let updated_proposal = sqlx::query!(
        "SELECT updated_at FROM route_proposals WHERE id = $1",
        proposal_id
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    // Verify updated_at changed
    assert!(
        updated_proposal.updated_at > initial_proposal.updated_at,
        "updated_at should be newer after status update"
    );
}
