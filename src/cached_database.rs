use crate::cache::CacheService;
use crate::database::Database;
use crate::mint_coordinator::MintCoordinator;
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
    mint_coordinator: Option<MintCoordinator>,
}

impl CachedDatabase {
    pub fn new(database: Database, cache: CacheService) -> Self {
        Self {
            database,
            cache,
            mint_coordinator: None,
        }
    }

    /// Set the mint coordinator (called after services are initialized)
    pub fn set_mint_coordinator(&mut self, coordinator: MintCoordinator) {
        self.mint_coordinator = Some(coordinator);
    }

    /// Pre-fill the cache with all main data types on startup
    pub async fn pre_fill_cache(&self) -> Result<()> {
        tracing::info!(
            target: "bitcoinmints_retyr::cached_database",
            "🔄 Starting cache pre-fill process..."
        );

        // Pre-fill all main cache keys in parallel for better performance
        let results = tokio::join!(
            self.pre_fill_mints_cache(),
            self.pre_fill_users_cache(),
            self.pre_fill_recommendations_cache(),
            self.pre_fill_raw_events_cache(),
            self.pre_fill_mint_info_cache(),
            self.pre_fill_health_summaries_cache(),
        );

        // Check for any errors in pre-filling
        let mut errors = Vec::new();

        if let Err(e) = results.0 {
            errors.push(format!("Mints cache: {}", e));
        }
        if let Err(e) = results.1 {
            errors.push(format!("Users cache: {}", e));
        }
        if let Err(e) = results.2 {
            errors.push(format!("Recommendations cache: {}", e));
        }
        if let Err(e) = results.3 {
            errors.push(format!("Raw events cache: {}", e));
        }
        if let Err(e) = results.4 {
            errors.push(format!("Mint info cache: {}", e));
        }
        if let Err(e) = results.5 {
            errors.push(format!("Health summaries cache: {}", e));
        }

        if !errors.is_empty() {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                errors = ?errors,
                "⚠️ Some cache pre-fills failed, but continuing..."
            );
        }

        tracing::info!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Cache pre-fill process completed"
        );

        Ok(())
    }

    /// Pre-fill mints cache (all variants)
    async fn pre_fill_mints_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling mints cache..."
        );

        // Fill all mints
        let all_mints = self.database.get_mints_with_recommendations(None).await?;
        self.cache.set(&CacheKey::AllMints, all_mints, 120).await;

        // Fill cashu mints
        let cashu_mints = self
            .database
            .get_mints_with_recommendations(Some("cashu"))
            .await?;
        self.cache
            .set(&CacheKey::CashuMints, cashu_mints, 120)
            .await;

        // Fill fedimint mints
        let fedimint_mints = self
            .database
            .get_mints_with_recommendations(Some("fedimint"))
            .await?;
        self.cache
            .set(&CacheKey::FedimintMints, fedimint_mints, 120)
            .await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Mints cache pre-filled"
        );

        Ok(())
    }

    /// Pre-fill users cache
    async fn pre_fill_users_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling users cache..."
        );

        let users = self.database.get_users_with_activity().await?;
        self.cache.set(&CacheKey::AllUsers, users, 600).await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Users cache pre-filled"
        );

        Ok(())
    }

    /// Pre-fill recommendations cache
    async fn pre_fill_recommendations_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling recommendations cache..."
        );

        let recommendations = self.database.get_all_recommendations().await?;
        self.cache
            .set(&CacheKey::AllRecommendations, recommendations, 120)
            .await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Recommendations cache pre-filled"
        );

        Ok(())
    }

    /// Pre-fill raw events cache
    async fn pre_fill_raw_events_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling raw events cache..."
        );

        let raw_events = self.database.get_all_raw_events().await?;
        self.cache
            .set(&CacheKey::AllRawEvents, raw_events, 120)
            .await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Raw events cache pre-filled"
        );

        Ok(())
    }

    /// Pre-fill mint info cache
    async fn pre_fill_mint_info_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling mint info cache..."
        );

        let mint_info = self.database.get_all_stored_mint_info().await?;
        self.cache.set(&CacheKey::AllMintInfo, mint_info, 600).await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Mint info cache pre-filled"
        );

        Ok(())
    }

    /// Pre-fill health summaries cache
    async fn pre_fill_health_summaries_cache(&self) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "Pre-filling health summaries cache..."
        );

        let health_summaries = self.database.get_all_mint_health_summaries().await?;
        self.cache
            .set(&CacheKey::AllHealthSummaries, health_summaries, 120)
            .await;

        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            "✅ Health summaries cache pre-filled"
        );

        Ok(())
    }

    /// Proactively refresh cache entries when data changes
    pub async fn refresh_cache_on_event(&self, event_kind: u16) -> Result<()> {
        tracing::debug!(
            target: "bitcoinmints_retyr::cached_database",
            event_kind = event_kind,
            "🔄 Proactively refreshing cache due to new event"
        );

        match event_kind {
            CASHU_MINT_KIND | FEDIMINT_KIND => {
                // Refresh mint-related caches in parallel
                let results = tokio::join!(
                    self.pre_fill_mints_cache(),
                    self.pre_fill_raw_events_cache()
                );

                if let Err(e) = results.0 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh mints cache"
                    );
                }
                if let Err(e) = results.1 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh raw events cache"
                    );
                }
            }
            RECOMMENDATION_KIND => {
                // Refresh recommendation and mint caches in parallel
                let results = tokio::join!(
                    self.pre_fill_recommendations_cache(),
                    self.pre_fill_mints_cache(),
                    self.pre_fill_raw_events_cache()
                );

                if let Err(e) = results.0 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh recommendations cache"
                    );
                }
                if let Err(e) = results.1 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh mints cache"
                    );
                }
                if let Err(e) = results.2 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh raw events cache"
                    );
                }
            }
            USER_METADATA_KIND => {
                // Refresh user-related caches
                let results = tokio::join!(
                    self.pre_fill_users_cache(),
                    self.pre_fill_raw_events_cache()
                );

                if let Err(e) = results.0 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh users cache"
                    );
                }
                if let Err(e) = results.1 {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh raw events cache"
                    );
                }
            }
            _ => {
                // For other event types, just refresh raw events
                if let Err(e) = self.pre_fill_raw_events_cache().await {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh raw events cache"
                    );
                }
            }
        }

        Ok(())
    }

    /// Start background task for periodic cache refresh
    pub fn start_cache_refresh_task(&self) -> tokio::task::JoinHandle<()> {
        let cached_database = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30)); // Check every 30 seconds

            loop {
                interval.tick().await;

                tracing::debug!(
                    target: "bitcoinmints_retyr::cached_database",
                    "🔄 Running periodic cache refresh"
                );

                // Refresh all main caches periodically to ensure they stay warm
                if let Err(e) = cached_database.pre_fill_cache().await {
                    tracing::warn!(
                        target: "bitcoinmints_retyr::cached_database",
                        error = %e,
                        "Failed to refresh cache during periodic task"
                    );
                }
            }
        })
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

        // Cache the result (2 minutes TTL for mint data)
        self.cache.set(&cache_key, data.clone(), 120).await;

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

        let mut data = self
            .database
            .get_mints_with_recommendations_and_info(query_params)
            .await?;

        // Enrich with fedimint federation information
        data = self
            .database
            .enrich_mints_with_federation_info(data)
            .await?;

        // Cache the result (2 minutes TTL for query-specific data)
        self.cache.set(&cache_key, data.clone(), 120).await;

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

        // Cache the result (2 minutes TTL for recommendation data)
        self.cache.set(&cache_key, data.clone(), 120).await;

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

        // Cache the result (2 minutes TTL for statistics)
        self.cache.set(&cache_key, data.clone(), 120).await;

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

    /// Store raw event and proactively refresh related caches
    pub async fn store_raw_event(&self, event: &nostr_sdk::Event) -> Result<()> {
        // Store in database first
        self.database.store_raw_event(event).await?;

        // Proactively refresh related caches based on event kind
        if let Err(e) = self.refresh_cache_on_event(event.kind.as_u16()).await {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                error = %e,
                event_kind = event.kind.as_u16(),
                "Failed to refresh cache after storing event, falling back to invalidation"
            );
            // Fall back to invalidation if refresh fails
            self.cache
                .invalidate_on_new_event(event.kind.as_u16())
                .await;
        }

        Ok(())
    }

    /// Store mint info and proactively refresh related caches
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

        // Proactively refresh mint info and health caches
        let refresh_results = tokio::join!(
            self.pre_fill_mint_info_cache(),
            self.pre_fill_health_summaries_cache()
        );

        let mut mint_info_failed = false;
        let mut health_summaries_failed = false;

        if let Err(e) = refresh_results.0 {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                error = %e,
                "Failed to refresh mint info cache, falling back to invalidation"
            );
            mint_info_failed = true;
        }
        if let Err(e) = refresh_results.1 {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                error = %e,
                "Failed to refresh health summaries cache, falling back to invalidation"
            );
            health_summaries_failed = true;
        }

        // If refresh failed, fall back to invalidation
        if mint_info_failed || health_summaries_failed {
            self.cache
                .invalidate_many(&[
                    CacheKey::AllMintInfo,
                    CacheKey::AllHealthSummaries,
                    CacheKey::MintHealthSummary(mint_url.to_string()),
                ])
                .await;
        }

        Ok(())
    }

    /// Store health record and proactively refresh health caches
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

        // Proactively refresh health caches
        if let Err(e) = self.pre_fill_health_summaries_cache().await {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                error = %e,
                "Failed to refresh health summaries cache, falling back to invalidation"
            );
            // Fall back to invalidation if refresh fails
            self.cache
                .invalidate_many(&[
                    CacheKey::AllHealthSummaries,
                    CacheKey::MintHealthSummary(mint_url.to_string()),
                ])
                .await;
        }

        Ok(())
    }

    // ===== PASS-THROUGH OPERATIONS (operations that don't need caching) =====

    /// Check if event exists (no caching needed for this check)
    pub async fn event_exists(&self, event_id: &str) -> Result<bool> {
        self.database.event_exists(event_id).await
    }

    /// Cleanup duplicate mints (refreshes caches)
    pub async fn cleanup_duplicate_mints(&self) -> Result<()> {
        let result = self.database.cleanup_duplicate_mints().await;

        // Proactively refresh mint-related caches after cleanup
        if let Err(e) = self.pre_fill_mints_cache().await {
            tracing::warn!(
                target: "bitcoinmints_retyr::cached_database",
                error = %e,
                "Failed to refresh mints cache after cleanup, falling back to invalidation"
            );
            // Fall back to invalidation if refresh fails
            self.cache
                .invalidate_many(&[
                    CacheKey::AllMints,
                    CacheKey::CashuMints,
                    CacheKey::FedimintMints,
                ])
                .await;
        }

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

    /// Get mints with recommendations and info for frontend (using new unified approach)
    pub async fn get_unified_mints_with_recommendations(
        &self,
        query_params: &MintQueryParams,
    ) -> Result<Vec<UnifiedMintWithRecommendations>> {
        // Create a hash of the query parameters for cache key
        let query_hash = self.hash_query_params(query_params);
        let cache_key = CacheKey::MintsByQuery(format!("unified_{}", query_hash));

        // Try cache first
        if let Some(cached_data) = self
            .cache
            .get::<Vec<UnifiedMintWithRecommendations>>(&cache_key)
            .await
        {
            debug!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🎯 Serving unified mints from cache"
            );
            return Ok(cached_data);
        }

        // Cache miss - use mint coordinator if available
        if let Some(coordinator) = &self.mint_coordinator {
            info!(
                target: "bitcoinmints_retyr::cached_database",
                cache_key = %cache_key,
                "🔄 Fetching unified mints using coordinator"
            );

            let data = coordinator
                .get_unified_mints_with_recommendations(query_params)
                .await?;

            // Cache the result (2 minutes TTL for query-specific data)
            self.cache.set(&cache_key, data.clone(), 120).await;

            return Ok(data);
        }

        // Fallback to old method if coordinator not available
        info!(
            target: "bitcoinmints_retyr::cached_database",
            cache_key = %cache_key,
            "🔄 Falling back to legacy enrichment method"
        );

        let legacy_data = self
            .get_mints_with_recommendations_and_info(query_params)
            .await?;

        // Convert legacy data to unified format (basic conversion)
        let unified_data = self.convert_legacy_to_unified(legacy_data);
        self.cache.set(&cache_key, unified_data.clone(), 120).await;

        Ok(unified_data)
    }

    /// Convert legacy MintWithRecommendationsAndInfo to unified format
    fn convert_legacy_to_unified(
        &self,
        legacy_mints: Vec<MintWithRecommendationsAndInfo>,
    ) -> Vec<UnifiedMintWithRecommendations> {
        legacy_mints
            .into_iter()
            .map(|legacy| {
                let mint_type = match legacy.mint.mint_type.as_str() {
                    "cashu" => MintType::Cashu,
                    "fedimint" => MintType::Fedimint,
                    _ => MintType::Cashu, // Default fallback
                };

                let (cashu_data, fedimint_data) = match mint_type {
                    MintType::Cashu => {
                        let cashu_data = Some(CashuMintData {
                            mint_url: legacy.mint.mint_url.clone(),
                            mint_pubkey: legacy.mint.mint_pubkey.clone(),
                            nuts: legacy.mint.nuts.clone(),
                            mint_info: legacy.parsed_info.clone(),
                            version: legacy.parsed_info.as_ref().and_then(|i| i.version.clone()),
                            supported_currencies: Vec::new(), // Would need to extract from nuts
                        });
                        (cashu_data, None)
                    }
                    MintType::Fedimint => {
                        let fedimint_data = Some(FedimintMintData {
                            federation_id: legacy.mint.federation_id.clone().unwrap_or_default(),
                            federation_name: legacy
                                .mint
                                .meta
                                .get("federation_name")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            invite_codes: legacy.mint.invite_codes.clone(),
                            modules: legacy.mint.modules.clone(),
                            guardians_count: legacy.mint.guardians_count,
                            welcome_message: legacy
                                .mint
                                .meta
                                .get("welcome_message")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string()),
                            config_available: legacy.mint.federation_id.is_some(),
                        });
                        (None, fedimint_data)
                    }
                };

                let health_info = legacy.stored_info.as_ref().map(|stored| MintHealthInfo {
                    health_score: stored.health_score,
                    uptime_percentage: stored.health_score * 100.0,
                    consecutive_failures: stored.consecutive_failures,
                    consecutive_successes: stored.consecutive_successes,
                    total_attempts: stored.total_attempts,
                    total_successes: stored.total_successes,
                    last_check: stored.last_fetched_at,
                });

                let unified_mint_data = UnifiedMintData {
                    event_id: legacy.mint.event_id,
                    mint_id: match mint_type {
                        MintType::Cashu => legacy.mint.mint_url.clone(),
                        MintType::Fedimint => legacy
                            .mint
                            .federation_id
                            .unwrap_or_else(|| format!("unknown-{}", legacy.mint.mint_pubkey)),
                    },
                    name: legacy.mint.name,
                    description: legacy.mint.description,
                    mint_type,
                    networks: legacy.mint.networks,
                    author_pubkey: legacy.mint.author_pubkey,
                    created_at: legacy.mint.created_at,
                    received_at: legacy.mint.received_at,
                    cashu_data,
                    fedimint_data,
                    health_info,
                    is_online: legacy
                        .stored_info
                        .as_ref()
                        .map(|s| s.is_currently_online)
                        .unwrap_or(true),
                    last_updated: legacy.stored_info.as_ref().map(|s| s.last_fetched_at),
                };

                UnifiedMintWithRecommendations {
                    mint_data: unified_mint_data,
                    recommendations: legacy.recommendations,
                    total_recommendations: legacy.total_recommendations,
                    average_rating: legacy.average_rating,
                }
            })
            .collect()
    }
}
