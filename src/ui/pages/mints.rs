use crate::models::{MintWithRecommendationsAndInfo, UnifiedMintWithRecommendations};
use crate::ui::components::mint_card::{
    extract_currency_capabilities, extract_module_capabilities, extract_nut_capabilities,
    get_mint_styles, render_cashu_filters, render_fedimint_filters, render_mint_card,
    render_unified_mint_card,
};
use crate::ui::components::{render_empty_state, render_layout, render_stats_row};
use maud::{html, Markup};

/// Render the complete mints page
pub fn render_mints_page(
    mints: &[MintWithRecommendationsAndInfo],
    current_filter: Option<&str>,
) -> Markup {
    // Extract capabilities for filters
    let all_mints_for_capabilities = mints; // Use all mints to get capabilities, not filtered ones
    let currency_capabilities = extract_currency_capabilities(all_mints_for_capabilities);
    let nut_capabilities = extract_nut_capabilities(all_mints_for_capabilities);
    let module_capabilities = extract_module_capabilities(all_mints_for_capabilities);

    let content = html! {
        style { (get_mint_styles()) }

        // Filter controls
        div class="filter-section" {
            h3 class="filter-title" { "Filter by Mint Type" }
            div style="margin-bottom: 1rem;" class="filter-buttons" {
                button
                    class={"filter-btn" @if current_filter.is_none() { " active" }}
                    onclick="filterMints('all')"
                { "All Mints" }
                button
                    class={"filter-btn" @if current_filter == Some("cashu") { " active" }}
                    onclick="filterMints('cashu'); toggleCashuFilters();"
                { "🟠 Cashu" }
                button
                    class={"filter-btn" @if current_filter == Some("fedimint") { " active" }}
                    onclick="filterMints('fedimint'); toggleFedimintFilters();"
                { "🔵 Fedimint" }
            }

            @if !mints.is_empty() {
                (render_stats_row(vec![
                    (mints.len().to_string(), {
                        match current_filter {
                            Some("cashu") => "Cashu Mints",
                            Some("fedimint") => "Fedimint Instances",
                            _ => "Total Mints"
                        }
                    }),
                    (mints.iter().map(|m| m.total_recommendations).sum::<usize>().to_string(), "Total Reviews"),
                    (
                        {
                            let ratings: Vec<f64> = mints.iter()
                                .filter_map(|m| m.average_rating)
                                .collect();
                            if let Some(avg) = if ratings.is_empty() {
                                None
                            } else {
                                Some(ratings.iter().sum::<f64>() / ratings.len() as f64)
                            } {
                                format!("{:.1}", avg)
                            } else {
                                "N/A".to_string()
                            }
                        },
                        "Avg Rating"
                    ),
                    (mints.iter().filter(|m| m.stored_info.is_some()).count().to_string(), "With Info")
                ]))
            }


            // Rating slider filter
            div class="rating-filter-section" {
                h4 class="rating-filter-title" { "⭐ Filter by Minimum Rating" }
                div class="rating-slider-container" {
                    div class="rating-slider-wrapper" {
                        input
                            type="range"
                            id="rating-slider"
                            class="rating-slider"
                            min="0"
                            max="5"
                            step="0.1"
                            value="0"
                            oninput="filterByRating(this.value)";
                        div class="rating-slider-track" {}
                    }
                    div class="rating-display" id="rating-display" {
                        span class="rating-value" id="rating-value" { "Show All" }
                        span class="rating-label" id="rating-label" { "(sorted by rating)" }
                    }
                    div class="rating-controls" {
                        button class="rating-reset-btn" onclick="resetRatingFilter()" {
                            "Show All Ratings"
                        }
                    }
                }
            }

            // Cashu-specific currency and NUT filters
            @if current_filter == Some("cashu") {
                (render_cashu_filters(&currency_capabilities, &nut_capabilities))
            }

            // Fedimint-specific module filters
            @if current_filter == Some("fedimint") {
                (render_fedimint_filters(&module_capabilities))
            }
        }


        @if mints.is_empty() {
            (render_empty_state("🔍", {
                match current_filter {
                    Some("cashu") => "No Cashu mints found",
                    Some("fedimint") => "No Fedimint instances found",
                    _ => "No mints discovered yet"
                }
            }, {
                match current_filter {
                    Some(_) => "Try selecting a different filter or check back later...",
                    _ => "The Nostr subscription is still collecting mint announcements..."
                }
            }))
        } @else {
            div class="mints-grid" id="mints-grid" {
                @for mint_with_recs in mints {
                    (render_mint_card(mint_with_recs))
                }
            }

            // Message for when all mints are filtered out by rating
            div class="no-rating-matches" id="no-rating-matches" style="display: none;" {
                div class="empty-state" {
                    div class="empty-icon" { "⭐" }
                    h3 { "No mints match the selected rating" }
                    p { "Try lowering the minimum rating or reset the filter to see all mints." }
                }
            }
        }
    };

    render_layout(
        "Bitcoin Mints Dashboard",
        "Decentralized Mint Discovery Dashboard",
        "mints",
        content,
    )
}

/// Render the complete mints page with unified data
pub fn render_unified_mints_page(
    unified_mints: &[UnifiedMintWithRecommendations],
    current_filter: Option<&str>,
) -> Markup {
    // Convert unified mints to legacy format for capability extraction
    // This is a temporary bridge until capabilities are fully unified
    let legacy_mints: Vec<MintWithRecommendationsAndInfo> = unified_mints
        .iter()
        .map(|unified| convert_unified_to_legacy(unified))
        .collect();

    // Extract capabilities for filters
    let currency_capabilities = extract_currency_capabilities(&legacy_mints);
    let nut_capabilities = extract_nut_capabilities(&legacy_mints);
    let module_capabilities = extract_module_capabilities(&legacy_mints);

    let content = html! {
        style { (get_mint_styles()) }

        // Filter controls
        div class="filter-section" {
            h3 class="filter-title" { "Filter by Mint Type" }
            div style="margin-bottom: 1rem;" class="filter-buttons" {
                button
                    class={"filter-btn" @if current_filter.is_none() { " active" }}
                    onclick="filterMints('all')"
                { "All Mints" }
                button
                    class={"filter-btn" @if current_filter == Some("cashu") { " active" }}
                    onclick="filterMints('cashu'); toggleCashuFilters();"
                { "🟠 Cashu" }
                button
                    class={"filter-btn" @if current_filter == Some("fedimint") { " active" }}
                    onclick="filterMints('fedimint'); toggleFedimintFilters();"
                { "🔵 Fedimint" }
            }

            @if !unified_mints.is_empty() {
                (render_stats_row(vec![
                    (unified_mints.len().to_string(), {
                        match current_filter {
                            Some("cashu") => "Cashu Mints",
                            Some("fedimint") => "Fedimint Instances",
                            _ => "Total Mints"
                        }
                    }),
                    (unified_mints.iter().map(|m| m.total_recommendations).sum::<usize>().to_string(), "Total Reviews"),
                    (
                        {
                            let ratings: Vec<f64> = unified_mints.iter()
                                .filter_map(|m| m.average_rating)
                                .collect();
                            if let Some(avg) = if ratings.is_empty() {
                                None
                            } else {
                                Some(ratings.iter().sum::<f64>() / ratings.len() as f64)
                            } {
                                format!("{:.1}", avg)
                            } else {
                                "N/A".to_string()
                            }
                        },
                        "Avg Rating"
                    ),
                    (unified_mints.iter().filter(|m| m.mint_data.is_online).count().to_string(), "Online")
                ]))
            }

            // Rating slider filter
            div class="rating-filter-section" {
                h4 class="rating-filter-title" { "⭐ Filter by Minimum Rating" }
                div class="rating-slider-container" {
                    div class="rating-slider-wrapper" {
                        input
                            type="range"
                            id="rating-slider"
                            class="rating-slider"
                            min="0"
                            max="5"
                            step="0.1"
                            value="0"
                            oninput="filterByRating(this.value)";
                        div class="rating-slider-track" {}
                    }
                    div class="rating-display" id="rating-display" {
                        span class="rating-value" id="rating-value" { "Show All" }
                        span class="rating-label" id="rating-label" { "(sorted by rating)" }
                    }
                    div class="rating-controls" {
                        button class="rating-reset-btn" onclick="resetRatingFilter()" {
                            "Show All Ratings"
                        }
                    }
                }
            }

            // Cashu-specific currency and NUT filters
            @if current_filter == Some("cashu") {
                (render_cashu_filters(&currency_capabilities, &nut_capabilities))
            }

            // Fedimint-specific module filters
            @if current_filter == Some("fedimint") {
                (render_fedimint_filters(&module_capabilities))
            }
        }

        @if unified_mints.is_empty() {
            (render_empty_state("🔍", {
                match current_filter {
                    Some("cashu") => "No Cashu mints found",
                    Some("fedimint") => "No Fedimint instances found",
                    _ => "No mints discovered yet"
                }
            }, {
                match current_filter {
                    Some(_) => "Try selecting a different filter or check back later...",
                    _ => "The Nostr subscription is still collecting mint announcements..."
                }
            }))
        } @else {
            div class="mints-grid" id="mints-grid" {
                @for unified_mint in unified_mints {
                    (render_unified_mint_card(unified_mint))
                }
            }

            // Message for when all mints are filtered out by rating
            div class="no-rating-matches" id="no-rating-matches" style="display: none;" {
                div class="empty-state" {
                    div class="empty-icon" { "⭐" }
                    h3 { "No mints match the selected rating" }
                    p { "Try lowering the minimum rating or reset the filter to see all mints." }
                }
            }
        }
    };

    render_layout(
        "Bitcoin Mints Dashboard",
        "Decentralized Mint Discovery Dashboard",
        "mints",
        content,
    )
}

/// Convert unified mint to legacy format for compatibility
fn convert_unified_to_legacy(
    unified: &UnifiedMintWithRecommendations,
) -> MintWithRecommendationsAndInfo {
    use crate::models::{Mint, MintType};

    let mint_data = &unified.mint_data;

    // Create a basic Mint struct from unified data
    let mint = Mint {
        event_id: mint_data.event_id.clone(),
        name: mint_data.name.clone(),
        mint_url: match mint_data.mint_type {
            MintType::Cashu => mint_data
                .cashu_data
                .as_ref()
                .map(|c| c.mint_url.clone())
                .unwrap_or_else(|| mint_data.mint_id.clone()),
            MintType::Fedimint => mint_data.mint_id.clone(), // Use federation_id as URL
        },
        description: mint_data.description.clone(),
        mint_pubkey: match mint_data.mint_type {
            MintType::Cashu => mint_data
                .cashu_data
                .as_ref()
                .map(|c| c.mint_pubkey.clone())
                .unwrap_or_default(),
            MintType::Fedimint => mint_data.author_pubkey.clone(), // Use author as mint pubkey
        },
        author_pubkey: mint_data.author_pubkey.clone(),
        mint_type: mint_data.mint_type.to_string(),
        networks: mint_data.networks.clone(),
        invite_codes: match mint_data.mint_type {
            MintType::Cashu => vec![mint_data.mint_id.clone()],
            MintType::Fedimint => mint_data
                .fedimint_data
                .as_ref()
                .map(|f| f.invite_codes.clone())
                .unwrap_or_default(),
        },
        nuts: match mint_data.mint_type {
            MintType::Cashu => mint_data
                .cashu_data
                .as_ref()
                .map(|c| c.nuts.clone())
                .unwrap_or_default(),
            MintType::Fedimint => cdk::nuts::Nuts::default(),
        },
        modules: match mint_data.mint_type {
            MintType::Fedimint => mint_data
                .fedimint_data
                .as_ref()
                .map(|f| f.modules.clone())
                .unwrap_or_default(),
            MintType::Cashu => Vec::new(),
        },
        federation_id: match mint_data.mint_type {
            MintType::Fedimint => Some(mint_data.mint_id.clone()),
            MintType::Cashu => None,
        },
        guardians_count: mint_data
            .fedimint_data
            .as_ref()
            .and_then(|f| f.guardians_count),
        meta: {
            let mut meta = std::collections::HashMap::new();
            if let Some(fedimint_data) = &mint_data.fedimint_data {
                if let Some(name) = &fedimint_data.federation_name {
                    meta.insert(
                        "federation_name".to_string(),
                        serde_json::Value::String(name.clone()),
                    );
                }
                if let Some(welcome) = &fedimint_data.welcome_message {
                    meta.insert(
                        "welcome_message".to_string(),
                        serde_json::Value::String(welcome.clone()),
                    );
                }
            }
            meta
        },
        created_at: mint_data.created_at,
        received_at: mint_data.received_at,
    };

    // Create stored info from health data
    let stored_info = mint_data.health_info.as_ref().map(|health| {
        use crate::models::StoredMintInfo;
        StoredMintInfo {
            id: format!("unified-{}", mint_data.mint_id),
            mint_url: mint_data.mint_id.clone(),
            info_json: match &mint_data.cashu_data {
                Some(cashu) => serde_json::to_string(&cashu.mint_info).unwrap_or_default(),
                None => "{}".to_string(),
            },
            last_fetched_at: health.last_check,
            fetch_success: mint_data.is_online,
            error_message: None,
            created_at: mint_data.received_at,
            updated_at: health.last_check,
            consecutive_failures: health.consecutive_failures,
            consecutive_successes: health.consecutive_successes,
            total_attempts: health.total_attempts,
            total_successes: health.total_successes,
            first_seen_at: mint_data.received_at,
            health_score: health.health_score,
            is_currently_online: mint_data.is_online,
        }
    });

    // Create parsed info from cashu data
    let parsed_info = mint_data
        .cashu_data
        .as_ref()
        .and_then(|cashu| cashu.mint_info.clone());

    MintWithRecommendationsAndInfo {
        mint,
        recommendations: unified.recommendations.clone(),
        total_recommendations: unified.total_recommendations,
        average_rating: unified.average_rating,
        stored_info,
        parsed_info,
    }
}
