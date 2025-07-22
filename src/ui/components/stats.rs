use maud::{html, Markup};

/// Render a single stat card
pub fn render_stat_card(number: String, label: &str) -> Markup {
    html! {
        div class="stat-card" {
            div class="stat-number" { (number) }
            div class="stat-label" { (label) }
        }
    }
}

/// Render a row of statistics cards
pub fn render_stats_row(stats: Vec<(String, &str)>) -> Markup {
    html! {
        div class="stats-row" {
            @for (number, label) in stats {
                (render_stat_card(number, label))
            }
        }
    }
}

/// Render an empty state component
pub fn render_empty_state(icon: &str, title: &str, description: &str) -> Markup {
    html! {
        div class="empty-state" {
            div class="empty-icon" { (icon) }
            h2 { (title) }
            p { (description) }
        }
    }
}
