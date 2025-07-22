use crate::models::RecommendationWithUser;
use maud::{html, Markup};

/// Get review-specific styles
pub fn get_review_styles() -> &'static str {
    "
    .reviews-grid {
        display: grid;
        gap: 2rem;
        grid-template-columns: repeat(auto-fit, minmax(450px, 1fr));
    }

    .review-card {
        background: var(--glass-bg);
        backdrop-filter: blur(16px);
        -webkit-backdrop-filter: blur(16px);
        border: 1px solid var(--glass-border);
        border-radius: 24px;
        padding: 2rem;
        box-shadow: var(--glass-shadow);
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
    }

    .review-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, var(--primary-orange), var(--secondary-orange));
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .review-card:hover {
        transform: translateY(-8px);
        box-shadow: 0 16px 50px rgba(31, 38, 135, 0.6);
    }

    .review-card:hover::before {
        opacity: 1;
    }

    .review-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1.5rem;
    }

    .reviewer-info {
        display: flex;
        align-items: center;
        gap: 1rem;
    }

    .reviewer-avatar {
        width: 50px;
        height: 50px;
        border-radius: 50%;
        object-fit: cover;
        border: 3px solid rgba(255, 255, 255, 0.3);
    }

    .no-avatar {
        width: 50px;
        height: 50px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 18px;
        font-weight: bold;
        border: 3px solid rgba(255, 255, 255, 0.3);
    }

    .reviewer-details {
        flex: 1;
    }

    .reviewer-name {
        font-weight: 700;
        font-size: 1.1rem;
        color: white;
        margin-bottom: 0.25rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .reviewer-pubkey {
        font-size: 0.8rem;
        color: rgba(255, 255, 255, 0.7);
        font-family: monospace;
        word-break: break-all;
    }

    .review-rating {
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        color: white;
        padding: 0.75rem 1.25rem;
        border-radius: 50px;
        font-size: 1.2rem;
        font-weight: 700;
        box-shadow: 0 4px 15px rgba(255, 107, 53, 0.4);
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.3);
    }

    .mint-info {
        background: rgba(255, 255, 255, 0.15);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 12px;
        padding: 1rem;
        margin-bottom: 1.5rem;
    }

    .mint-label {
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
        margin-bottom: 0.5rem;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
    }

    .mint-pubkey {
        font-family: monospace;
        font-size: 0.85rem;
        color: white;
        word-break: break-all;
        background: rgba(255, 255, 255, 0.1);
        padding: 0.5rem;
        border-radius: 6px;
        border: 1px solid rgba(255, 255, 255, 0.15);
    }

    .review-content {
        color: rgba(255, 255, 255, 0.95);
        margin-bottom: 1.5rem;
        font-size: 1rem;
        line-height: 1.7;
        white-space: pre-wrap;
        background: rgba(255, 255, 255, 0.1);
        padding: 1.25rem;
        border-radius: 12px;
        border: 1px solid rgba(255, 255, 255, 0.15);
    }

    .review-meta {
        display: flex;
        justify-content: space-between;
        align-items: center;
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.85rem;
        border-top: 1px solid rgba(255, 255, 255, 0.2);
        padding-top: 1rem;
        font-weight: 500;
    }

    .invite-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .invite-badge {
        background: linear-gradient(135deg, var(--primary-blue), var(--light-blue));
        color: white;
        padding: 0.25rem 0.75rem;
        border-radius: 12px;
        font-size: 0.75rem;
        font-weight: 600;
        box-shadow: 0 2px 8px rgba(30, 64, 175, 0.3);
    }

    @media (max-width: 768px) {
        .reviews-grid {
            grid-template-columns: 1fr;
        }

        .review-header {
            flex-direction: column;
            gap: 1rem;
            align-items: flex-start;
        }

        .reviewer-info {
            width: 100%;
        }
    }
    "
}

/// Render a single review card
pub fn render_review_card(rec_with_user: &RecommendationWithUser) -> Markup {
    html! {
        div class="review-card" {
            div class="review-header" {
                div class="reviewer-info" {
                    @if let Some(user_profile) = &rec_with_user.user_profile {
                        @if let Some(picture) = &user_profile.picture {
                            img class="reviewer-avatar" src=(picture) alt="Avatar";
                        } @else {
                            div class="no-avatar" {
                                @if let Some(name) = &user_profile.display_name.as_ref().or(user_profile.name.as_ref()) {
                                    (name.chars().next().unwrap_or('?').to_uppercase())
                                } @else {
                                    "?"
                                }
                            }
                        }
                        div class="reviewer-details" {
                            div class="reviewer-name" {
                                @if let Some(name) = &user_profile.display_name.as_ref().or(user_profile.name.as_ref()) {
                                    (name)
                                } @else {
                                    "Anonymous"
                                }
                            }
                            div class="reviewer-pubkey" {
                                (format!("{}...{}",
                                    &user_profile.pubkey[0..16],
                                    &user_profile.pubkey[user_profile.pubkey.len()-16..]
                                ))
                            }
                        }
                    } @else {
                        div class="no-avatar" { "?" }
                        div class="reviewer-details" {
                            div class="reviewer-name" { "Anonymous" }
                            div class="reviewer-pubkey" {
                                (format!("{}...{}",
                                    &rec_with_user.recommendation.reviewer_pubkey[0..16],
                                    &rec_with_user.recommendation.reviewer_pubkey[rec_with_user.recommendation.reviewer_pubkey.len()-16..]
                                ))
                            }
                        }
                    }
                }
                div class="review-rating" {
                    "⭐ " (rec_with_user.recommendation.rating)
                }
            }

            div class="mint-info" {
                div class="mint-label" { "Reviewed Mint:" }
                div class="mint-pubkey" { (rec_with_user.recommendation.mint_pubkey) }
            }

            @if let Some(content) = &rec_with_user.recommendation.content {
                div class="review-content" { (content) }
            }

            div class="review-meta" {
                div class="invite-info" {
                    @if !rec_with_user.recommendation.invite_codes.is_empty() {
                        span class="invite-badge" {
                            (rec_with_user.recommendation.invite_codes.len()) " Invite Codes"
                        }
                    } @else {
                        span { "No invite codes" }
                    }
                }
                div {
                    (chrono::DateTime::<chrono::Utc>::from_timestamp(
                        rec_with_user.recommendation.created_at as i64, 0
                    ).unwrap_or_default().format("%Y-%m-%d %H:%M UTC"))
                }
            }
        }
    }
}
