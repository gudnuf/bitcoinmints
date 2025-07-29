use crate::models::{MintType, MintWithRecommendationsAndInfo, UnifiedMintWithRecommendations};
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

/// Fedimint module capability information
#[derive(Debug, Clone)]
pub struct ModuleCapability {
    pub module_id: String,
    pub module_name: String,
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

/// Extract all fedimint module capabilities from a list of mints
pub fn extract_module_capabilities(
    mints: &[MintWithRecommendationsAndInfo],
) -> Vec<ModuleCapability> {
    let mut modules: HashMap<String, bool> = HashMap::new();

    // Define module descriptions
    let module_descriptions: HashMap<&str, &str> = [
        ("0", "Lightning Network Gateway"),
        ("1", "Mint (Bitcoin backing)"),
        ("2", "Wallet (On-chain Bitcoin)"),
        ("3", "Unknown Module"),
    ]
    .iter()
    .cloned()
    .collect();

    for mint in mints {
        // Only process Fedimint mints
        if mint.mint.mint_type.to_lowercase() != "fedimint" {
            continue;
        }

        for module in &mint.mint.modules {
            modules.insert(module.clone(), true);
        }
    }

    let mut result: Vec<ModuleCapability> = modules
        .into_iter()
        .map(|(module_id, _)| ModuleCapability {
            module_name: module_descriptions
                .get(module_id.as_str())
                .unwrap_or(&"Custom Module")
                .to_string(),
            module_id,
            is_supported: true,
        })
        .collect();

    result.sort_by(|a, b| a.module_id.cmp(&b.module_id));
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

/// Render Fedimint-specific module filters
pub fn render_fedimint_filters(module_capabilities: &[ModuleCapability]) -> Markup {
    html! {
        div class="fedimint-filters" id="fedimint-filters" {
            div class="fedimint-filter-header" {
                span class="filter-icon" { "🔵" }
                span class="filter-title" { "Fedimint Filters" }
            }

            @if module_capabilities.is_empty() {
                div class="no-filters-message" {
                    "No Fedimint instances with module information available"
                }
            } @else {
                div class="filter-sections" {
                    // Module Filters Section
                    div class="filter-section" {
                        div class="section-title" { "Module Support" }
                        div class="modules-grid" {
                            @for module in module_capabilities {
                                label class="module-option" {
                                    input
                                        type="checkbox"
                                        class="filter-checkbox module-filter"
                                        data-module=(module.module_id)
                                        onchange="updateFedimintFilters()";
                                    span class="module-label" { "Module " (module.module_id) }
                                    span class="module-description" { (module.module_name) }
                                }
                            }
                        }
                    }
                }

                div class="filter-actions" {
                    button class="filter-action-btn clear" onclick="clearFedimintFilters()" {
                        "Clear All"
                    }
                    button class="filter-action-btn select" onclick="selectAllFedimintFilters()" {
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
        border-radius: 24px;
        padding: 1.5rem;
        margin-bottom: 1.5rem;
        box-shadow: var(--glass-shadow);

        background: rgba(0, 0, 0, 0.5);
        border: 2px solid rgba(255, 255, 255, 0.15);
        backdrop-filter: blur(30px);
        -webkit-backdrop-filter: blur(30px);
        /* iOS color fallback */
        background-color: rgba(15, 23, 42, 0.9);
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

    .fedimint-filters {
        background: rgba(0, 0, 0, 0.6);
        backdrop-filter: blur(15px);
        -webkit-backdrop-filter: blur(15px);
        border: 2px solid rgba(59, 130, 246, 0.3);
        border-radius: 16px;
        padding: 1rem;
        margin-top: 1rem;
        display: none;
    }

    .fedimint-filters.active {
        display: block;
        animation: slideDown 0.3s ease;
    }

    .fedimint-filter-header {
        color: var(--text-primary);
        font-size: 0.95rem;
        font-weight: 700;
        margin-bottom: 0.75rem;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .modules-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
        gap: 0.4rem;
    }

    .module-option {
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

    .module-option:hover {
        background: rgba(255, 255, 255, 0.1);
        border-color: rgba(59, 130, 246, 0.3);
    }

    .module-label {
        color: var(--primary-blue);
        font-weight: 600;
        font-size: 0.75rem;
        min-width: 70px;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .module-description {
        color: rgba(255, 255, 255, 0.8);
        font-size: 0.7rem;
        flex: 1;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
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
        /* iOS optimizations */
        -webkit-appearance: none;
        appearance: none;
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        -webkit-tap-highlight-color: transparent;
    }

    .filter-action-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        color: white;
        transform: translateY(-1px) translateZ(0);
    }

    .filter-action-btn.clear {
        background: rgba(239, 68, 68, 0.2);
        border-color: rgba(239, 68, 68, 0.3);
        color: #FCA5A5;
    }

    .filter-action-btn.clear:hover {
        background: rgba(239, 68, 68, 0.4);
        color: white;
        transform: translateY(-2px) translateZ(0);
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
        text-decoration: none;
        display: inline-block;
        transition: all 0.3s ease;
        cursor: pointer;
    }

    a.nut-badge:hover {
        background: linear-gradient(135deg, #059669, var(--accent-green));
        transform: translateY(-2px);
        box-shadow: 0 6px 20px rgba(16, 185, 129, 0.6);
    }

    a.nut-badge:active {
        transform: translateY(0);
        box-shadow: 0 4px 12px rgba(16, 185, 129, 0.4);
    }

    .module-badge {
        background: linear-gradient(135deg, var(--primary-blue), var(--light-blue));
        color: white;
        padding: 0.375rem 0.875rem;
        border-radius: 16px;
        font-size: 0.75rem;
        font-weight: 700;
        box-shadow: 0 4px 12px rgba(30, 64, 175, 0.4);
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

    .reviewer-rating {
        color: var(--secondary-orange);
        font-weight: 700;
        font-size: 0.85rem;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .reviewer-rating.no-rating {
        color: rgba(255, 255, 255, 0.5);
        font-style: italic;
        font-weight: 500;
    }

    .no-reviews {
        color: var(--text-secondary);
        font-style: italic;
        text-align: center;
        padding: 1rem;
        font-weight: 500;
    }

    .recommendations-section {
        margin-bottom: 1.5rem;
    }

    .recommendations-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        cursor: pointer;
        padding: 1rem;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 12px;
        backdrop-filter: blur(6px);
        -webkit-backdrop-filter: blur(6px);
        border: 1px solid rgba(255, 255, 255, 0.1);
        transition: all 0.3s ease;
        min-height: 44px;
    }

    .recommendations-header:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .recommendations-summary {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex: 1;
    }

    .recommendations-info {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        color: var(--text-primary);
        font-weight: 600;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .recommendations-icon {
        font-size: 1.1rem;
    }

    .recommendations-count {
        font-size: 0.9rem;
    }

    .average-rating {
        color: var(--secondary-orange);
        font-size: 0.85rem;
        font-weight: 700;
    }

    .reviewers-stack {
        display: flex;
        align-items: center;
        position: relative;
        margin-left: 0.5rem;
    }

    .reviewer-bubble {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        border: 2px solid rgba(255, 255, 255, 0.3);
        margin-left: -8px;
        transition: all 0.3s ease;
        position: relative;
        z-index: 1;
        overflow: hidden;
        background: rgba(255, 255, 255, 0.1);
        flex-shrink: 0;
    }

    .reviewer-bubble:first-child {
        margin-left: 0;
    }

    .reviewer-bubble:hover {
        transform: translateY(-4px) scale(1.1);
        z-index: 10;
        border-color: var(--primary-orange);
        box-shadow: 0 6px 16px rgba(255, 107, 53, 0.4);
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
        font-size: 10px;
        font-weight: 700;
        display: flex;
        align-items: center;
        justify-content: center;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
    }

    .expand-indicator {
        color: var(--primary-orange);
        font-size: 1.2rem;
        transition: transform 0.3s ease;
        filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.3));
        flex-shrink: 0;
    }

    .expand-indicator.expanded {
        transform: rotate(180deg);
    }

    .recommendations-content {
        display: none;
        margin-top: 1rem;
        animation: slideDown 0.3s ease;
    }

    .recommendations-content.expanded {
        display: block;
    }

    .reviewers-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
        gap: 0.75rem;
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

        .recommendations-header {
            padding: 0.75rem;
            flex-direction: column;
            align-items: stretch;
            gap: 0.75rem;
        }

        .recommendations-summary {
            flex-direction: column;
            align-items: stretch;
            gap: 0.5rem;
        }

        .recommendations-info {
            justify-content: center;
        }

        .reviewers-stack {
            justify-content: center;
            margin-left: 0;
        }

        .reviewer-bubble {
            width: 28px;
            height: 28px;
            margin-left: -6px;
        }

        .expand-indicator {
            align-self: center;
            font-size: 1rem;
        }

        .reviewers-grid {
            grid-template-columns: 1fr;
            gap: 0.5rem;
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

        .reviewers-grid {
            grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
        }

        .recommendations-header {
            padding: 0.875rem;
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

    /* Rating Filter Styles */
    .rating-filter-section {
        background: var(--glass-bg);
        backdrop-filter: blur(20px);
        -webkit-backdrop-filter: blur(20px);
        border: 2px solid var(--glass-border);
        border-radius: 20px;
        padding: 1.5rem;
        margin: 1rem 0;
        box-shadow: var(--glass-shadow);
    }

    .rating-filter-title {
        color: var(--text-primary);
        font-size: 1.1rem;
        font-weight: 700;
        margin-bottom: 1rem;
        text-align: center;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
    }

    .rating-slider-container {
        display: flex;
        flex-direction: column;
        gap: 1rem;
        align-items: center;
    }

    .rating-slider-wrapper {
        position: relative;
        width: 100%;
        max-width: 400px;
    }

    .rating-slider {
        -webkit-appearance: none;
        appearance: none;
        width: 100%;
        height: 8px;
        background: rgba(255, 255, 255, 0.2);
        border-radius: 10px;
        outline: none;
        transition: all 0.3s ease;
        cursor: pointer;
    }

    .rating-slider.slider-inactive {
        background: rgba(255, 255, 255, 0.1);
        opacity: 0.6;
    }

    .rating-slider.slider-active {
        background: rgba(255, 255, 255, 0.2);
        opacity: 1;
    }

    .rating-slider::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 24px;
        height: 24px;
        border-radius: 50%;
        cursor: pointer;
        transition: all 0.3s ease;
        border: 2px solid rgba(255, 255, 255, 0.3);
        /* iOS optimizations */
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
    }

    .rating-slider.slider-inactive::-webkit-slider-thumb {
        background: rgba(255, 255, 255, 0.4);
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .rating-slider.slider-active::-webkit-slider-thumb {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        box-shadow: 0 4px 12px rgba(255, 107, 53, 0.4);
    }

    .rating-slider::-webkit-slider-thumb:hover {
        transform: scale(1.1) translateZ(0);
        box-shadow: 0 6px 20px rgba(255, 107, 53, 0.6);
    }

    .rating-slider::-moz-range-thumb {
        width: 24px;
        height: 24px;
        border-radius: 50%;
        cursor: pointer;
        transition: all 0.3s ease;
        border: 2px solid rgba(255, 255, 255, 0.3);
    }

    .rating-slider.slider-inactive::-moz-range-thumb {
        background: rgba(255, 255, 255, 0.4);
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
    }

    .rating-slider.slider-active::-moz-range-thumb {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        box-shadow: 0 4px 12px rgba(255, 107, 53, 0.4);
    }

    .rating-slider::-moz-range-thumb:hover {
        transform: scale(1.1);
        box-shadow: 0 6px 20px rgba(255, 107, 53, 0.6);
    }

    .rating-slider:focus {
        box-shadow: 0 0 0 3px rgba(255, 107, 53, 0.3);
    }

    .rating-display {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        font-size: 1.1rem;
        color: var(--text-primary);
        font-weight: 600;
        text-shadow: 0 2px 4px rgba(0, 0, 0, 0.5);
        transition: all 0.3s ease;
    }

    .rating-display.rating-inactive {
        opacity: 0.7;
    }

    .rating-display.rating-active {
        opacity: 1;
    }

    .rating-value {
        font-weight: 800;
        font-size: 1.3rem;
        min-width: 4rem;
        text-align: center;
        transition: all 0.3s ease;
    }

    .rating-inactive .rating-value {
        color: var(--text-secondary);
        background: none;
        -webkit-text-fill-color: var(--text-secondary);
    }

    .rating-active .rating-value {
        background: linear-gradient(135deg, var(--primary-orange), var(--secondary-orange));
        -webkit-background-clip: text;
        -webkit-text-fill-color: transparent;
        background-clip: text;
    }

    .rating-label {
        color: var(--text-secondary);
        font-size: 0.95rem;
        font-style: italic;
        transition: all 0.3s ease;
    }

    .rating-inactive .rating-label {
        opacity: 0.8;
    }

    .rating-active .rating-label {
        opacity: 1;
        font-style: normal;
    }

    .rating-controls {
        display: flex;
        justify-content: center;
    }

    .rating-reset-btn {
        background: rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
        -webkit-backdrop-filter: blur(10px);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: var(--text-primary);
        padding: 0.5rem 1rem;
        border-radius: 20px;
        font-size: 0.85rem;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.3s ease;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.5);
        /* iOS optimizations */
        -webkit-appearance: none;
        appearance: none;
        -webkit-transform: translateZ(0);
        transform: translateZ(0);
        -webkit-tap-highlight-color: transparent;
        min-height: 44px;
    }

    .rating-reset-btn:hover {
        background: rgba(255, 255, 255, 0.2);
        border-color: var(--primary-orange);
        transform: translateY(-2px) translateZ(0);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    }

    .rating-reset-btn:active {
        transform: translateY(0) translateZ(0);
    }

    /* Mobile responsive design for rating filter */
    @media (max-width: 480px) {
        .rating-filter-section {
            padding: 1rem;
            border-radius: 16px;
            margin: 0.75rem 0;
        }

        .rating-filter-title {
            font-size: 1rem;
            margin-bottom: 0.75rem;
        }

        .rating-slider-container {
            gap: 0.75rem;
        }

        .rating-slider-wrapper {
            max-width: 300px;
        }

        .rating-slider {
            height: 6px;
        }

        .rating-slider::-webkit-slider-thumb {
            width: 20px;
            height: 20px;
        }

        .rating-slider::-moz-range-thumb {
            width: 20px;
            height: 20px;
        }

        .rating-display {
            font-size: 1rem;
        }

        .rating-value {
            font-size: 1.15rem;
        }

        .rating-label {
            font-size: 0.85rem;
        }

        .rating-reset-btn {
            padding: 0.4rem 0.8rem;
            font-size: 0.8rem;
            border-radius: 16px;
        }
    }

    @media (min-width: 481px) and (max-width: 768px) {
        .rating-filter-section {
            padding: 1.25rem;
        }

        .rating-slider-wrapper {
            max-width: 350px;
        }
    }

    @media (min-width: 769px) {
        .rating-filter-section {
            padding: 2rem;
        }

        .rating-slider-wrapper {
            max-width: 500px;
        }

        .rating-display {
            font-size: 1.2rem;
        }

        .rating-value {
            font-size: 1.4rem;
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

    // NUT 21 and 22 are not commonly implemented yet, commenting out
    // #[cfg(feature = "auth")]
    // {
    //     if nuts.nut21.is_some() {
    //         supported.push("21".to_string());
    //     }
    //     if nuts.nut22.is_some() {
    //         supported.push("22".to_string());
    //     }
    // }

    supported
}

/// Render a unified mint card with proper type separation
pub fn render_unified_mint_card(unified_mint: &UnifiedMintWithRecommendations) -> Markup {
    let mint_data = &unified_mint.mint_data;

    html! {
        div class="mint-card" {
            div class="mint-header" {
                div class="mint-title-section" {
                    div class="mint-name" { (mint_data.name) }

                    // Display appropriate URL/identifier based on mint type
                    div class="mint-url" {
                        @match mint_data.mint_type {
                            MintType::Cashu => {
                                @if let Some(cashu_data) = &mint_data.cashu_data {
                                    a href=(cashu_data.mint_url) target="_blank" {
                                        (cashu_data.mint_url)
                                    }
                                } @else {
                                    span { (mint_data.mint_id) }
                                }
                            }
                            MintType::Fedimint => {
                                @if let Some(fedimint_data) = &mint_data.fedimint_data {
                                    @if !fedimint_data.invite_codes.is_empty() {
                                        div class="federation-invites" {
                                            span class="invite-label" { "Invite codes: " }
                                            @for (i, invite_code) in fedimint_data.invite_codes.iter().take(2).enumerate() {
                                                @if i > 0 { ", " }
                                                span class="invite-code" {
                                                    (format!("{}...{}",
                                                        &invite_code[0..12.min(invite_code.len())],
                                                        &invite_code[invite_code.len().saturating_sub(8)..]
                                                    ))
                                                }
                                            }
                                            @if fedimint_data.invite_codes.len() > 2 {
                                                span { " +" (fedimint_data.invite_codes.len() - 2) " more" }
                                            }
                                        }
                                    } @else {
                                        span { "Federation ID: " (mint_data.mint_id) }
                                    }
                                } @else {
                                    span { "Federation ID: " (mint_data.mint_id) }
                                }
                            }
                        }
                    }

                    // Health status indicator
                    div class="health-status" {
                        @if mint_data.is_online {
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

                        // Uptime percentage for Cashu mints
                        @if let Some(health_info) = &mint_data.health_info {
                            @let uptime_percent = (health_info.uptime_percentage) as i32;
                            div style="flex: 1; min-width: 120px;" {
                                div style="font-size: 0.7rem; color: rgba(255, 255, 255, 0.7); margin-bottom: 0.25rem;" {
                                    "Uptime: " (uptime_percent) "%"
                                }
                                div class="uptime-bar" {
                                    @let uptime_class = if uptime_percent >= 95 { "uptime-excellent" } else if uptime_percent >= 80 { "uptime-good" } else { "uptime-poor" };
                                    div class=(format!("uptime-fill {}", uptime_class)) style=(format!("width: {}%", uptime_percent)) {}
                                }
                            }
                        } @else if mint_data.mint_type == MintType::Fedimint {
                            @if let Some(fedimint_data) = &mint_data.fedimint_data {
                                div style="flex: 1; min-width: 120px;" {
                                    div style="font-size: 0.7rem; color: rgba(255, 255, 255, 0.7);" {
                                        @if fedimint_data.config_available {
                                            "✅ Config Available"
                                        } @else {
                                            "❌ Config Unavailable"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div class="mint-type" { (mint_data.mint_type) }
            }

            @if let Some(description) = &mint_data.description {
                div class="mint-description" { (description) }
            }

            // Display mint info section based on type
            @match mint_data.mint_type {
                MintType::Cashu => {
                    @if let Some(cashu_data) = &mint_data.cashu_data {
                        (render_cashu_info_section(cashu_data, &mint_data.health_info))
                    }
                }
                MintType::Fedimint => {
                    @if let Some(fedimint_data) = &mint_data.fedimint_data {
                        (render_fedimint_info_section(fedimint_data))
                    }
                }
            }

            // Recommendations section (same for both types)
            @if unified_mint.total_recommendations > 0 {
                div class="recommendations-section" {
                    div class="recommendations-header" onclick="toggleReviewers(this)" {
                        div class="recommendations-summary" {
                            div class="recommendations-info" {
                                span class="recommendations-icon" { "💬" }
                                span class="recommendations-count" { (unified_mint.total_recommendations) " Reviews" }
                                @if let Some(avg_rating) = unified_mint.average_rating {
                                    span class="average-rating" { "(" (format!("{:.1}", avg_rating)) "⭐)" }
                                }
                            }

                            div class="reviewers-stack" {
                                @let max_visible = 4;
                                @let total_reviewers = unified_mint.recommendations.len();

                                @for (index, rec_with_user) in unified_mint.recommendations.iter().take(max_visible).enumerate() {
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
                        }

                        span class="expand-indicator" { "▼" }
                    }

                    // Expanded reviewers grid (hidden by default)
                    div class="recommendations-content" {
                        div class="reviewers-grid" {
                            @for rec_with_user in &unified_mint.recommendations {
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

                                        @if let Some(rating) = rec_with_user.recommendation.rating {
                                            div class="reviewer-rating" { (rating) "⭐" }
                                        } @else {
                                            div class="reviewer-rating no-rating" { "No rating" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render Cashu-specific info section
fn render_cashu_info_section(
    cashu_data: &crate::models::CashuMintData,
    health_info: &Option<crate::models::MintHealthInfo>,
) -> Markup {
    html! {
        div class="info-section" {
            div class="info-header" onclick="toggleMintInfo(this)" {
                div class="info-header-left" {
                    span { "🏦" }
                    span { "Cashu Mint Information" }
                }
                span class="info-expand-indicator" { "▼" }
            }

            div class="info-content" {
                @if let Some(mint_info) = &cashu_data.mint_info {
                    div class="info-grid" {
                        @if let Some(version) = &mint_info.version {
                            div class="info-item" {
                                div class="info-label" { "Version" }
                                div class="info-value" { (version) }
                            }
                        }

                        @if let Some(pubkey) = &mint_info.pubkey {
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

                        @if let Some(motd) = &mint_info.motd {
                            div class="info-item" {
                                div class="info-label" { "Message of the Day" }
                                div class="info-value" { (motd) }
                            }
                        }

                        @if let Some(health) = health_info {
                            div class="info-item" {
                                div class="info-label" { "Health Status" }
                                div class="info-value" {
                                    (format!("{:.1}%", health.uptime_percentage)) " uptime • "
                                    (health.total_successes) "/" (health.total_attempts) " successful checks"
                                }
                            }
                        }

                        // Display supported NUTs
                        @let supported_nuts = get_supported_nuts(&cashu_data.nuts);
                        @if !supported_nuts.is_empty() {
                            div class="info-item" style="grid-column: 1 / -1;" {
                                div class="info-label" { "Supported NUTs (Protocols)" }
                                div class="nuts-container" {
                                    @for nut in supported_nuts {
                                        @let padded_nut = if nut.len() == 1 { format!("0{}", nut) } else { nut.clone() };
                                        @let spec_url = format!("https://github.com/cashubtc/nuts/blob/main/{}.md", padded_nut);
                                        a href=(spec_url) target="_blank" class="nut-badge" { "NUT-" (nut) }
                                    }
                                }
                            }
                        }

                        // Display supported currencies
                        @if !cashu_data.supported_currencies.is_empty() {
                            div class="info-item" style="grid-column: 1 / -1;" {
                                div class="info-label" { "Supported Currencies" }
                                div class="nuts-container" {
                                    @for currency in &cashu_data.supported_currencies {
                                        span class="currency-badge" { (currency.to_uppercase()) }
                                    }
                                }
                            }
                        }
                    }
                } @else {
                    div class="no-info" { "Mint info not available" }
                }
            }
        }
    }
}

/// Render Fedimint-specific info section  
fn render_fedimint_info_section(fedimint_data: &crate::models::FedimintMintData) -> Markup {
    html! {
        div class="info-section" {
            div class="info-header" onclick="toggleMintInfo(this)" {
                div class="info-header-left" {
                    span { "🔵" }
                    span { "Fedimint Federation Information" }
                }
                span class="info-expand-indicator" { "▼" }
            }

            div class="info-content" {
                div class="info-grid" {
                    div class="info-item" {
                        div class="info-label" { "Federation ID" }
                        div class="info-value" {
                            (format!("{}...{}",
                                &fedimint_data.federation_id[0..16.min(fedimint_data.federation_id.len())],
                                &fedimint_data.federation_id[fedimint_data.federation_id.len().saturating_sub(16)..]
                            ))
                        }
                    }

                    @if let Some(federation_name) = &fedimint_data.federation_name {
                        div class="info-item" {
                            div class="info-label" { "Federation Name" }
                            div class="info-value" { (federation_name) }
                        }
                    }

                    @if let Some(guardians_count) = fedimint_data.guardians_count {
                        div class="info-item" {
                            div class="info-label" { "Guardians" }
                            div class="info-value" { (guardians_count) " guardians" }
                        }
                    }

                    div class="info-item" {
                        div class="info-label" { "Configuration" }
                        div class="info-value" {
                            @if fedimint_data.config_available {
                                span style="color: #10B981;" { "✅ Available" }
                            } @else {
                                span style="color: #EF4444;" { "❌ Unavailable" }
                            }
                        }
                    }

                    @if !fedimint_data.modules.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Supported Modules" }
                            div class="nuts-container" {
                                @for module in &fedimint_data.modules {
                                    span class="module-badge" { "Module " (module) }
                                }
                            }
                        }
                    }

                    @if !fedimint_data.invite_codes.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Federation Invite Codes" }
                            div class="nuts-container" {
                                @for invite_code in &fedimint_data.invite_codes {
                                    span class="module-badge" style="font-family: monospace; font-size: 0.8rem;" {
                                        (format!("{}...{}",
                                            &invite_code[0..12.min(invite_code.len())],
                                            &invite_code[invite_code.len().saturating_sub(12)..]
                                        ))
                                    }
                                }
                            }
                        }
                    }

                    @if let Some(welcome_message) = &fedimint_data.welcome_message {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Welcome Message" }
                            div class="info-value" { (welcome_message) }
                        }
                    }
                }
            }
        }
    }
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

                    // Display supported NUTs for Cashu
                    @if mint_with_recs.mint.mint_type == "cashu" {
                        @let supported_nuts = get_supported_nuts(&mint_with_recs.mint.nuts);
                        @if !supported_nuts.is_empty() {
                            div class="info-item" style="grid-column: 1 / -1;" {
                                div class="info-label" { "Supported NUTs (Protocols)" }
                                div class="nuts-container" {
                                    @for nut in supported_nuts {
                                        @let padded_nut = if nut.len() == 1 { format!("0{}", nut) } else { nut.clone() };
                                        @let spec_url = format!("https://github.com/cashubtc/nuts/blob/main/{}.md", padded_nut);
                                        a href=(spec_url) target="_blank" class="nut-badge" { "NUT-" (nut) }
                                    }
                                }
                            }
                        }
                    }

                    // Display supported modules for Fedimint
                    @if mint_with_recs.mint.mint_type == "fedimint" && !mint_with_recs.mint.modules.is_empty() {
                        div class="info-item" style="grid-column: 1 / -1;" {
                            div class="info-label" { "Supported Modules" }
                            div class="nuts-container" {
                                @for module in &mint_with_recs.mint.modules {
                                    span class="module-badge" { "Module " (module) }
                                }
                            }
                        }
                    }

                    // Display federation-specific info for Fedimint
                    @if mint_with_recs.mint.mint_type == "fedimint" {
                        @if let Some(federation_id) = &mint_with_recs.mint.federation_id {
                            div class="info-item" {
                                div class="info-label" { "Federation ID" }
                                div class="info-value" {
                                    (format!("{}...{}",
                                        &federation_id[0..16.min(federation_id.len())],
                                        &federation_id[federation_id.len().saturating_sub(16)..]
                                    ))
                                }
                            }
                        }

                        @if !mint_with_recs.mint.invite_codes.is_empty() {
                            div class="info-item" style="grid-column: 1 / -1;" {
                                div class="info-label" { "Federation Invite Codes" }
                                div class="nuts-container" {
                                    @for invite_code in &mint_with_recs.mint.invite_codes {
                                        a href=(format!("https://fmo.sirion.io/config/{}", invite_code))
                                          target="_blank"
                                          class="module-badge"
                                          style="text-decoration: none; color: white; cursor: pointer; transition: all 0.3s ease;"
                                          onmouseover="setHoverStyle(this, 'rgba(59, 130, 246, 0.8)')"
                                          onmouseout="clearHoverStyle(this)" {
                                            (format!("{}...{}",
                                                &invite_code[0..12.min(invite_code.len())],
                                                &invite_code[invite_code.len().saturating_sub(12)..]
                                            ))
                                        }
                                    }
                                }
                            }
                        }

                        @if let Some(guardians_count) = mint_with_recs.mint.guardians_count {
                            div class="info-item" {
                                div class="info-label" { "Guardians" }
                                div class="info-value" { (guardians_count) " guardians" }
                            }
                        }

                        @if let Some(welcome_message) = mint_with_recs.mint.meta.get("welcome_message").and_then(|v| v.as_str()) {
                            div class="info-item" style="grid-column: 1 / -1;" {
                                div class="info-label" { "Welcome Message" }
                                div class="info-value" { (welcome_message) }
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
                div class="recommendations-section" {
                    div class="recommendations-header" onclick="toggleReviewers(this)" {
                        div class="recommendations-summary" {
                            div class="recommendations-info" {
                                span class="recommendations-icon" { "💬" }
                                span class="recommendations-count" { (mint_with_recs.total_recommendations) " Reviews" }
                                @if let Some(avg_rating) = mint_with_recs.average_rating {
                                    span class="average-rating" { "(" (format!("{:.1}", avg_rating)) "⭐)" }
                                }
                            }

                            div class="reviewers-stack" {
                                @let max_visible = 4;
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
                        }

                        span class="expand-indicator" { "▼" }
                    }

                    // Expanded reviewers grid (hidden by default)
                    div class="recommendations-content" {
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

                                        @if let Some(rating) = rec_with_user.recommendation.rating {
                                            div class="reviewer-rating" { (rating) "⭐" }
                                        } @else {
                                            div class="reviewer-rating no-rating" { "No rating" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } @else {
                div class="recommendations-section" {
                    div class="no-reviews" { "No reviews yet" }
                }
            }
        }
    }
}
