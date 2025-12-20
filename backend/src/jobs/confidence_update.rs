use anyhow::Result;
use chrono::{Utc, Duration};
use crate::db::DbPool;

/// Confidence Update Job
///
/// Periodically updates confidence scores based on:
/// - Age of verification
/// - Community usage
/// - Cross-validation with other sources
pub struct ConfidenceUpdateJob;

impl ConfidenceUpdateJob {
    pub async fn run(&self, pool: &DbPool) -> Result<()> {
        tracing::info!("Starting confidence update job");

        // Downgrade old unverified tracks
        self.downgrade_old_tracks(pool).await?;

        // Upgrade frequently used tracks
        self.upgrade_popular_tracks(pool).await?;

        tracing::info!("Confidence update job completed");
        Ok(())
    }

    async fn downgrade_old_tracks(&self, pool: &DbPool) -> Result<()> {
        let cutoff_date = Utc::now().naive_utc().date() - Duration::days(365);

        sqlx::query!(
            r#"
            UPDATE curated_tracks
            SET confidence = GREATEST(confidence - 1, 1)
            WHERE last_verified < $1 AND confidence > 2
            "#,
            cutoff_date
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn upgrade_popular_tracks(&self, pool: &DbPool) -> Result<()> {
        // TODO: Implement usage tracking and upgrade logic
        // This would require a usage_stats table to track which tracks
        // are frequently included in routes

        Ok(())
    }

    /// Cross-validate track with multiple sources
    pub async fn cross_validate_track(
        &self,
        pool: &DbPool,
        track_id: uuid::Uuid,
    ) -> Result<i32> {
        // Check if track appears in multiple sources
        let count: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(DISTINCT source)
            FROM curated_tracks
            WHERE ST_DWithin(
                geometry::geography,
                (SELECT geometry::geography FROM curated_tracks WHERE id = $1),
                100
            )
            "#,
            track_id
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        // More sources = higher confidence
        Ok(match count {
            0..=1 => 2,
            2 => 3,
            3 => 4,
            _ => 5,
        })
    }
}
