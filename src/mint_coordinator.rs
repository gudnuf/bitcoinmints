use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::database::Database;
use crate::fedimint_service::FedimintService;
use crate::mint_info_service::MintInfoService;
use crate::models::*;

/// Service that coordinates between different mint data sources
/// and provides a unified interface for both Cashu and Fedimint mints
#[derive(Clone)]
pub struct MintCoordinator {
    database: Database,
    mint_info_service: MintInfoService,
    fedimint_service: FedimintService,
}

impl MintCoordinator {
    pub fn new(
        database: Database,
        mint_info_service: MintInfoService,
        fedimint_service: FedimintService,
    ) -> Self {
        Self {
            database,
            mint_info_service,
            fedimint_service,
        }
    }

    /// Get unified mint data with proper type separation and enrichment
    pub async fn get_unified_mints_with_recommendations(
        &self,
        query_params: &MintQueryParams,
    ) -> Result<Vec<UnifiedMintWithRecommendations>> {
        info!(
            target: "bitcoinmints_retyr::mint_coordinator",
            "🔄 Fetching unified mints with recommendations"
        );

        // Get raw mints from database (without the complex enrichment)
        let raw_mints = self
            .database
            .get_mints_with_recommendations(query_params.mint_type.as_deref())
            .await?;

        let mut unified_mints = Vec::new();

        for mint_with_recs in raw_mints {
            let mint = &mint_with_recs.mint;

            // Create unified mint data based on type
            let unified_mint_data = match mint.mint_type.as_str() {
                "cashu" => self.create_cashu_mint_data(mint).await?,
                "fedimint" => self.create_fedimint_mint_data(mint).await?,
                _ => {
                    warn!(
                        target: "bitcoinmints_retyr::mint_coordinator",
                        mint_type = %mint.mint_type,
                        mint_name = %mint.name,
                        "⚠️ Unknown mint type, skipping"
                    );
                    continue;
                }
            };

            // Apply filters if specified
            if !self.passes_filters(&unified_mint_data, query_params) {
                continue;
            }

            unified_mints.push(UnifiedMintWithRecommendations {
                mint_data: unified_mint_data,
                recommendations: mint_with_recs.recommendations,
                total_recommendations: mint_with_recs.total_recommendations,
                average_rating: mint_with_recs.average_rating,
            });
        }

        // Group fedimint mints by federation_id to avoid duplicates
        unified_mints = self.group_fedimint_mints(unified_mints).await?;

        info!(
            target: "bitcoinmints_retyr::mint_coordinator",
            total_unified_mints = unified_mints.len(),
            "✅ Successfully created unified mints"
        );

        Ok(unified_mints)
    }

    /// Create Cashu mint data with proper enrichment
    async fn create_cashu_mint_data(&self, mint: &Mint) -> Result<UnifiedMintData> {
        debug!(
            target: "bitcoinmints_retyr::mint_coordinator",
            mint_url = %mint.mint_url,
            "🟠 Creating Cashu mint data"
        );

        // Get stored mint info if available
        let stored_info = self.database.get_stored_mint_info(&mint.mint_url).await?;

        let mut mint_info = None;
        let mut version = None;
        let mut is_online = true;
        let mut health_info = None;
        let mut last_updated = None;

        if let Some(stored) = &stored_info {
            is_online = stored.is_currently_online;
            last_updated = Some(stored.last_fetched_at);

            health_info = Some(MintHealthInfo {
                health_score: stored.health_score,
                uptime_percentage: stored.health_score * 100.0,
                consecutive_failures: stored.consecutive_failures,
                consecutive_successes: stored.consecutive_successes,
                total_attempts: stored.total_attempts,
                total_successes: stored.total_successes,
                last_check: stored.last_fetched_at,
            });

            if stored.fetch_success && !stored.info_json.is_empty() {
                if let Ok(parsed_info) = serde_json::from_str::<FlexibleMintInfo>(&stored.info_json)
                {
                    version = parsed_info.version.clone();
                    mint_info = Some(parsed_info);
                }
            }
        }

        // Extract supported currencies from nuts
        let supported_currencies = self.extract_supported_currencies(&mint.nuts);

        let cashu_data = CashuMintData {
            mint_url: mint.mint_url.clone(),
            mint_pubkey: mint.mint_pubkey.clone(),
            nuts: mint.nuts.clone(),
            mint_info,
            version,
            supported_currencies,
        };

        Ok(UnifiedMintData {
            event_id: mint.event_id.clone(),
            mint_id: mint.mint_url.clone(),
            name: mint.name.clone(),
            description: mint.description.clone(),
            mint_type: MintType::Cashu,
            networks: mint.networks.clone(),
            author_pubkey: mint.author_pubkey.clone(),
            created_at: mint.created_at,
            received_at: mint.received_at,
            cashu_data: Some(cashu_data),
            fedimint_data: None,
            health_info,
            is_online,
            last_updated,
        })
    }

    /// Create Fedimint mint data with proper enrichment
    async fn create_fedimint_mint_data(&self, mint: &Mint) -> Result<UnifiedMintData> {
        debug!(
            target: "bitcoinmints_retyr::mint_coordinator",
            mint_name = %mint.name,
            invite_codes = ?mint.invite_codes,
            "🔵 Creating Fedimint mint data"
        );

        // Try to find federation info for any of the invite codes
        let federation_enrichment = self.get_federation_enrichment(&mint.invite_codes).await;

        let (
            federation_id,
            federation_name,
            modules,
            guardians_count,
            invite_codes,
            welcome_message,
            config_available,
        ) = if let Some(enrichment) = &federation_enrichment {
            (
                enrichment.federation_id.clone(),
                enrichment.federation_name.clone(),
                enrichment.modules.clone(),
                Some(enrichment.guardians_count),
                enrichment.invite_codes.clone(),
                enrichment
                    .meta
                    .get("welcome_message")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                enrichment.config_available,
            )
        } else {
            // No federation info found - use basic data from the event
            warn!(
                target: "bitcoinmints_retyr::mint_coordinator",
                mint_name = %mint.name,
                invite_codes = ?mint.invite_codes,
                "⚠️ No federation info found, using basic event data"
            );
            (
                format!("unknown-{}", mint.mint_pubkey),
                None,
                mint.modules.clone(),
                None,
                mint.invite_codes.clone(),
                None,
                false,
            )
        };

        let fedimint_data = FedimintMintData {
            federation_id: federation_id.clone(),
            federation_name: federation_name.clone(),
            invite_codes,
            modules,
            guardians_count,
            welcome_message,
            config_available,
        };

        // Use federation name if available, otherwise use event name
        let display_name = federation_name.unwrap_or_else(|| mint.name.clone());

        Ok(UnifiedMintData {
            event_id: mint.event_id.clone(),
            mint_id: federation_id,
            name: display_name,
            description: mint.description.clone(),
            mint_type: MintType::Fedimint,
            networks: mint.networks.clone(),
            author_pubkey: mint.author_pubkey.clone(),
            created_at: mint.created_at,
            received_at: mint.received_at,
            cashu_data: None,
            fedimint_data: Some(fedimint_data),
            health_info: None, // Fedimint doesn't have HTTP health checks
            is_online: config_available,
            last_updated: None,
        })
    }

    /// Get federation enrichment data for invite codes
    async fn get_federation_enrichment(
        &self,
        invite_codes: &[String],
    ) -> Option<FederationEnrichmentResult> {
        for invite_code in invite_codes {
            // Try to get federation_id for this invite code
            if let Ok(Some(federation_id)) = self
                .database
                .get_federation_id_for_invite_code(invite_code)
                .await
            {
                // Get federation info by federation_id
                if let Ok(Some(federation_info)) = self
                    .database
                    .get_stored_federation_info(&federation_id)
                    .await
                {
                    if federation_info.fetch_success && !federation_info.config_json.is_empty() {
                        // Get all invite codes for this federation
                        let all_invite_codes = self
                            .database
                            .get_invite_codes_for_federation(&federation_id)
                            .await
                            .unwrap_or_default();

                        // Extract metadata
                        let mut meta = HashMap::new();
                        if let Some(federation_name) = &federation_info.federation_name {
                            meta.insert(
                                "federation_name".to_string(),
                                serde_json::Value::String(federation_name.clone()),
                            );
                        }
                        if let Some(welcome_message) = &federation_info.welcome_message {
                            meta.insert(
                                "welcome_message".to_string(),
                                serde_json::Value::String(welcome_message.clone()),
                            );
                        }

                        return Some(FederationEnrichmentResult {
                            federation_id,
                            federation_name: federation_info.federation_name,
                            modules: federation_info.modules,
                            guardians_count: federation_info.guardians_count,
                            invite_codes: all_invite_codes,
                            meta,
                            config_available: true,
                        });
                    }
                }
            }
        }

        None
    }

    /// Extract supported currencies from Cashu nuts
    fn extract_supported_currencies(&self, nuts: &cdk::nuts::Nuts) -> Vec<String> {
        let mut currencies = std::collections::HashSet::new();

        // From minting methods (NUT-04)
        for method in &nuts.nut04.methods {
            currencies.insert(format!("{:?}", method.unit).to_lowercase());
        }

        // From melting methods (NUT-05)
        for method in &nuts.nut05.methods {
            currencies.insert(format!("{:?}", method.unit).to_lowercase());
        }

        let mut result: Vec<String> = currencies.into_iter().collect();
        result.sort();
        result
    }

    /// Check if mint passes the specified filters
    fn passes_filters(&self, mint_data: &UnifiedMintData, query_params: &MintQueryParams) -> bool {
        // Type filter
        if let Some(type_filter) = &query_params.mint_type {
            let expected_type = match type_filter.as_str() {
                "cashu" => MintType::Cashu,
                "fedimint" => MintType::Fedimint,
                _ => return true, // Unknown filter, don't filter
            };
            if mint_data.mint_type != expected_type {
                return false;
            }
        }

        // Cashu-specific filters
        if mint_data.mint_type == MintType::Cashu {
            if let Some(cashu_data) = &mint_data.cashu_data {
                // Currency filters
                if let Some(minting) = &query_params.minting {
                    let required: Vec<&str> = minting.split(',').map(|s| s.trim()).collect();
                    if !required
                        .iter()
                        .all(|&curr| cashu_data.supported_currencies.contains(&curr.to_string()))
                    {
                        return false;
                    }
                }

                if let Some(melting) = &query_params.melting {
                    let required: Vec<&str> = melting.split(',').map(|s| s.trim()).collect();
                    if !required
                        .iter()
                        .all(|&curr| cashu_data.supported_currencies.contains(&curr.to_string()))
                    {
                        return false;
                    }
                }

                // NUT filters
                if let Some(nuts) = &query_params.nuts {
                    let required: Vec<&str> = nuts.split(',').map(|s| s.trim()).collect();
                    let supported_nuts =
                        crate::ui::components::mint_card::get_supported_nuts(&cashu_data.nuts);
                    if !required
                        .iter()
                        .all(|&nut| supported_nuts.contains(&nut.to_string()))
                    {
                        return false;
                    }
                }
            }
        }

        // Fedimint-specific filters
        if mint_data.mint_type == MintType::Fedimint {
            if let Some(fedimint_data) = &mint_data.fedimint_data {
                // Module filters
                if let Some(modules) = &query_params.modules {
                    let required: Vec<&str> = modules.split(',').map(|s| s.trim()).collect();
                    if !required
                        .iter()
                        .all(|&module| fedimint_data.modules.contains(&module.to_string()))
                    {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Group fedimint mints by federation_id to avoid duplicates
    async fn group_fedimint_mints(
        &self,
        mints: Vec<UnifiedMintWithRecommendations>,
    ) -> Result<Vec<UnifiedMintWithRecommendations>> {
        let mut grouped_mints = Vec::new();
        let mut fedimint_federations: HashMap<String, UnifiedMintWithRecommendations> =
            HashMap::new();

        for mint in mints {
            if mint.mint_data.mint_type == MintType::Fedimint {
                let federation_id = mint.mint_data.mint_id.clone();

                if let Some(existing) = fedimint_federations.get_mut(&federation_id) {
                    // Merge recommendations
                    existing.recommendations.extend(mint.recommendations);
                    existing.total_recommendations += mint.total_recommendations;

                    // Recalculate average rating
                    let ratings: Vec<i32> = existing
                        .recommendations
                        .iter()
                        .filter_map(|r| r.recommendation.rating)
                        .collect();
                    existing.average_rating = if !ratings.is_empty() {
                        let sum: i32 = ratings.iter().sum();
                        Some(sum as f64 / ratings.len() as f64)
                    } else {
                        None
                    };
                } else {
                    fedimint_federations.insert(federation_id, mint);
                }
            } else {
                // Non-fedimint mints (Cashu) go directly to the result
                grouped_mints.push(mint);
            }
        }

        // Add all grouped federations
        grouped_mints.extend(fedimint_federations.into_values());

        Ok(grouped_mints)
    }
}
