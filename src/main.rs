use anyhow::Result;
use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::{services::ServeDir, trace::TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod cache;
mod cached_database;
mod database;
mod fedimint_service;
mod handlers;
mod mint_info_service;
mod models;
mod nostr;
mod ui;
mod utils;

use cache::CacheService;
use cached_database::CachedDatabase;
use database::Database;
use fedimint_service::FedimintService;
use handlers::{
    cleanup_mints, clear_cache, get_cache_stats, get_health_status, get_mint_health, get_mint_info,
    get_mints, get_raw_events, get_users, health_check, mint_stats, mints_page, review_detail_page,
    reviews_page,
};
use mint_info_service::MintInfoService;
use nostr::NostrService;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize enhanced tracing with proper filtering and formatting
    init_logging();

    tracing::info!(
        target: "bitcoinmints_retyr",
        version = env!("CARGO_PKG_VERSION"),
        "🚀 Starting bitcoinmints-retyr - NIP-87 Nostr event collector with proactive caching"
    );

    // Initialize database
    let database = Database::new().await?;
    database.migrate().await?;
    tracing::info!(
        target: "bitcoinmints_retyr::database",
        "✅ Database initialized and migrated successfully"
    );

    // Initialize cache service
    let cache_service = CacheService::new();
    tracing::info!(
        target: "bitcoinmints_retyr::cache",
        "💾 Cache service initialized"
    );

    // Start cache cleanup task
    let _cleanup_task = cache_service.start_cleanup_task();
    tracing::info!(
        target: "bitcoinmints_retyr::cache",
        "🧹 Cache cleanup task started"
    );

    // Initialize cached database
    let cached_database = CachedDatabase::new(database.clone(), cache_service.clone());
    tracing::info!(
        target: "bitcoinmints_retyr::cached_database",
        "🎯 Cached database service initialized"
    );

    // Pre-fill cache with all data on startup
    if let Err(e) = cached_database.pre_fill_cache().await {
        tracing::error!(
            target: "bitcoinmints_retyr::cached_database",
            error = %e,
            "❌ Failed to pre-fill cache on startup, continuing with empty cache"
        );
    } else {
        tracing::info!(
            target: "bitcoinmints_retyr::cached_database",
            "🎯 Cache pre-filled successfully with all data"
        );
    }

    // Start cache refresh background task
    let _cache_refresh_task = cached_database.start_cache_refresh_task();
    tracing::info!(
        target: "bitcoinmints_retyr::cached_database",
        "🔄 Cache refresh background task started"
    );

    // Start cache refresh monitoring task (using proactive refresh instead of invalidation)
    let cached_database_for_refresh = cached_database.clone();
    tokio::spawn(async move {
        let mut last_check = chrono::Utc::now();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10)); // Check every 10 seconds

        loop {
            interval.tick().await;

            // Check for new events since last check and proactively refresh cache
            if let Ok(events) = cached_database_for_refresh
                .database()
                .get_all_raw_events()
                .await
            {
                let mut new_event_kinds = std::collections::HashSet::new();

                for event in events {
                    if event.received_at > last_check {
                        // New event found, collect event kinds for refresh
                        new_event_kinds.insert(event.kind);
                    }
                }

                if !new_event_kinds.is_empty() {
                    tracing::debug!(
                        target: "bitcoinmints_retyr::cache",
                        event_kinds = ?new_event_kinds,
                        "🔄 Proactively refreshing cache due to new events"
                    );

                    // Refresh cache for each event kind found
                    for event_kind in new_event_kinds {
                        if let Err(e) = cached_database_for_refresh
                            .refresh_cache_on_event(event_kind)
                            .await
                        {
                            tracing::warn!(
                                target: "bitcoinmints_retyr::cache",
                                error = %e,
                                event_kind = event_kind,
                                "Failed to refresh cache for event kind"
                            );
                        }
                    }
                }

                last_check = chrono::Utc::now();
            }
        }
    });

    tracing::info!(
        target: "bitcoinmints_retyr::cache",
        "👁️ Cache refresh monitor started"
    );

    // Initialize Nostr service
    let nostr_service = NostrService::new(database.clone()).await?;

    // Start Nostr subscription in background
    let nostr_clone = nostr_service.clone();
    tokio::spawn(async move {
        if let Err(e) = nostr_clone.start().await {
            tracing::error!(
                target: "bitcoinmints_retyr::nostr",
                error = %e,
                "❌ Failed to start Nostr subscription service"
            );
        }
    });

    // Start historical sync in background (after a delay to let real-time subscription start)
    let nostr_clone = nostr_service.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        if let Err(e) = nostr_clone.sync_historical_events().await {
            tracing::error!(
                target: "bitcoinmints_retyr::nostr",
                error = %e,
                "❌ Failed to sync historical events"
            );
        }
    });

    // Initialize Mint Info Service
    let mint_info_service = MintInfoService::new(database.clone()).await?;

    // Start mint info fetching in background
    let mint_info_clone = mint_info_service.clone();
    tokio::spawn(async move {
        if let Err(e) = mint_info_clone.start().await {
            tracing::error!(
                target: "bitcoinmints_retyr::mint_info",
                error = %e,
                "❌ Failed to start mint info service"
            );
        }
    });

    // Initialize Fedimint Service
    let fedimint_service = FedimintService::new(database.clone()).await?;

    // Start fedimint federation config fetching in background
    let fedimint_clone = fedimint_service.clone();
    tokio::spawn(async move {
        if let Err(e) = fedimint_clone.start().await {
            tracing::error!(
                target: "bitcoinmints_retyr::fedimint",
                error = %e,
                "❌ Failed to start fedimint service"
            );
        }
    });

    // Build the router with cached database
    let app = Router::new()
        // Static assets
        .nest_service("/assets", ServeDir::new("src/assets"))
        // Frontend routes
        .route("/", get(mints_page))
        .route("/mints", get(mints_page))
        .route("/reviews", get(reviews_page))
        .route("/review/:event_id", get(review_detail_page))
        // API routes
        .route("/api", get(root_handler))
        .route("/api/health", get(health_check))
        .route("/api/mints", get(get_mints))
        .route("/api/users", get(get_users))
        .route("/api/events/raw", get(get_raw_events))
        .route("/api/cleanup", axum::routing::post(cleanup_mints))
        .route("/api/stats", get(mint_stats))
        .route("/api/mint-info", get(get_mint_info))
        .route("/api/health/status", get(get_health_status))
        .route("/api/health/mint/:mint_url", get(get_mint_health))
        // Cache management routes (for debugging/admin)
        .route("/api/cache/stats", get(get_cache_stats))
        .route("/api/cache/clear", axum::routing::post(clear_cache))
        .with_state(cached_database)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Get port from environment variable or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);

    let bind_address = format!("0.0.0.0:{}", port);

    // Start the server
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    tracing::info!(
        target: "bitcoinmints_retyr::server",
        address = %bind_address,
        port = port,
        "🌐 Server starting with proactive pre-filled caching enabled"
    );

    tracing::info!(
        target: "bitcoinmints_retyr::server",
        "📍 Available endpoints:",
    );
    let endpoints = [
        ("GET /", "Frontend mint list"),
        ("GET /mints", "Frontend mint list"),
        ("GET /reviews", "Frontend reviews list"),
        ("GET /review/{event_id}", "Individual review detail page"),
        ("GET /api", "API information"),
        ("GET /api/health", "Health check"),
        (
            "GET /api/mints",
            "Get all mints with recommendations (cached)",
        ),
        ("GET /api/users", "Get all users with activity (cached)"),
        ("GET /api/events/raw", "Get all raw events (cached)"),
        ("POST /api/cleanup", "Clean up duplicate mints"),
        ("GET /api/stats", "Get mint statistics (cached)"),
        (
            "GET /api/mint-info",
            "Get detailed mint info from /v1/info endpoints (cached)",
        ),
        (
            "GET /api/health/status",
            "Get health status summary for all mints (cached)",
        ),
        (
            "GET /api/health/mint/{mint_url}",
            "Get detailed health information for a specific mint (cached)",
        ),
        ("GET /api/cache/stats", "Get cache statistics"),
        ("POST /api/cache/clear", "Clear all caches"),
    ];

    for (endpoint, description) in endpoints {
        tracing::info!(
            target: "bitcoinmints_retyr::server",
            endpoint = endpoint,
            description = description,
            "  "
        );
    }

    axum::serve(listener, app).await?;

    Ok(())
}

/// Initialize logging with proper filtering and formatting
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,bitcoinmints_retyr=debug"));

    let formatting_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    tracing_subscriber::registry()
        .with(env_filter)
        .with(formatting_layer)
        .init();
}

/// Root API handler
async fn root_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "bitcoinmints-retyr",
        "description": "NIP-87 Nostr event collector for ecash mint discoverability with proactive pre-filled caching",
        "version": "0.1.0",
        "features": [
            "Real-time Nostr event collection",
            "Proactive cache pre-filling on startup",
            "Intelligent cache refresh based on data changes",
            "Background cache warming and maintenance",
            "Health monitoring",
            "Mint discovery and recommendations"
        ],
        "endpoints": {
            "/": "API information (this endpoint)",
            "/api/health": "Health check",
            "/api/mints": "Get all mints with recommendations (cached)",
            "/api/users": "Get all users with activity (cached)",
            "/api/events/raw": "Get all raw events (cached)",
            "/api/cleanup": "POST - Clean up duplicate mints by normalizing URLs",
            "/api/stats": "Get mint statistics and duplicate counts (cached)",
            "/api/mint-info": "Get detailed mint information from /v1/info endpoints (cached)",
            "/api/health/status": "Get health status summary for all mints (cached)",
            "/api/health/mint/{mint_url}": "Get detailed health information for a specific mint (cached)",
            "/api/cache/stats": "Get cache performance statistics",
            "/api/cache/clear": "POST - Clear all caches"
        },
        "nip87_events": {
            "cashu_mint": 38172,
            "fedimint": 38173,
            "recommendation": 38000,
            "user_metadata": 0
        },
        "caching": {
            "enabled": true,
            "strategy": "Proactive pre-filled cache with intelligent refresh",
            "pre_filled_on_startup": true,
            "ttl_seconds": {
                "mints": 120,
                "users": 600,
                "recommendations": 120,
                "raw_events": 120,
                "mint_info": 600,
                "health_summaries": 120,
                "user_profiles": 1800,
                "individual_recommendations": 900,
                "mint_health": 60,
                "query_specific": 120
            },
            "refresh_strategy": "Smart refresh based on event types with periodic background refresh",
            "refresh_intervals": {
                "event_driven": "10 seconds",
                "periodic_full_refresh": "30 seconds"
            }
        }
    }))
}
