use crate::models::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Thread-safe cache service with TTL support
#[derive(Clone)]
pub struct CacheService {
    // Use Arc<RwLock<>> for thread-safe access to the cache
    cache: Arc<RwLock<HashMap<String, Box<dyn std::any::Any + Send + Sync>>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl CacheService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::new())),
        }
    }

    /// Get cached data if it exists and hasn't expired
    pub async fn get<T: Clone + Send + Sync + 'static>(&self, key: &CacheKey) -> Option<T> {
        let key_str = key.to_string();

        // Check cache in a scoped block to release the lock
        let cached_result = {
            let cache = self.cache.read().await;
            if let Some(boxed_data) = cache.get(&key_str) {
                if let Some(cached_data) = boxed_data.downcast_ref::<CachedData<T>>() {
                    if !cached_data.is_expired() {
                        Some(cached_data.data.clone())
                    } else {
                        debug!(
                            target: "bitcoinmints_retyr::cache",
                            key = %key,
                            expired_at = %cached_data.expires_at(),
                            "⏰ Cache entry expired"
                        );
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Update stats after releasing the cache lock
        let mut stats = self.stats.write().await;
        if cached_result.is_some() {
            stats.hit_count += 1;
            debug!(
                target: "bitcoinmints_retyr::cache",
                key = %key,
                "📋 Cache hit"
            );
        } else {
            stats.miss_count += 1;
            debug!(
                target: "bitcoinmints_retyr::cache",
                key = %key,
                "❌ Cache miss"
            );
        }
        stats.calculate_hit_rate();

        cached_result
    }

    /// Store data in cache with TTL
    pub async fn set<T: Clone + Send + Sync + 'static>(
        &self,
        key: &CacheKey,
        data: T,
        ttl_seconds: i64,
    ) {
        let cached_data = CachedData::new(data, ttl_seconds);
        let key_str = key.to_string();

        let mut cache = self.cache.write().await;
        cache.insert(key_str, Box::new(cached_data));

        // Update stats
        drop(cache); // Release write lock before updating stats
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.read().await.len();

        info!(
            target: "bitcoinmints_retyr::cache",
            key = %key,
            ttl_seconds = ttl_seconds,
            "💾 Cached data"
        );
    }

    /// Invalidate specific cache key
    pub async fn invalidate(&self, key: &CacheKey) {
        let key_str = key.to_string();
        let mut cache = self.cache.write().await;

        if cache.remove(&key_str).is_some() {
            info!(
                target: "bitcoinmints_retyr::cache",
                key = %key,
                "🗑️ Invalidated cache entry"
            );
        }

        // Update stats
        drop(cache); // Release write lock before updating stats
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.read().await.len();
    }

    /// Invalidate multiple cache keys (for related data)
    pub async fn invalidate_many(&self, keys: &[CacheKey]) {
        let mut cache = self.cache.write().await;
        let mut invalidated_count = 0;

        for key in keys {
            let key_str = key.to_string();
            if cache.remove(&key_str).is_some() {
                invalidated_count += 1;
            }
        }

        if invalidated_count > 0 {
            info!(
                target: "bitcoinmints_retyr::cache",
                count = invalidated_count,
                "🗑️ Invalidated multiple cache entries"
            );
        }

        // Update stats
        drop(cache); // Release write lock before updating stats
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.read().await.len();
    }

    /// Invalidate all cached data (nuclear option)
    pub async fn clear_all(&self) {
        let mut cache = self.cache.write().await;
        let cleared_count = cache.len();
        cache.clear();

        // Reset stats
        let mut stats = self.stats.write().await;
        *stats = CacheStats::new();

        warn!(
            target: "bitcoinmints_retyr::cache",
            cleared_count = cleared_count,
            "💥 Cleared entire cache"
        );
    }

    /// Get cache statistics
    pub async fn get_stats(&self) -> CacheStats {
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.read().await.len();

        // For now, we'll skip counting expired entries as it requires complex type checking
        // In a production system, we could maintain this separately or use a different approach
        stats.expired_entries = 0;

        stats.clone()
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) {
        // For now, we'll implement a simple cleanup that removes all entries periodically
        // In a production system, we could track expiration times separately
        let cache_size = self.cache.read().await.len();

        if cache_size > 1000 {
            // If cache gets too large, clear it
            warn!(
                target: "bitcoinmints_retyr::cache",
                cache_size = cache_size,
                "🧹 Cache size exceeded limit, clearing all entries"
            );
            self.clear_all().await;
        } else {
            debug!(
                target: "bitcoinmints_retyr::cache",
                cache_size = cache_size,
                "✅ Cache size within limits"
            );
        }
    }

    /// Invalidate cache when new events are stored (smart invalidation)
    pub async fn invalidate_on_new_event(&self, event_kind: u16) {
        let mut keys_to_invalidate = Vec::new();

        match event_kind {
            CASHU_MINT_KIND | FEDIMINT_KIND => {
                // New mint events invalidate mint-related caches
                keys_to_invalidate.extend([
                    CacheKey::AllMints,
                    CacheKey::CashuMints,
                    CacheKey::FedimintMints,
                ]);

                // Also invalidate any query-specific caches (we'll need to clear all MintsByQuery)
                // For now, we'll clear the entire cache when mint data changes
                // In a more sophisticated implementation, we could track which queries exist
            }
            RECOMMENDATION_KIND => {
                // New recommendations invalidate recommendation and mint caches
                keys_to_invalidate.extend([
                    CacheKey::AllRecommendations,
                    CacheKey::AllMints,
                    CacheKey::CashuMints,
                    CacheKey::FedimintMints,
                ]);
            }
            USER_METADATA_KIND => {
                // New user profiles invalidate user-related caches
                keys_to_invalidate.push(CacheKey::AllUsers);
            }
            _ => {
                // For other event types, invalidate raw events cache
                keys_to_invalidate.push(CacheKey::AllRawEvents);
            }
        }

        self.invalidate_many(&keys_to_invalidate).await;

        info!(
            target: "bitcoinmints_retyr::cache",
            event_kind = event_kind,
            invalidated_keys = keys_to_invalidate.len(),
            "🔄 Smart cache invalidation on new event"
        );
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(&self) -> tokio::task::JoinHandle<()> {
        let cache_service = self.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;
                cache_service.cleanup_expired().await;
            }
        })
    }
}
