use crate::models::RecommendationWithUser;
use crate::ui::components::review_card::get_review_styles;
use crate::ui::components::{render_empty_state, render_layout};
use maud::{html, Markup};

/// Render the individual review detail page
pub fn render_review_detail_page(recommendation: Option<&RecommendationWithUser>) -> Markup {
    let content = html! {
        style { (get_review_styles()) }
        style { (get_review_detail_styles()) }

        @if let Some(rec_with_user) = recommendation {
            div class="review-detail-container" {
                // Back navigation
                div class="back-nav" {
                    a href="/reviews" class="back-link" {
                        "← Back to All Reviews"
                    }
                }

                // Main review detail card
                div class="review-detail-card" {
                    div class="review-detail-header" {
                        div class="reviewer-section" {
                            @if let Some(user_profile) = &rec_with_user.user_profile {
                                @if let Some(picture) = &user_profile.picture {
                                    img class="reviewer-avatar-large" src=(picture) alt="Avatar";
                                } @else {
                                    div class="no-avatar-large" {
                                        @if let Some(name) = &user_profile.display_name.as_ref().or(user_profile.name.as_ref()) {
                                            (name.chars().next().unwrap_or('?').to_uppercase())
                                        } @else {
                                            "?"
                                        }
                                    }
                                }
                                div class="reviewer-info" {
                                    div class="reviewer-name-large" {
                                        @if let Some(name) = &user_profile.display_name.as_ref().or(user_profile.name.as_ref()) {
                                            (name)
                                        } @else {
                                            "Anonymous Reviewer"
                                        }
                                    }
                                    div class="reviewer-pubkey-large" {
                                        "Pubkey: " (format!("{}...{}",
                                            &user_profile.pubkey[0..16],
                                            &user_profile.pubkey[user_profile.pubkey.len()-16..]
                                        ))
                                    }
                                    @if let Some(about) = &user_profile.about {
                                        div class="reviewer-about" {
                                            (about)
                                        }
                                    }
                                    @if let Some(website) = &user_profile.website {
                                        div class="reviewer-website" {
                                            a href=(website) target="_blank" rel="noopener noreferrer" {
                                                "🌐 " (website)
                                            }
                                        }
                                    }
                                    @if let Some(nip05) = &user_profile.nip05 {
                                        div class="reviewer-nip05" {
                                            "✉️ " (nip05)
                                        }
                                    }
                                }
                            } @else {
                                div class="no-avatar-large" { "?" }
                                div class="reviewer-info" {
                                    div class="reviewer-name-large" { "Anonymous Reviewer" }
                                    div class="reviewer-pubkey-large" {
                                        "Pubkey: " (format!("{}...{}",
                                            &rec_with_user.recommendation.reviewer_pubkey[0..16],
                                            &rec_with_user.recommendation.reviewer_pubkey[rec_with_user.recommendation.reviewer_pubkey.len()-16..]
                                        ))
                                    }
                                }
                            }
                        }

                        div class="rating-section" {
                            @if let Some(rating) = rec_with_user.recommendation.rating {
                                div class="rating-display" {
                                    div class="rating-stars" {
                                        @for i in 1..=5 {
                                            @if i <= rating {
                                                span class="star filled" { "⭐" }
                                            } @else {
                                                span class="star" { "☆" }
                                            }
                                        }
                                    }
                                    div class="rating-value" {
                                        (rating) "/5"
                                    }
                                }
                            } @else {
                                div class="no-rating" {
                                    "No numerical rating provided"
                                }
                            }
                        }
                    }

                    // Mint information section
                    div class="mint-section" {
                        div class="section-title" {
                            "🏦 Mint Being Reviewed"
                        }
                        div class="mint-pubkey-display" {
                            div class="mint-label" { "Mint Public Key:" }
                            div class="mint-pubkey-value" { (rec_with_user.recommendation.mint_pubkey) }
                        }
                        @if !rec_with_user.recommendation.invite_codes.is_empty() {
                            div class="invite-codes-section" {
                                div class="invite-label" { "Invite Codes / URLs:" }
                                div class="invite-codes-list" {
                                    @for invite_code in &rec_with_user.recommendation.invite_codes {
                                        div class="invite-code-item" {
                                            a href=(invite_code) target="_blank" rel="noopener noreferrer" {
                                                (invite_code)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Review content section
                    @if let Some(content) = &rec_with_user.recommendation.content {
                        div class="content-section" {
                            div class="section-title" {
                                "📝 Review Content"
                            }
                            div class="review-content-display" {
                                (content)
                            }
                        }
                    }

                    // Metadata section
                    div class="metadata-section" {
                        div class="section-title" {
                            "ℹ️ Review Metadata"
                        }
                        div class="metadata-grid" {
                            div class="metadata-item" {
                                div class="metadata-label" { "Event ID:" }
                                div class="metadata-value event-id" { (rec_with_user.recommendation.event_id) }
                            }
                            div class="metadata-item" {
                                div class="metadata-label" { "Created:" }
                                div class="metadata-value" {
                                    (chrono::DateTime::<chrono::Utc>::from_timestamp(
                                        rec_with_user.recommendation.created_at as i64, 0
                                    ).unwrap_or_default().format("%Y-%m-%d %H:%M:%S UTC"))
                                }
                            }
                            div class="metadata-item" {
                                div class="metadata-label" { "Received:" }
                                div class="metadata-value" {
                                    (rec_with_user.recommendation.received_at.format("%Y-%m-%d %H:%M:%S UTC"))
                                }
                            }
                            div class="metadata-item" {
                                div class="metadata-label" { "D-Tag:" }
                                div class="metadata-value" { (rec_with_user.recommendation.d_tag) }
                            }
                            div class="metadata-item" {
                                div class="metadata-label" { "K-Tag:" }
                                div class="metadata-value" { (rec_with_user.recommendation.k_tag) }
                            }
                        }
                    }
                }
            }
        } @else {
            (render_empty_state("❓", "Review Not Found", "The requested review could not be found or may have been removed."))
        }
    };

    render_layout(
        "Review Detail",
        "Detailed Review Information",
        "reviews",
        content,
    )
}

/// Additional styles specific to the review detail page
fn get_review_detail_styles() -> &'static str {
    "
    .review-detail-container {
        max-width: 900px;
        margin: 0 auto;
    }

    .back-nav {
        margin-bottom: 2rem;
    }

    .back-link {
        display: inline-flex;
        align-items: center;
        gap: 0.5rem;
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 20px;
        padding: 0.75rem 1.5rem;
        color: rgba(255, 255, 255, 0.9);
        text-decoration: none;
        font-weight: 500;
        transition: all 0.3s ease;
    }

    .back-link:hover {
        background: rgba(255, 107, 53, 0.2);
        border-color: var(--primary-orange);
        color: white;
        transform: translateY(-2px);
        box-shadow: 0 8px 25px rgba(255, 107, 53, 0.3);
    }

    .review-detail-card {
        background: var(--glass-bg);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid var(--glass-border);
        border-radius: 24px;
        padding: 2.5rem;
        box-shadow: var(--glass-shadow);
        position: relative;
        overflow: hidden;
    }

    .review-detail-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, var(--primary-orange), var(--secondary-orange));
    }

    .review-detail-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 2rem;
        gap: 2rem;
    }

    .reviewer-section {
        display: flex;
        gap: 1.5rem;
        flex: 1;
    }

    .reviewer-avatar-large {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        object-fit: cover;
        border: 3px solid rgba(255, 255, 255, 0.3);
    }

    .no-avatar-large {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 28px;
        font-weight: bold;
        border: 3px solid rgba(255, 255, 255, 0.3);
    }

    .reviewer-info {
        flex: 1;
    }

    .reviewer-name-large {
        font-size: 1.5rem;
        font-weight: 700;
        color: white;
        margin-bottom: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .reviewer-pubkey-large {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.7);
        font-family: monospace;
        margin-bottom: 0.75rem;
    }

    .reviewer-about {
        color: rgba(255, 255, 255, 0.9);
        line-height: 1.6;
        margin-bottom: 0.75rem;
        font-size: 0.95rem;
    }

    .reviewer-website,
    .reviewer-nip05 {
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.8);
        margin-bottom: 0.5rem;
    }

    .reviewer-website a {
        color: var(--primary-orange);
        text-decoration: none;
        transition: color 0.3s ease;
    }

    .reviewer-website a:hover {
        color: var(--secondary-orange);
    }

    .rating-section {
        text-align: center;
        min-width: 150px;
    }

    .rating-display {
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 20px;
        padding: 1.5rem;
    }

    .rating-stars {
        font-size: 1.5rem;
        margin-bottom: 0.5rem;
        letter-spacing: 0.25rem;
    }

    .star.filled {
        color: var(--secondary-orange);
        text-shadow: 0 2px 4px rgba(247, 147, 26, 0.5);
    }

    .star {
        color: rgba(255, 255, 255, 0.3);
    }

    .rating-value {
        color: white;
        font-size: 1.2rem;
        font-weight: 700;
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .no-rating {
        color: rgba(255, 255, 255, 0.7);
        font-style: italic;
        padding: 1.5rem;
    }

    .section-title {
        font-size: 1.2rem;
        font-weight: 700;
        color: white;
        margin-bottom: 1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .mint-section,
    .content-section,
    .metadata-section {
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 16px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
    }

    .mint-pubkey-display {
        margin-bottom: 1rem;
    }

    .mint-label,
    .invite-label {
        color: rgba(255, 255, 255, 0.8);
        font-weight: 600;
        margin-bottom: 0.5rem;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .mint-pubkey-value {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 0.75rem;
        font-family: monospace;
        font-size: 0.85rem;
        color: white;
        word-break: break-all;
    }

    .invite-codes-list {
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
    }

    .invite-code-item {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 0.75rem;
    }

    .invite-code-item a {
        color: var(--primary-orange);
        text-decoration: none;
        word-break: break-all;
        transition: color 0.3s ease;
    }

    .invite-code-item a:hover {
        color: var(--secondary-orange);
    }

    .review-content-display {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 12px;
        padding: 1.5rem;
        color: rgba(255, 255, 255, 0.95);
        line-height: 1.7;
        white-space: pre-wrap;
        font-size: 1rem;
    }

    .metadata-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        gap: 1rem;
    }

    .metadata-item {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 0.75rem;
    }

    .metadata-label {
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 0.25rem;
        font-weight: 600;
    }

    .metadata-value {
        color: white;
        font-size: 0.9rem;
        word-break: break-word;
    }

    .metadata-value.event-id {
        font-family: monospace;
        font-size: 0.75rem;
    }

    @media (max-width: 768px) {
        .review-detail-header {
            flex-direction: column;
            gap: 1.5rem;
        }

        .reviewer-section {
            flex-direction: column;
            text-align: center;
            gap: 1rem;
        }

        .metadata-grid {
            grid-template-columns: 1fr;
        }

        .review-detail-card {
            padding: 1.5rem;
        }
    }
    "
}
