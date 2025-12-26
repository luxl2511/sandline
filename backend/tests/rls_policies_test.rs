/// Integration tests for Row Level Security (RLS) policies
///
/// These tests verify that RLS policies are properly enforced at the database level.
/// They use the RlsTransaction wrapper to set auth context and test access control.
///
/// Prerequisites:
/// - Test users must exist in production Supabase
/// - .env.test must be configured
/// - Run tests with: cargo test --test rls_policies_test
mod common;

use common::{cleanup_test_data, create_test_pool, TestUser};
use dakar_planner_backend::db::RlsTransaction;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn test_rls_user_can_insert_own_route() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user = TestUser::test_user_1().await;

    // Create an RLS transaction with user auth context
    let auth_user = user.to_auth_user();
    let mut tx = RlsTransaction::begin(&pool, &auth_user)
        .await
        .expect("Failed to begin RLS transaction");

    // Insert a route as the authenticated user
    let route_id = Uuid::new_v4();
    let result = sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        RETURNING id
        "#,
    )
    .bind(&route_id)
    .bind(&user.id)
    .bind("RLS Test Route")
    .bind(serde_json::json!([
        {"lng": 2.5, "lat": 35.2},
        {"lng": 2.6, "lat": 35.3}
    ]))
    .fetch_one(&mut **tx)
    .await;

    assert!(
        result.is_ok(),
        "User should be able to insert their own route"
    );

    // Rollback to clean up
    tx.rollback().await.expect("Rollback failed");
}

#[tokio::test]
async fn test_rls_user_cannot_insert_route_for_other_user() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user1 = TestUser::test_user_1().await;
    let user2 = TestUser::test_user_2().await;

    // User 1 tries to insert a route claiming to be owned by User 2
    let auth_user1 = user1.to_auth_user();
    let mut tx = RlsTransaction::begin(&pool, &auth_user1)
        .await
        .expect("Failed to begin RLS transaction");

    let route_id = Uuid::new_v4();
    let result = sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        "#,
    )
    .bind(&route_id)
    .bind(&user2.id) // Trying to insert as different user!
    .bind("Malicious Route")
    .bind(serde_json::json!([
        {"lng": 2.5, "lat": 35.2}
    ]))
    .execute(&mut **tx)
    .await;

    assert!(
        result.is_err(),
        "User should NOT be able to insert route for another user (RLS policy violation)"
    );

    tx.rollback().await.ok();
}

#[tokio::test]
async fn test_rls_user_can_update_own_route() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user = TestUser::test_user_1().await;

    // First, create a route outside RLS context (for setup)
    let route_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        "#,
    )
    .bind(&route_id)
    .bind(&user.id)
    .bind("Original Name")
    .bind(serde_json::json!([{"lng": 2.5, "lat": 35.2}]))
    .execute(&pool)
    .await
    .expect("Failed to insert test route");

    // Now try to update it within RLS context
    let auth_user = user.to_auth_user();
    let mut tx = RlsTransaction::begin(&pool, &auth_user)
        .await
        .expect("Failed to begin RLS transaction");

    let result = sqlx::query(
        r#"
        UPDATE routes
        SET name = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind("Updated Name")
    .bind(&route_id)
    .execute(&mut **tx)
    .await;

    assert!(
        result.is_ok(),
        "User should be able to update their own route"
    );

    let rows_affected = result.unwrap().rows_affected();
    assert_eq!(rows_affected, 1, "Exactly one route should be updated");

    tx.commit().await.expect("Commit failed");

    // Cleanup
    cleanup_test_data(&pool, user.id).await;
}

#[tokio::test]
async fn test_rls_user_cannot_update_other_users_route() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user1 = TestUser::test_user_1().await;
    let user2 = TestUser::test_user_2().await;

    // User 1 creates a route
    let route_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        "#,
    )
    .bind(&route_id)
    .bind(&user1.id)
    .bind("User 1 Route")
    .bind(serde_json::json!([{"lng": 2.5, "lat": 35.2}]))
    .execute(&pool)
    .await
    .expect("Failed to insert test route");

    // User 2 tries to update User 1's route (should fail)
    let auth_user2 = user2.to_auth_user();
    let mut tx = RlsTransaction::begin(&pool, &auth_user2)
        .await
        .expect("Failed to begin RLS transaction");

    let result = sqlx::query(
        r#"
        UPDATE routes
        SET name = $1
        WHERE id = $2
        "#,
    )
    .bind("Hacked Name")
    .bind(&route_id)
    .execute(&mut **tx)
    .await;

    // Result should succeed (no error) but affect 0 rows due to RLS filtering
    assert!(result.is_ok(), "Query should execute without error");
    let rows_affected = result.unwrap().rows_affected();
    assert_eq!(
        rows_affected, 0,
        "RLS should prevent updating other user's route (0 rows affected)"
    );

    tx.rollback().await.ok();

    // Cleanup
    cleanup_test_data(&pool, user1.id).await;
}

#[tokio::test]
async fn test_rls_all_users_can_read_all_routes() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user1 = TestUser::test_user_1().await;
    let user2 = TestUser::test_user_2().await;

    // User 1 creates a route
    let route_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        "#,
    )
    .bind(&route_id)
    .bind(&user1.id)
    .bind("Publicly Visible Route")
    .bind(serde_json::json!([{"lng": 2.5, "lat": 35.2}]))
    .execute(&pool)
    .await
    .expect("Failed to insert test route");

    // User 2 should be able to read User 1's route
    let auth_user2 = user2.to_auth_user();
    let mut tx = RlsTransaction::begin(&pool, &auth_user2)
        .await
        .expect("Failed to begin RLS transaction");

    let result = sqlx::query(
        r#"
        SELECT id, name, owner_id
        FROM routes
        WHERE id = $1
        "#,
    )
    .bind(&route_id)
    .fetch_one(&mut **tx)
    .await;

    assert!(
        result.is_ok(),
        "All users should be able to read all routes"
    );

    let row = result.unwrap();
    let fetched_id: Uuid = row.get("id");
    let fetched_name: String = row.get("name");

    assert_eq!(fetched_id, route_id);
    assert_eq!(fetched_name, "Publicly Visible Route");

    tx.rollback().await.ok();

    // Cleanup
    cleanup_test_data(&pool, user1.id).await;
}

#[tokio::test]
async fn test_rls_proposal_update_owner_only() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;
    let user1 = TestUser::test_user_1().await; // Route owner
    let user2 = TestUser::test_user_2().await; // Proposer

    // User 1 creates a route
    let route_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO routes (id, owner_id, name, control_points, created_at, updated_at)
        VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
        "#,
    )
    .bind(&route_id)
    .bind(&user1.id)
    .bind("Route with Proposals")
    .bind(serde_json::json!([{"lng": 2.5, "lat": 35.2}]))
    .execute(&pool)
    .await
    .expect("Failed to insert test route");

    // User 2 creates a proposal (point change)
    let proposal_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO route_point_changes (id, route_id, user_id, user_email, feature_index, point_index, original_position, new_position, status, created_at)
        VALUES ($1, $2, $3, $4, 0, 0, $5::jsonb, $6::jsonb, 'pending', NOW())
        "#,
    )
    .bind(&proposal_id)
    .bind(&route_id)
    .bind(&user2.id)
    .bind(&user2.email)
    .bind(serde_json::json!({"lng": 2.5, "lat": 35.2}))
    .bind(serde_json::json!({"lng": 2.6, "lat": 35.3}))
    .execute(&pool)
    .await
    .expect("Failed to insert proposal");

    // User 2 (proposer, non-owner) tries to accept their own proposal (should fail)
    let auth_user2 = user2.to_auth_user();
    let mut tx2 = RlsTransaction::begin(&pool, &auth_user2)
        .await
        .expect("Failed to begin RLS transaction");

    let result = sqlx::query(
        r#"
        UPDATE route_point_changes
        SET status = 'accepted'
        WHERE id = $1
        "#,
    )
    .bind(&proposal_id)
    .execute(&mut **tx2)
    .await;

    assert!(result.is_ok(), "Query should execute");
    let rows_affected = result.unwrap().rows_affected();
    assert_eq!(
        rows_affected, 0,
        "Non-owner should NOT be able to update proposal status (RLS)"
    );

    tx2.rollback().await.ok();

    // User 1 (route owner) accepts the proposal (should succeed)
    let auth_user1 = user1.to_auth_user();
    let mut tx1 = RlsTransaction::begin(&pool, &auth_user1)
        .await
        .expect("Failed to begin RLS transaction");

    let result = sqlx::query(
        r#"
        UPDATE route_point_changes
        SET status = 'accepted', updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(&proposal_id)
    .execute(&mut **tx1)
    .await;

    assert!(
        result.is_ok(),
        "Route owner should be able to update proposal status"
    );
    let rows_affected = result.unwrap().rows_affected();
    assert_eq!(rows_affected, 1, "Exactly one proposal should be updated");

    tx1.commit().await.expect("Commit failed");

    // Cleanup
    cleanup_test_data(&pool, user1.id).await;
    common::cleanup_proposals_by_user(&pool, user2.id).await;
}

#[tokio::test]
async fn test_rls_transaction_validation_rejects_invalid_uuid() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;

    // Create an AuthUser with invalid UUID
    use dakar_planner_backend::middleware::auth::{AuthUser, Claims};

    let claims = Claims {
        sub: "not-a-uuid".to_string(),
        aud: "authenticated".to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as i64,
        role: "authenticated".to_string(),
        email: Some("test@example.com".to_string()),
    };

    let auth_user = AuthUser {
        id: "not-a-uuid".to_string(),
        role: "authenticated".to_string(),
        full_claims: claims,
    };

    let result = RlsTransaction::begin(&pool, &auth_user).await;

    assert!(result.is_err(), "RlsTransaction should reject invalid UUID");
}

#[tokio::test]
async fn test_rls_transaction_validation_rejects_invalid_role() {
    dotenvy::from_filename(".env.test").ok();
    let pool = create_test_pool().await;

    // Create an AuthUser with invalid role
    use dakar_planner_backend::middleware::auth::{AuthUser, Claims};

    let user_id = Uuid::new_v4();
    let claims = Claims {
        sub: user_id.to_string(),
        aud: "authenticated".to_string(),
        exp: (chrono::Utc::now().timestamp() + 3600) as i64,
        role: "hacker".to_string(),
        email: Some("hacker@evil.com".to_string()),
    };

    let auth_user = AuthUser {
        id: user_id.to_string(),
        role: "hacker".to_string(),
        full_claims: claims,
    };

    let result = RlsTransaction::begin(&pool, &auth_user).await;

    assert!(result.is_err(), "RlsTransaction should reject invalid role");
}
