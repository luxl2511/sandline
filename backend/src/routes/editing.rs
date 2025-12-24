use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::db::DbPool;
use crate::middleware::AuthUser;
use crate::models::{
    CreateEditingSession, CreatePointChange, EditingSession, EditingSessionInfo,
    EditingSessionResponse, PointChange, UpdatePointChangeStatus,
};

// ============================================
// Editing Session Endpoints
// ============================================

pub async fn join_editing_session(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(route_id): Path<Uuid>,
    Json(payload): Json<CreateEditingSession>,
) -> Result<Json<EditingSessionResponse>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Upsert editing session (handles re-joins)
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
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create editing session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get all active sessions for this route
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
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch active sessions: {}", e);
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

pub async fn leave_editing_session(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(route_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "DELETE FROM route_editing_sessions WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete editing session: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn heartbeat_editing_session(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(route_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "UPDATE route_editing_sessions SET last_heartbeat = NOW() WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update heartbeat: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(StatusCode::OK)
}

// ============================================
// Point Change Endpoints
// ============================================

pub async fn create_point_change(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(route_id): Path<Uuid>,
    Json(payload): Json<CreatePointChange>,
) -> Result<Json<PointChange>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Get user email from editing session
    let session = sqlx::query!(
        "SELECT user_email FROM route_editing_sessions WHERE route_id = $1 AND user_id = $2",
        route_id,
        user_id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!("User not in editing session for route {}", route_id);
            StatusCode::FORBIDDEN
        }
        _ => {
            tracing::error!("Failed to fetch user session: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

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
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create point change: {}", e);
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

pub async fn list_point_changes(
    State(pool): State<DbPool>,
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
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch point changes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(changes))
}

pub async fn update_point_change_status(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(change_id): Path<Uuid>,
    Json(payload): Json<UpdatePointChangeStatus>,
) -> Result<Json<PointChange>, StatusCode> {
    let user_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Start transaction for accept flow
    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fetch the change
    let change = sqlx::query_as!(
        PointChange,
        "SELECT * FROM route_point_changes WHERE id = $1",
        change_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Failed to fetch point change: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Verify route ownership
    let route = sqlx::query!("SELECT owner_id FROM routes WHERE id = $1", change.route_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch route: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if route.owner_id != user_id {
        tracing::warn!(
            "User {} attempted to update point change for route {} owned by {}",
            user_id,
            change.route_id,
            route.owner_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // If accepting the change, apply it to route geometry
    if payload.status == "accepted" {
        // Fetch current geometry
        let current_version = sqlx::query!(
            "SELECT geometry FROM route_versions WHERE route_id = $1 ORDER BY created_at DESC LIMIT 1",
            change.route_id
        )
        .fetch_one(&mut *tx)
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
            .and_then(|f| f.get_mut(change.point_index as usize))
            .ok_or_else(|| {
                tracing::error!("Invalid feature/point index in change");
                StatusCode::BAD_REQUEST
            })?;

        *coords = change.new_position.clone();

        // Create new route version
        sqlx::query!(
            "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
            change.route_id,
            geometry
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create route version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Update routes.updated_at to trigger real-time notifications
        sqlx::query!(
            "UPDATE routes SET updated_at = NOW() WHERE id = $1",
            change.route_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to update route timestamp: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // Update change status
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
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update point change status: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(updated_change))
}
