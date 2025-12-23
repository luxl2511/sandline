use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::DbPool;
use crate::models::{CuratedTrack, TrackQuery};

pub async fn list_tracks(
    State(pool): State<DbPool>,
    Query(query): Query<TrackQuery>,
) -> Result<Json<Vec<CuratedTrack>>, StatusCode> {
    let mut sql = String::from(
        "SELECT id, ST_AsGeoJSON(geometry)::jsonb as geometry, source, surface, confidence, last_verified, region
         FROM curated_tracks
         WHERE 1=1",
    );

    if query.source.is_some() {
        sql.push_str(" AND source = $1");
    }
    if query.min_confidence.is_some() {
        sql.push_str(" AND confidence >= $2");
    }
    if query.region.is_some() {
        sql.push_str(" AND region = $3");
    }

    sql.push_str(" LIMIT 1000");

    let tracks = sqlx::query_as::<_, CuratedTrack>(&sql)
        .persistent(false)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch tracks: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Json(tracks))
}

pub async fn get_track(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<CuratedTrack>, StatusCode> {
    let track = sqlx::query_as::<_, CuratedTrack>(
        "SELECT id, ST_AsGeoJSON(geometry)::jsonb as geometry, source, surface, confidence, last_verified, region
         FROM curated_tracks
         WHERE id = $1",
    )
    .bind(id)
    .persistent(false)
    .fetch_one(&pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Failed to fetch track: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(track))
}
