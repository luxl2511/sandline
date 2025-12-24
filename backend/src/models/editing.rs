use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EditingSession {
    pub id: Uuid,
    pub route_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub user_avatar_url: Option<String>,
    pub started_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateEditingSession {
    pub user_email: String,
    pub user_avatar_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EditingSessionResponse {
    pub session_id: Uuid,
    pub route_id: Uuid,
    pub user_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub active_sessions: Vec<EditingSessionInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditingSessionInfo {
    pub user_id: Uuid,
    pub user_email: String,
    pub user_avatar_url: Option<String>,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PointChange {
    pub id: Uuid,
    pub route_id: Uuid,
    pub user_id: Uuid,
    pub user_email: String,
    pub feature_index: i32,
    pub point_index: i32,
    pub original_position: serde_json::Value,
    pub new_position: serde_json::Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePointChange {
    pub feature_index: i32,
    pub point_index: i32,
    pub original_position: serde_json::Value,
    pub new_position: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePointChangeStatus {
    pub status: String,
}
