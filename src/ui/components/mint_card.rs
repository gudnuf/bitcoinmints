use crate::models::MintWithRecommendationsAndInfo;
use maud::{html, Markup};

/// Get mint-specific styles
pub fn get_mint_styles() -> &'static str {
    "
    .mints-grid {
        display: grid;
        gap: 2rem;
        grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
    }

    .mint-card {
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

    .mint-card::before {
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

    .mint-card:hover {
        transform: translateY(-8px);
        box-shadow: 0 16px 50px rgba(31, 38, 135, 0.6);
    }

    .mint-card:hover::before {
        opacity: 1;
    }

    .mint-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1.5rem;
    }

    .mint-title-section {
        flex: 1;
    }

    .mint-name {
        font-size: 1.5rem;
        font-weight: 700;
        color: white;
        margin-bottom: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
    }

    .mint-url {
        font-size: 0.9rem;
        color: rgba(255, 255, 255, 0.8);
        margin-bottom: 0.5rem;
        word-break: break-all;
    }

    .mint-url a {
        color: inherit;
        text-decoration: none;
        transition: color 0.3s ease;
    }

    .mint-url a:hover {
        color: var(--primary-orange);
    }

    .mint-type {
        background: linear-gradient(135deg, var(--primary-blue), var(--light-blue));
        color: white;
        padding: 0.5rem 1rem;
        border-radius: 20px;
        font-size: 0.8rem;
        text-transform: uppercase;
        font-weight: 600;
        letter-spacing: 0.5px;
        box-shadow: 0 4px 12px rgba(30, 64, 175, 0.3);
        white-space: nowrap;
    }

    .mint-description {
        color: rgba(255, 255, 255, 0.9);
        margin-bottom: 1.5rem;
        font-size: 0.95rem;
        line-height: 1.6;
    }

    .info-section {
        background: rgba(255, 255, 255, 0.15);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 16px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
    }

    .info-header {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        margin-bottom: 1rem;
        font-weight: 600;
        color: white;
        font-size: 1.1rem;
    }

    .info-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .info-item {
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 12px;
        padding: 1rem;
    }

    .info-label {
        font-weight: 600;
        color: rgba(255, 255, 255, 0.7);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        margin-bottom: 0.5rem;
    }

    .info-value {
        color: white;
        font-size: 0.9rem;
        word-break: break-word;
        font-weight: 500;
    }

    .nuts-container {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }

    .nut-badge {
        background: linear-gradient(135deg, #10B981, #059669);
        color: white;
        padding: 0.25rem 0.75rem;
        border-radius: 16px;
        font-size: 0.75rem;
        font-weight: 600;
        box-shadow: 0 2px 8px rgba(16, 185, 129, 0.3);
        text-transform: uppercase;
        letter-spacing: 0.25px;
    }

    .contact-list {
        margin-top: 0.5rem;
    }

    .contact-item {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 0.75rem;
        margin-bottom: 0.5rem;
        font-size: 0.85rem;
        color: rgba(255, 255, 255, 0.9);
    }

    .method-grid {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-top: 0.5rem;
    }

    .method-item {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 8px;
        padding: 0.75rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
    }

    .method-unit {
        background: linear-gradient(135deg, #8B5CF6, #A855F7);
        color: white;
        padding: 0.25rem 0.75rem;
        border-radius: 12px;
        font-size: 0.75rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.25px;
        box-shadow: 0 2px 8px rgba(139, 92, 246, 0.3);
        white-space: nowrap;
    }

    .method-details {
        color: rgba(255, 255, 255, 0.9);
        font-size: 0.85rem;
        font-weight: 500;
        text-align: right;
    }

    .ratings-section {
        margin-bottom: 1.5rem;
    }

    .ratings-summary {
        display: flex;
        align-items: center;
        gap: 1rem;
        margin-bottom: 1rem;
        padding: 1rem;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        backdrop-filter: blur(4px);
        -webkit-backdrop-filter: blur(4px);
    }

    .rating-score {
        font-size: 2rem;
        font-weight: 700;
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .rating-details {
        color: rgba(255, 255, 255, 0.8);
        font-weight: 500;
    }

    .reviewers-grid {
        display: flex;
        flex-wrap: wrap;
        gap: 0.75rem;
    }

    .reviewer-card {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 25px;
        padding: 0.75rem 1rem;
        font-size: 0.85rem;
        transition: all 0.3s ease;
    }

    .reviewer-card:hover {
        background: rgba(255, 107, 53, 0.2);
        border-color: var(--primary-orange);
        transform: translateY(-2px);
    }

    .reviewer-avatar {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        object-fit: cover;
        border: 2px solid rgba(255, 255, 255, 0.3);
    }

    .reviewer-name {
        font-weight: 600;
        color: white;
    }

    .reviewer-rating {
        color: var(--secondary-orange);
        font-weight: 700;
        font-size: 0.9rem;
    }

    .no-avatar {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 14px;
        font-weight: bold;
        border: 2px solid rgba(255, 255, 255, 0.3);
    }

    .no-info {
        color: rgba(255, 255, 255, 0.7);
        font-style: italic;
        text-align: center;
        padding: 1rem;
    }

    @media (max-width: 768px) {
        .mints-grid {
            grid-template-columns: 1fr;
        }

        .mint-header {
            flex-direction: column;
            gap: 1rem;
        }
    }
    "
}

/// Helper function to extract supported NUTs from a Nuts struct
fn get_supported_nuts(nuts: &cdk::nuts::Nuts) -> Vec<String> {
    let mut supported = Vec::new();

    // Check each NUT and see if it's supported
    if !nuts.nut04.methods.is_empty() {
        supported.push("04".to_string());
    }
    if !nuts.nut05.methods.is_empty() {
        supported.push("05".to_string());
    }
    if nuts.nut07.supported {
        supported.push("07".to_string());
    }
    if nuts.nut08.supported {
        supported.push("08".to_string());
    }
    if nuts.nut09.supported {
        supported.push("09".to_string());
    }
    if nuts.nut10.supported {
        supported.push("10".to_string());
    }
    if nuts.nut11.supported {
        supported.push("11".to_string());
    }
    if nuts.nut12.supported {
        supported.push("12".to_string());
    }
    if nuts.nut14.supported {
        supported.push("14".to_string());
    }
    if !nuts.nut15.methods.is_empty() {
        supported.push("15".to_string());
    }
    if !nuts.nut17.supported.is_empty() {
        supported.push("17".to_string());
    }
    if !nuts.nut19.cached_endpoints.is_empty() {
        supported.push("19".to_string());
    }
    if nuts.nut20.supported {
        supported.push("20".to_string());
    }

    #[cfg(feature = "auth")]
    {
        if nuts.nut21.is_some() {
            supported.push("21".to_string());
        }
        if nuts.nut22.is_some() {
            supported.push("22".to_string());
        }
    }

    supported
}

/// Render a single mint card
pub fn render_mint_card(mint_with_recs: &MintWithRecommendationsAndInfo) -> Markup {
    html! {
        div class="mint-card" {
            div class="mint-header" {
                div class="mint-title-section" {
                    div class="mint-name" { (mint_with_recs.mint.name) }
                    div class="mint-url" {
                        a href=(mint_with_recs.mint.mint_url) target="_blank" {
                            (mint_with_recs.mint.mint_url)
                        }
                    }
                }
                div class="mint-type" { (mint_with_recs.mint.mint_type) }
            }

            @if let Some(description) = &mint_with_recs.mint.description {
                div class="mint-description" { (description) }
            }

            // Display mint info section if available
            @if let Some(parsed_info) = &mint_with_recs.parsed_info {
                div class="info-section" {
                    div class="info-header" {
                        span { "🏦" }
                        span { "Mint Information" }
                    }

                    div class="info-grid" {
                        @if let Some(version) = &parsed_info.version {
                            div class="info-item" {
                                div class="info-label" { "Version" }
                                div class="info-value" { (version) }
                            }
                        }

                        @if let Some(pubkey) = &parsed_info.pubkey {
                            div class="info-item" {
                                div class="info-label" { "Public Key" }
                                div class="info-value" {
                                    (format!("{}...{}",
                                        &pubkey[0..16.min(pubkey.len())],
                                        &pubkey[pubkey.len().saturating_sub(16)..]
                                    ))
                                }
                            }
                        }

                        @if let Some(motd) = &parsed_info.motd {
                            div class="info-item" {
                                div class="info-label" { "Message of the Day" }
                                div class="info-value" { (motd) }
                            }
                        }

                        @if let Some(stored_info) = &mint_with_recs.stored_info {
                            div class="info-item" {
                                div class="info-label" { "Last Updated" }
                                div class="info-value" {
                                    (stored_info.last_fetched_at.format("%Y-%m-%d %H:%M UTC"))
                                }
                            }
                        }
                    }

                    @if let Some(description_long) = &parsed_info.description_long {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Detailed Description" }
                            div class="info-value" { (description_long) }
                        }
                    }

                    // Display supported NUTs
                    @let supported_nuts = get_supported_nuts(&mint_with_recs.mint.nuts);
                    @if !supported_nuts.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Supported NUTs (Protocols)" }
                            div class="nuts-container" {
                                @for nut in supported_nuts {
                                    span class="nut-badge" { "NUT-" (nut) }
                                }
                            }
                        }
                    }

                    // Display mint settings (NUT-04)
                    @if !mint_with_recs.mint.nuts.nut04.methods.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Minting Support (NUT-04)" }
                            div class="method-grid" {
                                @for method in &mint_with_recs.mint.nuts.nut04.methods {
                                    div class="method-item" {
                                        span class="method-unit" { (format!("{:?}", method.unit)) }
                                        span class="method-details" { (format!("Min: {} - Max: {}", method.min_amount.unwrap_or(cdk::Amount::ZERO), method.max_amount.unwrap_or(cdk::Amount::ZERO))) }
                                    }
                                }
                            }
                        }
                    }

                    // Display melt settings (NUT-05)
                    @if !mint_with_recs.mint.nuts.nut05.methods.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Melting Support (NUT-05)" }
                            div class="method-grid" {
                                @for method in &mint_with_recs.mint.nuts.nut05.methods {
                                    div class="method-item" {
                                        span class="method-unit" { (format!("{:?}", method.unit)) }
                                        span class="method-details" { (format!("Min: {} - Max: {}", method.min_amount.unwrap_or(cdk::Amount::ZERO), method.max_amount.unwrap_or(cdk::Amount::ZERO))) }
                                    }
                                }
                            }
                        }
                    }

                    // Display contact information
                    @if let Some(contact) = &parsed_info.contact {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Contact Information" }
                            div class="contact-list" {
                                @if let Some(contact_array) = contact.as_array() {
                                    @for contact_item in contact_array {
                                        @if let Some(contact_info) = contact_item.as_array() {
                                            @if contact_info.len() >= 2 {
                                                div class="contact-item" {
                                                    strong { (contact_info[0].as_str().unwrap_or("Unknown")) ": " }
                                                    (contact_info[1].as_str().unwrap_or("No info"))
                                                }
                                            }
                                        }
                                    }
                                } @else {
                                    div class="contact-item" { (contact.to_string()) }
                                }
                            }
                        }
                    }
                }
            } @else if let Some(stored_info) = &mint_with_recs.stored_info {
                @if !stored_info.fetch_success {
                    div class="info-section" {
                        div class="info-header" {
                            span { "⚠️" }
                            span { "Mint Information" }
                        }
                        div class="no-info" {
                            "Unable to fetch mint info: "
                            @if let Some(error) = &stored_info.error_message {
                                (error)
                            } @else {
                                "Unknown error"
                            }
                        }
                    }
                }
            } @else {
                div class="info-section" {
                    div class="info-header" {
                        span { "📡" }
                        span { "Mint Information" }
                    }
                    div class="no-info" { "Mint info not yet fetched or available" }
                }
            }

            @if mint_with_recs.total_recommendations > 0 {
                div class="ratings-section" {
                    div class="ratings-summary" {
                        @if let Some(avg_rating) = mint_with_recs.average_rating {
                            div class="rating-score" {
                                "⭐ " (format!("{:.1}", avg_rating))
                            }
                        }
                        div class="rating-details" {
                            (mint_with_recs.total_recommendations) " review"
                            @if mint_with_recs.total_recommendations != 1 { "s" }
                        }
                    }

                    div class="reviewers-grid" {
                        @for rec_with_user in &mint_with_recs.recommendations {
                            div class="reviewer-card" {
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
                                    div class="reviewer-name" {
                                        @if let Some(name) = &user_profile.display_name.as_ref().or(user_profile.name.as_ref()) {
                                            (name)
                                        } @else {
                                            (format!("{}...{}",
                                                &user_profile.pubkey[0..8],
                                                &user_profile.pubkey[user_profile.pubkey.len()-8..]
                                            ))
                                        }
                                    }
                                } @else {
                                    div class="no-avatar" { "?" }
                                    div class="reviewer-name" {
                                        (format!("{}...{}",
                                            &rec_with_user.recommendation.reviewer_pubkey[0..8],
                                            &rec_with_user.recommendation.reviewer_pubkey[rec_with_user.recommendation.reviewer_pubkey.len()-8..]
                                        ))
                                    }
                                }
                                div class="reviewer-rating" {
                                    (rec_with_user.recommendation.rating)
                                }
                            }
                        }
                    }
                }
            } @else {
                div class="ratings-section" {
                    p style="color: #6c757d; font-style: italic;" { "No reviews yet" }
                }
            }
        }
    }
}
