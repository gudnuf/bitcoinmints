use anyhow::{Context, Result};
use fedimint_api_client::api::net::Connector;
use fedimint_core::config::{ClientConfig, FederationId};
use fedimint_core::invite_code::InviteCode;
use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::database::Database;

/// Service for fetching and managing fedimint federation information
#[derive(Clone)]
pub struct FedimintService {
    database: Database,
    connector: Connector,
}

/// Fedimint federation information extracted from client config
#[derive(Debug, Clone)]
pub struct FederationInfo {
    pub federation_id: FederationId,
    pub invite_codes: Vec<String>,
    pub modules: Vec<String>,
    pub config: ClientConfig,
    pub guardians_count: usize,
    pub meta: HashMap<String, serde_json::Value>,
}

impl FedimintService {
    /// Create a new FedimintService
    pub async fn new(database: Database) -> Result<Self> {
        let connector = Connector::Tcp;

        Ok(Self {
            database,
            connector,
        })
    }

    /// Start the background service that periodically fetches federation configs
    pub async fn start(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::fedimint",
            "🔄 Starting fedimint federation config fetching service"
        );

        // Initial fetch
        self.fetch_all_federation_configs().await?;

        // Set up periodic fetching (every 6 hours)
        let mut fetch_interval = interval(Duration::from_secs(6 * 3600));

        loop {
            tokio::select! {
                _ = fetch_interval.tick() => {
                    if let Err(e) = self.fetch_all_federation_configs().await {
                        error!(
                            target: "bitcoinmints_retyr::fedimint",
                            error = %e,
                            "❌ Error in periodic federation config fetch"
                        );
                    }
                }
            }
        }
    }

    /// Fetch federation configs for all fedimint invite codes
    pub async fn fetch_all_federation_configs(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::fedimint",
            "🔍 Starting federation config fetch cycle"
        );

        // Get all fedimint invite codes from raw events
        let invite_codes = self.get_fedimint_invite_codes().await?;
        let total_count = invite_codes.len();

        info!(
            target: "bitcoinmints_retyr::fedimint",
            total_invites = total_count,
            "📊 Found fedimint invite codes"
        );

        let mut success_count = 0;
        let mut error_count = 0;
        let mut federation_infos: HashMap<FederationId, FederationInfo> = HashMap::new();

        for invite_code in invite_codes {
            match self.fetch_federation_config(&invite_code).await {
                Ok(fed_info) => {
                    success_count += 1;
                    info!(
                        target: "bitcoinmints_retyr::fedimint",
                        invite_code = %invite_code,
                        federation_id = %fed_info.federation_id,
                        "✅ Successfully fetched federation config"
                    );

                    // Group by federation ID - multiple invite codes can point to same federation
                    if let Some(existing) = federation_infos.get_mut(&fed_info.federation_id) {
                        // Add this invite code to existing federation
                        if !existing.invite_codes.contains(&invite_code) {
                            existing.invite_codes.push(invite_code);
                        }
                    } else {
                        // New federation
                        federation_infos.insert(fed_info.federation_id, fed_info);
                    }
                }
                Err(e) => {
                    error_count += 1;
                    warn!(
                        target: "bitcoinmints_retyr::fedimint",
                        invite_code = %invite_code,
                        error = %e,
                        "⚠️ Failed to fetch federation config"
                    );
                }
            }

            // Add a small delay between requests to be respectful
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        // Store the federation info in the database
        for fed_info in federation_infos.values() {
            if let Err(e) = self.store_federation_info(fed_info).await {
                error!(
                    target: "bitcoinmints_retyr::fedimint",
                    federation_id = %fed_info.federation_id,
                    error = %e,
                    "❌ Failed to store federation info"
                );
            }
        }

        info!(
            target: "bitcoinmints_retyr::fedimint",
            successful = success_count,
            errors = error_count,
            unique_federations = federation_infos.len(),
            "✨ Federation config fetch cycle completed"
        );
        Ok(())
    }

    /// Fetch federation config for a specific invite code
    pub async fn fetch_federation_config(&self, invite_code: &str) -> Result<FederationInfo> {
        let invite =
            InviteCode::from_str(invite_code).context("Failed to parse fedimint invite code")?;

        info!(
            target: "bitcoinmints_retyr::fedimint",
            invite_code = %invite_code,
            "🔍 Fetching federation config for invite code: {}", invite_code
        );

        // Use the proper fedimint API to download the config
        let client_config = self
            .connector
            .download_from_invite_code(&invite)
            .await
            .context(format!(
                "Failed to download federation config for invite: {}",
                invite_code
            ))?;

        // Calculate federation ID
        let federation_id = client_config.calculate_federation_id();

        // Extract modules from the config
        let modules: Vec<String> = client_config
            .modules
            .keys()
            .map(|id| format!("{}", id))
            .collect();

        // Extract metadata
        let mut meta = HashMap::new();
        if let Ok(Some(federation_name)) = client_config.meta::<String>("federation_name") {
            meta.insert(
                "federation_name".to_string(),
                serde_json::Value::String(federation_name),
            );
        }
        if let Ok(Some(welcome_message)) = client_config.meta::<String>("welcome_message") {
            meta.insert(
                "welcome_message".to_string(),
                serde_json::Value::String(welcome_message),
            );
        }

        // Count guardians
        let guardians_count = client_config.global.api_endpoints.len();

        Ok(FederationInfo {
            federation_id,
            invite_codes: vec![invite_code.to_string()],
            modules,
            config: client_config,
            guardians_count,
            meta,
        })
    }

    /// Get all fedimint invite codes from the database
    async fn get_fedimint_invite_codes(&self) -> Result<Vec<String>> {
        let raw_events = self.database.get_all_raw_events().await?;
        let mut invite_codes = Vec::new();

        for event in raw_events {
            // Only process fedimint events (kind 38173)
            if event.kind == crate::models::FEDIMINT_KIND {
                let tags: Vec<nostr_sdk::Tag> = serde_json::from_str(&event.tags)?;

                for tag in tags {
                    let tag_vec = tag.to_vec();
                    if tag_vec.len() >= 2 && tag_vec[0] == "u" {
                        // This is an invite code/URL tag
                        let invite_code = &tag_vec[1];
                        if invite_code.starts_with("fed11") {
                            invite_codes.push(invite_code.clone());
                        }
                    }
                }
            }
        }

        // Remove duplicates
        invite_codes.sort();
        invite_codes.dedup();

        Ok(invite_codes)
    }

    /// Store federation information in the database
    async fn store_federation_info(&self, fed_info: &FederationInfo) -> Result<()> {
        let config_json = serde_json::to_string(&fed_info.config)
            .context("Failed to serialize federation config")?;

        let federation_name = fed_info
            .meta
            .get("federation_name")
            .and_then(|v| v.as_str());
        let welcome_message = fed_info
            .meta
            .get("welcome_message")
            .and_then(|v| v.as_str());

        // Store federation info using the new schema
        self.database
            .store_federation_info(
                &fed_info.federation_id.to_string(),
                &config_json,
                fed_info.guardians_count,
                &fed_info.modules,
                federation_name,
                welcome_message,
                &fed_info.invite_codes,
                None, // no error
            )
            .await
            .context("Failed to store federation config")?;

        Ok(())
    }

    /// Get stored federation info for an invite code
    pub async fn get_stored_federation_info(
        &self,
        invite_code: &str,
    ) -> Result<Option<FederationInfo>> {
        // First, get the federation_id for this invite code
        if let Some(federation_id) = self
            .database
            .get_federation_id_for_invite_code(invite_code)
            .await?
        {
            // Then get the federation info by federation_id
            if let Some(stored_info) = self
                .database
                .get_stored_federation_info(&federation_id)
                .await?
            {
                if stored_info.fetch_success && !stored_info.config_json.is_empty() {
                    let client_config: ClientConfig =
                        serde_json::from_str(&stored_info.config_json)
                            .context("Failed to parse stored federation config")?;

                    let federation_id = client_config.calculate_federation_id();

                    // Get all invite codes for this federation
                    let invite_codes = self
                        .database
                        .get_invite_codes_for_federation(&federation_id.to_string())
                        .await?;

                    // Extract metadata
                    let mut meta = HashMap::new();
                    if let Some(federation_name) = &stored_info.federation_name {
                        meta.insert(
                            "federation_name".to_string(),
                            serde_json::Value::String(federation_name.clone()),
                        );
                    }
                    if let Some(welcome_message) = &stored_info.welcome_message {
                        meta.insert(
                            "welcome_message".to_string(),
                            serde_json::Value::String(welcome_message.clone()),
                        );
                    }

                    return Ok(Some(FederationInfo {
                        federation_id,
                        invite_codes,
                        modules: stored_info.modules,
                        config: client_config,
                        guardians_count: stored_info.guardians_count,
                        meta,
                    }));
                }
            }
        }

        Ok(None)
    }
}
