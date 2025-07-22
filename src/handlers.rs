use crate::database::Database;
use crate::models::MintQueryParams;
use crate::models::*;
use crate::ui::{render_mints_page, render_reviews_page};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Html,
    Json,
};
use tracing::error;

/// GET /api/mints - Get all mints with their recommendations
pub async fn get_mints(
    State(database): State<Database>,
) -> Result<Json<MintsResponse>, (StatusCode, String)> {
    match database.get_mints_with_recommendations(None).await {
        Ok(mints) => Ok(Json(MintsResponse { mints })),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get mints"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get mints: {}", e),
            ))
        }
    }
}

/// GET /api/users - Get all users with their activity
pub async fn get_users(
    State(database): State<Database>,
) -> Result<Json<UsersResponse>, (StatusCode, String)> {
    match database.get_users_with_activity().await {
        Ok(users) => Ok(Json(UsersResponse { users })),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get users"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get users: {}", e),
            ))
        }
    }
}

/// GET /api/events/raw - Get all raw events (for debugging)
pub async fn get_raw_events(
    State(database): State<Database>,
) -> Result<Json<Vec<RawEvent>>, (StatusCode, String)> {
    match database.get_all_raw_events().await {
        Ok(events) => Ok(Json(events)),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get raw events"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get raw events: {}", e),
            ))
        }
    }
}

/// GET /api/health - Health check endpoint
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "bitcoinmints-retyr",
        "message": "NIP-87 Nostr event collector is running"
    }))
}

/// POST /api/cleanup - Clean up duplicate mints by normalizing URLs
pub async fn cleanup_mints(
    State(database): State<Database>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match database.cleanup_duplicate_mints().await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "success",
            "message": "Mint URL normalization and duplicate cleanup completed"
        }))),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to cleanup mints"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to cleanup mints: {}", e),
            ))
        }
    }
}

/// GET /api/stats - Get mint statistics for debugging
pub async fn mint_stats(
    State(database): State<Database>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match database.get_mint_statistics().await {
        Ok(stats) => Ok(Json(stats)),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get mint statistics"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get mint statistics: {}", e),
            ))
        }
    }
}

/// GET /mints - Frontend mint list page
pub async fn mints_page(
    State(database): State<Database>,
    Query(params): Query<MintQueryParams>,
) -> Result<Html<String>, (StatusCode, String)> {
    match database
        .get_mints_with_recommendations_and_info(&params)
        .await
    {
        Ok(mints) => {
            let markup = render_mints_page(&mints, params.mint_type.as_deref());
            Ok(Html(markup.into_string()))
        }
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get mints for frontend"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load mints: {}", e),
            ))
        }
    }
}

/// GET /reviews - Frontend reviews page
pub async fn reviews_page(
    State(database): State<Database>,
) -> Result<Html<String>, (StatusCode, String)> {
    match database.get_all_recommendations().await {
        Ok(recommendations) => {
            let markup = render_reviews_page(&recommendations);
            Ok(Html(markup.into_string()))
        }
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get recommendations for frontend"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to load reviews: {}", e),
            ))
        }
    }
}

/// GET /api/mint-info - Get detailed mint information from /v1/info endpoints
pub async fn get_mint_info(
    State(database): State<Database>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // Get all stored mint info from the database
    let mint_info_records = match database.get_all_stored_mint_info().await {
        Ok(records) => records,
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get stored mint info"
            );
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get stored mint info: {}", e),
            ));
        }
    };

    let mut mint_info_list = Vec::new();

    for stored_info in mint_info_records {
        let mut info_obj = serde_json::json!({
            "mint_url": stored_info.mint_url,
            "last_fetched_at": stored_info.last_fetched_at,
            "fetch_success": stored_info.fetch_success,
            "error_message": stored_info.error_message,
            "created_at": stored_info.created_at,
            "updated_at": stored_info.updated_at
        });

        // Try to parse the stored JSON as FlexibleMintInfo
        if stored_info.fetch_success {
            if let Ok(parsed_info) =
                serde_json::from_str::<crate::models::FlexibleMintInfo>(&stored_info.info_json)
            {
                info_obj["mint_info"] = serde_json::to_value(parsed_info).unwrap_or_default();
            } else {
                // If flexible parsing fails, include the raw JSON for debugging
                if let Ok(raw_json) =
                    serde_json::from_str::<serde_json::Value>(&stored_info.info_json)
                {
                    info_obj["raw_mint_info"] = raw_json;
                }
            }
        }

        mint_info_list.push(info_obj);
    }

    Ok(Json(serde_json::json!({
        "mint_info": mint_info_list,
        "total_count": mint_info_list.len()
    })))
}

/// GET /api/health/status - Get health status summary for all mints
pub async fn get_health_status(
    State(database): State<Database>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    match database.get_all_mint_health_summaries().await {
        Ok(health_summaries) => {
            let online_count = health_summaries.iter().filter(|h| h.is_online).count();
            let total_count = health_summaries.len();
            let average_uptime = if total_count > 0 {
                health_summaries
                    .iter()
                    .map(|h| h.uptime_percentage)
                    .sum::<f64>()
                    / total_count as f64
            } else {
                0.0
            };

            Ok(Json(serde_json::json!({
                "summary": {
                    "total_mints": total_count,
                    "online_mints": online_count,
                    "offline_mints": total_count - online_count,
                    "average_uptime": format!("{:.1}%", average_uptime)
                },
                "mints": health_summaries
            })))
        }
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                "❌ Failed to get health summaries"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get health summaries: {}", e),
            ))
        }
    }
}

/// GET /api/health/mint/{mint_url} - Get detailed health information for a specific mint
pub async fn get_mint_health(
    State(database): State<Database>,
    axum::extract::Path(mint_url): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    // URL decode the mint_url parameter
    let decoded_url = match urlencoding::decode(&mint_url) {
        Ok(url) => url.to_string(),
        Err(_) => {
            return Err((StatusCode::BAD_REQUEST, "Invalid URL encoding".to_string()));
        }
    };

    match database.get_mint_health_summary(&decoded_url).await {
        Ok(Some(health_summary)) => Ok(Json(
            serde_json::to_value(health_summary).unwrap_or_default(),
        )),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            format!("Mint not found: {}", decoded_url),
        )),
        Err(e) => {
            error!(
                target: "bitcoinmints_retyr::handlers",
                error = %e,
                mint_url = %decoded_url,
                "❌ Failed to get mint health"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get mint health: {}", e),
            ))
        }
    }
}
