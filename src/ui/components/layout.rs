use crate::ui::styles::get_shared_styles;
use maud::{html, Markup, DOCTYPE};

/// Render the page header with logo and tagline
pub fn render_header(tagline: &str) -> Markup {
    html! {
        div class="header" {
            div class="header-glass" {
                div class="logo" {
                    img src="/assets/d.png" alt="Bitcoin Mints" class="logo-image";
                }
                div class="tagline" { (tagline) }
            }
        }
    }
}

/// Render the navigation bar
pub fn render_navigation(active_page: &str) -> Markup {
    html! {
        div class="nav-container" {
            a class=(format!("nav-link{}", if active_page == "mints" { " active" } else { "" }))
              href="/mints" { "🏦 Mints" }
            a class=(format!("nav-link{}", if active_page == "reviews" { " active" } else { "" }))
              href="/reviews" { "📝 Reviews" }
        }
    }
}

/// Render a complete page layout with header, navigation, and content
pub fn render_layout(title: &str, tagline: &str, active_page: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover";
                meta name="apple-mobile-web-app-capable" content="yes";
                meta name="apple-mobile-web-app-status-bar-style" content="black-translucent";
                meta name="format-detection" content="telephone=no";
                title { (title) }
                link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700;800&display=swap" rel="stylesheet";
                style { (get_shared_styles()) }
            }
            body {
                div class="dashboard-container" {
                    (render_header(tagline))
                    (render_navigation(active_page))
                    (content)
                }
            }
        }
    }
}
