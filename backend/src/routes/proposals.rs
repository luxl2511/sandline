use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{CreateProposal, UpdateProposalStatus, RouteProposal};

pub async fn list_proposals(
    State(pool): State<DbPool>,
    Path(route_id): Path<Uuid>,
) -> Result<Json<Vec<RouteProposal>>, StatusCode> {
    let proposals = sqlx::query_as!(
        RouteProposal,
        "SELECT * FROM route_proposals WHERE route_id = $1 ORDER BY created_at DESC",
        route_id
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch proposals: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(proposals))
}

pub async fn create_proposal(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateProposal>,
) -> Result<Json<RouteProposal>, StatusCode> {
    // TODO: Get actual user ID from auth
    let created_by = Uuid::new_v4();

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
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create proposal: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(proposal))
}

pub async fn update_proposal_status(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateProposalStatus>,
) -> Result<Json<RouteProposal>, StatusCode> {
    let proposal = sqlx::query_as!(
        RouteProposal,
        "UPDATE route_proposals SET status = $1 WHERE id = $2 RETURNING *",
        payload.status,
        id
    )
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Failed to update proposal: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(proposal))
}
