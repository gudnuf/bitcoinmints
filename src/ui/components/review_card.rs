use crate::models::RecommendationWithUser;
use maud::{html, Markup};

/// Get review-specific styles
pub fn get_review_styles() -> &'static str {
    "
    .reviews-grid {
        display: grid;
        gap: 1.5rem;
        grid-template-columns: 1fr;
    }

    .review-card-link {
        text-decoration: none;
        color: inherit;
        display: block;
        transition: all 0.3s ease;
    }

    .review-card {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 24px;
        padding: 1.5rem;
        box-shadow: var(--glass-shadow);
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
        cursor: pointer;
    }

    .review-card::before {
        content: '';
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 4px;
        background: linear-gradient(90deg, var(--primary-orange), var(--secondary-orange), var(--accent-purple));
        opacity: 0;
        transition: opacity 0.3s ease;
    }

    .review-card:hover {
        transform: translateY(-8px);
        box-shadow: 0 20px 60px rgba(31, 38, 135, 0.7);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .review-card:hover::before {
        opacity: 1;
    }

    .review-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1.5rem;
        gap: 1rem;
    }

    .reviewer-info {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex: 1;
        min-width: 0;
    }

    .reviewer-avatar {
        width: 56px;
        height: 56px;
        border-radius: 50%;
        object-fit: cover;
        border: 2px solid rgba(255, 255, 255, 0.2);
        flex-shrink: 0;
    }

    .no-avatar {
        width: 56px;
        height: 56px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 20px;
        font-weight: bold;
        border: 2px solid rgba(255, 255, 255, 0.2);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        flex-shrink: 0;
    }

    .reviewer-details {
        flex: 1;
        min-width: 0;
    }

    .reviewer-name {
        font-weight: 800;
        font-size: 1.15rem;
        color: var(--text-primary);
        margin-bottom: 0.25rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
        word-break: break-word;
    }

    .reviewer-pubkey {
        font-size: 0.8rem;
        color: var(--text-secondary);
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        word-break: break-all;
        line-height: 1.3;
    }

    .review-rating {
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        color: white;
        padding: 0.875rem 1.5rem;
        border-radius: 50px;
        font-size: 1.3rem;
        font-weight: 800;
        box-shadow: 0 6px 20px rgba(255, 107, 53, 0.5);
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
        white-space: nowrap;
        flex-shrink: 0;
    }

    .mint-info {
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 16px;
        padding: 1.25rem;
        margin-bottom: 1.5rem;
    }

    .mint-label {
        font-weight: 700;
        color: var(--text-secondary);
        margin-bottom: 0.75rem;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.75px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .mint-pubkey {
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.8rem;
        color: var(--text-primary);
        word-break: break-all;
        background: rgba(255, 255, 255, 0.05);
        padding: 0.75rem;
        border-radius: 8px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        line-height: 1.4;
        font-weight: 500;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .review-content {
        color: var(--text-primary);
        margin-bottom: 1.5rem;
        font-size: 1rem;
        line-height: 1.7;
        white-space: pre-wrap;
        background: rgba(255, 255, 255, 0.05);
        padding: 1.5rem;
        border-radius: 16px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        font-weight: 500;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .review-meta {
        display: flex;
        justify-content: space-between;
        align-items: center;
        color: var(--text-secondary);
        font-size: 0.85rem;
        border-top: 2px solid rgba(255, 255, 255, 0.25);
        padding-top: 1rem;
        font-weight: 600;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .invite-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .invite-badge {
        background: linear-gradient(135deg, var(--primary-blue), var(--light-blue));
        color: white;
        padding: 0.375rem 0.875rem;
        border-radius: 16px;
        font-size: 0.75rem;
        font-weight: 700;
        box-shadow: 0 4px 12px rgba(30, 64, 175, 0.4);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        letter-spacing: 0.25px;
    }

    /* Mobile responsive design */
    @media (max-width: 480px) {
        .reviews-grid {
            gap: 1rem;
        }

        .review-card {
            padding: 1rem;
            border-radius: 20px;
        }

        .review-header {
            flex-direction: column;
            gap: 1rem;
            align-items: stretch;
        }

        .reviewer-info {
            gap: 0.75rem;
        }

        .reviewer-avatar, .no-avatar {
            width: 48px;
            height: 48px;
            font-size: 18px;
        }

        .reviewer-name {
            font-size: 1rem;
        }

        .reviewer-pubkey {
            font-size: 0.75rem;
        }

        .review-rating {
            padding: 0.75rem 1.25rem;
            font-size: 1.1rem;
            align-self: flex-start;
        }

        .mint-info {
            padding: 1rem;
            border-radius: 12px;
        }

        .mint-label {
            font-size: 0.8rem;
        }

        .mint-pubkey {
            font-size: 0.75rem;
            padding: 0.625rem;
        }

        .review-content {
            padding: 1rem;
            font-size: 0.95rem;
            border-radius: 12px;
        }

        .review-meta {
            font-size: 0.8rem;
            flex-direction: column;
            align-items: stretch;
            gap: 0.75rem;
        }

        .invite-badge {
            padding: 0.5rem 0.75rem;
            font-size: 0.7rem;
        }
    }

    @media (min-width: 481px) and (max-width: 768px) {
        .reviews-grid {
            gap: 1.25rem;
        }

        .review-header {
            flex-wrap: wrap;
            gap: 1rem;
        }

        .reviewer-info {
            min-width: 250px;
        }
    }

    @media (min-width: 769px) {
        .reviews-grid {
            grid-template-columns: repeat(auto-fit, minmax(500px, 1fr));
            gap: 2rem;
        }

        .review-card {
            padding: 2rem;
        }

        .mint-info {
            padding: 1.5rem;
        }

        .review-content {
            padding: 1.75rem;
        }
    }
    "
}

/// Render a single review card
pub fn render_review_card(rec_with_user: &RecommendationWithUser) -> Markup {
    html! {
        a href=(format!("/review/{}", rec_with_user.recommendation.event_id)) class="review-card-link" {
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
                    @if let Some(rating) = rec_with_user.recommendation.rating {
                        "⭐ " (rating)
                    } @else {
                        span style="color: rgba(255, 255, 255, 0.5); font-style: italic;" { "No rating" }
                    }
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
}
