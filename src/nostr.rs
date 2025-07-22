use crate::database::Database;
use crate::models::*;
use anyhow::Result;
use nostr_sdk::{Client, EventId, Filter, Keys, Kind, RelayUrl};
use std::collections::HashSet;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Nostr subscription service that listens for NIP-87 events
#[derive(Clone)]
pub struct NostrService {
    client: Client,
    database: Database,
}

impl NostrService {
    pub async fn new(database: Database) -> Result<Self> {
        // Generate random keys for the client (we're only subscribing, not publishing)
        let keys = Keys::generate();
        let nostr_client_builder = nostr_sdk::client::ClientBuilder::new()
            .signer(keys)
            .opts(nostr_sdk::client::Options::new().gossip(true));

        let client = nostr_client_builder.build();

        Ok(Self { client, database })
    }

    /// Start the subscription service
    pub async fn start(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::nostr",
            "🔗 Starting Nostr subscription service"
        );

        // Add popular relays
        let relays = vec![
            "wss://relay.snort.social",
            "wss://bitcoiner.social",
            "wss://relay.primal.net",
            "wss://relay.damus.io",
            "wss://relay.nostr.band",
            "wss://nos.lol",
            "wss://relay.cashumints.space",
        ];

        let mut connected_relays = Vec::new();
        let mut failed_relays = Vec::new();

        for relay_url in &relays {
            match RelayUrl::parse(relay_url) {
                Ok(url) => {
                    if let Err(e) = self.client.add_relay(url).await {
                        warn!(
                            target: "bitcoinmints_retyr::nostr",
                            relay = %relay_url,
                            error = %e,
                            "⚠️ Failed to add relay"
                        );
                        failed_relays.push(relay_url);
                    } else {
                        connected_relays.push(relay_url);
                    }
                }
                Err(e) => {
                    error!(
                        target: "bitcoinmints_retyr::nostr",
                        relay = %relay_url,
                        error = %e,
                        "❌ Invalid relay URL"
                    );
                    failed_relays.push(relay_url);
                }
            }
        }

        // Connect to relays
        self.client.connect().await;
        info!(
            target: "bitcoinmints_retyr::nostr",
            connected = connected_relays.len(),
            failed = failed_relays.len(),
            relays = ?connected_relays,
            "🌐 Connected to Nostr relays"
        );

        // Create subscription filters for NIP-87 events (no user metadata in main subscription)
        let filter = Filter::new()
            .kinds([
                Kind::Custom(CASHU_MINT_KIND),     // 38172
                Kind::Custom(FEDIMINT_KIND),       // 38173
                Kind::Custom(RECOMMENDATION_KIND), // 38000
            ])
            .limit(1000); // Get recent events

        info!(
            target: "bitcoinmints_retyr::nostr",
            kinds = "[38172, 38173, 38000]",
            "📡 Subscribing to NIP-87 events"
        );

        // Subscribe to events
        let (tx, mut rx) = mpsc::channel(1000);

        // Clone client for the subscription task
        let client_clone = self.client.clone();
        let tx_clone = tx.clone();

        tokio::spawn(async move {
            let subscription_result = client_clone.subscribe(filter, None).await;

            match subscription_result {
                Ok(_sub_id) => {
                    info!(
                        target: "bitcoinmints_retyr::nostr",
                        "✅ Successfully subscribed to NIP-87 events"
                    );

                    // Handle incoming events
                    let mut notifications = client_clone.notifications();
                    while let Ok(notification) = notifications.recv().await {
                        if let nostr_sdk::RelayPoolNotification::Event { event, .. } = notification
                        {
                            if let Err(e) = tx_clone.send(event).await {
                                error!(
                                    target: "bitcoinmints_retyr::nostr",
                                    error = %e,
                                    "❌ Failed to send event to processing queue"
                                );
                            }
                        }
                    }
                }
                Err(e) => {
                    error!(
                        target: "bitcoinmints_retyr::nostr",
                        error = %e,
                        "❌ Failed to subscribe to events"
                    );
                }
            }
        });

        // Process events in a separate task with deduplication
        let database = self.database.clone();
        let client_for_profiles = self.client.clone();

        tokio::spawn(async move {
            let mut seen_events: HashSet<EventId> = HashSet::new();
            let mut processed_count = 0;

            while let Some(event) = rx.recv().await {
                // Dedupe by event ID
                if seen_events.contains(&event.id) {
                    continue; // Skip duplicate
                }
                seen_events.insert(event.id);

                // Check if we already have this event in database
                match database.event_exists(&event.id.to_string()).await {
                    Ok(exists) => {
                        if exists {
                            continue; // Skip duplicate
                        }
                    }
                    Err(e) => {
                        error!(
                            target: "bitcoinmints_retyr::nostr",
                            event_id = %event.id,
                            error = %e,
                            "❌ Failed to check event existence"
                        );
                        continue;
                    }
                }

                // Store the raw event
                if let Err(e) = database.store_raw_event(&event).await {
                    error!(
                        target: "bitcoinmints_retyr::nostr",
                        event_id = %event.id,
                        kind = event.kind.as_u16(),
                        error = %e,
                        "❌ Failed to store event"
                    );
                } else {
                    processed_count += 1;

                    // Only log every 10 events to reduce verbosity
                    if processed_count % 10 == 0 {
                        info!(
                            target: "bitcoinmints_retyr::nostr",
                            processed = processed_count,
                            "📝 Processed events"
                        );
                    }

                    debug!(
                        target: "bitcoinmints_retyr::nostr",
                        event_id = %event.id,
                        kind = event.kind.as_u16(),
                        "📝 Stored event"
                    );
                }

                // If this is a recommendation event, fetch the author's profile if we don't have it
                if event.kind.as_u16() == RECOMMENDATION_KIND {
                    Self::fetch_user_profile_if_missing(
                        &database,
                        &client_for_profiles,
                        &event.pubkey.to_string(),
                    )
                    .await;
                }
            }
        });

        info!(
            target: "bitcoinmints_retyr::nostr",
            "✅ Nostr subscription service started successfully"
        );
        Ok(())
    }

    /// Fetch historical events (run once on startup)
    pub async fn sync_historical_events(&self) -> Result<()> {
        info!(
            target: "bitcoinmints_retyr::nostr",
            "📚 Syncing historical NIP-87 events"
        );

        // Create filter for historical events
        let since = nostr_sdk::Timestamp::now() - Duration::from_secs(30 * 24 * 60 * 60); // 30 days ago

        let filter = Filter::new()
            .kinds([
                Kind::Custom(CASHU_MINT_KIND),
                Kind::Custom(FEDIMINT_KIND),
                Kind::Custom(RECOMMENDATION_KIND),
            ])
            .since(since)
            .limit(5000); // Get more historical events

        debug!(
            target: "bitcoinmints_retyr::nostr",
            filter = ?filter,
            "🔍 Fetching with filter"
        );

        // Get events from all connected relays
        let events = self
            .client
            .fetch_events(filter, Duration::from_secs(30))
            .await?;

        info!(
            target: "bitcoinmints_retyr::nostr",
            total_events = events.len(),
            "📊 Found historical events to process"
        );

        // Dedupe events by event ID
        let mut unique_events = std::collections::HashMap::new();
        for event in events {
            unique_events.insert(event.id, event);
        }

        info!(
            target: "bitcoinmints_retyr::nostr",
            unique_events = unique_events.len(),
            "🔄 After deduplication"
        );

        // Store each unique historical event
        let mut stored_count = 0;
        for (_, event) in unique_events {
            // Check if we already have this event
            match self.database.event_exists(&event.id.to_string()).await {
                Ok(exists) => {
                    if exists {
                        continue; // Skip duplicate
                    }
                }
                Err(e) => {
                    error!(
                        target: "bitcoinmints_retyr::nostr",
                        event_id = %event.id,
                        error = %e,
                        "❌ Failed to check event existence"
                    );
                    continue;
                }
            }

            if let Err(e) = self.database.store_raw_event(&event).await {
                warn!(
                    target: "bitcoinmints_retyr::nostr",
                    event_id = %event.id,
                    error = %e,
                    "⚠️ Failed to store historical event"
                );
            } else {
                stored_count += 1;
            }

            // If this is a recommendation event, fetch the author's profile if we don't have it
            if event.kind.as_u16() == RECOMMENDATION_KIND {
                Self::fetch_user_profile_if_missing(
                    &self.database,
                    &self.client,
                    &event.pubkey.to_string(),
                )
                .await;
            }
        }

        info!(
            target: "bitcoinmints_retyr::nostr",
            stored = stored_count,
            "✨ Historical sync completed"
        );
        Ok(())
    }

    /// Fetch user profile if we don't already have it
    async fn fetch_user_profile_if_missing(database: &Database, client: &Client, pubkey: &str) {
        // Check if we already have a profile for this user
        let profile_exists = match database.get_user_profile(pubkey).await {
            Ok(profile) => profile.is_some(),
            Err(e) => {
                warn!(
                    target: "bitcoinmints_retyr::nostr",
                    pubkey = %pubkey,
                    error = %e,
                    "⚠️ Failed to check user profile existence"
                );
                false
            }
        };

        if profile_exists {
            return; // We already have this user's profile
        }

        // Create filter for this specific user's metadata
        let user_pubkey = match nostr_sdk::PublicKey::parse(pubkey) {
            Ok(pk) => pk,
            Err(e) => {
                warn!(
                    target: "bitcoinmints_retyr::nostr",
                    pubkey = %pubkey,
                    error = %e,
                    "⚠️ Invalid pubkey"
                );
                return;
            }
        };

        let filter = Filter::new()
            .kind(Kind::Custom(USER_METADATA_KIND))
            .author(user_pubkey)
            .limit(1);

        // Fetch the user's profile
        match client.fetch_events(filter, Duration::from_secs(5)).await {
            Ok(events) => {
                if let Some(event) = events.first() {
                    // Store the user profile event
                    if let Err(e) = database.store_raw_event(event).await {
                        warn!(
                            target: "bitcoinmints_retyr::nostr",
                            pubkey = %pubkey,
                            error = %e,
                            "⚠️ Failed to store user profile"
                        );
                    } else {
                        info!(
                            target: "bitcoinmints_retyr::nostr",
                            pubkey = %pubkey,
                            "👤 Fetched and stored user profile"
                        );
                    }
                }
            }
            Err(e) => {
                warn!(
                    target: "bitcoinmints_retyr::nostr",
                    pubkey = %pubkey,
                    error = %e,
                    "⚠️ Failed to fetch user profile"
                );
            }
        }
    }
}
