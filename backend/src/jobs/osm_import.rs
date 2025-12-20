use anyhow::Result;
use uuid::Uuid;
use crate::db::DbPool;

/// OSM Import Job - Imports relevant OSM tracks
///
/// Filters for:
/// - highway=track, path, unclassified
/// - surface=sand, gravel, dirt
/// - tracktype=grade2-5
pub struct OsmImportJob {
    pub region: String,
    pub osm_pbf_path: String,
}

impl OsmImportJob {
    pub async fn run(&self, pool: &DbPool) -> Result<()> {
        tracing::info!("Starting OSM import for region: {}", self.region);

        // TODO: Implement actual OSM parsing using osmpbf or osmium
        // For PoC, this is a placeholder structure

        // Example: Parse PBF file and filter relevant ways
        // let ways = parse_osm_pbf(&self.osm_pbf_path)?;
        // for way in ways {
        //     if is_relevant_way(&way) {
        //         insert_track(pool, way).await?;
        //     }
        // }

        tracing::info!("OSM import completed for region: {}", self.region);
        Ok(())
    }

    async fn insert_track(
        &self,
        pool: &DbPool,
        geometry: serde_json::Value,
        surface: Option<String>,
    ) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO curated_tracks (geometry, source, surface, confidence, region)
            VALUES ($1, 'osm', $2, 3, $3)
            ON CONFLICT DO NOTHING
            "#,
            geometry,
            surface,
            &self.region
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

// Helper functions for OSM filtering
fn is_relevant_way(tags: &std::collections::HashMap<String, String>) -> bool {
    // Check highway type
    if let Some(highway) = tags.get("highway") {
        if !["track", "path", "unclassified"].contains(&highway.as_str()) {
            return false;
        }
    } else {
        return false;
    }

    // Prefer ways with surface tags
    if let Some(surface) = tags.get("surface") {
        if ["sand", "gravel", "dirt", "unpaved", "ground"].contains(&surface.as_str()) {
            return true;
        }
    }

    // Or tracktype
    if let Some(tracktype) = tags.get("tracktype") {
        if ["grade2", "grade3", "grade4", "grade5"].contains(&tracktype.as_str()) {
            return true;
        }
    }

    false
}
