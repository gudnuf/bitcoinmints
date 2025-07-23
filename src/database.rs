use crate::models::*;
use crate::ui::components::mint_card::get_supported_nuts;
use crate::utils::normalize_mint_url;
use anyhow::Result;
use chrono::Utc;
use sqlx::{sqlite::SqlitePoolOptions, Row, SqlitePool};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new() -> Result<Self> {
        // Create data directory if it doesn't exist
        std::fs::create_dir_all("data")?;

        let database_url = "sqlite:data/bitcoinmints.db?mode=rwc";
        info!(
            target: "bitcoinmints_retyr::database",
            path = "data/bitcoinmints.db",
            "🗄️ Connecting to database"
        );

        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::database",
            "🔧 Running database migrations"
        );

        // Create raw_events table to store all events as received
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS raw_events (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL UNIQUE,
                kind INTEGER NOT NULL,
                pubkey TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT NOT NULL,
                sig TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                received_at TEXT NOT NULL,
                processed BOOLEAN DEFAULT FALSE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_raw_events_kind ON raw_events (kind)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_raw_events_pubkey ON raw_events (pubkey)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_raw_events_processed ON raw_events (processed)",
        )
        .execute(&self.pool)
        .await?;

        // Create mint_info table to store detailed mint information from /v1/info endpoints
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS mint_info (
                id TEXT PRIMARY KEY,
                mint_url TEXT NOT NULL UNIQUE,
                info_json TEXT NOT NULL,
                last_fetched_at TEXT NOT NULL,
                fetch_success BOOLEAN NOT NULL,
                error_message TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add health tracking columns if they don't exist (for existing databases)
        let add_health_columns = [
            "ALTER TABLE mint_info ADD COLUMN consecutive_failures INTEGER DEFAULT 0",
            "ALTER TABLE mint_info ADD COLUMN consecutive_successes INTEGER DEFAULT 0",
            "ALTER TABLE mint_info ADD COLUMN total_attempts INTEGER DEFAULT 0",
            "ALTER TABLE mint_info ADD COLUMN total_successes INTEGER DEFAULT 0",
            "ALTER TABLE mint_info ADD COLUMN first_seen_at TEXT",
            "ALTER TABLE mint_info ADD COLUMN health_score REAL DEFAULT 1.0",
            "ALTER TABLE mint_info ADD COLUMN is_currently_online BOOLEAN DEFAULT TRUE",
        ];

        for alter_sql in add_health_columns {
            // Ignore errors if column already exists
            let _ = sqlx::query(alter_sql).execute(&self.pool).await;
        }

        // Create health_records table for detailed health history
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS health_records (
                id TEXT PRIMARY KEY,
                mint_url TEXT NOT NULL,
                checked_at TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                response_time_ms INTEGER,
                error_message TEXT,
                http_status INTEGER,
                FOREIGN KEY (mint_url) REFERENCES mint_info (mint_url)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for mint_info table
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_mint_info_url ON mint_info (mint_url)")
            .execute(&self.pool)
            .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_mint_info_fetched ON mint_info (last_fetched_at)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_mint_info_health ON mint_info (health_score, is_currently_online)",
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for health_records table
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_health_records_url ON health_records (mint_url)",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_health_records_checked ON health_records (checked_at)",
        )
        .execute(&self.pool)
        .await?;

        info!(
            target: "bitcoinmints_retyr::database",
            "✅ Database migrations completed"
        );
        Ok(())
    }

    /// Store a raw Nostr event
    pub async fn store_raw_event(&self, event: &nostr_sdk::Event) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let tags_json = serde_json::to_string(&event.tags)?;

        sqlx::query(
            r#"
            INSERT OR IGNORE INTO raw_events 
            (id, event_id, kind, pubkey, content, tags, sig, created_at, received_at, processed)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(event.id.to_string())
        .bind(event.kind.as_u16() as i64)
        .bind(event.pubkey.to_string())
        .bind(&event.content)
        .bind(&tags_json)
        .bind(event.sig.to_string())
        .bind(event.created_at.as_u64() as i64)
        .bind(now.to_rfc3339())
        .bind(false)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all raw events (for debugging/inspection)
    pub async fn get_all_raw_events(&self) -> Result<Vec<RawEvent>> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at, processed
            FROM raw_events 
            ORDER BY received_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut events = Vec::new();
        for row in rows {
            let event = RawEvent {
                id: row.get("id"),
                event_id: row.get("event_id"),
                kind: row.get::<i64, _>("kind") as u16,
                pubkey: row.get("pubkey"),
                content: row.get("content"),
                tags: row.get("tags"),
                sig: row.get("sig"),
                created_at: row.get::<i64, _>("created_at") as u64,
                received_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("received_at"),
                )?
                .into(),
                processed: row.get("processed"),
            };
            events.push(event);
        }
        Ok(events)
    }

    /// Get all mints with their recommendations (filtered by mint type if specified)
    pub async fn get_mints_with_recommendations(
        &self,
        mint_type: Option<&str>,
    ) -> Result<Vec<MintWithRecommendations>> {
        // Build SQL query based on mint type filter
        let sql_query = match mint_type {
            Some("cashu") => {
                r#"
                SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
                FROM raw_events 
                WHERE kind = 38172
                ORDER BY created_at DESC
                "#
            }
            Some("fedimint") => {
                r#"
                SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
                FROM raw_events 
                WHERE kind = 38173
                ORDER BY created_at DESC
                "#
            }
            _ => {
                r#"
                SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
                FROM raw_events 
                WHERE kind IN (38172, 38173)
                ORDER BY created_at DESC
                "#
            }
        };

        // Get filtered mint events
        let mint_rows = sqlx::query(sql_query).fetch_all(&self.pool).await?;

        // Use a HashMap to deduplicate by normalized URL, keeping the most recent mint
        let mut unique_mints: std::collections::HashMap<String, Mint> =
            std::collections::HashMap::new();

        for row in mint_rows {
            // Parse the mint event
            if let Ok(mint) = self.parse_mint_event(&row).await {
                let url = mint.mint_url.clone();

                // Only keep the first (most recent) mint for each normalized URL
                if !unique_mints.contains_key(&url) {
                    unique_mints.insert(url, mint);
                }
            }
        }

        let mut mints_with_recommendations = Vec::new();

        for mint in unique_mints.into_values() {
            // Get recommendations for this mint using the mint URL
            let recommendations = self
                .get_recommendations_for_mint_url(&mint.mint_url)
                .await?;

            // Calculate stats
            let total_recommendations = recommendations.len();
            let ratings_with_values: Vec<i32> = recommendations
                .iter()
                .filter_map(|r| r.recommendation.rating)
                .collect();
            let average_rating = if !ratings_with_values.is_empty() {
                let sum: i32 = ratings_with_values.iter().sum();
                Some(sum as f64 / ratings_with_values.len() as f64)
            } else {
                None
            };

            mints_with_recommendations.push(MintWithRecommendations {
                mint,
                recommendations,
                total_recommendations,
                average_rating,
            });
        }

        // Sort by name for consistent output
        mints_with_recommendations.sort_by(|a, b| a.mint.name.cmp(&b.mint.name));

        Ok(mints_with_recommendations)
    }

    /// Get all mints with their recommendations and stored mint info for frontend display
    pub async fn get_mints_with_recommendations_and_info(
        &self,
        query_params: &crate::models::MintQueryParams,
    ) -> Result<Vec<MintWithRecommendationsAndInfo>> {
        let mints_with_recs = self
            .get_mints_with_recommendations(query_params.mint_type.as_deref())
            .await?;

        let mut mints_with_info = Vec::new();

        for mint_with_recs in mints_with_recs {
            // Apply Cashu currency and NUT filters if specified
            if query_params.mint_type.as_deref() == Some("cashu")
                && (query_params.minting.is_some()
                    || query_params.melting.is_some()
                    || query_params.nuts.is_some())
            {
                let mut passes_filter = true;

                // Check minting currencies filter
                if let Some(minting_currencies_str) = &query_params.minting {
                    let required_minting_currencies: Vec<&str> = minting_currencies_str
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if !required_minting_currencies.is_empty() {
                        let supported_minting_currencies: Vec<String> = mint_with_recs
                            .mint
                            .nuts
                            .nut04
                            .methods
                            .iter()
                            .map(|method| format!("{:?}", method.unit).to_lowercase())
                            .collect();

                        // Check if all required minting currencies are supported
                        for required_currency in required_minting_currencies {
                            if !supported_minting_currencies
                                .contains(&required_currency.to_lowercase())
                            {
                                passes_filter = false;
                                break;
                            }
                        }
                    }
                }

                // Check melting currencies filter
                if passes_filter {
                    if let Some(melting_currencies_str) = &query_params.melting {
                        let required_melting_currencies: Vec<&str> = melting_currencies_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        if !required_melting_currencies.is_empty() {
                            let supported_melting_currencies: Vec<String> = mint_with_recs
                                .mint
                                .nuts
                                .nut05
                                .methods
                                .iter()
                                .map(|method| format!("{:?}", method.unit).to_lowercase())
                                .collect();

                            // Check if all required melting currencies are supported
                            for required_currency in required_melting_currencies {
                                if !supported_melting_currencies
                                    .contains(&required_currency.to_lowercase())
                                {
                                    passes_filter = false;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Check NUT filters
                if passes_filter {
                    if let Some(nuts_str) = &query_params.nuts {
                        let required_nuts: Vec<&str> = nuts_str
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect();

                        if !required_nuts.is_empty() {
                            // Get supported NUTs for this mint
                            let supported_nuts = get_supported_nuts(&mint_with_recs.mint.nuts);

                            // Check if all required NUTs are supported
                            for required_nut in required_nuts {
                                if !supported_nuts.contains(&required_nut.to_string()) {
                                    passes_filter = false;
                                    break;
                                }
                            }
                        }
                    }
                }

                // Skip this mint if it doesn't pass the filters
                if !passes_filter {
                    continue;
                }
            }

            // Get stored mint info for this mint
            let stored_info = self
                .get_stored_mint_info(&mint_with_recs.mint.mint_url)
                .await
                .ok()
                .flatten();

            // Parse the info JSON if available
            let parsed_info = if let Some(ref stored) = stored_info {
                if stored.fetch_success && !stored.info_json.is_empty() && stored.info_json != "{}"
                {
                    serde_json::from_str::<FlexibleMintInfo>(&stored.info_json).ok()
                } else {
                    None
                }
            } else {
                None
            };

            mints_with_info.push(MintWithRecommendationsAndInfo {
                mint: mint_with_recs.mint,
                recommendations: mint_with_recs.recommendations,
                total_recommendations: mint_with_recs.total_recommendations,
                average_rating: mint_with_recs.average_rating,
                stored_info,
                parsed_info,
            });
        }

        Ok(mints_with_info)
    }

    /// Get user profiles with their activity
    pub async fn get_users_with_activity(&self) -> Result<Vec<UserWithActivity>> {
        // Get all user profile events (kind 0)
        let user_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 0
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut users_with_activity = Vec::new();

        for row in user_rows {
            if let Ok(profile) = self.parse_user_profile_event(&row).await {
                // Count recommendations by this user
                let recommendations_count = sqlx::query(
                    "SELECT COUNT(*) as count FROM raw_events WHERE kind = 38000 AND pubkey = ?",
                )
                .bind(&profile.pubkey)
                .fetch_one(&self.pool)
                .await?
                .get::<i64, _>("count") as usize;

                // Count mints published by this user
                let mints_count = sqlx::query(
                    "SELECT COUNT(*) as count FROM raw_events WHERE kind IN (38172, 38173) AND pubkey = ?"
                )
                .bind(&profile.pubkey)
                .fetch_one(&self.pool)
                .await?
                .get::<i64, _>("count") as usize;

                users_with_activity.push(UserWithActivity {
                    profile,
                    recommendations_count,
                    mints_count,
                });
            }
        }

        Ok(users_with_activity)
    }

    /// Get recommendations for a specific mint URL
    async fn get_recommendations_for_mint_url(
        &self,
        mint_url: &str,
    ) -> Result<Vec<RecommendationWithUser>> {
        let recommendation_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 38000 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut recommendations_with_users = Vec::new();

        for row in recommendation_rows {
            if let Ok(recommendation) = self.parse_recommendation_event(&row).await {
                // Check if this recommendation is for our mint by comparing URLs
                let matches_mint = recommendation.invite_codes.iter().any(|url| {
                    // Normalize both URLs for comparison
                    if let (Ok(recommendation_url), Ok(target_url)) = (
                        crate::utils::normalize_mint_url(url),
                        crate::utils::normalize_mint_url(mint_url),
                    ) {
                        recommendation_url == target_url
                    } else {
                        // Fallback to direct string comparison if normalization fails
                        url == mint_url
                    }
                });

                if matches_mint {
                    // Get user profile for the reviewer
                    let user_profile = self
                        .get_user_profile(&recommendation.reviewer_pubkey)
                        .await?;

                    recommendations_with_users.push(RecommendationWithUser {
                        recommendation,
                        user_profile,
                    });
                }
            }
        }

        Ok(recommendations_with_users)
    }

    /// Get recommendations for a specific mint pubkey (legacy method for compatibility)
    async fn get_recommendations_for_mint(
        &self,
        mint_pubkey: &str,
    ) -> Result<Vec<RecommendationWithUser>> {
        let recommendation_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 38000 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut recommendations_with_users = Vec::new();

        for row in recommendation_rows {
            if let Ok(recommendation) = self.parse_recommendation_event(&row).await {
                // Check if this recommendation is for our mint
                if recommendation.mint_pubkey == mint_pubkey {
                    // Get user profile for the reviewer
                    let user_profile = self
                        .get_user_profile(&recommendation.reviewer_pubkey)
                        .await?;

                    recommendations_with_users.push(RecommendationWithUser {
                        recommendation,
                        user_profile,
                    });
                }
            }
        }

        Ok(recommendations_with_users)
    }

    /// Get user profile by pubkey
    pub async fn get_user_profile(&self, pubkey: &str) -> Result<Option<UserProfile>> {
        let rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 0 AND pubkey = ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(pubkey)
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = rows.first() {
            Ok(Some(self.parse_user_profile_event(row).await?))
        } else {
            Ok(None)
        }
    }

    /// Get all recommendations with user profiles
    pub async fn get_all_recommendations(&self) -> Result<Vec<RecommendationWithUser>> {
        let recommendation_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 38000 
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut recommendations_with_users = Vec::new();

        for row in recommendation_rows {
            if let Ok(recommendation) = self.parse_recommendation_event(&row).await {
                // Get user profile for the reviewer
                let user_profile = self
                    .get_user_profile(&recommendation.reviewer_pubkey)
                    .await?;

                recommendations_with_users.push(RecommendationWithUser {
                    recommendation,
                    user_profile,
                });
            }
        }

        Ok(recommendations_with_users)
    }

    /// Get a specific recommendation by event_id with user profile
    pub async fn get_recommendation_by_event_id(
        &self,
        event_id: &str,
    ) -> Result<Option<RecommendationWithUser>> {
        let recommendation_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind = 38000 AND event_id = ?
            "#,
        )
        .bind(event_id)
        .fetch_all(&self.pool)
        .await?;

        if let Some(row) = recommendation_rows.first() {
            if let Ok(recommendation) = self.parse_recommendation_event(row).await {
                // Get user profile for the reviewer
                let user_profile = self
                    .get_user_profile(&recommendation.reviewer_pubkey)
                    .await?;

                return Ok(Some(RecommendationWithUser {
                    recommendation,
                    user_profile,
                }));
            }
        }

        Ok(None)
    }

    /// Parse mint event from database row
    async fn parse_mint_event(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Mint> {
        let tags_json: String = row.get("tags");
        let tags: Vec<nostr_sdk::Tag> = serde_json::from_str(&tags_json)?;
        let content: String = row.get("content");
        let kind = row.get::<i64, _>("kind") as u16;

        let mut mint_pubkey = String::new();
        let mut networks = Vec::new();
        let mut invite_codes = Vec::new();
        let mut nuts = cdk::nuts::Nuts::default(); // Initialize with default
        let mut modules = Vec::new();
        let mut name = String::new();
        let mut description = None;

        // Parse tags according to NIP-87
        for tag in tags {
            let tag_vec = tag.to_vec();
            if tag_vec.len() >= 2 {
                match tag_vec[0].as_str() {
                    "d" => mint_pubkey = tag_vec[1].clone(),
                    "u" => invite_codes.push(tag_vec[1].clone()),
                    "n" => networks.push(tag_vec[1].clone()),
                    "modules" => {
                        modules = tag_vec[1]
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .collect()
                    }
                    _ => {}
                }
            }
        }

        // Parse content for name and description
        if !content.is_empty() {
            if let Ok(content_data) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(content_name) = content_data.get("name").and_then(|v| v.as_str()) {
                    name = content_name.to_string();
                }
                if let Some(content_desc) = content_data.get("about").and_then(|v| v.as_str()) {
                    description = Some(content_desc.to_string());
                }
            }
        }

        // Get the first invite code and normalize it only for cashu mints
        let raw_mint_url = invite_codes.first().cloned().unwrap_or_default();
        let normalized_mint_url = if kind == crate::models::CASHU_MINT_KIND {
            // Only normalize URLs for cashu mints
            match normalize_mint_url(&raw_mint_url) {
                Ok(url) => {
                    if url != raw_mint_url {
                        info!(
                            target: "bitcoinmints_retyr::database",
                            original = %raw_mint_url,
                            normalized = %url,
                            "🔗 Normalized cashu mint URL"
                        );
                    }
                    url
                }
                Err(e) => {
                    warn!(
                        target: "bitcoinmints_retyr::database",
                        url = %raw_mint_url,
                        error = %e,
                        "⚠️ Failed to normalize cashu mint URL, using original"
                    );
                    raw_mint_url.clone()
                }
            }
        } else {
            // For fedimint, use the raw invite code as-is (no normalization needed)
            raw_mint_url.clone()
        };

        // Try to get nuts from stored mint info by parsing the JSON
        if let Ok(Some(stored_info)) = self.get_stored_mint_info(&normalized_mint_url).await {
            if stored_info.fetch_success
                && !stored_info.info_json.is_empty()
                && stored_info.info_json != "{}"
            {
                // Try to parse the stored mint info JSON to get nuts
                if let Ok(mint_info) =
                    serde_json::from_str::<cdk::nuts::nut06::MintInfo>(&stored_info.info_json)
                {
                    nuts = mint_info.nuts;
                }
            }
        }

        // Use first invite code as mint URL if no name
        if name.is_empty() {
            name = normalized_mint_url
                .split("://")
                .nth(1)
                .and_then(|s| s.split('/').next())
                .unwrap_or("Unknown Mint")
                .to_string();
        }

        Ok(Mint {
            event_id: row.get("event_id"),
            name,
            mint_url: normalized_mint_url,
            description,
            mint_pubkey,
            author_pubkey: row.get("pubkey"),
            mint_type: if kind == CASHU_MINT_KIND {
                "cashu".to_string()
            } else {
                "fedimint".to_string()
            },
            networks,
            invite_codes,
            nuts,
            modules,
            created_at: row.get::<i64, _>("created_at") as u64,
            received_at: chrono::DateTime::parse_from_rfc3339(
                &row.get::<String, _>("received_at"),
            )?
            .into(),
        })
    }

    /// Parse recommendation event from database row
    async fn parse_recommendation_event(
        &self,
        row: &sqlx::sqlite::SqliteRow,
    ) -> Result<Recommendation> {
        let tags_json: String = row.get("tags");
        let tags: Vec<nostr_sdk::Tag> = serde_json::from_str(&tags_json)?;
        let content: String = row.get("content");

        let mut d_tag = String::new();
        let mut k_tag = String::new();
        let mut invite_codes = Vec::new();
        let mut rating: Option<i32> = None;

        // Parse tags
        for tag in tags {
            let tag_vec = tag.to_vec();
            if tag_vec.len() >= 2 {
                match tag_vec[0].as_str() {
                    "d" => d_tag = tag_vec[1].clone(),
                    "k" => k_tag = tag_vec[1].clone(),
                    "u" => invite_codes.push(tag_vec[1].clone()),
                    "rating" => {
                        if let Ok(r) = tag_vec[1].parse::<i32>() {
                            rating = Some(r);
                        }
                    }
                    _ => {}
                }
            }
        }
        // Parse rating from content if not found in tags
        if rating.is_none() && !content.is_empty() {
            // Look for rating pattern like [5/5] or [3/5] in the content
            if let Some(captures) = regex::Regex::new(r"\[(\d+)/5\]")
                .unwrap()
                .captures(&content)
            {
                if let Some(rating_match) = captures.get(1) {
                    if let Ok(parsed_rating) = rating_match.as_str().parse::<i32>() {
                        rating = Some(parsed_rating);
                    }
                }
            }
        }

        Ok(Recommendation {
            event_id: row.get("event_id"),
            reviewer_pubkey: row.get("pubkey"),
            mint_pubkey: d_tag.clone(),
            rating,
            content: if content.is_empty() {
                None
            } else {
                Some(content)
            },
            d_tag,
            k_tag,
            invite_codes,
            created_at: row.get::<i64, _>("created_at") as u64,
            received_at: chrono::DateTime::parse_from_rfc3339(
                &row.get::<String, _>("received_at"),
            )?
            .into(),
        })
    }

    /// Parse user profile event from database row
    async fn parse_user_profile_event(&self, row: &sqlx::sqlite::SqliteRow) -> Result<UserProfile> {
        let content: String = row.get("content");

        let mut name = None;
        let mut display_name = None;
        let mut about = None;
        let mut picture = None;
        let mut banner = None;
        let mut website = None;
        let mut nip05 = None;

        if !content.is_empty() {
            if let Ok(metadata) = serde_json::from_str::<serde_json::Value>(&content) {
                name = metadata
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                display_name = metadata
                    .get("display_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                about = metadata
                    .get("about")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                picture = metadata
                    .get("picture")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                banner = metadata
                    .get("banner")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                website = metadata
                    .get("website")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                nip05 = metadata
                    .get("nip05")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }

        Ok(UserProfile {
            pubkey: row.get("pubkey"),
            name,
            display_name,
            about,
            picture,
            banner,
            website,
            nip05,
            created_at: row.get::<i64, _>("created_at") as u64,
            received_at: chrono::DateTime::parse_from_rfc3339(
                &row.get::<String, _>("received_at"),
            )?
            .into(),
        })
    }

    /// Check if an event already exists
    pub async fn event_exists(&self, event_id: &str) -> Result<bool> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM raw_events WHERE event_id = ?")
            .bind(event_id)
            .fetch_one(&self.pool)
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    /// Clean up duplicate mints by normalizing URLs and merging data
    pub async fn cleanup_duplicate_mints(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::database",
            "🧹 Starting mint URL normalization and duplicate cleanup"
        );

        // Get all mint events from the database
        let mint_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind IN (38172, 38173)
            ORDER BY created_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut normalized_mints: std::collections::HashMap<String, Vec<Mint>> =
            std::collections::HashMap::new();
        let mut processed_count = 0;
        let mut normalized_count = 0;

        // Parse and group mints by normalized URL
        for row in mint_rows {
            if let Ok(mint) = self.parse_mint_event(&row).await {
                let normalized_url = mint.mint_url.clone(); // Already normalized in parse_mint_event

                if let Some(original_url) = mint.invite_codes.first() {
                    if normalized_url != *original_url {
                        normalized_count += 1;
                    }
                }

                normalized_mints
                    .entry(normalized_url)
                    .or_insert_with(Vec::new)
                    .push(mint);

                processed_count += 1;
            }
        }

        let total_unique_mints = normalized_mints.len();
        let duplicate_count = processed_count - total_unique_mints;

        if duplicate_count > 0 {
            info!(
                target: "bitcoinmints_retyr::database",
                duplicates = duplicate_count,
                total = processed_count,
                unique = total_unique_mints,
                "📊 Found duplicate mints"
            );
        } else {
            info!(
                target: "bitcoinmints_retyr::database",
                "✅ No duplicate mints found after normalization"
            );
        }

        info!(
            target: "bitcoinmints_retyr::database",
            processed = processed_count,
            normalized = normalized_count,
            duplicates = duplicate_count,
            "✨ Mint cleanup completed"
        );

        Ok(())
    }

    /// Get mint statistics for debugging
    pub async fn get_mint_statistics(&self) -> Result<serde_json::Value> {
        // Get total mint events
        let total_events =
            sqlx::query("SELECT COUNT(*) as count FROM raw_events WHERE kind IN (38172, 38173)")
                .fetch_one(&self.pool)
                .await?
                .get::<i64, _>("count");

        // Parse all mints and count unique URLs
        let mint_rows = sqlx::query(
            r#"
            SELECT id, event_id, kind, pubkey, content, tags, sig, created_at, received_at
            FROM raw_events 
            WHERE kind IN (38172, 38173)
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut unique_urls = std::collections::HashSet::new();
        let mut parsed_count = 0;

        for row in mint_rows {
            if let Ok(mint) = self.parse_mint_event(&row).await {
                unique_urls.insert(mint.mint_url);
                parsed_count += 1;
            }
        }

        Ok(serde_json::json!({
            "total_mint_events": total_events,
            "successfully_parsed": parsed_count,
            "unique_mint_urls": unique_urls.len(),
            "duplicate_events": total_events - unique_urls.len() as i64,
            "sample_urls": unique_urls.iter().take(10).collect::<Vec<_>>()
        }))
    }

    /// Store mint info from /v1/info endpoint with health tracking
    pub async fn store_mint_info(
        &self,
        mint_url: &str,
        mint_info_json: Option<&str>,
        error: Option<&str>,
    ) -> Result<()> {
        let now = Utc::now();
        let info_json = mint_info_json.unwrap_or("{}");
        let success = mint_info_json.is_some();

        // Get existing record to update health counters
        let existing = self.get_stored_mint_info(mint_url).await?;

        let (
            consecutive_failures,
            consecutive_successes,
            total_attempts,
            total_successes,
            first_seen_at,
            health_score,
        ) = if let Some(existing_info) = existing {
            let new_total_attempts = existing_info.total_attempts + 1;
            let new_total_successes = if success {
                existing_info.total_successes + 1
            } else {
                existing_info.total_successes
            };

            let (new_consecutive_failures, new_consecutive_successes) = if success {
                (0, existing_info.consecutive_successes + 1)
            } else {
                (existing_info.consecutive_failures + 1, 0)
            };

            // Calculate health score based on recent success rate
            let new_health_score = if new_total_attempts > 0 {
                new_total_successes as f64 / new_total_attempts as f64
            } else {
                1.0
            };

            (
                new_consecutive_failures,
                new_consecutive_successes,
                new_total_attempts,
                new_total_successes,
                existing_info.first_seen_at,
                new_health_score,
            )
        } else {
            // First time seeing this mint
            let initial_successes = if success { 1 } else { 0 };
            let initial_failures = if success { 0 } else { 1 };
            let initial_health_score = if success { 1.0 } else { 0.0 };

            (
                initial_failures,
                initial_successes,
                1,
                initial_successes,
                now,
                initial_health_score,
            )
        };

        let id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO mint_info 
            (id, mint_url, info_json, last_fetched_at, fetch_success, error_message, created_at, updated_at,
             consecutive_failures, consecutive_successes, total_attempts, total_successes, 
             first_seen_at, health_score, is_currently_online)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(mint_url)
        .bind(info_json)
        .bind(now.to_rfc3339())
        .bind(success)
        .bind(error)
        .bind(now.to_rfc3339())
        .bind(now.to_rfc3339())
        .bind(consecutive_failures)
        .bind(consecutive_successes)
        .bind(total_attempts)
        .bind(total_successes)
        .bind(first_seen_at.to_rfc3339())
        .bind(health_score)
        .bind(success)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all mint URLs that need info fetching (or haven't been fetched recently)
    pub async fn get_mints_needing_info_fetch(&self, max_age_hours: i64) -> Result<Vec<String>> {
        // First, get all unique mint URLs from parsed mint events
        let mint_rows = sqlx::query(
            r#"
            SELECT DISTINCT tags, content
            FROM raw_events 
            WHERE kind IN (38172, 38173)
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut mint_urls = Vec::new();

        for row in mint_rows {
            // Quick parse to extract mint URL without full parsing
            if let Ok(mint_url) = self.extract_mint_url_from_row(&row).await {
                mint_urls.push(mint_url);
            }
        }

        // Remove duplicates
        mint_urls.sort();
        mint_urls.dedup();

        // Filter out URLs that have been fetched recently
        let cutoff_time = Utc::now() - chrono::Duration::hours(max_age_hours);

        let mut urls_needing_fetch = Vec::new();

        for url in mint_urls {
            let needs_fetch = sqlx::query(
                r#"
                SELECT COUNT(*) as count FROM mint_info 
                WHERE mint_url = ? 
                AND last_fetched_at > ? 
                AND fetch_success = TRUE
                "#,
            )
            .bind(&url)
            .bind(cutoff_time.to_rfc3339())
            .fetch_one(&self.pool)
            .await?
            .get::<i64, _>("count")
                == 0;

            if needs_fetch {
                urls_needing_fetch.push(url);
            }
        }

        Ok(urls_needing_fetch)
    }

    /// Get stored mint info for a URL
    pub async fn get_stored_mint_info(
        &self,
        mint_url: &str,
    ) -> Result<Option<crate::models::StoredMintInfo>> {
        let row = sqlx::query(
            r#"
            SELECT id, mint_url, info_json, last_fetched_at, fetch_success, error_message, created_at, updated_at,
                   consecutive_failures, consecutive_successes, total_attempts, total_successes, 
                   first_seen_at, health_score, is_currently_online
            FROM mint_info
            WHERE mint_url = ?
            "#,
        )
        .bind(mint_url)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            Ok(Some(crate::models::StoredMintInfo {
                id: row.get("id"),
                mint_url: row.get("mint_url"),
                info_json: row.get("info_json"),
                last_fetched_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("last_fetched_at"),
                )?
                .into(),
                fetch_success: row.get("fetch_success"),
                error_message: row.get("error_message"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .into(),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )?
                .into(),
                consecutive_failures: row
                    .get::<Option<i32>, _>("consecutive_failures")
                    .unwrap_or(0),
                consecutive_successes: row
                    .get::<Option<i32>, _>("consecutive_successes")
                    .unwrap_or(0),
                total_attempts: row.get::<Option<i32>, _>("total_attempts").unwrap_or(0),
                total_successes: row.get::<Option<i32>, _>("total_successes").unwrap_or(0),
                first_seen_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<Option<String>, _>("first_seen_at")
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                )?
                .into(),
                health_score: row.get::<Option<f64>, _>("health_score").unwrap_or(1.0),
                is_currently_online: row
                    .get::<Option<bool>, _>("is_currently_online")
                    .unwrap_or(true),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all stored mint info
    pub async fn get_all_stored_mint_info(&self) -> Result<Vec<crate::models::StoredMintInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT id, mint_url, info_json, last_fetched_at, fetch_success, error_message, created_at, updated_at,
                   consecutive_failures, consecutive_successes, total_attempts, total_successes, 
                   first_seen_at, health_score, is_currently_online
            FROM mint_info
            ORDER BY last_fetched_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut stored_infos = Vec::new();
        for row in rows {
            stored_infos.push(crate::models::StoredMintInfo {
                id: row.get("id"),
                mint_url: row.get("mint_url"),
                info_json: row.get("info_json"),
                last_fetched_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("last_fetched_at"),
                )?
                .into(),
                fetch_success: row.get("fetch_success"),
                error_message: row.get("error_message"),
                created_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("created_at"),
                )?
                .into(),
                updated_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<String, _>("updated_at"),
                )?
                .into(),
                consecutive_failures: row
                    .get::<Option<i32>, _>("consecutive_failures")
                    .unwrap_or(0),
                consecutive_successes: row
                    .get::<Option<i32>, _>("consecutive_successes")
                    .unwrap_or(0),
                total_attempts: row.get::<Option<i32>, _>("total_attempts").unwrap_or(0),
                total_successes: row.get::<Option<i32>, _>("total_successes").unwrap_or(0),
                first_seen_at: chrono::DateTime::parse_from_rfc3339(
                    &row.get::<Option<String>, _>("first_seen_at")
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339()),
                )?
                .into(),
                health_score: row.get::<Option<f64>, _>("health_score").unwrap_or(1.0),
                is_currently_online: row
                    .get::<Option<bool>, _>("is_currently_online")
                    .unwrap_or(true),
            });
        }

        Ok(stored_infos)
    }

    /// Extract mint URL from database row without full parsing
    async fn extract_mint_url_from_row(&self, row: &sqlx::sqlite::SqliteRow) -> Result<String> {
        let tags_json: String = row.get("tags");
        let tags: Vec<nostr_sdk::Tag> = serde_json::from_str(&tags_json)?;

        // Find the first 'u' tag (invite code/URL)
        for tag in tags {
            let tag_vec = tag.to_vec();
            if tag_vec.len() >= 2 && tag_vec[0] == "u" {
                return normalize_mint_url(&tag_vec[1]).or_else(|_| Ok(tag_vec[1].clone()));
            }
        }

        anyhow::bail!("No mint URL found in tags");
    }

    /// Get mint health summary for API responses
    pub async fn get_mint_health_summary(
        &self,
        mint_url: &str,
    ) -> Result<Option<crate::models::MintHealthSummary>> {
        let stored_info = self.get_stored_mint_info(mint_url).await?;

        if let Some(info) = stored_info {
            // Get 24h statistics from health_records if available
            let cutoff_24h = Utc::now() - chrono::Duration::hours(24);

            let health_stats = sqlx::query(
                r#"
                SELECT 
                    COUNT(*) as total_checks,
                    SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful_checks,
                    AVG(response_time_ms) as avg_response_time
                FROM health_records 
                WHERE mint_url = ? AND checked_at > ?
                "#,
            )
            .bind(mint_url)
            .bind(cutoff_24h.to_rfc3339())
            .fetch_one(&self.pool)
            .await
            .ok();

            let (total_checks_24h, successful_checks_24h, avg_response_time_24h) =
                if let Some(stats) = health_stats {
                    (
                        stats.get::<i64, _>("total_checks") as i32,
                        stats.get::<i64, _>("successful_checks") as i32,
                        stats.get::<Option<f64>, _>("avg_response_time"),
                    )
                } else {
                    (0, 0, None)
                };

            // Get last online/offline times
            let last_online = if info.is_currently_online {
                Some(info.last_fetched_at)
            } else {
                sqlx::query(
                    r#"
                    SELECT checked_at FROM health_records 
                    WHERE mint_url = ? AND success = 1 
                    ORDER BY checked_at DESC LIMIT 1
                    "#,
                )
                .bind(mint_url)
                .fetch_optional(&self.pool)
                .await?
                .and_then(|row| {
                    chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("checked_at"))
                        .ok()
                        .map(|dt| dt.into())
                })
            };

            let last_offline = if !info.is_currently_online {
                Some(info.last_fetched_at)
            } else {
                sqlx::query(
                    r#"
                    SELECT checked_at FROM health_records 
                    WHERE mint_url = ? AND success = 0 
                    ORDER BY checked_at DESC LIMIT 1
                    "#,
                )
                .bind(mint_url)
                .fetch_optional(&self.pool)
                .await?
                .and_then(|row| {
                    chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("checked_at"))
                        .ok()
                        .map(|dt| dt.into())
                })
            };

            Ok(Some(crate::models::MintHealthSummary {
                mint_url: info.mint_url,
                is_online: info.is_currently_online,
                uptime_percentage: info.health_score * 100.0,
                last_online,
                last_offline,
                consecutive_failures: info.consecutive_failures,
                total_checks_24h,
                successful_checks_24h,
                average_response_time_24h: avg_response_time_24h,
            }))
        } else {
            Ok(None)
        }
    }

    /// Store a detailed health record
    pub async fn store_health_record(
        &self,
        mint_url: &str,
        success: bool,
        response_time_ms: Option<i64>,
        error_message: Option<&str>,
        http_status: Option<u16>,
    ) -> Result<()> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO health_records 
            (id, mint_url, checked_at, success, response_time_ms, error_message, http_status)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(mint_url)
        .bind(now.to_rfc3339())
        .bind(success)
        .bind(response_time_ms)
        .bind(error_message)
        .bind(http_status.map(|s| s as i32))
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all mint health summaries for dashboard
    pub async fn get_all_mint_health_summaries(
        &self,
    ) -> Result<Vec<crate::models::MintHealthSummary>> {
        let all_infos = self.get_all_stored_mint_info().await?;
        let mut summaries = Vec::new();

        for info in all_infos {
            if let Ok(Some(summary)) = self.get_mint_health_summary(&info.mint_url).await {
                summaries.push(summary);
            }
        }

        Ok(summaries)
    }

    /// Cleanup old health records to prevent database bloat (keep last 30 days)
    pub async fn cleanup_old_health_records(&self) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::days(30);

        let deleted_count = sqlx::query(
            r#"
            DELETE FROM health_records 
            WHERE checked_at < ?
            "#,
        )
        .bind(cutoff.to_rfc3339())
        .execute(&self.pool)
        .await?
        .rows_affected();

        if deleted_count > 0 {
            info!(
                target: "bitcoinmints_retyr::database",
                deleted_records = deleted_count,
                "🧹 Cleaned up old health records"
            );
        }

        Ok(())
    }
}
