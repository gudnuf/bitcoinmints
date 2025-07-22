use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Raw Nostr event as stored in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawEvent {
    pub id: String,                 // Database ID
    pub event_id: String,           // Nostr event ID
    pub kind: u16,                  // Event kind
    pub pubkey: String,             // Author pubkey
    pub content: String,            // Event content
    pub tags: String,               // JSON-serialized tags
    pub sig: String,                // Event signature
    pub created_at: u64,            // Event timestamp
    pub received_at: DateTime<Utc>, // When we received it
    pub processed: bool,            // Whether we've processed it
}

/// Mint information extracted from kind 38172/38173 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mint {
    pub event_id: String,
    pub name: String,
    pub mint_url: String,
    pub description: Option<String>,
    pub mint_pubkey: String,
    pub author_pubkey: String,
    pub mint_type: String, // "cashu" or "fedimint"
    pub networks: Vec<String>,
    pub invite_codes: Vec<String>,
    pub nuts: cdk::nuts::Nuts, // CDK nuts structure for Cashu
    pub modules: Vec<String>,  // For Fedimint
    pub created_at: u64,
    pub received_at: DateTime<Utc>,
}

/// Recommendation from kind 38000 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub event_id: String,
    pub reviewer_pubkey: String,
    pub mint_pubkey: String,
    pub rating: Option<i32>,
    pub content: Option<String>,
    pub d_tag: String,
    pub k_tag: String,
    pub invite_codes: Vec<String>,
    pub created_at: u64,
    pub received_at: DateTime<Utc>,
}

/// User profile from kind 0 events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub pubkey: String,
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub about: Option<String>,
    pub picture: Option<String>,
    pub banner: Option<String>,
    pub website: Option<String>,
    pub nip05: Option<String>,
    pub created_at: u64,
    pub received_at: DateTime<Utc>,
}

/// API response for mints endpoint
#[derive(Debug, Serialize)]
pub struct MintsResponse {
    pub mints: Vec<MintWithRecommendations>,
}

/// API response for users endpoint
#[derive(Debug, Serialize)]
pub struct UsersResponse {
    pub users: Vec<UserWithActivity>,
}

/// Mint with its recommendations and stored mint info for frontend display
#[derive(Debug, Serialize, Clone)]
pub struct MintWithRecommendationsAndInfo {
    pub mint: Mint,
    pub recommendations: Vec<RecommendationWithUser>,
    pub total_recommendations: usize,
    pub average_rating: Option<f64>,
    pub stored_info: Option<StoredMintInfo>,
    pub parsed_info: Option<FlexibleMintInfo>,
}

/// Mint with its recommendations
#[derive(Debug, Serialize, Clone)]
pub struct MintWithRecommendations {
    pub mint: Mint,
    pub recommendations: Vec<RecommendationWithUser>,
    pub total_recommendations: usize,
    pub average_rating: Option<f64>,
}

/// Recommendation with user profile
#[derive(Debug, Serialize, Clone)]
pub struct RecommendationWithUser {
    pub recommendation: Recommendation,
    pub user_profile: Option<UserProfile>,
}

/// User with their activity
#[derive(Debug, Serialize, Clone)]
pub struct UserWithActivity {
    pub profile: UserProfile,
    pub recommendations_count: usize,
    pub mints_count: usize,
}

/// Nostr event kinds we're interested in
pub const CASHU_MINT_KIND: u16 = 38172;
pub const FEDIMINT_KIND: u16 = 38173;
pub const RECOMMENDATION_KIND: u16 = 38000;
pub const USER_METADATA_KIND: u16 = 0;

/// Flexible mint info structure that can handle various formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlexibleMintInfo {
    pub name: Option<String>,
    pub pubkey: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub description_long: Option<String>,
    pub contact: Option<serde_json::Value>, // Can be array of arrays or other formats
    pub motd: Option<String>,
    pub nuts: Option<HashMap<String, serde_json::Value>>, // Flexible nuts structure
}

/// Database representation of stored mint info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredMintInfo {
    pub id: String,
    pub mint_url: String,
    pub info_json: String, // JSON string of mint info
    pub last_fetched_at: DateTime<Utc>,
    pub fetch_success: bool,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // Health tracking fields
    pub consecutive_failures: i32,
    pub consecutive_successes: i32,
    pub total_attempts: i32,
    pub total_successes: i32,
    pub first_seen_at: DateTime<Utc>,
    pub health_score: f64, // 0.0 to 1.0 representing uptime percentage
    pub is_currently_online: bool,
}

/// Individual health check record for detailed history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintHealthRecord {
    pub id: String,
    pub mint_url: String,
    pub checked_at: DateTime<Utc>,
    pub success: bool,
    pub response_time_ms: Option<i64>,
    pub error_message: Option<String>,
    pub http_status: Option<u16>,
}

/// Health summary for API responses
#[derive(Debug, Serialize, Clone)]
pub struct MintHealthSummary {
    pub mint_url: String,
    pub is_online: bool,
    pub uptime_percentage: f64,
    pub last_online: Option<DateTime<Utc>>,
    pub last_offline: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub total_checks_24h: i32,
    pub successful_checks_24h: i32,
    pub average_response_time_24h: Option<f64>,
}

/// Query parameters for filtering mints
#[derive(Debug, Deserialize, Hash)]
pub struct MintQueryParams {
    /// Optional mint type filter: "cashu", "fedimint", or None for all
    #[serde(rename = "type")]
    pub mint_type: Option<String>,
    /// Comma-separated list of currencies that must support minting (only for Cashu mints)
    pub minting: Option<String>,
    /// Comma-separated list of currencies that must support melting (only for Cashu mints)
    pub melting: Option<String>,
}

/// Cache key types for different cached data
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum CacheKey {
    AllMints,
    CashuMints,
    FedimintMints,
    AllUsers,
    AllRecommendations,
    AllRawEvents,
    AllMintInfo,
    AllHealthSummaries,
    MintsByQuery(String), // Serialized query parameters
    RecommendationByEventId(String),
    UserProfile(String),       // pubkey
    MintHealthSummary(String), // mint_url
}

impl std::fmt::Display for CacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheKey::AllMints => write!(f, "all_mints"),
            CacheKey::CashuMints => write!(f, "cashu_mints"),
            CacheKey::FedimintMints => write!(f, "fedimint_mints"),
            CacheKey::AllUsers => write!(f, "all_users"),
            CacheKey::AllRecommendations => write!(f, "all_recommendations"),
            CacheKey::AllRawEvents => write!(f, "all_raw_events"),
            CacheKey::AllMintInfo => write!(f, "all_mint_info"),
            CacheKey::AllHealthSummaries => write!(f, "all_health_summaries"),
            CacheKey::MintsByQuery(query) => write!(f, "mints_by_query_{}", query),
            CacheKey::RecommendationByEventId(event_id) => write!(f, "recommendation_{}", event_id),
            CacheKey::UserProfile(pubkey) => write!(f, "user_profile_{}", pubkey),
            CacheKey::MintHealthSummary(mint_url) => write!(f, "mint_health_{}", mint_url),
        }
    }
}

/// Cached data wrapper with timestamp for TTL
#[derive(Debug, Clone)]
pub struct CachedData<T> {
    pub data: T,
    pub cached_at: DateTime<Utc>,
    pub ttl_seconds: i64,
}

impl<T> CachedData<T> {
    pub fn new(data: T, ttl_seconds: i64) -> Self {
        Self {
            data,
            cached_at: Utc::now(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.cached_at);
        elapsed.num_seconds() > self.ttl_seconds
    }

    pub fn expires_at(&self) -> DateTime<Utc> {
        self.cached_at + chrono::Duration::seconds(self.ttl_seconds)
    }
}

/// Cache statistics for monitoring
#[derive(Debug, Clone, Serialize)]
pub struct CacheStats {
    pub total_entries: usize,
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub expired_entries: usize,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            total_entries: 0,
            hit_count: 0,
            miss_count: 0,
            hit_rate: 0.0,
            expired_entries: 0,
        }
    }

    pub fn calculate_hit_rate(&mut self) {
        let total = self.hit_count + self.miss_count;
        self.hit_rate = if total > 0 {
            self.hit_count as f64 / total as f64
        } else {
            0.0
        };
    }
}
