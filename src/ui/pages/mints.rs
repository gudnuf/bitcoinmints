use crate::models::MintWithRecommendationsAndInfo;
use crate::ui::components::mint_card::{
    extract_currency_capabilities, extract_module_capabilities, extract_nut_capabilities,
    get_mint_styles, render_cashu_filters, render_fedimint_filters, render_mint_card,
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
