use crate::models::MintWithRecommendationsAndInfo;
use maud::{html, Markup};
use std::collections::HashMap;

/// Currency capability information
#[derive(Debug, Clone)]
pub struct CurrencyCapability {
    pub unit: String,
    pub can_mint: bool,
    pub can_melt: bool,
}

/// NUT capability information
#[derive(Debug, Clone)]
pub struct NutCapability {
    pub nut_number: String,
    pub description: String,
    pub is_supported: bool,
}

/// Extract all currency capabilities from a list of mints
pub fn extract_currency_capabilities(
    mints: &[MintWithRecommendationsAndInfo],
) -> Vec<CurrencyCapability> {
    let mut currencies: HashMap<String, CurrencyCapability> = HashMap::new();

    for mint in mints {
        // Only process Cashu mints
        if mint.mint.mint_type.to_lowercase() != "cashu" {
            continue;
        }

        // Check minting capabilities (NUT-04)
        for method in &mint.mint.nuts.nut04.methods {
            let unit = format!("{:?}", method.unit).to_lowercase();
            currencies
                .entry(unit.clone())
                .or_insert_with(|| CurrencyCapability {
                    unit: unit.clone(),
                    can_mint: false,
                    can_melt: false,
                })
                .can_mint = true;
        }

        // Check melting capabilities (NUT-05)
        for method in &mint.mint.nuts.nut05.methods {
            let unit = format!("{:?}", method.unit).to_lowercase();
            currencies
                .entry(unit.clone())
                .or_insert_with(|| CurrencyCapability {
                    unit: unit.clone(),
                    can_mint: false,
                    can_melt: false,
                })
                .can_melt = true;
        }
    }

    let mut result: Vec<CurrencyCapability> = currencies.into_values().collect();
    result.sort_by(|a, b| a.unit.cmp(&b.unit));
    result
}

/// Extract all NUT capabilities from a list of mints
pub fn extract_nut_capabilities(mints: &[MintWithRecommendationsAndInfo]) -> Vec<NutCapability> {
    let mut nuts: HashMap<String, bool> = HashMap::new();

    // Define NUT descriptions
    let nut_descriptions: HashMap<&str, &str> = [
        ("04", "Minting tokens"),
        ("05", "Melting tokens"),
        ("07", "Token state check"),
        ("08", "Overpaid fees"),
        ("09", "Restore signatures"),
        ("10", "Spending conditions"),
        ("11", "Pay-to-Public-Key-Hash"),
        ("12", "DLEQ proofs"),
        ("14", "Hashed Time Locked Contracts"),
        ("15", "Partial multi-path payments"),
        ("17", "WebSocket subscriptions"),
        ("19", "Cached endpoints"),
        ("20", "Faster payments"),
        ("21", "Authentication"),
        ("22", "Multi-party payments"),
    ]
    .iter()
    .cloned()
    .collect();

    for mint in mints {
        // Only process Cashu mints
        if mint.mint.mint_type.to_lowercase() != "cashu" {
            continue;
        }

        let supported_nuts = get_supported_nuts(&mint.mint.nuts);
        for nut in supported_nuts {
            nuts.insert(nut, true);
        }
    }

    let mut result: Vec<NutCapability> = nuts
        .into_iter()
        .map(|(nut_number, _)| NutCapability {
            description: nut_descriptions
                .get(nut_number.as_str())
                .unwrap_or(&"Unknown protocol")
                .to_string(),
            nut_number,
            is_supported: true,
        })
        .collect();

    result.sort_by(|a, b| a.nut_number.cmp(&b.nut_number));
    result
}

/// Render Cashu-specific currency and NUT filters
pub fn render_cashu_filters(
    capabilities: &[CurrencyCapability],
    nut_capabilities: &[NutCapability],
) -> Markup {
    html! {
        div class="cashu-filters" id="cashu-filters" {
            div class="cashu-filter-header" {
                span class="filter-icon" { "🟠" }
                span class="filter-title" { "Cashu Filters" }
            }

            @if capabilities.is_empty() && nut_capabilities.is_empty() {
                div class="no-filters-message" {
                    "No Cashu mints with filter information available"
                }
            } @else {
                div class="filter-sections" {
                    // Currency Filters Section
                    @if !capabilities.is_empty() {
                        div class="filter-section" {
                            div class="section-title" { "Currency Support" }
                            div class="currency-grid" {
                                @for capability in capabilities {
                                    div class="currency-item" {
                                        div class="currency-header" { (capability.unit.to_uppercase()) }
                                        div class="currency-options" {
                                            @if capability.can_mint {
                                                label class="filter-option" {
                                                    input
                                                        type="checkbox"
                                                        class="filter-checkbox mint-filter"
                                                        data-currency=(capability.unit)
                                                        onchange="updateCashuFilters()";
                                                    span class="option-label" { "Mint" }
                                                }
                                            }
                                            @if capability.can_melt {
                                                label class="filter-option" {
                                                    input
                                                        type="checkbox"
                                                        class="filter-checkbox melt-filter"
                                                        data-currency=(capability.unit)
                                                        onchange="updateCashuFilters()";
                                                    span class="option-label" { "Melt" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // NUT Filters Section
                    @if !nut_capabilities.is_empty() {
                        div class="filter-section" {
                            div class="section-title" { "Protocol Support (NUTs)" }
                            div class="nuts-grid" {
                                @for nut in nut_capabilities {
                                    label class="nut-option" {
                                        input
                                            type="checkbox"
                                            class="filter-checkbox nut-filter"
                                            data-nut=(nut.nut_number)
                                            onchange="updateCashuFilters()";
                                        span class="nut-label" { "NUT-" (nut.nut_number) }
                                        span class="nut-description" { (nut.description) }
                                    }
                                }
                            }
                        }
                    }
                }

                div class="filter-actions" {
                    button class="filter-action-btn clear" onclick="clearCashuFilters()" {
                        "Clear All"
                    }
                    button class="filter-action-btn select" onclick="selectAllCashuFilters()" {
                        "Select All"
                    }
                }
            }
        }
    }
}

/// Get mint-specific styles
pub fn get_mint_styles() -> &'static str {
    "
    .mints-grid {
        display: grid;
        gap: 1.5rem;
        grid-template-columns: 1fr;
    }

    .filter-section {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 24px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
        box-shadow: var(--glass-shadow);
    }

    .filter-title {
        color: var(--text-primary);
        font-size: 1.1rem;
        font-weight: 700;
        margin-bottom: 1rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
    }

    .filter-buttons {
        display: flex;
        gap: 0.75rem;
        flex-wrap: wrap;
    }

    .filter-btn {
        background: rgba(255, 255, 255, 0.08);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 20px;
        padding: 0.875rem 1.5rem;
        color: var(--text-primary);
        font-size: 16px; /* Prevent iOS zoom */
        font-weight: 700;
        cursor: pointer;
        transition: all 0.3s ease;
        white-space: nowrap;
        min-height: 44px;
        display: flex;
        align-items: center;
        justify-content: center;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        /* iOS optimizations */
        -webkit-appearance: none;
        appearance: none;
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        -webkit-tap-highlight-color: transparent;
    }

    .filter-btn:hover,
    .filter-btn:active {
        background: rgba(255, 107, 53, 0.25);
        border-color: var(--primary-orange);
        color: white;
        transform: translateY(-3px) translateZ(0);
        box-shadow: 0 10px 30px rgba(255, 107, 53, 0.4);
    }

    .filter-btn.active {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        border-color: var(--primary-orange);
        color: white;
        box-shadow: 0 10px 30px rgba(255, 107, 53, 0.5);
        transform: translateY(-2px) translateZ(0);
    }

    .filter-btn.active:hover,
    .filter-btn.active:active {
        transform: translateY(-4px) translateZ(0);
        box-shadow: 0 15px 40px rgba(255, 107, 53, 0.6);
    }

    .cashu-filters {
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 2px solid rgba(255, 107, 53, 0.3);
        border-radius: 16px;
        padding: 1rem;
        margin-top: 1rem;
        display: none;
    }

    .cashu-filters.active {
        display: block;
        animation: slideDown 0.3s ease;
    }

    .cashu-filter-header {
        color: var(--text-primary);
        font-size: 0.95rem;
        font-weight: 700;
        margin-bottom: 0.75rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .filter-icon {
        font-size: 1.1rem;
    }

    .filter-title {
        color: var(--text-primary);
    }

    .no-filters-message {
        color: rgba(255, 255, 255, 0.7);
        font-style: italic;
        text-align: center;
        padding: 0.75rem;
        font-size: 0.85rem;
    }

    .filter-sections {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
    }

    .filter-section {
        background: rgba(255, 255, 255, 0.08);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 12px;
        padding: 0.75rem;
    }

    .section-title {
        color: var(--text-primary);
        font-weight: 600;
        font-size: 0.8rem;
        margin-bottom: 0.5rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        opacity: 0.9;
    }

    .currency-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
        gap: 0.5rem;
    }

    .currency-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.5rem;
    }

    .currency-header {
        color: var(--text-primary);
        font-weight: 600;
        font-size: 0.75rem;
        margin-bottom: 0.4rem;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .currency-options {
        display: flex;
        gap: 0.4rem;
        flex-wrap: wrap;
    }

    .filter-option {
        display: flex;
        align-items: center;
        gap: 0.3rem;
        cursor: pointer;
        min-height: 32px;
        padding: 0.2rem 0.4rem;
        border-radius: 6px;
        transition: background 0.2s ease;
    }

    .filter-option:hover {
        background: rgba(255, 255, 255, 0.08);
    }

    .nuts-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 0.4rem;
    }

    .nut-option {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        cursor: pointer;
        min-height: 36px;
        padding: 0.4rem 0.6rem;
        border-radius: 8px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        transition: all 0.2s ease;
    }

    .nut-option:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 107, 53, 0.3);
    }

    .nut-label {
        color: var(--primary-orange);
        font-weight: 600;
        font-size: 0.75rem;
        min-width: 50px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .nut-description {
        color: rgba(255, 255, 255, 0.8);
        font-size: 0.7rem;
        flex: 1;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .filter-checkbox {
        appearance: none;
        -webkit-appearance: none;
        width: 16px;
        height: 16px;
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 4px;
        background: transparent;
        cursor: pointer;
        position: relative;
        transition: all 0.3s ease;
        flex-shrink: 0;
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        -webkit-tap-highlight-color: transparent;
    }

    .filter-checkbox:checked {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        border-color: var(--primary-orange);
        box-shadow: 0 2px 8px rgba(255, 107, 53, 0.4);
        transform: translateZ(0);
    }

    .filter-checkbox:checked::after {
        content: '✓';
        position: absolute;
        top: 50%;
        left: 50%;
        transform: translate(-50%, -50%);
        color: white;
        font-size: 11px;
        font-weight: bold;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .option-label {
        color: var(--text-primary);
        font-size: 0.7rem;
        font-weight: 500;
        cursor: pointer;
        user-select: none;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .filter-actions {
        display: flex;
        gap: 0.5rem;
        margin-top: 0.75rem;
        justify-content: center;
        flex-wrap: wrap;
    }

    .filter-action-btn {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        border-radius: 12px;
        padding: 0.5rem 0.75rem;
        color: var(--text-primary);
        font-size: 0.75rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
        min-height: 32px;
        display: flex;
        align-items: center;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .filter-action-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        color: white;
        transform: translateY(-1px);
    }

    .filter-action-btn.clear {
        background: rgba(239, 68, 68, 0.2);
        border-color: rgba(239, 68, 68, 0.3);
        color: #FCA5A5;
    }

    .filter-action-btn.clear:hover {
        background: rgba(239, 68, 68, 0.4);
        color: white;
        transform: translateY(-2px);
    }

    .mint-card {
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid rgba(255, 107, 53, 0.3);
        border-radius: 24px;
        padding: 1.5rem;
        box-shadow: var(--glass-shadow);
        transition: all 0.3s ease;
        position: relative;
        overflow: hidden;
        /* iOS optimizations */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        -webkit-tap-highlight-color: transparent;
    }

    .mint-card::before {
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

    .mint-card:hover,
    .mint-card:active {
        transform: translateY(-8px) translateZ(0);
        box-shadow: 0 20px 60px rgba(31, 38, 135, 0.7);
        border-color: rgba(255, 107, 53, 0.5);
        background: rgba(0, 0, 0, 0.7);
    }

    .mint-card:hover::before,
    .mint-card:active::before {
        opacity: 1;
    }

    .mint-header {
        display: flex;
        justify-content: space-between;
        align-items: flex-start;
        margin-bottom: 1.5rem;
        gap: 1rem;
    }

    .mint-title-section {
        flex: 1;
        min-width: 0;
    }

    .mint-name {
        font-size: 1.4rem;
        font-weight: 800;
        color: var(--text-primary);
        margin-bottom: 0.5rem;
        text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
        word-break: break-word;
    }

    .mint-url {
        font-size: 0.85rem;
        color: var(--text-secondary);
        margin-bottom: 0.5rem;
        word-break: break-all;
        line-height: 1.4;
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
        padding: 0.625rem 1.25rem;
        border-radius: 20px;
        font-size: 0.8rem;
        text-transform: uppercase;
        font-weight: 700;
        letter-spacing: 0.75px;
        box-shadow: 0 6px 16px rgba(30, 64, 175, 0.4);
        white-space: nowrap;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        flex-shrink: 0;
    }

    .health-status {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        margin-top: 0.75rem;
        flex-wrap: wrap;
    }

    .health-indicator {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.625rem 1.25rem;
        border-radius: 20px;
        font-size: 0.8rem;
        font-weight: 700;
        letter-spacing: 0.5px;
        white-space: nowrap;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .health-online {
        background: linear-gradient(135deg, var(--accent-green), #059669);
        color: white;
        box-shadow: 0 6px 16px rgba(16, 185, 129, 0.4);
    }

    .health-offline {
        background: linear-gradient(135deg, var(--accent-red), #DC2626);
        color: white;
        box-shadow: 0 6px 16px rgba(239, 68, 68, 0.4);
    }

    .health-warning {
        background: linear-gradient(135deg, var(--accent-yellow), #D97706);
        color: white;
        box-shadow: 0 6px 16px rgba(245, 158, 11, 0.4);
    }

    .health-dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: currentColor;
        animation: pulse 2s infinite;
    }

    .uptime-bar {
        background: rgba(255, 255, 255, 0.25);
        border-radius: 8px;
        height: 8px;
        overflow: hidden;
        margin-top: 0.5rem;
        flex: 1;
        min-width: 100px;
    }

    .uptime-fill {
        height: 100%;
        border-radius: 8px;
        transition: width 0.3s ease;
    }

    .uptime-excellent {
        background: linear-gradient(90deg, var(--accent-green), #34D399);
    }

    .uptime-good {
        background: linear-gradient(90deg, var(--accent-yellow), #FBBF24);
    }

    .uptime-poor {
        background: linear-gradient(90deg, var(--accent-red), #F87171);
    }

    @keyframes pulse {
        0%, 100% { opacity: 1; }
        50% { opacity: 0.6; }
    }

    .mint-description {
        color: var(--text-secondary);
        margin-bottom: 1.5rem;
        font-size: 0.95rem;
        line-height: 1.6;
    }

    .info-section {
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 1px solid rgba(255, 255, 255, 0.12);
        border-radius: 16px;
        padding: 1.25rem;
        margin-bottom: 1.25rem;
    }

    .info-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 1rem;
        font-weight: 700;
        color: var(--text-primary);
        font-size: 1.05rem;
        cursor: pointer;
        padding: 0.75rem;
        border-radius: 12px;
        transition: background 0.3s ease;
        min-height: 44px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .info-header:hover {
        background: rgba(255, 255, 255, 0.08);
    }

    .info-header-left {
        display: flex;
        align-items: center;
        gap: 0.5rem;
    }

    .info-expand-indicator {
        color: var(--primary-orange);
        font-size: 1.2rem;
        transition: transform 0.3s ease;
        filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
    }

    .info-expand-indicator.expanded {
        transform: rotate(180deg);
    }

    .info-content {
        display: none;
        animation: slideDown 0.3s ease;
    }

    .info-content.expanded {
        display: block;
    }

    .info-grid {
        display: grid;
        grid-template-columns: 1fr;
        gap: 1rem;
        margin-bottom: 1rem;
    }

    .info-item {
        background: rgba(255, 255, 255, 0.05);
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        padding: 1rem;
    }

    .info-label {
        font-weight: 700;
        color: var(--text-secondary);
        font-size: 0.8rem;
        text-transform: uppercase;
        letter-spacing: 0.75px;
        margin-bottom: 0.5rem;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .info-value {
        color: var(--text-primary);
        font-size: 0.9rem;
        word-break: break-word;
        font-weight: 600;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .nuts-container {
        display: flex;
        flex-wrap: wrap;
        gap: 0.5rem;
        margin-top: 0.5rem;
    }

    .nut-badge {
        background: linear-gradient(135deg, var(--accent-green), #059669);
        color: white;
        padding: 0.375rem 0.875rem;
        border-radius: 16px;
        font-size: 0.75rem;
        font-weight: 700;
        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
        text-transform: uppercase;
        letter-spacing: 0.5px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .contact-list {
        margin-top: 0.5rem;
    }

    .contact-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.875rem;
        margin-bottom: 0.5rem;
        font-size: 0.85rem;
        color: var(--text-secondary);
        font-weight: 500;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .method-grid {
        display: flex;
        flex-direction: column;
        gap: 0.75rem;
        margin-top: 0.5rem;
    }

    .method-item {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 0.875rem;
        display: flex;
        justify-content: space-between;
        align-items: center;
        gap: 1rem;
        flex-wrap: wrap;
    }

    .method-unit {
        background: linear-gradient(135deg, var(--accent-purple), #A855F7);
        color: white;
        padding: 0.375rem 0.875rem;
        border-radius: 12px;
        font-size: 0.75rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.5px;
        box-shadow: 0 4px 12px rgba(139, 92, 246, 0.4);
        white-space: nowrap;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .method-details {
        color: var(--text-secondary);
        font-size: 0.85rem;
        font-weight: 600;
        text-align: right;
        flex: 1;
        min-width: 0;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
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
        background: rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        flex-wrap: wrap;
    }

    .rating-score {
        font-size: 2rem;
        font-weight: 800;
        background: linear-gradient(135deg, var(--secondary-orange), var(--primary-orange));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    }

    .rating-details {
        color: var(--text-secondary);
        font-weight: 600;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .reviews-summary-container {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 1rem;
        cursor: pointer;
        padding: 0.75rem;
        border-radius: 12px;
        transition: background 0.3s ease;
        min-height: 44px;
    }

    .reviews-summary-container:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .reviewers-stack {
        display: flex;
        align-items: center;
        position: relative;
    }

    .reviewer-bubble {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        border: 2px solid rgba(255, 255, 255, 0.2);
        margin-left: -10px;
        transition: all 0.3s ease;
        position: relative;
        z-index: 1;
    }

    .reviewer-bubble:first-child {
        margin-left: 0;
    }

    .reviewer-bubble:hover {
        transform: translateY(-6px) scale(1.15);
        z-index: 10;
        border-color: var(--primary-orange);
        box-shadow: 0 8px 20px rgba(255, 107, 53, 0.4);
    }

    .reviewer-avatar {
        width: 100%;
        height: 100%;
        border-radius: 50%;
        object-fit: cover;
    }

    .no-avatar {
        width: 100%;
        height: 100%;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 14px;
        font-weight: bold;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .more-reviewers {
        background: rgba(255, 255, 255, 0.25);
        color: white;
        font-size: 11px;
        font-weight: 700;
        display: flex;
        align-items: center;
        justify-content: center;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .expand-indicator {
        color: var(--primary-orange);
        font-size: 1.4rem;
        transition: transform 0.3s ease;
        filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
    }

    .expand-indicator.expanded {
        transform: rotate(180deg);
    }

    .reviewers-grid {
        display: none;
        flex-wrap: wrap;
        gap: 0.75rem;
        margin-top: 1rem;
        animation: slideDown 0.3s ease;
    }

    .reviewers-grid.expanded {
        display: flex;
    }

    @keyframes slideDown {
        from {
            opacity: 0;
            transform: translateY(-10px);
        }
        to {
            opacity: 1;
            transform: translateY(0);
        }
    }

    .reviewer-card-link {
        text-decoration: none;
        color: inherit;
        display: block;
        transition: all 0.3s ease;
    }

    .reviewer-card-link:hover {
        transform: translateY(-3px);
    }

    .reviewer-card {
        display: flex;
        align-items: center;
        gap: 0.75rem;
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(8px);
        -webkit-backdrop-filter: blur(8px);
        border: 1px solid rgba(255, 255, 255, 0.15);
        border-radius: 25px;
        padding: 0.875rem 1.25rem;
        font-size: 0.85rem;
        transition: all 0.3s ease;
        cursor: pointer;
        min-height: 44px;
    }

    .reviewer-card:hover {
        background: rgba(255, 107, 53, 0.2);
        border-color: var(--primary-orange);
        box-shadow: 0 8px 20px rgba(255, 107, 53, 0.3);
    }

    .reviewer-card .reviewer-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        object-fit: cover;
        border: 1px solid rgba(255, 255, 255, 0.2);
    }

    .reviewer-card .no-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 16px;
        font-weight: bold;
        border: 1px solid rgba(255, 255, 255, 0.2);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .reviewer-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        object-fit: cover;
        border: 1px solid rgba(255, 255, 255, 0.2);
    }

    .reviewer-name {
        font-weight: 700;
        color: var(--text-primary);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .reviewer-rating {
        color: var(--secondary-orange);
        font-weight: 800;
        font-size: 0.9rem;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .no-avatar {
        width: 36px;
        height: 36px;
        border-radius: 50%;
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        display: flex;
        align-items: center;
        justify-content: center;
        color: white;
        font-size: 16px;
        font-weight: bold;
        border: 1px solid rgba(255, 255, 255, 0.2);
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .no-info {
        color: var(--text-secondary);
        font-style: italic;
        text-align: center;
        padding: 1rem;
        font-weight: 500;
    }

    /* Mobile responsive design */
    @media (max-width: 480px) {
        .filter-section {
            padding: 1rem;
            border-radius: 20px;
        }

        .filter-title {
            font-size: 1rem;
        }

        .filter-buttons {
            gap: 0.5rem;
        }

        .filter-btn {
            padding: 0.75rem 1rem;
            font-size: 0.8rem;
            min-width: 80px;
        }

        .currency-filters {
            gap: 0.75rem;
        }

        .currency-filter {
            padding: 0.75rem;
        }

        .mint-card {
            padding: 1rem;
            border-radius: 20px;
        }

        .mint-header {
            flex-direction: column;
            gap: 0.75rem;
            align-items: stretch;
        }

        .mint-name {
            font-size: 1.2rem;
        }

        .mint-type {
            align-self: flex-start;
            padding: 0.5rem 1rem;
            font-size: 0.75rem;
        }

        .health-status {
            flex-direction: column;
            align-items: stretch;
            gap: 0.5rem;
        }

        .uptime-bar {
            min-width: auto;
        }

        .info-grid {
            gap: 0.75rem;
        }

        .info-item {
            padding: 0.75rem;
        }

        .method-item {
            flex-direction: column;
            align-items: stretch;
            gap: 0.5rem;
        }

        .method-details {
            text-align: left;
        }

        .ratings-summary {
            flex-direction: column;
            text-align: center;
            gap: 0.75rem;
        }

        .rating-score {
            font-size: 1.75rem;
        }

        .reviews-summary-container {
            flex-direction: column;
            gap: 0.75rem;
        }

        .reviewers-grid {
            gap: 0.5rem;
        }

        .reviewer-card {
            padding: 0.75rem 1rem;
            font-size: 0.8rem;
        }
    }

    @media (min-width: 481px) and (max-width: 768px) {
        .mints-grid {
            gap: 1.25rem;
        }

        .currency-filters {
            grid-template-columns: repeat(2, 1fr);
        }

        .info-grid {
            grid-template-columns: repeat(2, 1fr);
        }

        .mint-header {
            flex-wrap: wrap;
            gap: 1rem;
        }
    }

    @media (min-width: 769px) {
        .mints-grid {
            grid-template-columns: repeat(auto-fit, minmax(450px, 1fr));
            gap: 2rem;
        }

        .currency-filters {
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        }

        .info-grid {
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        }

        .mint-card {
            padding: 2rem;
        }

        .filter-section {
            padding: 2rem;
        }
    }
    "
}

/// Helper function to extract supported NUTs from a Nuts struct
pub fn get_supported_nuts(nuts: &cdk::nuts::Nuts) -> Vec<String> {
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
                    // Health status indicator
                    @if let Some(stored_info) = &mint_with_recs.stored_info {
                        div class="health-status" {
                            @if stored_info.is_currently_online {
                                div class="health-indicator health-online" {
                                    span class="health-dot" {}
                                    span { "Online" }
                                }
                            } @else {
                                div class="health-indicator health-offline" {
                                    span class="health-dot" {}
                                    span { "Offline" }
                                }
                            }

                            // Uptime percentage
                            @let uptime_percent = (stored_info.health_score * 100.0) as i32;
                            div style="flex: 1; min-width: 120px;" {
                                div style="font-size: 0.7rem; color: rgba(255, 255, 255, 0.7); margin-bottom: 0.25rem;" {
                                    "Uptime: " (uptime_percent) "%"
                                }
                                                                div class="uptime-bar" {
                                    @let uptime_class = if uptime_percent >= 95 { "uptime-excellent" } else if uptime_percent >= 80 { "uptime-good" } else { "uptime-poor" };
                                    div class=(format!("uptime-fill {}", uptime_class)) style=(format!("width: {}%", uptime_percent)) {}
                                }
                            }
                        }
                    } @else {
                        div class="health-status" {
                            div class="health-indicator health-warning" {
                                span class="health-dot" {}
                                span { "Unknown" }
                            }
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
                    div class="info-header" onclick="toggleMintInfo(this)" {
                        div class="info-header-left" {
                            span { "🏦" }
                            span { "Mint Information" }
                        }
                        span class="info-expand-indicator" { "▼" }
                    }

                    div class="info-content" {
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

                            div class="info-item" {
                                div class="info-label" { "Health Status" }
                                div class="info-value" {
                                    @if stored_info.is_currently_online {
                                        span style="color: #10B981;" { "🟢 Online" }
                                    } @else {
                                        span style="color: #EF4444;" { "🔴 Offline" }
                                    }
                                    " • "
                                    (format!("{:.1}%", stored_info.health_score * 100.0)) " uptime"
                                }
                            }

                            @if stored_info.total_attempts > 0 {
                                div class="info-item" {
                                    div class="info-label" { "Reliability" }
                                    div class="info-value" {
                                        (stored_info.total_successes) "/" (stored_info.total_attempts) " successful checks"
                                        @if stored_info.consecutive_failures > 0 {
                                            " • " (stored_info.consecutive_failures) " consecutive failures"
                                        } @else if stored_info.consecutive_successes > 0 {
                                            " • " (stored_info.consecutive_successes) " consecutive successes"
                                        }
                                    }
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
                }
            } @else if let Some(stored_info) = &mint_with_recs.stored_info {
                @if !stored_info.fetch_success {
                    div class="info-section" {
                        div class="info-header" onclick="toggleMintInfo(this)" {
                            div class="info-header-left" {
                                span { "⚠️" }
                                span { "Mint Information" }
                            }
                            span class="info-expand-indicator" { "▼" }
                        }
                        div class="info-content" {
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
                }
            } @else {
                div class="info-section" {
                    div class="info-header" onclick="toggleMintInfo(this)" {
                        div class="info-header-left" {
                            span { "📡" }
                            span { "Mint Information" }
                        }
                        span class="info-expand-indicator" { "▼" }
                    }
                    div class="info-content" {
                        div class="no-info" { "Mint info not yet fetched or available" }
                    }
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

                    // Collapsible reviews summary
                    div class="reviews-summary-container" onclick="toggleReviewers(this)" {
                        div class="reviewers-stack" {
                            @let max_visible = 5;
                            @let total_reviewers = mint_with_recs.recommendations.len();

                            @for (index, rec_with_user) in mint_with_recs.recommendations.iter().take(max_visible).enumerate() {
                                div class="reviewer-bubble" style=(format!("z-index: {}", max_visible - index)) {
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
                                    } @else {
                                        div class="no-avatar" { "?" }
                                    }
                                }
                            }

                            @if total_reviewers > max_visible {
                                div class="reviewer-bubble more-reviewers" {
                                    "+" (total_reviewers - max_visible)
                                }
                            }
                        }

                        span class="expand-indicator" { "▼" }
                    }

                    // Expanded reviewers grid (hidden by default)
                    div class="reviewers-grid" {
                        @for rec_with_user in &mint_with_recs.recommendations {
                            a href=(format!("/review/{}", rec_with_user.recommendation.event_id)) class="reviewer-card-link" {
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
                                        @if let Some(rating) = rec_with_user.recommendation.rating {
                                            (rating)
                                        } @else {
                                            span style="color: rgba(255, 255, 255, 0.5); font-style: italic;" { "No rating" }
                                        }
                                    }
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
