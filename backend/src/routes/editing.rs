use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

use crate::db::RlsTransaction;
use crate::middleware::AuthUser;
use crate::models::{
    CreateEditingSession, CreatePointChange, EditingSession, EditingSessionInfo,
    EditingSessionResponse, PointChange, UpdatePointChangeStatus,
};
use crate::AppState;

// ============================================
// Editing Session Endpoints
// ============================================

/// Join an editing session for a route (requires authentication)
///
/// Creates or updates an editing session for the authenticated user on the specified route.
/// Returns information about all active editing sessions for the route.
pub async fn join_editing_session(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
    Json(payload): Json<CreateEditingSession>,
) -> Result<Json<EditingSessionResponse>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Upsert editing session (handles re-joins)
    // RLS policy "Users manage own session" checks: user_id = auth.uid()
    let session = sqlx::query_as!(
        EditingSession,
        r#"
        INSERT INTO route_editing_sessions (route_id, user_id, user_email, user_avatar_url)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (route_id, user_id)
        DO UPDATE SET last_heartbeat = NOW()
        RETURNING *
        "#,
        route_id,
        user_id,
        payload.user_email,
        payload.user_avatar_url
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create editing session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get all active sessions for this route (excluding current user)
    // RLS policy allows seeing sessions for owned or joined routes
    let active_sessions = sqlx::query_as!(
        EditingSessionInfo,
        r#"
        SELECT user_id, user_email, user_avatar_url, started_at
        FROM route_editing_sessions
        WHERE route_id = $1 AND user_id != $2
        ORDER BY started_at
        "#,
        route_id,
        user_id
    )
    .fetch_all(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch active sessions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(EditingSessionResponse {
        session_id: session.id,
        route_id: session.route_id,
        user_id: session.user_id,
        started_at: session.started_at,
        active_sessions,
    }))
}

/// Leave an editing session (requires authentication)
///
/// Deletes the authenticated user's editing session for the specified route.
pub async fn leave_editing_session(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Delete editing session
    // RLS policy "Users manage own session" ensures only own sessions can be deleted
    sqlx::query!(
        "DELETE FROM route_editing_sessions WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete editing session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

/// Send heartbeat for editing session (requires authentication)
///
/// Updates the last_heartbeat timestamp for the authenticated user's editing session.
/// Used to keep the session active and show presence to other editors.
pub async fn heartbeat_editing_session(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update heartbeat
    // RLS policy "Users manage own session" ensures only own sessions can be updated
    sqlx::query!(
        "UPDATE route_editing_sessions SET last_heartbeat = NOW() WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update heartbeat: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}

// ============================================
// Point Change Endpoints
// ============================================

/// Create a point change suggestion (requires authentication)
///
/// Creates a suggestion to move a specific point in a route's geometry.
/// User must be in an active editing session for the route.
pub async fn create_point_change(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
    Json(payload): Json<CreatePointChange>,
) -> Result<Json<PointChange>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Get user email from editing session
    let session = sqlx::query!(
        "SELECT user_email FROM route_editing_sessions WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!(
                "User {} not in editing session for route {}",
                user_id,
                route_id
            );
            StatusCode::FORBIDDEN
        }
        _ => {
            tracing::error!("Failed to fetch user session: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Create point change
    // RLS policy "Users create point changes" checks: user_id = auth.uid()
    let point_change = sqlx::query_as!(
        PointChange,
        r#"
        INSERT INTO route_point_changes (
            route_id, user_id, user_email, feature_index, point_index,
            original_position, new_position
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
        route_id,
        user_id,
        session.user_email,
        payload.feature_index,
        payload.point_index,
        payload.original_position,
        payload.new_position
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create point change: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(point_change))
}

#[derive(Deserialize)]
pub struct ListPointChangesQuery {
    #[serde(default = "default_status")]
    status: String,
}

fn default_status() -> String {
    "pending".to_string()
}

/// List point changes for a route (public endpoint)
///
/// Returns all point changes for the specified route with the given status.
pub async fn list_point_changes(
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
    Query(query): Query<ListPointChangesQuery>,
) -> Result<Json<Vec<PointChange>>, StatusCode> {
    let changes = sqlx::query_as!(
        PointChange,
        r#"
        SELECT *
        FROM route_point_changes
        WHERE route_id = $1 AND status = $2
        ORDER BY created_at DESC
        "#,
        route_id,
        query.status
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch point changes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(changes))
}

/// Update point change status (requires authentication and route ownership)
///
/// Accepts or rejects a point change suggestion.
/// If accepted, applies the change to the route geometry by creating a new version.
///
/// **IMPORTANT**: This handler relies entirely on RLS for authorization.
/// No manual ownership checks are performed - RLS policy ensures that
/// only the route owner can update point changes.
pub async fn update_point_change_status(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(change_id): Path<Uuid>,
    Json(payload): Json<UpdatePointChangeStatus>,
) -> Result<Json<PointChange>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Fetch the change
    // RLS policy "Users read point changes" allows seeing own changes or changes for owned routes
    let change = sqlx::query_as!(
        PointChange,
        "SELECT * FROM route_point_changes WHERE id = $1",
        change_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!(
                "Point change {} not found or not accessible by user {}",
                change_id,
                user_id
            );
            StatusCode::NOT_FOUND
        }
        _ => {
            tracing::error!("Failed to fetch point change: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // If accepting the change, apply it to route geometry
    if payload.status == "accepted" {
        // Fetch current geometry
        let current_version = sqlx::query!(
            "SELECT geometry FROM route_versions WHERE route_id = $1 ORDER BY created_at DESC LIMIT 1",
            change.route_id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch current route version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Parse and modify geometry
        let mut geometry: serde_json::Value = current_version.geometry.clone();

        // Apply the point change
        let coords = geometry["coordinates"]
            .get_mut(change.feature_index as usize)
            .and_then(|f: &mut Value| f.get_mut(change.point_index as usize))
            .ok_or_else(|| {
                tracing::error!("Invalid feature/point index in change");
                StatusCode::BAD_REQUEST
            })?;

        *coords = change.new_position.clone();

        // Create new route version
        // RLS policy "Owner create route versions" ensures only owner can create versions
        sqlx::query!(
            "INSERT INTO route_versions (route_id, geometry, created_by) VALUES ($1, $2, $3)",
            change.route_id,
            geometry,
            user_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            // Check if error is due to RLS policy violation
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("permission denied")
                    || db_err.message().contains("violates row-level security policy")
                {
                    tracing::warn!(
                        "User {} attempted to accept point change for route {} (permission denied by RLS)",
                        user_id,
                        change.route_id
                    );
                    return StatusCode::FORBIDDEN;
                }
            }

            tracing::error!("Failed to create route version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Update routes.updated_at to trigger real-time notifications
        // RLS policy ensures only owner can update
        sqlx::query!(
            "UPDATE routes SET updated_at = NOW() WHERE id = $1",
            change.route_id
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update route timestamp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Update control_points to match the new geometry
        // Fetch current control points
        let route = sqlx::query!(
            "SELECT control_points FROM routes WHERE id = $1",
            change.route_id
        )
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch route for control points update: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Update control points with new position
        let mut control_points_value = route.control_points;
        if let Some(control_points) = control_points_value.as_array_mut() {
            if let Some(point) = control_points.get_mut(change.point_index as usize) {
                // Update the point with new position
                // Convert [lng, lat] array to {lng, lat} object format
                if let Some(coords) = change.new_position.as_array() {
                    if coords.len() == 2 {
                        *point = serde_json::json!({
                            "lng": coords[0],
                            "lat": coords[1]
                        });
                    }
                }
            }

            // Save updated control_points back to database
            sqlx::query("UPDATE routes SET control_points = $1 WHERE id = $2")
                .bind(&control_points_value)
                .bind(change.route_id)
                .execute(&mut **tx)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to update control points: {}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
        }
    }

    // Update change status
    // RLS policy "Owners update point changes" ensures only route owner can update
    let updated_change = sqlx::query_as!(
        PointChange,
        r#"
        UPDATE route_point_changes
        SET status = $1, resolved_at = NOW(), resolved_by = $2
        WHERE id = $3
        RETURNING *
        "#,
        payload.status,
        user_id,
        change_id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!(
                "Point change {} not found or user {} not authorized to update it (RLS)",
                change_id,
                user_id
            );
            StatusCode::FORBIDDEN
        }
        _ => {
            tracing::error!("Failed to update point change status: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated_change))
}
