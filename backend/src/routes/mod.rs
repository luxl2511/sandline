pub mod editing;
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
        // Collaborative Editing
        .route(
            "/routes/:id/editing-session",
            post(editing::join_editing_session).delete(editing::leave_editing_session),
        )
        .route(
            "/routes/:id/editing-session/heartbeat",
            post(editing::heartbeat_editing_session),
        )
        .route(
            "/routes/:id/point-changes",
            post(editing::create_point_change).get(editing::list_point_changes),
        )
        .route(
            "/point-changes/:id",
            patch(editing::update_point_change_status),
        )
}
