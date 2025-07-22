use anyhow::Result;
use axum::{routing::get, Router};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

mod database;
mod handlers;
mod mint_info_service;
mod models;
mod nostr;
mod ui;
mod utils;

use database::Database;
use handlers::{
    cleanup_mints, get_health_status, get_mint_health, get_mint_info, get_mints, get_raw_events,
    get_users, health_check, mint_stats, mints_page, reviews_page,
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
        "🚀 Starting bitcoinmints-retyr - NIP-87 Nostr event collector"
    );

    // Initialize database
    let database = Database::new().await?;
    database.migrate().await?;
    tracing::info!(
        target: "bitcoinmints_retyr::database",
        "✅ Database initialized and migrated successfully"
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
    tracing::info!(
        target: "bitcoinmints_retyr::mint_info",
        "✅ Mint info service initialized successfully"
    );

    // Start mint info fetching in background (after a delay to let other services start)
    let mint_info_clone = mint_info_service.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        if let Err(e) = mint_info_clone.start().await {
            tracing::error!(
                target: "bitcoinmints_retyr::mint_info",
                error = %e,
                "❌ Failed to start mint info fetching service"
            );
        }
    });

    // Build the Axum application with routes
    let app = Router::new()
        // Frontend routes
        .route("/", get(mints_page))
        .route("/mints", get(mints_page))
        .route("/reviews", get(reviews_page))
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
        .with_state(database)
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    // Start the server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!(
        target: "bitcoinmints_retyr::server",
        address = "0.0.0.0:3000",
        "🌐 Server starting"
    );

    tracing::info!(
        target: "bitcoinmints_retyr::server",
        "📍 Available endpoints:",
    );
    let endpoints = [
        ("GET /", "Frontend mint list"),
        ("GET /mints", "Frontend mint list"),
        ("GET /reviews", "Frontend reviews list"),
        ("GET /api", "API information"),
        ("GET /api/health", "Health check"),
        ("GET /api/mints", "Get all mints with recommendations"),
        ("GET /api/users", "Get all users with activity"),
        ("GET /api/events/raw", "Get all raw events (debugging)"),
        ("POST /api/cleanup", "Clean up duplicate mints"),
        ("GET /api/stats", "Get mint statistics"),
        (
            "GET /api/mint-info",
            "Get detailed mint info from /v1/info endpoints",
        ),
        (
            "GET /api/health/status",
            "Get health status summary for all mints",
        ),
        (
            "GET /api/health/mint/{mint_url}",
            "Get detailed health information for a specific mint",
        ),
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
    // Support for RUST_LOG environment variable override
    let default_filter = "bitcoinmints_retyr=info,\
                          sqlx=warn,\
                          hyper=warn,\
                          reqwest=warn,\
                          nostr_relay_pool=warn,\
                          nostr_sdk=info,\
                          tower_http=warn,\
                          axum=warn,\
                          tokio=warn,\
                          rustls=warn,\
                          tungstenite=warn";

    let filter = EnvFilter::builder()
        .with_default_directive(tracing::Level::WARN.into())
        .from_env_lossy()
        .add_directive(
            default_filter
                .parse()
                .unwrap_or_else(|_| tracing::Level::INFO.into()),
        );

    // Check if we should use JSON formatting (for production)
    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v.to_lowercase() == "json")
        .unwrap_or(false);

    if use_json {
        // JSON formatting for production environments
        let json_layer = tracing_subscriber::fmt::layer()
            .json()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_filter(filter);

        tracing_subscriber::registry().with(json_layer).init();
    } else {
        // Pretty formatting for development
        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_target(true)
            .with_level(true)
            .with_thread_ids(false)
            .with_file(false)
            .with_line_number(false)
            .with_ansi(true)
            .compact()
            .with_filter(filter);

        tracing_subscriber::registry().with(fmt_layer).init();
    }
}

/// Root handler that returns API information
async fn root_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "name": "bitcoinmints-retyr",
        "description": "NIP-87 Nostr event collector for ecash mint discoverability",
        "version": "0.1.0",
        "endpoints": {
            "/": "API information (this endpoint)",
            "/api/health": "Health check",
            "/api/mints": "Get all mints with recommendations",
            "/api/users": "Get all users with activity",
            "/api/events/raw": "Get all raw events (debugging)",
            "/api/cleanup": "POST - Clean up duplicate mints by normalizing URLs",
            "/api/stats": "Get mint statistics and duplicate counts",
            "/api/mint-info": "Get detailed mint information from /v1/info endpoints",
            "/api/health/status": "Get health status summary for all mints",
            "/api/health/mint/{mint_url}": "Get detailed health information for a specific mint"
        },
        "nip87_events": {
            "cashu_mint": 38172,
            "fedimint": 38173,
            "recommendation": 38000,
            "user_metadata": 0
        }
    }))
}
