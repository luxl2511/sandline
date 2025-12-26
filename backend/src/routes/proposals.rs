use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::RlsTransaction;
use crate::middleware::AuthUser;
use crate::models::{CreateProposal, RouteProposal, UpdateProposalStatus};
use crate::AppState;

/// List all proposals for a route (public endpoint)
///
/// Returns all proposals for the specified route.
/// Currently public - all users can see all proposals.
pub async fn list_proposals(
    State(state): State<AppState>,
    Path(route_id): Path<Uuid>,
) -> Result<Json<Vec<RouteProposal>>, StatusCode> {
    let proposals = sqlx::query_as!(
        RouteProposal,
        "SELECT * FROM route_proposals WHERE route_id = $1 ORDER BY created_at DESC",
        route_id
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch proposals: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(proposals))
}

/// Create a new proposal (requires authentication)
///
/// Creates a proposal for a route owned by the authenticated user.
/// RLS policy enforces that `created_by = auth.uid()`.
pub async fn create_proposal(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateProposal>,
) -> Result<Json<RouteProposal>, StatusCode> {
    // Parse creator ID from authenticated user
    let created_by = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Begin RLS transaction - sets auth.uid() to the authenticated user
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create proposal - RLS policy "Users create proposals" checks: created_by = auth.uid()
    let proposal = sqlx::query_as!(
        RouteProposal,
        r#"
        INSERT INTO route_proposals (route_id, geometry, comment, created_by)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#,
        payload.route_id,
        payload.geometry,
        payload.comment,
        created_by
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create proposal: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(proposal))
}

/// Update a proposal's status (requires authentication and route ownership)
///
/// Updates the status of a proposal (e.g., accept or reject).
/// RLS policy enforces that only the route owner can update proposals.
///
/// **IMPORTANT**: This handler relies entirely on RLS for authorization.
/// No manual ownership checks are performed - RLS policy ensures that
/// only the route owner can update proposals.
pub async fn update_proposal_status(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProposalStatus>,
) -> Result<Json<RouteProposal>, StatusCode> {
    // Begin RLS transaction - sets auth.uid() to the authenticated user
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Update proposal status
    // RLS policy "Owners update proposals" checks:
    // EXISTS (SELECT 1 FROM routes WHERE id = route_id AND owner_id = auth.uid())
    //
    // If user doesn't own the route, UPDATE will return no rows (RLS filters them out)
    let proposal = sqlx::query_as!(
        RouteProposal,
        "UPDATE route_proposals SET status = $1 WHERE id = $2 RETURNING *",
        payload.status,
        id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!(
                "Proposal {} not found or user {} not authorized to update it (RLS)",
                id,
                auth_user.id
            );
            StatusCode::FORBIDDEN
        }
        _ => {
            tracing::error!("Failed to update proposal: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(proposal))
}
