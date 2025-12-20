pub mod tracks;
pub mod routes;
pub mod proposals;

use axum::{Router, routing::{get, post, put, patch}};
use crate::db::DbPool;

pub fn api_routes() -> Router<DbPool> {
    Router::new()
        // Tracks
        .route("/tracks", get(tracks::list_tracks))
        .route("/tracks/:id", get(tracks::get_track))
        // Routes
        .route("/routes", get(routes::list_routes).post(routes::create_route))
        .route("/routes/:id", get(routes::get_route).put(routes::update_route))
        .route("/routes/:id/proposals", get(proposals::list_proposals))
        // Proposals
        .route("/proposals", post(proposals::create_proposal))
        .route("/proposals/:id", patch(proposals::update_proposal_status))
}
