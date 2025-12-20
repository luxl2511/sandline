use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RouteProposal {
    pub id: Uuid,
    pub route_id: Uuid,
    #[sqlx(json)]
    pub geometry: serde_json::Value,
    pub comment: String,
    pub status: String,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateProposal {
    pub route_id: Uuid,
    pub geometry: serde_json::Value,
    pub comment: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProposalStatus {
    pub status: String, // "accepted" | "rejected"
}
