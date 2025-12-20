use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CuratedTrack {
    pub id: Uuid,
    #[sqlx(json)]
    pub geometry: serde_json::Value,
    pub source: String,
    pub surface: Option<String>,
    pub confidence: i32,
    pub last_verified: Option<NaiveDate>,
    pub region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrack {
    pub geometry: serde_json::Value,
    pub source: String,
    pub surface: Option<String>,
    pub confidence: i32,
    pub region: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TrackQuery {
    pub bbox: Option<String>, // "minLon,minLat,maxLon,maxLat"
    pub source: Option<String>,
    pub min_confidence: Option<i32>,
    pub region: Option<String>,
}
