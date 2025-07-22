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
    pub rating: i32,
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
#[derive(Debug, Serialize)]
pub struct MintWithRecommendationsAndInfo {
    pub mint: Mint,
    pub recommendations: Vec<RecommendationWithUser>,
    pub total_recommendations: usize,
    pub average_rating: Option<f64>,
    pub stored_info: Option<StoredMintInfo>,
    pub parsed_info: Option<FlexibleMintInfo>,
}

/// Mint with its recommendations
#[derive(Debug, Serialize)]
pub struct MintWithRecommendations {
    pub mint: Mint,
    pub recommendations: Vec<RecommendationWithUser>,
    pub total_recommendations: usize,
    pub average_rating: Option<f64>,
}

/// Recommendation with user profile
#[derive(Debug, Serialize)]
pub struct RecommendationWithUser {
    pub recommendation: Recommendation,
    pub user_profile: Option<UserProfile>,
}

/// User with their activity
#[derive(Debug, Serialize)]
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
}
