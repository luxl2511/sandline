use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
    Router,
};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use sqlx::PgPool;
use tower::ServiceExt;

/// Build a test application with the given database pool
pub fn build_test_app(pool: PgPool) -> Router {
    // Import the api_routes function from the routes module
    // This mirrors the main.rs setup but without CORS for simpler testing
    Router::new()
        .nest("/api", dakar_planner_backend::routes::api_routes())
        .with_state(pool)
}

/// Helper to make a GET request to the test app
pub async fn make_get_request(app: &Router, uri: &str) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    (status, body.to_vec())
}

/// Helper to make a POST request to the test app
pub async fn make_post_request(
    app: &Router,
    uri: &str,
    json_body: serde_json::Value,
) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    (status, body.to_vec())
}

/// Helper to make a PUT request to the test app
pub async fn make_put_request(
    app: &Router,
    uri: &str,
    json_body: serde_json::Value,
) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method(Method::PUT)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    (status, body.to_vec())
}

/// Helper to make a PATCH request to the test app
pub async fn make_patch_request(
    app: &Router,
    uri: &str,
    json_body: serde_json::Value,
) -> (StatusCode, Vec<u8>) {
    let request = Request::builder()
        .method(Method::PATCH)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_string()))
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();

    (status, body.to_vec())
}

/// Parse JSON response body into a typed struct
pub fn parse_json_response<T: DeserializeOwned>(body: &[u8]) -> T {
    serde_json::from_slice(body).expect("Failed to parse JSON response")
}

/// Assert that a JSON response matches an expected value
pub fn assert_json_eq<T: DeserializeOwned + std::fmt::Debug + PartialEq>(body: &[u8], expected: T) {
    let actual: T = parse_json_response(body);
    assert_eq!(actual, expected);
}

/// Assert that response status is OK (200)
pub fn assert_status_ok(status: StatusCode) {
    assert_eq!(status, StatusCode::OK, "Expected 200 OK, got {}", status);
}

/// Assert that response status is Created (201)
pub fn assert_status_created(status: StatusCode) {
    assert_eq!(
        status,
        StatusCode::CREATED,
        "Expected 201 Created, got {}",
        status
    );
}

/// Assert that response status is Not Found (404)
pub fn assert_status_not_found(status: StatusCode) {
    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "Expected 404 Not Found, got {}",
        status
    );
}

/// Assert that response status is Internal Server Error (500)
pub fn assert_status_internal_server_error(status: StatusCode) {
    assert_eq!(
        status,
        StatusCode::INTERNAL_SERVER_ERROR,
        "Expected 500 Internal Server Error, got {}",
        status
    );
}

/// Assert that a GeoJSON geometry has the correct structure
pub fn assert_valid_geojson_geometry(geometry: &serde_json::Value) {
    assert!(geometry.is_object(), "Geometry must be an object");
    assert!(
        geometry.get("type").is_some(),
        "Geometry must have a 'type' field"
    );
    assert!(
        geometry.get("coordinates").is_some(),
        "Geometry must have a 'coordinates' field"
    );

    let geometry_type = geometry["type"].as_str().unwrap();
    assert_eq!(
        geometry_type, "LineString",
        "Expected LineString geometry type"
    );

    let coords = geometry["coordinates"].as_array().unwrap();
    assert!(!coords.is_empty(), "Coordinates array must not be empty");
}
