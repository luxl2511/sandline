use anyhow::{anyhow, Result};
use sqlx::{PgPool, Postgres, Transaction};
use std::ops::{Deref, DerefMut};

use crate::middleware::AuthUser;

pub struct RlsTransaction<'a> {
    inner: Transaction<'a, Postgres>,
}

impl<'a> RlsTransaction<'a> {
    /// Begin a new transaction with RLS context
    ///
    /// Sets the following session variables:
    /// - `role`: The user's role (e.g., "authenticated")
    /// - `request.jwt.claim.sub`: The user's UUID
    /// - `request.jwt.claim.role`: The user's role
    ///
    /// These variables enable RLS policies to use `auth.uid()` for authorization.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The transaction cannot be started
    /// - The user ID is not a valid UUID
    /// - The role is not whitelisted
    /// - Setting any session variable fails
    ///
    /// If an error occurs, the transaction is automatically rolled back.
    pub async fn begin(pool: &'a PgPool, auth: &AuthUser) -> Result<Self> {
        let mut tx = pool.begin().await?;

        // Validate user ID is a valid UUID (security check)
        uuid::Uuid::parse_str(&auth.id)
            .map_err(|_| anyhow!("Invalid user ID format: {}", auth.id))?;

        // Validate role is whitelisted (security check)
        let allowed_roles = ["authenticated", "anon", "service_role"];
        if !allowed_roles.contains(&auth.role.as_str()) {
            return Err(anyhow!("Invalid role: {}", auth.role));
        }

        // Set RLS context using SET LOCAL (transaction-scoped)
        //
        // SECURITY NOTE: Using format!() with validated inputs only
        // - auth.id is validated as a UUID above
        // - auth.role is validated against whitelist above
        //
        // Alternative: PostgreSQL doesn't support parameterized SET LOCAL yet

        // Set the PostgreSQL role
        let set_role_query = format!("SET LOCAL role {}", auth.role);
        sqlx::query(&set_role_query)
            .execute(&mut *tx)
            .await
            .map_err(|e| anyhow!("Failed to set role: {}. Query: {}", e, set_role_query))?;

        // Set JWT claim: sub (user UUID)
        let set_sub_query = format!("SET LOCAL \"request.jwt.claim.sub\" TO '{}'", auth.id);
        sqlx::query(&set_sub_query)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to set request.jwt.claim.sub: {}. Query: {}",
                    e,
                    set_sub_query
                )
            })?;

        // Set JWT claim: role
        let set_role_claim_query =
            format!("SET LOCAL \"request.jwt.claim.role\" TO '{}'", auth.role);
        sqlx::query(&set_role_claim_query)
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to set request.jwt.claim.role: {}. Query: {}",
                    e,
                    set_role_claim_query
                )
            })?;

        tracing::debug!("RLS context set: user={}, role={}", auth.id, auth.role);

        Ok(Self { inner: tx })
    }

    /// Commit the transaction
    ///
    /// # Errors
    ///
    /// Returns an error if the commit fails
    pub async fn commit(self) -> Result<()> {
        self.inner.commit().await?;
        Ok(())
    }

    /// Rollback the transaction
    ///
    /// # Errors
    ///
    /// Returns an error if the rollback fails
    pub async fn rollback(self) -> Result<()> {
        self.inner.rollback().await?;
        Ok(())
    }
}

impl<'a> Deref for RlsTransaction<'a> {
    type Target = Transaction<'a, Postgres>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a> DerefMut for RlsTransaction<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
