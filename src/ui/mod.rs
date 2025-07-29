pub mod components;
pub mod pages;
pub mod styles;

// Re-export main page functions for easy access
pub use pages::mints::render_unified_mints_page;
pub use pages::reviews::render_reviews_page;
