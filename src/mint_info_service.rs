use anyhow::{Context, Result};
use reqwest::Client;
use std::net::IpAddr;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info, warn};
use url::Url;

use crate::database::Database;
use crate::models::FlexibleMintInfo;

/// Service for fetching and managing mint information
#[derive(Clone)]
pub struct MintInfoService {
    database: Database,
    http_client: Client,
}

impl MintInfoService {
    /// Check if a URL points to a public domain (not localhost or private networks)
    fn is_public_domain_url(url_str: &str) -> bool {
        // Parse the URL
        let url = match Url::parse(url_str) {
            Ok(url) => url,
            Err(_) => return false,
        };

        // Get the host
        let host = match url.host_str() {
            Some(host) => host,
            None => return false,
        };

        // Check for localhost patterns
        if host == "localhost"
            || host == "127.0.0.1"
            || host == "::1"
            || host.ends_with(".localhost")
            || host.starts_with("localhost:")
        {
            return false;
        }

        // Try to parse as IP address
        if let Ok(ip) = host.parse::<IpAddr>() {
            // Filter out private/reserved IP ranges
            let is_private = match ip {
                IpAddr::V4(ipv4) => {
                    ipv4.is_private()
                        || ipv4.is_loopback()
                        || ipv4.is_link_local()
                        || ipv4.is_broadcast()
                        || ipv4.is_documentation()
                }
                IpAddr::V6(ipv6) => {
                    ipv6.is_loopback() || ipv6.is_unicast_link_local() || ipv6.is_unique_local()
                }
            };

            if is_private {
                return false;
            }
        }

        // Check for private domain patterns
        if host.ends_with(".local")
            || host.ends_with(".internal")
            || host.ends_with(".corp")
            || host.contains("192.168.")
            || host.contains("10.")
            || host.starts_with("172.")
        {
            return false;
        }

        true
    }

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

        // Set up periodic cleanup (every 24 hours)
        let mut cleanup_interval = interval(Duration::from_secs(24 * 3600));

        loop {
            tokio::select! {
                _ = fetch_interval.tick() => {
                    if let Err(e) = self.fetch_all_mint_info().await {
                        error!(
                            target: "bitcoinmints_retyr::mint_info",
                            error = %e,
                            "❌ Error in periodic mint info fetch"
                        );
                    }
                }
                _ = cleanup_interval.tick() => {
                    if let Err(e) = self.database.cleanup_old_health_records().await {
                        error!(
                            target: "bitcoinmints_retyr::mint_info",
                            error = %e,
                            "❌ Error in periodic health record cleanup"
                        );
                    }
                }
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

        // Filter out Fedimint federation invite codes and localhost/private network URLs
        let mut fed11_filtered = 0;
        let mut localhost_filtered = 0;

        let mint_urls: Vec<String> = all_mint_urls
            .into_iter()
            .filter(|url| {
                if url.starts_with("https://fed11") {
                    fed11_filtered += 1;
                    false
                } else if !Self::is_public_domain_url(url) {
                    localhost_filtered += 1;
                    false
                } else {
                    true
                }
            })
            .collect();

        let total_filtered = fed11_filtered + localhost_filtered;

        info!(
            target: "bitcoinmints_retyr::mint_info",
            total_mints = mint_urls.len(),
            fed11_filtered = fed11_filtered,
            localhost_filtered = localhost_filtered,
            total_filtered = total_filtered,
            "📊 Found mints needing info fetch (filtered {} fedimint and {} localhost/private URLs)", fed11_filtered, localhost_filtered
        );

        let mut success_count = 0;
        let mut error_count = 0;

        for mint_url in mint_urls {
            match self.fetch_mint_info_with_retries(&mint_url).await {
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
                        "⚠️ Failed to fetch mint info after retries"
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

    /// Fetch mint info with retry logic and health tracking
    pub async fn fetch_mint_info_with_retries(&self, mint_url: &str) -> Result<FlexibleMintInfo> {
        const MAX_RETRIES: u32 = 3;
        const BASE_DELAY_MS: u64 = 1000; // Start with 1 second

        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            let start_time = std::time::Instant::now();

            match self.fetch_mint_info_attempt(mint_url).await {
                Ok(info) => {
                    let response_time = start_time.elapsed().as_millis() as i64;

                    // Store successful health record
                    if let Err(e) = self
                        .database
                        .store_health_record(mint_url, true, Some(response_time), None, None)
                        .await
                    {
                        warn!(
                            target: "bitcoinmints_retyr::mint_info",
                            mint_url = %mint_url,
                            error = %e,
                            "⚠️ Failed to store health record"
                        );
                    }

                    return Ok(info);
                }
                Err(e) => {
                    let response_time = start_time.elapsed().as_millis() as i64;
                    let error_msg = e.to_string();

                    // Extract HTTP status if available
                    let http_status = if error_msg.contains("HTTP error") {
                        error_msg
                            .split_whitespace()
                            .find(|&s| s.chars().all(|c| c.is_ascii_digit()))
                            .and_then(|s| s.parse::<u16>().ok())
                    } else {
                        None
                    };

                    // Store failed health record
                    if let Err(store_err) = self
                        .database
                        .store_health_record(
                            mint_url,
                            false,
                            Some(response_time),
                            Some(&error_msg),
                            http_status,
                        )
                        .await
                    {
                        warn!(
                            target: "bitcoinmints_retyr::mint_info",
                            mint_url = %mint_url,
                            error = %store_err,
                            "⚠️ Failed to store health record"
                        );
                    }

                    last_error = Some(e);

                    // Don't retry on certain permanent errors
                    if error_msg.contains("404")
                        || error_msg.contains("403")
                        || error_msg.contains("invalid URL")
                    {
                        warn!(
                            target: "bitcoinmints_retyr::mint_info",
                            mint_url = %mint_url,
                            error = %error_msg,
                            "🚫 Permanent error detected, skipping retries"
                        );
                        break;
                    }

                    // If this isn't the last attempt, wait before retrying
                    if attempt < MAX_RETRIES {
                        let delay = Duration::from_millis(BASE_DELAY_MS * 2_u64.pow(attempt));
                        info!(
                            target: "bitcoinmints_retyr::mint_info",
                            mint_url = %mint_url,
                            attempt = attempt + 1,
                            delay_ms = delay.as_millis(),
                            "🔄 Retrying mint info fetch"
                        );
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // All retries failed
        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Unknown error during fetch")))
    }

    /// Single attempt to fetch mint info (used by retry logic)
    async fn fetch_mint_info_attempt(&self, mint_url: &str) -> Result<FlexibleMintInfo> {
        let info_url = format!("{}/v1/info", mint_url.trim_end_matches('/'));

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
        let mint_info: FlexibleMintInfo = serde_json::from_str(&response_text)
            .context(format!("Failed to parse JSON response from {}", info_url))?;

        // Store the successful result in the database (store the raw JSON)
        self.database
            .store_mint_info(mint_url, Some(&response_text), None)
            .await
            .context("Failed to store mint info in database")?;

        Ok(mint_info)
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
