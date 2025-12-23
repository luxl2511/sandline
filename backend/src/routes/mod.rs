pub mod proposals;
pub mod route_handlers;
pub mod tracks;

use crate::db::DbPool;
use axum::{
    routing::{get, patch, post},
    Router,
};

pub fn api_routes() -> Router<DbPool> {
    Router::new()
        // Tracks
        .route("/tracks", get(tracks::list_tracks))
        .route("/tracks/:id", get(tracks::get_track))
        // Routes
        .route(
            "/routes",
            get(route_handlers::list_routes).post(route_handlers::create_route),
        )
        .route(
            "/routes/:id",
            get(route_handlers::get_route).put(route_handlers::update_route),
        )
        .route("/routes/:id/proposals", get(proposals::list_proposals))
        // Proposals
        .route("/proposals", post(proposals::create_proposal))
        .route("/proposals/:id", patch(proposals::update_proposal_status))
}
