use crate::models::MintWithRecommendationsAndInfo;
use crate::ui::components::mint_card::{
    extract_currency_capabilities, extract_nut_capabilities, get_mint_styles, render_cashu_filters,
    render_mint_card,
};
use crate::ui::components::{render_empty_state, render_layout, render_stats_row};
use maud::{html, Markup, PreEscaped};

/// Render the complete mints page
pub fn render_mints_page(
    mints: &[MintWithRecommendationsAndInfo],
    current_filter: Option<&str>,
) -> Markup {
    // Extract currency and NUT capabilities for Cashu filters
    let all_mints_for_capabilities = mints; // Use all mints to get capabilities, not filtered ones
    let currency_capabilities = extract_currency_capabilities(all_mints_for_capabilities);
    let nut_capabilities = extract_nut_capabilities(all_mints_for_capabilities);

    let content = html! {
        style { (get_mint_styles()) }

        script {
            (PreEscaped(r#"
            function toggleReviewers(element) {
                const reviewersGrid = element.nextElementSibling;
                const expandIndicator = element.querySelector('.expand-indicator');
                
                if (reviewersGrid.classList.contains('expanded')) {
                    reviewersGrid.classList.remove('expanded');
                    expandIndicator.classList.remove('expanded');
                } else {
                    reviewersGrid.classList.add('expanded');
                    expandIndicator.classList.add('expanded');
                }
            }
            
            function toggleMintInfo(element) {
                const infoContent = element.nextElementSibling;
                const expandIndicator = element.querySelector('.info-expand-indicator');
                
                if (infoContent.classList.contains('expanded')) {
                    infoContent.classList.remove('expanded');
                    expandIndicator.classList.remove('expanded');
                } else {
                    infoContent.classList.add('expanded');
                    expandIndicator.classList.add('expanded');
                }
            }
            
            function filterMints(type) {
                const url = new URL(window.location);
                
                // Clear Cashu-specific filters when switching types
                url.searchParams.delete('minting');
                url.searchParams.delete('melting');
                url.searchParams.delete('nuts');
                
                if (type === 'all') {
                    url.searchParams.delete('type');
                } else {
                    url.searchParams.set('type', type);
                }
                
                window.location.href = url.toString();
            }
            
            function toggleCashuFilters() {
                const cashuFilters = document.getElementById('cashu-filters');
                const filterButtons = document.querySelectorAll('.filter-btn');
                let cashuBtn = null;
                
                for (let i = 0; i < filterButtons.length; i++) {
                    if (filterButtons[i].textContent.includes('Cashu')) {
                        cashuBtn = filterButtons[i];
                        break;
                    }
                }
                
                if (cashuBtn && cashuBtn.classList.contains('active')) {
                    cashuFilters.classList.add('active');
                } else {
                    cashuFilters.classList.remove('active');
                }
            }
            
            function updateCashuFilters() {
                const url = new URL(window.location);
                const mintFilters = Array.from(document.querySelectorAll('.mint-filter:checked')).map(cb => cb.dataset.currency);
                const meltFilters = Array.from(document.querySelectorAll('.melt-filter:checked')).map(cb => cb.dataset.currency);
                const nutFilters = Array.from(document.querySelectorAll('.nut-filter:checked')).map(cb => cb.dataset.nut);
                
                if (mintFilters.length > 0) {
                    url.searchParams.set('minting', mintFilters.join(','));
                } else {
                    url.searchParams.delete('minting');
                }
                
                if (meltFilters.length > 0) {
                    url.searchParams.set('melting', meltFilters.join(','));
                } else {
                    url.searchParams.delete('melting');
                }
                
                if (nutFilters.length > 0) {
                    url.searchParams.set('nuts', nutFilters.join(','));
                } else {
                    url.searchParams.delete('nuts');
                }
                
                window.location.href = url.toString();
            }
            
            function clearCashuFilters() {
                const url = new URL(window.location);
                url.searchParams.delete('minting');
                url.searchParams.delete('melting');
                url.searchParams.delete('nuts');
                window.location.href = url.toString();
            }
            
            function selectAllCashuFilters() {
                const checkboxes = document.querySelectorAll('.filter-checkbox');
                checkboxes.forEach(cb => cb.checked = true);
                updateCashuFilters();
            }
            
            // Initialize filters on page load
            document.addEventListener('DOMContentLoaded', function() {
                toggleCashuFilters();
                
                // Restore filter state from URL
                const url = new URL(window.location);
                const mintingCurrencies = url.searchParams.get('minting')?.split(',') || [];
                const meltingCurrencies = url.searchParams.get('melting')?.split(',') || [];
                const nutProtocols = url.searchParams.get('nuts')?.split(',') || [];
                
                mintingCurrencies.forEach(currency => {
                    const checkbox = document.querySelector(`.mint-filter[data-currency="${currency}"]`);
                    if (checkbox) checkbox.checked = true;
                });
                
                meltingCurrencies.forEach(currency => {
                    const checkbox = document.querySelector(`.melt-filter[data-currency="${currency}"]`);
                    if (checkbox) checkbox.checked = true;
                });
                
                nutProtocols.forEach(nut => {
                    const checkbox = document.querySelector(`.nut-filter[data-nut="${nut}"]`);
                    if (checkbox) checkbox.checked = true;
                });
            });
            "#))
        }

        // Filter controls
        div class="filter-section" {
            h3 class="filter-title" { "Filter by Mint Type" }
            div class="filter-buttons" {
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
                    onclick="filterMints('fedimint')"
                { "🔵 Fedimint" }
            }

            // Cashu-specific currency and NUT filters
            @if current_filter == Some("cashu") {
                (render_cashu_filters(&currency_capabilities, &nut_capabilities))
            }
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
            div class="mints-grid" {
                @for mint_with_recs in mints {
                    (render_mint_card(mint_with_recs))
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
