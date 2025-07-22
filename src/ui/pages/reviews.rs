use crate::models::RecommendationWithUser;
use crate::ui::components::review_card::{get_review_styles, render_review_card};
use crate::ui::components::{render_empty_state, render_layout, render_stats_row};
use maud::{html, Markup};

/// Render the complete reviews page
pub fn render_reviews_page(recommendations: &[RecommendationWithUser]) -> Markup {
    let content = html! {
        style { (get_review_styles()) }

        @if !recommendations.is_empty() {
            (render_stats_row(vec![
                (recommendations.len().to_string(), "Total Reviews"),
                (
                    {
                        let ratings: Vec<i32> = recommendations.iter()
                            .map(|r| r.recommendation.rating)
                            .collect();
                        if let Some(avg) = if ratings.is_empty() {
                            None
                        } else {
                            Some(ratings.iter().sum::<i32>() as f64 / ratings.len() as f64)
                        } {
                            format!("{:.1}", avg)
                        } else {
                            "N/A".to_string()
                        }
                    },
                    "Avg Rating"
                ),
                (
                    recommendations.iter()
                        .map(|r| r.recommendation.invite_codes.len())
                        .sum::<usize>().to_string(),
                    "Invite Codes"
                ),
                (
                    recommendations.iter()
                        .filter(|r| r.user_profile.is_some())
                        .count().to_string(),
                    "Known Users"
                )
            ]))
        }

        @if recommendations.is_empty() {
            (render_empty_state("📭", "No reviews found yet", "The Nostr subscription is still collecting recommendations..."))
        } @else {
            div class="reviews-grid" {
                @for rec_with_user in recommendations {
                    (render_review_card(rec_with_user))
                }
            }
        }
    };

    render_layout(
        "Bitcoin Mint Reviews",
        "Community Mint Reviews & Recommendations",
        "reviews",
        content,
    )
}
