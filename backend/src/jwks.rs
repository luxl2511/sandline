use anyhow::{anyhow, Result};
use jsonwebtoken::jwk::{
    AlgorithmParameters, EllipticCurve, EllipticCurveKeyParameters, Jwk, JwkSet,
};
use jsonwebtoken::DecodingKey;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

/// JWKS cache that fetches and caches public keys from Supabase
/// with automatic background refresh every 24 hours
pub struct JwksCache {
    keys: Arc<RwLock<HashMap<String, Jwk>>>,
    jwks_url: String,
}

impl JwksCache {
    /// Create a new JWKS cache and perform initial fetch
    ///
    /// # Errors
    /// Returns an error if the initial JWKS fetch fails or the key set is empty
    pub async fn new(jwks_url: String) -> Result<Self> {
        let cache = Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
            jwks_url,
        };

        // Initial fetch - must succeed for app to start
        cache.refresh().await?;

        Ok(cache)
    }

    /// Fetch JWKS from the endpoint and update the cache
    ///
    /// # Errors
    /// Returns an error if the HTTP request fails, JSON parsing fails, or the key set is empty
    async fn refresh(&self) -> Result<()> {
        tracing::info!("Fetching JWKS from {}", self.jwks_url);

        let response = reqwest::get(&self.jwks_url)
            .await
            .map_err(|e| anyhow!("Failed to fetch JWKS: {}", e))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "JWKS endpoint returned status {}",
                response.status()
            ));
        }

        let jwks: JwkSet = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse JWKS JSON: {}", e))?;

        if jwks.keys.is_empty() {
            tracing::error!("JWKS endpoint returned empty key set - possible configuration issue");
            return Err(anyhow!("Empty JWKS key set"));
        }

        // Convert JwkSet to HashMap for O(1) lookup by kid
        let key_map: HashMap<String, Jwk> = jwks
            .keys
            .into_iter()
            .filter_map(|k| k.common.key_id.clone().map(|kid| (kid, k)))
            .collect();

        tracing::info!("JWKS refreshed, {} keys loaded", key_map.len());

        *self.keys.write().await = key_map;

        Ok(())
    }

    /// Get a decoding key by kid (Key ID)
    ///
    /// # Arguments
    /// * `kid` - The Key ID from the JWT header
    ///
    /// # Returns
    /// * `Some(DecodingKey)` if the key is found
    /// * `None` if the key is not found in the cache
    pub async fn get_key(&self, kid: &str) -> Option<DecodingKey> {
        let keys = self.keys.read().await;

        keys.get(kid).and_then(|jwk| {
            // Convert JWK to DecodingKey based on algorithm
            match &jwk.algorithm {
                AlgorithmParameters::EllipticCurve(ec) => Self::ec_to_decoding_key(ec)
                    .map_err(|e| {
                        tracing::error!("Failed to convert EC key {}: {}", kid, e);
                        e
                    })
                    .ok(),
                AlgorithmParameters::RSA(rsa) => {
                    // If Supabase switches to RSA in the future
                    DecodingKey::from_rsa_components(&rsa.n, &rsa.e)
                        .map_err(|e| {
                            tracing::error!("Failed to convert RSA key {}: {}", kid, e);
                            e
                        })
                        .ok()
                }
                _ => {
                    tracing::warn!("Unsupported key algorithm for kid: {}", kid);
                    None
                }
            }
        })
    }

    /// Convert EC JWK parameters to DecodingKey
    ///
    /// Supabase uses ES256 (ECDSA with P-256 curve and SHA-256)
    fn ec_to_decoding_key(ec: &EllipticCurveKeyParameters) -> Result<DecodingKey> {
        // Supabase ES256 keys use P-256 curve
        if ec.curve != EllipticCurve::P256 {
            return Err(anyhow!(
                "Unsupported elliptic curve: {:?}. Expected P-256",
                ec.curve
            ));
        }

        // Convert base64url-encoded coordinates to DecodingKey
        DecodingKey::from_ec_components(&ec.x, &ec.y)
            .map_err(|e| anyhow!("Failed to create EC decoding key: {}", e))
    }

    /// Spawn a background task that refreshes the JWKS every 24 hours
    ///
    /// The task will continue running even if individual refresh attempts fail,
    /// keeping the cached keys available.
    pub fn spawn_refresh_task(self: Arc<Self>) {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(24 * 60 * 60)); // 24 hours
            interval.tick().await; // First tick completes immediately

            loop {
                interval.tick().await;

                tracing::info!("Starting scheduled JWKS refresh");

                if let Err(e) = self.refresh().await {
                    tracing::error!("JWKS refresh failed: {}. Continuing with cached keys.", e);
                    // Don't panic - keep using cached keys until next refresh succeeds
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_jwks_fetch() {
        // Test against real Supabase JWKS endpoint
        let url = "https://mptcysneksvjwiwqoqmt.supabase.co/auth/v1/.well-known/jwks.json";
        let cache = JwksCache::new(url.to_string()).await;

        assert!(
            cache.is_ok(),
            "JWKS fetch should succeed for valid Supabase project"
        );

        let cache = cache.unwrap();
        let keys = cache.keys.read().await;
        assert!(!keys.is_empty(), "JWKS should contain at least one key");
    }

    #[tokio::test]
    async fn test_invalid_url() {
        let url = "https://invalid-url-that-does-not-exist.example.com/jwks";
        let cache = JwksCache::new(url.to_string()).await;

        assert!(cache.is_err(), "JWKS fetch should fail for invalid URL");
    }
}
