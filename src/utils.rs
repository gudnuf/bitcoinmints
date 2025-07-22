use anyhow::Result;
use url::Url;

/// Normalize a mint URL to ensure consistency and prevent duplicates
///
/// Normalization rules:
/// 1. Convert to lowercase
/// 2. Remove trailing slashes
/// 3. Ensure HTTPS if no protocol specified
/// 4. Remove default ports (80 for HTTP, 443 for HTTPS)
/// 5. Remove www. prefix
/// 6. Sort query parameters (if any)
pub fn normalize_mint_url(url_str: &str) -> Result<String> {
    let url_str = url_str.trim();

    if url_str.is_empty() {
        return Err(anyhow::anyhow!("Empty URL"));
    }

    // Add protocol if missing (case-insensitive check)
    let url_lower = url_str.to_lowercase();
    let url_with_protocol = if url_lower.starts_with("http://") || url_lower.starts_with("https://")
    {
        url_str.to_string()
    } else {
        format!("https://{}", url_str)
    };

    let mut url = Url::parse(&url_with_protocol)
        .map_err(|e| anyhow::anyhow!("Invalid URL '{}': {}", url_str, e))?;

    // Normalize host to lowercase and remove www. prefix
    if let Some(host) = url.host_str() {
        let normalized_host = host.to_lowercase();
        let final_host = if normalized_host.starts_with("www.") && normalized_host.len() > 4 {
            &normalized_host[4..]
        } else {
            &normalized_host
        };

        url.set_host(Some(final_host))
            .map_err(|e| anyhow::anyhow!("Failed to set normalized host: {}", e))?;
    }

    // Remove default ports
    if let Some(port) = url.port() {
        let scheme = url.scheme();
        if (scheme == "http" && port == 80) || (scheme == "https" && port == 443) {
            let _ = url.set_port(None);
        }
    }

    // Handle path normalization - remove trailing slash unless it's root
    let path = url.path().to_string();
    if path.ends_with('/') && path != "/" {
        url.set_path(&path[..path.len() - 1]);
    }

    // Sort query parameters for consistency
    let query_pairs: Vec<(String, String)> = url
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();

    if !query_pairs.is_empty() {
        let mut sorted_pairs = query_pairs;
        sorted_pairs.sort_by(|a, b| a.0.cmp(&b.0));
        let query_string = sorted_pairs
            .into_iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        url.set_query(Some(&query_string));
    }

    let mut result = url.to_string();

    // Handle slash before query parameters: "https://example.com/?a=1" -> "https://example.com?a=1"
    if let Some(query_pos) = result.find('?') {
        if query_pos > 0 && result.chars().nth(query_pos - 1) == Some('/') {
            // Check if this is just the root path slash before query params
            let before_query = &result[..query_pos];
            if before_query.ends_with('/') && before_query.matches('/').count() == 3 {
                result.remove(query_pos - 1);
            }
        }
    }

    // Final cleanup: ensure no trailing slash for domains without paths and no query params
    if result.ends_with('/') && result.matches('/').count() == 3 && !result.contains('?') {
        // URL like "https://example.com/" should become "https://example.com"
        result = result.trim_end_matches('/').to_string();
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_basic_url() {
        assert_eq!(
            normalize_mint_url("example.com").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_normalize_with_trailing_slash() {
        assert_eq!(
            normalize_mint_url("https://example.com/").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_normalize_with_www() {
        assert_eq!(
            normalize_mint_url("https://www.example.com").unwrap(),
            "https://example.com"
        );
    }

    #[test]
    fn test_normalize_uppercase() {
        assert_eq!(
            normalize_mint_url("HTTPS://Example.COM/Path").unwrap(),
            "https://example.com/Path"
        );
    }

    #[test]
    fn test_normalize_default_ports() {
        assert_eq!(
            normalize_mint_url("https://example.com:443/").unwrap(),
            "https://example.com"
        );
        assert_eq!(
            normalize_mint_url("http://example.com:80/").unwrap(),
            "http://example.com"
        );
    }

    #[test]
    fn test_normalize_query_params() {
        assert_eq!(
            normalize_mint_url("https://example.com?b=2&a=1").unwrap(),
            "https://example.com?a=1&b=2"
        );
    }
}
