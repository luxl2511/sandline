use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Route {
    pub id: Uuid,
    pub name: String,
    pub owner_id: Uuid,
    pub control_points: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateRoute {
    pub name: String,
    pub geometry: serde_json::Value,
    pub control_points: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoute {
    pub geometry: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRouteControlPoints {
    pub control_points: serde_json::Value,
    pub feature_index: i32,
    pub point_index: i32,
}

#[derive(Debug, Serialize)]
pub struct RouteWithGeometry {
    #[serde(flatten)]
    pub route: Route,
    pub geometry: serde_json::Value,
    pub control_points: Option<serde_json::Value>,
}
