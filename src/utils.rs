// src/utils.rs
use web_sys::window;

/// Get the base URL for the application
/// This handles both local development and GitHub Pages deployment
pub fn get_base_url() -> String {
    if let Some(window) = window() {
        if let Some(location) = window.location().pathname().ok() {
            // Check if we're on GitHub Pages (path starts with /tei-viewer/)
            if location.starts_with("/tei-viewer/") {
                return "/tei-viewer".to_string();
            }
        }
    }
    // Local development - no base path needed
    String::new()
}

/// Build a resource URL with the correct base path
pub fn resource_url(path: &str) -> String {
    let base = get_base_url();
    let clean_path = path.trim_start_matches('/');

    if base.is_empty() {
        format!("/{}", clean_path)
    } else {
        format!("{}/{}", base, clean_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_url_formatting() {
        // Note: These tests won't actually detect the window location
        // They're mainly for documentation of expected behavior

        // With leading slash
        let url1 = resource_url("/public/projects/test.xml");
        assert!(url1.contains("public/projects/test.xml"));

        // Without leading slash
        let url2 = resource_url("public/projects/test.xml");
        assert!(url2.contains("public/projects/test.xml"));
    }
}
