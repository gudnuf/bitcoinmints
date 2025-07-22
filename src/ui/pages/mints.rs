use crate::models::MintWithRecommendationsAndInfo;
use crate::ui::components::mint_card::{get_mint_styles, render_mint_card};
use crate::ui::components::{render_empty_state, render_layout, render_stats_row};
use maud::{html, Markup};

/// Render the complete mints page
pub fn render_mints_page(mints: &[MintWithRecommendationsAndInfo]) -> Markup {
    let content = html! {
        style { (get_mint_styles()) }

        @if !mints.is_empty() {
            (render_stats_row(vec![
                (mints.len().to_string(), "Total Mints"),
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
            (render_empty_state("🔍", "No mints discovered yet", "The Nostr subscription is still collecting mint announcements..."))
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
