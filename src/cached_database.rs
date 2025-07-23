use crate::cache::CacheService;
use crate::database::Database;
use crate::models::*;
use anyhow::Result;
use serde_json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use tracing::{debug, info};

/// Cached database service that wraps the original database with intelligent caching
#[derive(Clone)]
pub struct CachedDatabase {
    database: Database,
    cache: CacheService,
}

impl CachedDatabase {
    pub fn new(database: Database, cache: CacheService) -> Self {
        Self { database, cache }
    }

    /// Get the underlying database (for operations that don't need caching)
    pub fn database(&self) -> &Database {
        &self.database
    }

    /// Get the cache service
    pub fn cache(&self) -> &CacheService {
        &self.cache
    }

    // ===== CACHED DATABASE OPERATIONS =====

    /// Get all mints with recommendations (cached)
    pub async fn get_mints_with_recommendations(
        &self,
        mint_type: Option<&str>,
    ) -> Result<Vec<MintWithRecommendations>> {
        let cache_key = match mint_type {
            Some("cashu") => CacheKey::CashuMints,
            Some("fedimint") => CacheKey::FedimintMints,
            _ => CacheKey::AllMints,
        };

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Vec<MintWithRecommendations>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving mints from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching mints from database"
        );

        let data = self
            .database
            .get_mints_with_recommendations(mint_type)
            .await?;

        // Cache the result (5 minutes TTL for mint data)
        self.cache.set(&cache_key, data.clone(), 300).await;

        Ok(data)
    }

    /// Get mints with recommendations and info for frontend (cached with query-specific keys)
    pub async fn get_mints_with_recommendations_and_info(
        &self,
        query_params: &MintQueryParams,
    ) -> Result<Vec<MintWithRecommendationsAndInfo>> {
        // Create a hash of the query parameters for cache key
        let query_hash = self.hash_query_params(query_params);
        let cache_key = CacheKey::MintsByQuery(query_hash);

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Vec<MintWithRecommendationsAndInfo>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving mints with info from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching mints with info from database"
        );

        let data = self
            .database
            .get_mints_with_recommendations_and_info(query_params)
            .await?;

        // Cache the result (3 minutes TTL for query-specific data)
        self.cache.set(&cache_key, data.clone(), 180).await;

        Ok(data)
    }

    /// Get all users with activity (cached)
    pub async fn get_users_with_activity(&self) -> Result<Vec<UserWithActivity>> {
        let cache_key = CacheKey::AllUsers;

        // Try cache first
        if let Some(cached_data) = self.cache.get::<Vec<UserWithActivity>>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving users from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching users from database"
        );

        let data = self.database.get_users_with_activity().await?;

        // Cache the result (10 minutes TTL for user data)
        self.cache.set(&cache_key, data.clone(), 600).await;

        Ok(data)
    }

    /// Get all recommendations (cached)
    pub async fn get_all_recommendations(&self) -> Result<Vec<RecommendationWithUser>> {
        let cache_key = CacheKey::AllRecommendations;

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Vec<RecommendationWithUser>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving recommendations from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching recommendations from database"
        );

        let data = self.database.get_all_recommendations().await?;

        // Cache the result (5 minutes TTL for recommendation data)
        self.cache.set(&cache_key, data.clone(), 300).await;

        Ok(data)
    }

    /// Get all raw events (cached)
    pub async fn get_all_raw_events(&self) -> Result<Vec<RawEvent>> {
        let cache_key = CacheKey::AllRawEvents;

        // Try cache first
        if let Some(cached_data) = self.cache.get::<Vec<RawEvent>>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving raw events from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching raw events from database"
        );

        let data = self.database.get_all_raw_events().await?;

        // Cache the result (2 minutes TTL for raw events - they change frequently)
        self.cache.set(&cache_key, data.clone(), 120).await;

        Ok(data)
    }

    /// Get all stored mint info (cached)
    pub async fn get_all_stored_mint_info(&self) -> Result<Vec<StoredMintInfo>> {
        let cache_key = CacheKey::AllMintInfo;

        // Try cache first
        if let Some(cached_data) = self.cache.get::<Vec<StoredMintInfo>>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving mint info from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching mint info from database"
        );

        let data = self.database.get_all_stored_mint_info().await?;

        // Cache the result (10 minutes TTL for mint info)
        self.cache.set(&cache_key, data.clone(), 600).await;

        Ok(data)
    }

    /// Get all mint health summaries (cached)
    pub async fn get_all_mint_health_summaries(&self) -> Result<Vec<MintHealthSummary>> {
        let cache_key = CacheKey::AllHealthSummaries;

        // Try cache first
        if let Some(cached_data) = self.cache.get::<Vec<MintHealthSummary>>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving health summaries from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Fetching health summaries from database"
        );

        let data = self.database.get_all_mint_health_summaries().await?;

        // Cache the result (2 minutes TTL for health data - it changes frequently)
        self.cache.set(&cache_key, data.clone(), 120).await;

        Ok(data)
    }

    /// Get mint statistics (cached)
    pub async fn get_mint_statistics(&self) -> Result<serde_json::Value> {
        let cache_key = CacheKey::AllMints; // Reuse mint cache key since stats depend on mint data

        // For statistics, we'll use a shorter cache time since they're computed data
        if let Some(cached_data) = self.cache.get::<serde_json::Value>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                "🎯 Serving mint statistics from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            "🔄 Computing mint statistics from database"
        );

        let data = self.database.get_mint_statistics().await?;

        // Cache the result (5 minutes TTL for statistics)
        self.cache.set(&cache_key, data.clone(), 300).await;

        Ok(data)
    }

    /// Get recommendation by event ID (cached)
    pub async fn get_recommendation_by_event_id(
        &self,
        event_id: &str,
    ) -> Result<Option<RecommendationWithUser>> {
        let cache_key = CacheKey::RecommendationByEventId(event_id.to_string());

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Option<RecommendationWithUser>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                event_id = %event_id,
                "🎯 Serving recommendation from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            event_id = %event_id,
            "🔄 Fetching recommendation from database"
        );

        let data = self
            .database
            .get_recommendation_by_event_id(event_id)
            .await?;

        // Cache the result (15 minutes TTL for individual recommendations)
        self.cache.set(&cache_key, data.clone(), 900).await;

        Ok(data)
    }

    /// Get user profile (cached)
    pub async fn get_user_profile(&self, pubkey: &str) -> Result<Option<UserProfile>> {
        let cache_key = CacheKey::UserProfile(pubkey.to_string());

        // Try cache first
        if let Some(cached_data) = self.cache.get::<Option<UserProfile>>(&cache_key).await {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                pubkey = %pubkey,
                "🎯 Serving user profile from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            pubkey = %pubkey,
            "🔄 Fetching user profile from database"
        );

        let data = self.database.get_user_profile(pubkey).await?;

        // Cache the result (30 minutes TTL for user profiles)
        self.cache.set(&cache_key, data.clone(), 1800).await;

        Ok(data)
    }

    /// Get mint health summary (cached)
    pub async fn get_mint_health_summary(
        &self,
        mint_url: &str,
    ) -> Result<Option<MintHealthSummary>> {
        let cache_key = CacheKey::MintHealthSummary(mint_url.to_string());

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Option<MintHealthSummary>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                mint_url = %mint_url,
                "🎯 Serving mint health summary from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - fetch from database
        info!(
            target: "bitcoinmints_retyr::cached_database",
            mint_url = %mint_url,
            "🔄 Fetching mint health summary from database"
        );

        let data = self.database.get_mint_health_summary(mint_url).await?;

        // Cache the result (1 minute TTL for health summaries - they change frequently)
        self.cache.set(&cache_key, data.clone(), 60).await;

        Ok(data)
    }

    // ===== WRITE OPERATIONS (with cache invalidation) =====

    /// Store raw event and invalidate related caches
    pub async fn store_raw_event(&self, event: &nostr_sdk::Event) -> Result<()> {
        // Store in database first
        self.database.store_raw_event(event).await?;

        // Invalidate related caches based on event kind
        self.cache
            .invalidate_on_new_event(event.kind.as_u16())
            .await;

        Ok(())
    }

    /// Store mint info and invalidate related caches
    pub async fn store_mint_info(
        &self,
        mint_url: &str,
        mint_info_json: Option<&str>,
        error: Option<&str>,
    ) -> Result<()> {
        // Store in database first
        self.database
            .store_mint_info(mint_url, mint_info_json, error)
            .await?;

        // Invalidate mint info caches
        self.cache
            .invalidate_many(&[
                CacheKey::AllMintInfo,
                CacheKey::AllHealthSummaries,
                CacheKey::MintHealthSummary(mint_url.to_string()),
            ])
            .await;

        Ok(())
    }

    /// Store health record and invalidate health caches
    pub async fn store_health_record(
        &self,
        mint_url: &str,
        success: bool,
        response_time_ms: Option<i64>,
        error_message: Option<&str>,
        http_status: Option<u16>,
    ) -> Result<()> {
        // Store in database first
        self.database
            .store_health_record(
                mint_url,
                success,
                response_time_ms,
                error_message,
                http_status,
            )
            .await?;

        // Invalidate health-related caches
        self.cache
            .invalidate_many(&[
                CacheKey::AllHealthSummaries,
                CacheKey::MintHealthSummary(mint_url.to_string()),
            ])
            .await;

        Ok(())
    }

    // ===== PASS-THROUGH OPERATIONS (operations that don't need caching) =====

    /// Check if event exists (no caching needed for this check)
    pub async fn event_exists(&self, event_id: &str) -> Result<bool> {
        self.database.event_exists(event_id).await
    }

    /// Cleanup duplicate mints (invalidates caches)
    pub async fn cleanup_duplicate_mints(&self) -> Result<()> {
        let result = self.database.cleanup_duplicate_mints().await;

        // Clear all mint-related caches after cleanup
        self.cache
            .invalidate_many(&[
                CacheKey::AllMints,
                CacheKey::CashuMints,
                CacheKey::FedimintMints,
            ])
            .await;

        result
    }

    /// Get mints needing info fetch (no caching needed)
    pub async fn get_mints_needing_info_fetch(&self, max_age_hours: i64) -> Result<Vec<String>> {
        self.database
            .get_mints_needing_info_fetch(max_age_hours)
            .await
    }

    /// Get stored mint info (individual lookups can be cached)
    pub async fn get_stored_mint_info(&self, mint_url: &str) -> Result<Option<StoredMintInfo>> {
        // For individual lookups, we'll go directly to the database for now
        // In the future, we could cache individual mint info lookups
        self.database.get_stored_mint_info(mint_url).await
    }

    /// Cleanup old health records (no caching impact)
    pub async fn cleanup_old_health_records(&self) -> Result<()> {
        self.database.cleanup_old_health_records().await
    }

    // ===== CACHE MANAGEMENT =====

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        self.cache.get_stats().await
    }

    /// Clear all caches (for debugging/admin purposes)
    pub async fn clear_all_caches(&self) {
        self.cache.clear_all().await
    }

    // ===== UTILITY METHODS =====

    /// Create a hash of query parameters for cache keys
    fn hash_query_params(&self, params: &MintQueryParams) -> String {
        let mut hasher = DefaultHasher::new();

        // Hash the query parameters to create a unique key
        params.mint_type.hash(&mut hasher);
        params.minting.hash(&mut hasher);
        params.melting.hash(&mut hasher);
        params.nuts.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}
