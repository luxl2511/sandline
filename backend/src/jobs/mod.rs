pub mod osm_import;
pub mod curated_tracks_import;
pub mod confidence_update;

// Job runner utilities
use anyhow::Result;
use crate::db::DbPool;

pub trait DataImportJob {
    async fn run(&self, pool: &DbPool) -> Result<()>;
}
