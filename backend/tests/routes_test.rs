/// Integration tests for route CRUD operations
///
/// These tests use the PRODUCTION database with test users.
/// Each test cleans up its data after execution.
///
/// Prerequisites:
/// - Test users must exist in production Supabase:
///   - test-user-1@example.com
///   - test-user-2@example.com
/// - .env.test must be configured with production credentials
/// - Run tests with: cargo test --test routes_test
mod common;

use axum::http::StatusCode;
use axum::Router;
use common::{
    cleanup_test_data, create_test_app_state, send_authed_request, send_request, TestUser,
};
use dakar_planner_backend::routes::api_routes;

/// Helper to create test app with routes
async fn create_test_app() -> Router {
    let state = create_test_app_state().await;
    api_routes().with_state(state)
}

#[tokio::test]
async fn test_list_routes_public() {
    let app = create_test_app().await;

    // List routes without authentication (should work - routes are public)
    let (status, body) = send_request(app, "GET", "/routes", None).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Listing routes should work without auth. Body: {}",
        body
    );

    // Should return valid JSON array
    let routes: serde_json::Value =
        serde_json::from_str(&body).expect("Response should be valid JSON");
    assert!(routes.is_array(), "Response should be an array");
}

#[tokio::test]
async fn test_create_route_requires_auth() {
    let app = create_test_app().await;

    let route_data = serde_json::json!({
        "name": "Unauthorized Test Route",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    // Try to create route without authentication
    let (status, body) = send_request(app, "POST", "/routes", Some(route_data.to_string())).await;

    assert_eq!(
        status,
        StatusCode::UNAUTHORIZED,
        "Creating route without auth should fail. Body: {}",
        body
    );
}

#[tokio::test]
async fn test_create_route_authenticated() {
    dotenvy::from_filename(".env.test").ok();
    let app = create_test_app().await;
    let user = TestUser::test_user_1().await;

    let route_data = serde_json::json!({
        "name": "Test Route - Create",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    // Create route with authentication
    let (status, body) = send_authed_request(
        app,
        "POST",
        "/routes",
        &user.jwt,
        Some(route_data.to_string()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Creating route with auth should succeed. Body: {}",
        body
    );

    // Verify response contains route data
    let response: serde_json::Value =
        serde_json::from_str(&body).expect("Response should be valid JSON");
    assert!(
        response.get("id").is_some(),
        "Response should contain route ID"
    );
    assert_eq!(
        response.get("name").and_then(|n| n.as_str()),
        Some("Test Route - Create"),
        "Route name should match"
    );

    // Cleanup
    let pool = common::create_test_pool().await;
    cleanup_test_data(&pool, user.id).await;
}

#[tokio::test]
async fn test_get_route_by_id() {
    dotenvy::from_filename(".env.test").ok();
    let app_create = create_test_app().await;
    let app_get = create_test_app().await;
    let user = TestUser::test_user_1().await;

    // First, create a route
    let route_data = serde_json::json!({
        "name": "Test Route - Get By ID",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    let (status, body) = send_authed_request(
        app_create,
        "POST",
        "/routes",
        &user.jwt,
        Some(route_data.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Route creation should succeed");

    let created_route: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    let route_id = created_route
        .get("id")
        .and_then(|id| id.as_str())
        .expect("Route should have ID");

    // Now get the route by ID (public endpoint, no auth required)
    let (status, body) = send_request(app_get, "GET", &format!("/routes/{}", route_id), None).await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Getting route by ID should succeed. Body: {}",
        body
    );

    let fetched_route: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    assert_eq!(
        fetched_route.get("id").and_then(|id| id.as_str()),
        Some(route_id),
        "Fetched route should have same ID"
    );

    // Cleanup
    let pool = common::create_test_pool().await;
    cleanup_test_data(&pool, user.id).await;
}

#[tokio::test]
async fn test_update_route_owner_only() {
    dotenvy::from_filename(".env.test").ok();
    let app_create = create_test_app().await;
    let app_update_owner = create_test_app().await;
    let app_update_other = create_test_app().await;

    let user1 = TestUser::test_user_1().await; // Route owner
    let user2 = TestUser::test_user_2().await; // Other user

    // User 1 creates a route
    let route_data = serde_json::json!({
        "name": "Test Route - Owner Only",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    let (status, body) = send_authed_request(
        app_create,
        "POST",
        "/routes",
        &user1.jwt,
        Some(route_data.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Route creation should succeed");

    let created_route: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    let route_id = created_route
        .get("id")
        .and_then(|id| id.as_str())
        .expect("Route should have ID");

    // User 2 tries to update the route geometry (should fail due to RLS)
    let update_data = serde_json::json!({
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.8, 35.5]]
        }
    });

    let (status, _body) = send_authed_request(
        app_update_other,
        "PUT",
        &format!("/routes/{}", route_id),
        &user2.jwt,
        Some(update_data.to_string()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "Non-owner should not be able to update route (RLS enforcement)"
    );

    // User 1 (owner) updates the route geometry successfully
    let update_data = serde_json::json!({
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.7, 35.4], [2.9, 35.6]]
        }
    });

    let (status, body) = send_authed_request(
        app_update_owner,
        "PUT",
        &format!("/routes/{}", route_id),
        &user1.jwt,
        Some(update_data.to_string()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Owner should be able to update route. Body: {}",
        body
    );

    // Cleanup
    let pool = common::create_test_pool().await;
    cleanup_test_data(&pool, user1.id).await;
}

#[tokio::test]
async fn test_update_control_points_owner_direct_update() {
    dotenvy::from_filename(".env.test").ok();
    let app_create = create_test_app().await;
    let app_update = create_test_app().await;
    let user = TestUser::test_user_1().await;

    // Create a route
    let route_data = serde_json::json!({
        "name": "Test Route - Control Points Owner",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    let (status, body) = send_authed_request(
        app_create,
        "POST",
        "/routes",
        &user.jwt,
        Some(route_data.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Route creation should succeed");

    let created_route: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    let route_id = created_route
        .get("id")
        .and_then(|id| id.as_str())
        .expect("Route should have ID");

    // Owner updates control points (direct update)
    let update_data = serde_json::json!({
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.7, "lat": 35.4}, // Modified point
            {"lng": 2.8, "lat": 35.5}  // New point
        ],
        "feature_index": 0,
        "point_index": 1
    });

    let (status, body) = send_authed_request(
        app_update,
        "PUT",
        &format!("/routes/{}/control-points", route_id),
        &user.jwt,
        Some(update_data.to_string()),
    )
    .await;

    assert_eq!(
        status,
        StatusCode::OK,
        "Owner should be able to update control points. Body: {}",
        body
    );

    // Verify response indicates direct update (not proposal)
    let response: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    // The response structure depends on your implementation
    // This is a placeholder assertion
    assert!(response.is_object(), "Response should be a valid object");

    // Cleanup
    let pool = common::create_test_pool().await;
    cleanup_test_data(&pool, user.id).await;
}

#[tokio::test]
async fn test_update_control_points_non_owner_creates_proposal() {
    dotenvy::from_filename(".env.test").ok();
    let app_create = create_test_app().await;
    let app_update = create_test_app().await;

    let user1 = TestUser::test_user_1().await; // Route owner
    let user2 = TestUser::test_user_2().await; // Other user

    // User 1 creates a route
    let route_data = serde_json::json!({
        "name": "Test Route - Control Points Non-Owner",
        "geometry": {
            "type": "LineString",
            "coordinates": [[2.5, 35.2], [2.6, 35.3]]
        },
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.6, "lat": 35.3}
        ]
    });

    let (status, body) = send_authed_request(
        app_create,
        "POST",
        "/routes",
        &user1.jwt,
        Some(route_data.to_string()),
    )
    .await;

    assert_eq!(status, StatusCode::OK, "Route creation should succeed");

    let created_route: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    let route_id = created_route
        .get("id")
        .and_then(|id| id.as_str())
        .expect("Route should have ID");

    // User 2 (non-owner) tries to update control points (should create proposal)
    let update_data = serde_json::json!({
        "control_points": [
            {"lng": 2.5, "lat": 35.2},
            {"lng": 2.7, "lat": 35.4}, // Modified point
        ],
        "feature_index": 0,
        "point_index": 1
    });

    let (status, body) = send_authed_request(
        app_update,
        "PUT",
        &format!("/routes/{}/control-points", route_id),
        &user2.jwt,
        Some(update_data.to_string()),
    )
    .await;

    // Status should indicate proposal created (could be 200 or 201 depending on implementation)
    assert!(
        status == StatusCode::OK || status == StatusCode::CREATED,
        "Non-owner control point update should create proposal. Status: {}, Body: {}",
        status,
        body
    );

    let response: serde_json::Value = serde_json::from_str(&body).expect("Valid JSON");
    // Verify it's a proposal, not a direct update
    // (Assertion depends on your API response structure)
    assert!(
        response.is_object(),
        "Response should be a valid object indicating proposal creation"
    );

    // Cleanup
    let pool = common::create_test_pool().await;
    cleanup_test_data(&pool, user1.id).await;
    common::cleanup_proposals_by_user(&pool, user2.id).await;
}

#[tokio::test]
async fn test_invalid_route_id_returns_404() {
    let app = create_test_app().await;

    let invalid_uuid = "00000000-0000-0000-0000-000000000000";

    let (status, _body) =
        send_request(app, "GET", &format!("/routes/{}", invalid_uuid), None).await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "Invalid route ID should return 404"
    );
}
