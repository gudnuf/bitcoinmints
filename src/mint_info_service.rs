use anyhow::{Context, Result};
use reqwest::Client;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};

use crate::database::Database;
use crate::models::FlexibleMintInfo;

/// Service for fetching and managing mint information
#[derive(Clone)]
pub struct MintInfoService {
    database: Database,
    http_client: Client,
}

impl MintInfoService {
    /// Create a new MintInfoService
    pub async fn new(database: Database) -> Result<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("bitcoinmints-retyr/0.1.0")
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            database,
            http_client,
        })
    }

    /// Start the background service that periodically fetches mint info
    pub async fn start(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::mint_info",
            "🔄 Starting mint info fetching service"
        );

        // Initial fetch
        self.fetch_all_mint_info().await?;

        // Set up periodic fetching (every 6 hours)
        let mut fetch_interval = interval(Duration::from_secs(6 * 3600));

        loop {
            fetch_interval.tick().await;

            if let Err(e) = self.fetch_all_mint_info().await {
                error!(
                    target: "bitcoinmints_retyr::mint_info",
                    error = %e,
                    "❌ Error in periodic mint info fetch"
                );
            }
        }
    }

    /// Fetch mint info for all mints that need updating
    pub async fn fetch_all_mint_info(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::mint_info",
            "🔍 Starting mint info fetch cycle"
        );

        // Get mints that need info fetching (not fetched in last 4 hours)
        //  TODO: refetch sooner
        let all_mint_urls = self.database.get_mints_needing_info_fetch(1).await?;
        let total_count = all_mint_urls.len();

        // Filter out Fedimint federation invite codes (URLs starting with https://fed11)
        let mint_urls: Vec<String> = all_mint_urls
            .into_iter()
            .filter(|url| !url.starts_with("https://fed11"))
            .collect();

        let filtered_out = total_count - mint_urls.len();

        info!(
            target: "bitcoinmints_retyr::mint_info",
            total_mints = mint_urls.len(),
            filtered_fed11 = filtered_out,
            "📊 Found mints needing info fetch"
        );

        let mut success_count = 0;
        let mut error_count = 0;

        for mint_url in mint_urls {
            match self.fetch_mint_info(&mint_url).await {
                Ok(_) => {
                    success_count += 1;
                    info!(
                        target: "bitcoinmints_retyr::mint_info",
                        mint_url = %mint_url,
                        "✅ Successfully fetched mint info"
                    );
                }
                Err(e) => {
                    error_count += 1;
                    warn!(
                        target: "bitcoinmints_retyr::mint_info",
                        mint_url = %mint_url,
                        error = %e,
                        "⚠️ Failed to fetch mint info"
                    );

                    // Store the error in the database
                    if let Err(store_err) = self
                        .database
                        .store_mint_info(&mint_url, None, Some(&e.to_string()))
                        .await
                    {
                        error!(
                            target: "bitcoinmints_retyr::mint_info",
                            mint_url = %mint_url,
                            error = %store_err,
                            "❌ Failed to store error for mint"
                        );
                    }
                }
            }

            // Add a small delay between requests to be respectful
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        info!(
            target: "bitcoinmints_retyr::mint_info",
            successful = success_count,
            errors = error_count,
            "✨ Mint info fetch cycle completed"
        );
        Ok(())
    }

    /// Fetch mint info for a specific mint
    pub async fn fetch_mint_info(&self, mint_url: &str) -> Result<FlexibleMintInfo> {
        let info_url = format!("{}/v1/info", mint_url.trim_end_matches('/'));

        // Remove the individual fetch log since it's too verbose - the summary above is enough

        let response = self
            .http_client
            .get(&info_url)
            .send()
            .await
            .context(format!("Failed to send request to {}", info_url))?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error {} from {}", response.status(), info_url);
        }

        let response_text = response
            .text()
            .await
            .context(format!("Failed to get response text from {}", info_url))?;

        // Try to parse as our flexible mint info structure
        // TODO: make sure this doesn't forget any fields
        let mint_info: FlexibleMintInfo = serde_json::from_str(&response_text)
            .context(format!("Failed to parse JSON response from {}", info_url))?;

        // Store the successful result in the database (store the raw JSON)
        self.database
            .store_mint_info(mint_url, Some(&response_text), None)
            .await
            .context("Failed to store mint info in database")?;

        Ok(mint_info)
    }

    /// Get the stored mint info for a mint URL
    pub async fn get_stored_mint_info(
        &self,
        mint_url: &str,
    ) -> Result<Option<crate::models::StoredMintInfo>> {
        self.database.get_stored_mint_info(mint_url).await
    }

    /// Force refresh mint info for a specific mint (bypassing cache)
    pub async fn force_refresh_mint_info(&self, mint_url: &str) -> Result<FlexibleMintInfo> {
        info!(
            target: "bitcoinmints_retyr::mint_info",
            mint_url = %mint_url,
            "🔄 Force refreshing mint info"
        );
        self.fetch_mint_info(mint_url).await
    }
}
