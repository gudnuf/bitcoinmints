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
        padding: 0 1rem;
    }

    .back-nav {
        margin-bottom: 1.5rem;
    }

    .back-link {
        display: inline-flex;
        align-items: center;
        gap: 0.75rem;
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 20px;
        padding: 0.875rem 1.75rem;
        color: var(--text-primary);
        text-decoration: none;
        font-weight: 600;
        transition: all 0.3s ease;
        min-height: 44px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .back-link:hover {
        background: rgba(255, 107, 53, 0.25);
        border-color: var(--primary-orange);
        color: white;
        transform: translateY(-3px);
        box-shadow: 0 10px 30px rgba(255, 107, 53, 0.4);
    }

    .review-detail-card {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 24px;
        padding: 2rem;
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
        background: linear-gradient(90deg, var(--primary-orange), var(--secondary-orange), var(--accent-purple));
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
        min-width: 0;
    }

    .reviewer-avatar-large {
        width: 80px;
        height: 80px;
        border-radius: 50%;
        object-fit: cover;
        border: 2px solid rgba(255, 255, 255, 0.2);
        flex-shrink: 0;
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
        border: 2px solid rgba(255, 255, 255, 0.2);
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
        flex-shrink: 0;
    }

    .reviewer-info {
        flex: 1;
        min-width: 0;
    }

    .reviewer-name-large {
        font-size: 1.5rem;
        font-weight: 800;
        color: var(--text-primary);
        margin-bottom: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
        word-break: break-word;
    }

    .reviewer-pubkey-large {
        font-size: 0.9rem;
        color: var(--text-secondary);
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        margin-bottom: 0.75rem;
        word-break: break-all;
        line-height: 1.4;
    }

    .reviewer-about {
        color: var(--text-primary);
        line-height: 1.6;
        margin-bottom: 0.75rem;
        font-size: 0.95rem;
        font-weight: 500;
    }

    .reviewer-website,
    .reviewer-nip05 {
        font-size: 0.85rem;
        color: var(--text-secondary);
        margin-bottom: 0.5rem;
        font-weight: 500;
    }

    .reviewer-website a {
        color: var(--primary-orange);
        text-decoration: none;
        transition: color 0.3s ease;
        font-weight: 600;
    }

    .reviewer-website a:hover {
        color: var(--secondary-orange);
    }

    .rating-section {
        text-align: center;
        min-width: 150px;
        flex-shrink: 0;
    }

    .rating-display {
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.12);
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
        color: var(--text-primary);
        font-size: 1.2rem;
        font-weight: 800;
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    .no-rating {
        color: var(--text-secondary);
        font-style: italic;
        padding: 1.5rem;
        font-weight: 500;
    }

    .section-title {
        font-size: 1.2rem;
        font-weight: 800;
        color: var(--text-primary);
        margin-bottom: 1rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
    }

    .mint-section,
    .content-section,
    .metadata-section {
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 16px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
    }

    .mint-pubkey-display {
        margin-bottom: 1rem;
    }

    .mint-label,
    .invite-label {
        color: var(--text-secondary);
        font-weight: 700;
        margin-bottom: 0.75rem;
        font-size: 0.9rem;
        text-transform: uppercase;
        letter-spacing: 0.75px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .mint-pubkey-value {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.875rem;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.85rem;
        color: var(--text-primary);
        word-break: break-all;
        line-height: 1.4;
        font-weight: 500;
    }

    .invite-codes-list {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .invite-code-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.875rem;
        transition: all 0.3s ease;
    }

    .invite-code-item:hover {
        background: rgba(255, 255, 255, 0.08);
        border-color: rgba(255, 255, 255, 0.2);
    }

    .invite-code-item a {
        color: var(--primary-orange);
        text-decoration: none;
        word-break: break-all;
        transition: color 0.3s ease;
        font-weight: 600;
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.85rem;
        line-height: 1.4;
    }

    .invite-code-item a:hover {
        color: var(--secondary-orange);
    }

    .review-content-display {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 1.5rem;
        color: var(--text-primary);
        line-height: 1.7;
        white-space: pre-wrap;
        font-size: 1rem;
        font-weight: 500;
    }

    .metadata-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 1rem;
    }

    .metadata-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.875rem;
    }

    .metadata-label {
        color: var(--text-secondary);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.75px;
        margin-bottom: 0.5rem;
        font-weight: 700;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .metadata-value {
        color: var(--text-primary);
        font-size: 0.9rem;
        word-break: break-word;
        font-weight: 600;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .metadata-value.event-id {
        font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
        font-size: 0.75rem;
        line-height: 1.4;
        font-weight: 500;
    }

    /* Mobile responsive design */
    @media (max-width: 480px) {
        .review-detail-container {
            padding: 0 0.75rem;
        }

        .back-nav {
            margin-bottom: 1rem;
        }

        .back-link {
            padding: 0.75rem 1.25rem;
            font-size: 0.9rem;
            gap: 0.5rem;
        }

        .review-detail-card {
            padding: 1rem;
            border-radius: 20px;
        }

        .review-detail-header {
            flex-direction: column;
            gap: 1.5rem;
            margin-bottom: 1.5rem;
        }

        .reviewer-section {
            flex-direction: column;
            text-align: center;
            gap: 1rem;
        }

        .reviewer-avatar-large, .no-avatar-large {
            width: 64px;
            height: 64px;
            font-size: 24px;
            align-self: center;
        }

        .reviewer-name-large {
            font-size: 1.25rem;
        }

        .reviewer-pubkey-large {
            font-size: 0.8rem;
        }

        .reviewer-about {
            font-size: 0.9rem;
            text-align: left;
        }

        .rating-section {
            min-width: auto;
            align-self: stretch;
        }

        .rating-display {
            padding: 1rem;
        }

        .rating-stars {
            font-size: 1.25rem;
        }

        .rating-value {
            font-size: 1.1rem;
        }

        .section-title {
            font-size: 1.1rem;
        }

        .mint-section,
        .content-section,
        .metadata-section {
            padding: 1rem;
            border-radius: 12px;
        }

        .mint-pubkey-value {
            font-size: 0.75rem;
            padding: 0.75rem;
        }

        .invite-code-item {
            padding: 0.75rem;
        }

        .invite-code-item a {
            font-size: 0.75rem;
        }

        .review-content-display {
            padding: 1rem;
            font-size: 0.95rem;
        }

        .metadata-grid {
            gap: 0.75rem;
        }

        .metadata-item {
            padding: 0.75rem;
        }

        .metadata-value.event-id {
            font-size: 0.7rem;
        }
    }

    @media (min-width: 481px) and (max-width: 768px) {
        .review-detail-container {
            padding: 0 1rem;
        }

        .review-detail-header {
            flex-wrap: wrap;
            gap: 1.5rem;
        }

        .reviewer-section {
            min-width: 300px;
        }

        .metadata-grid {
            grid-template-columns: repeat(2, 1fr);
        }
    }

    @media (min-width: 769px) {
        .review-detail-container {
            padding: 0 2rem;
        }

        .back-nav {
            margin-bottom: 2rem;
        }

        .review-detail-card {
            padding: 2.5rem;
        }

        .metadata-grid {
            grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
        }
    }
    "
}
