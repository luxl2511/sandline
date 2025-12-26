use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::RlsTransaction;
use crate::geometry::{route_geometry, simplify_geometry, RoutingConfig};
use crate::middleware::AuthUser;
use crate::models::{CreateRoute, Route, RouteWithGeometry, UpdateRoute, UpdateRouteControlPoints};
use crate::AppState;

/// Process geometry: route via hybrid approach (Mapbox + curated tracks) + simplify
///
/// # Steps
/// 1. Route geometry using hybrid approach (on-road vs off-road detection)
/// 2. Simplify routed geometry using Douglas-Peucker algorithm
/// 3. Return processed geometry with confidence score
async fn process_geometry(pool: &PgPool, raw_geometry: &Value) -> Result<Value, StatusCode> {
    let routing_config = RoutingConfig::default();

    // Step 1: Hybrid routing (Mapbox Directions + curated tracks)
    let (routed_geometry, confidence) = route_geometry(pool, raw_geometry, &routing_config)
        .await
        .map_err(|e| {
        tracing::error!("Geometry routing failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!(
        "Geometry routing complete - confidence: {:.2}%",
        confidence * 100.0
    );

    // Step 2: Geometric simplification (0.0001° ≈ 11m tolerance)
    let simplified_geometry = simplify_geometry(&routed_geometry, 0.0001).map_err(|e| {
        tracing::error!("Geometric simplification failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tracing::info!("Geometry simplification complete");

    Ok(simplified_geometry)
}

/// List all routes (public endpoint)
///
/// Returns routes with their latest geometry version.
/// Currently public - all users can see all routes.
pub async fn list_routes(
    State(state): State<AppState>,
) -> Result<Json<Vec<RouteWithGeometry>>, StatusCode> {
    let routes = sqlx::query!(
        r#"
        SELECT
            r.id, r.name, r.owner_id, r.control_points, r.created_at as "created_at!", r.updated_at as "updated_at!",
            rv.geometry
        FROM routes r
        LEFT JOIN LATERAL (
            SELECT geometry
            FROM route_versions
            WHERE route_id = r.id
            ORDER BY created_at DESC
            LIMIT 1
        ) rv ON true
        ORDER BY r.created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch routes: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let result = routes
        .into_iter()
        .map(|row| RouteWithGeometry {
            route: Route {
                id: row.id,
                name: row.name,
                owner_id: row.owner_id,
                control_points: row.control_points.clone(),
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            geometry: row.geometry,
            control_points: row.control_points,
        })
        .collect();

    Ok(Json(result))
}

/// Get a single route by ID (public endpoint)
///
/// Returns the route with its latest geometry version.
pub async fn get_route(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    let route = sqlx::query!(
        r#"
        SELECT
            r.id, r.name, r.owner_id, r.control_points, r.created_at as "created_at!", r.updated_at as "updated_at!",
            rv.geometry
        FROM routes r
        LEFT JOIN LATERAL (
            SELECT geometry
            FROM route_versions
            WHERE route_id = r.id
            ORDER BY created_at DESC
            LIMIT 1
        ) rv ON true
        WHERE r.id = $1
        "#,
        id
    )
    .fetch_one(&state.pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
        _ => {
            tracing::error!("Failed to fetch route: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    Ok(Json(RouteWithGeometry {
        route: Route {
            id: route.id,
            name: route.name,
            owner_id: route.owner_id,
            control_points: route.control_points.clone(),
            created_at: route.created_at,
            updated_at: route.updated_at,
        },
        geometry: route.geometry,
        control_points: route.control_points,
    }))
}

/// Create a new route (requires authentication)
///
/// Creates a route owned by the authenticated user.
/// RLS policy enforces that `owner_id = auth.uid()`.
pub async fn create_route(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(payload): Json<CreateRoute>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    // Parse owner ID from authenticated user
    let owner_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // STEP 1: Process geometry (route + simplify)
    tracing::info!("Processing geometry for new route '{}'", payload.name);
    let processed_geometry = process_geometry(&state.pool, &payload.geometry).await?;

    // Begin RLS transaction - sets auth.uid() to the authenticated user
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Create route - RLS policy "Users create routes" checks: owner_id = auth.uid()
    let route = sqlx::query_as!(
        Route,
        "INSERT INTO routes (name, owner_id, control_points) VALUES ($1, $2, $3) RETURNING *",
        payload.name,
        owner_id,
        payload.control_points
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create route: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // STEP 2: Store PROCESSED geometry (not raw payload)
    sqlx::query!(
        "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
        route.id,
        processed_geometry
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create route version: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(RouteWithGeometry {
        route,
        geometry: processed_geometry,
        control_points: Some(payload.control_points),
    }))
}

/// Update a route's geometry (requires authentication and ownership)
///
/// Creates a new version of the route's geometry.
/// RLS policy enforces that only the route owner can create new versions.
///
/// **IMPORTANT**: This handler relies entirely on RLS for authorization.
/// No manual ownership checks are performed - RLS policy ensures that
/// only the route owner can insert into route_versions.
pub async fn update_route(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRoute>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    // STEP 1: Process geometry (route + simplify)
    tracing::info!("Processing geometry for route update (route_id: {})", id);
    let processed_geometry = process_geometry(&state.pool, &payload.geometry).await?;

    // Begin RLS transaction - sets auth.uid() to the authenticated user
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // STEP 2: Store PROCESSED geometry version
    // RLS policy "Owner create route versions" checks:
    // EXISTS (SELECT 1 FROM routes WHERE id = route_id AND owner_id = auth.uid())
    //
    // If user doesn't own the route, INSERT will fail with permission denied error
    sqlx::query!(
        "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
        id,
        processed_geometry // <-- Routed + simplified geometry
    )
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        // Check if error is due to RLS policy violation
        if let sqlx::Error::Database(ref db_err) = e {
            if db_err.message().contains("permission denied")
                || db_err
                    .message()
                    .contains("violates row-level security policy")
            {
                tracing::warn!(
                    "User {} attempted to update route {} (permission denied by RLS)",
                    auth_user.id,
                    id
                );
                return StatusCode::FORBIDDEN;
            }
        }

        tracing::error!("Failed to create route version: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Fetch the route with new geometry (RLS ensures we can only see owned routes)
    let route = sqlx::query!(
        r#"
        SELECT
            r.id, r.name, r.owner_id, r.control_points, r.created_at as "created_at!", r.updated_at as "updated_at!"
        FROM routes r
        WHERE r.id = $1
        "#,
        id
    )
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!("Route {} not found or not accessible by user {}", id, auth_user.id);
            StatusCode::NOT_FOUND
        }
        _ => {
            tracing::error!("Failed to fetch route: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    // Commit transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(RouteWithGeometry {
        route: Route {
            id: route.id,
            name: route.name,
            owner_id: route.owner_id,
            control_points: route.control_points.clone(),
            created_at: route.created_at,
            updated_at: route.updated_at,
        },
        geometry: processed_geometry, // <-- Return processed geometry
        control_points: route.control_points,
    }))
}

pub async fn update_route_control_points(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRouteControlPoints>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    // IMPORTANT: Process geometry BEFORE starting transaction to avoid connection pool deadlock
    // The transaction holds one connection, and process_geometry needs another for routing queries
    let coordinates: Vec<serde_json::Value> = payload
        .control_points
        .as_array()
        .ok_or(StatusCode::BAD_REQUEST)?
        .iter()
        .map(|p| {
            // Handle both {lng, lat} and {coordinates: [lng, lat]} formats
            if let Some(coords) = p.get("coordinates") {
                Ok(coords.clone())
            } else if let (Some(lng), Some(lat)) = (p.get("lng"), p.get("lat")) {
                Ok(serde_json::json!([lng, lat]))
            } else {
                Err(StatusCode::BAD_REQUEST)
            }
        })
        .collect::<Result<Vec<_>, _>>()?;

    let geometry = serde_json::json!({
        "type": "MultiLineString",
        "coordinates": [coordinates]
    });

    let processed_geometry = process_geometry(&state.pool, &geometry).await?;

    // Now begin RLS transaction
    let mut tx = RlsTransaction::begin(&state.pool, &auth_user)
        .await
        .map_err(|e| {
            tracing::error!("Failed to start RLS transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Fetch the route to check ownership
    let existing_route = sqlx::query_as!(
        Route,
        r#"
        SELECT
            id, name, owner_id, control_points, created_at, updated_at
        FROM routes
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(&mut **tx) // Use the transaction for fetching
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => {
            tracing::warn!(
                "Route {} not found or not accessible by user {}",
                id,
                auth_user.id
            );
            StatusCode::NOT_FOUND
        }
        _ => {
            tracing::error!("Failed to fetch route: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    let auth_user_uuid = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID from auth_user: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if auth_user_uuid == existing_route.owner_id {
        // User owns the route, apply change directly
        tracing::info!(
            "Processing geometry for route update from control points (route_id: {}) - owned route",
            id
        );

        // Update the control points on the route
        sqlx::query("UPDATE routes SET control_points = $1 WHERE id = $2")
            .bind(&payload.control_points)
            .bind(id)
            .execute(&mut **tx)
            .await
            .map_err(|e| {
                tracing::error!("Failed to update control points: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

        // STEP 2: Store PROCESSED geometry version
        sqlx::query!(
            "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
            id,
            processed_geometry
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(ref db_err) = e {
                if db_err.message().contains("permission denied")
                    || db_err
                        .message()
                        .contains("violates row-level security policy")
                {
                    tracing::warn!(
                        "User {} attempted to update route {} (permission denied by RLS)",
                        auth_user.id,
                        id
                    );
                    return StatusCode::FORBIDDEN;
                }
            }

            tracing::error!("Failed to create route version: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Commit transaction
        tx.commit().await.map_err(|e| {
            tracing::error!("Failed to commit transaction: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        Ok(Json(RouteWithGeometry {
            route: existing_route,
            geometry: processed_geometry,
            control_points: Some(payload.control_points),
        }))
    } else {
        // User does NOT own the route, create a proposal

        tracing::info!(
            "User {} proposing change to route {} owned by {}",
            auth_user.id,
            id,
            existing_route.owner_id
        );

        let original_control_points = existing_route
            .control_points
            .clone()
            .ok_or(StatusCode::BAD_REQUEST)?;

        // Handle both {lng, lat} and {coordinates: [lng, lat]} formats
        let original_position = original_control_points
            .as_array()
            .and_then(|arr| arr.get(payload.point_index as usize))
            .and_then(|p| {
                // Handle both formats
                if let Some(coords) = p.get("coordinates") {
                    Some(coords.clone())
                } else if let (Some(lng), Some(lat)) = (p.get("lng"), p.get("lat")) {
                    Some(serde_json::json!([lng, lat]))
                } else {
                    None
                }
            })
            .ok_or(StatusCode::BAD_REQUEST)?;

        let new_position = payload
            .control_points
            .as_array()
            .and_then(|arr| arr.get(payload.point_index as usize))
            .and_then(|p| {
                // Handle both formats
                if let Some(coords) = p.get("coordinates") {
                    Some(coords.clone())
                } else if let (Some(lng), Some(lat)) = (p.get("lng"), p.get("lat")) {
                    Some(serde_json::json!([lng, lat]))
                } else {
                    None
                }
            })
            .ok_or(StatusCode::BAD_REQUEST)?;

        sqlx::query!(
            r#"
            INSERT INTO route_point_changes (route_id, user_id, user_email, feature_index, point_index, original_position, new_position, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending')
            "#,
            id,
            auth_user_uuid,
            auth_user.full_claims.email,
            payload.feature_index,
            payload.point_index,
            original_position,
            new_position,
        )
        .execute(&mut **tx)
        .await
        .map_err(|e| {
            tracing::error!("Failed to create route_point_change: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        tx.commit().await.map_err(|e| {
            tracing::error!(
                "Failed to commit transaction for point change proposal: {}",
                e
            );
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        // Fetch the latest processed geometry for the original route
        let original_processed_geometry = sqlx::query!(
            r#"
            SELECT geometry FROM route_versions
            WHERE route_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            id
        )
        .fetch_optional(&state.pool)
        .await
        .map_err(|e| {
            tracing::error!("Failed to fetch original route geometry: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .map(|row| row.geometry)
        .ok_or(StatusCode::NOT_FOUND)?; // Should always have geometry if route exists

        // Return the original route, indicating a proposal was created
        let control_points_clone = existing_route.control_points.clone();
        Ok(Json(RouteWithGeometry {
            route: existing_route,
            geometry: original_processed_geometry,
            control_points: control_points_clone,
        }))
    }
}
