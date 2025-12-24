use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::db::DbPool;
use crate::middleware::AuthUser;
use crate::models::{CreateRoute, Route, RouteWithGeometry, UpdateRoute};

pub async fn list_routes(
    State(pool): State<DbPool>,
) -> Result<Json<Vec<RouteWithGeometry>>, StatusCode> {
    let routes = sqlx::query!(
        r#"
        SELECT
            r.id, r.name, r.owner_id, r.created_at as "created_at!", r.updated_at as "updated_at!",
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
    .fetch_all(&pool)
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
                created_at: row.created_at,
                updated_at: row.updated_at,
            },
            geometry: row.geometry,
        })
        .collect();

    Ok(Json(result))
}

pub async fn get_route(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    let route = sqlx::query!(
        r#"
        SELECT
            r.id, r.name, r.owner_id, r.created_at as "created_at!", r.updated_at as "updated_at!",
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
    .fetch_one(&pool)
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
            created_at: route.created_at,
            updated_at: route.updated_at,
        },
        geometry: route.geometry,
    }))
}

pub async fn create_route(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Json(payload): Json<CreateRoute>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    // Use authenticated user ID as owner
    let owner_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let mut tx = pool.begin().await.map_err(|e| {
        tracing::error!("Failed to start transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let route = sqlx::query_as!(
        Route,
        "INSERT INTO routes (name, owner_id) VALUES ($1, $2) RETURNING *",
        payload.name,
        owner_id
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create route: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    sqlx::query!(
        "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
        route.id,
        payload.geometry
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create route version: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(RouteWithGeometry {
        route,
        geometry: payload.geometry,
    }))
}

pub async fn update_route(
    auth_user: AuthUser,
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateRoute>,
) -> Result<Json<RouteWithGeometry>, StatusCode> {
    let owner_id = Uuid::parse_str(&auth_user.id).map_err(|e| {
        tracing::error!("Failed to parse user ID: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let route = sqlx::query_as!(Route, "SELECT * FROM routes WHERE id = $1", id)
        .fetch_one(&pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => StatusCode::NOT_FOUND,
            _ => {
                tracing::error!("Failed to fetch route: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            }
        })?;

    // Verify ownership
    if route.owner_id != owner_id {
        tracing::warn!(
            "User {} attempted to update route {} owned by {}",
            owner_id,
            route.id,
            route.owner_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    sqlx::query!(
        "INSERT INTO route_versions (route_id, geometry) VALUES ($1, $2)",
        id,
        payload.geometry
    )
    .execute(&pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create route version: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(RouteWithGeometry {
        route,
        geometry: payload.geometry,
    }))
}
