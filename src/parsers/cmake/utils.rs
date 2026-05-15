// Shared utility functions for CMake parsers

use regex::Regex;

/// Extract value of a CMake argument from raw content
///
/// Handles patterns like:
/// - GIT_TAG v1.2.3
/// - URL https://example.com
/// - GIT_REPOSITORY https://github.com/owner/repo.git
/// - URL "https://example.com/path with spaces.tar.gz" (quoted strings with spaces)
pub fn extract_cmake_arg(content: &str, arg_name: &str) -> Option<String> {
    // Match: ARG_NAME <whitespace> value
    // Value can be:
    // - Quoted string: "value with spaces"
    // - Unquoted string: value_without_spaces
    let pattern = format!(r#"{}\s+(?:"([^"]*)"|([^\s)]+))"#, regex::escape(arg_name));
    let regex = Regex::new(&pattern).ok()?;

    regex.captures(content).and_then(|cap| {
        // Try quoted capture first (group 1), then unquoted (group 2)
        cap.get(1)
            .or_else(|| cap.get(2))
            .map(|m| m.as_str().to_string())
    })
}

/// Extract semantic version from Git tag or URL
///
/// Handles patterns like:
/// - v5 → 5 (single number with v prefix)
/// - release-1 → 1 (single number after dash/slash)
/// - v1.2.3 → 1.2.3
/// - release-2.0.1 → 2.0.1
/// - https://github.com/owner/repo/archive/refs/tags/v3.1.0.tar.gz → 3.1.0
pub fn extract_version_from_url(url: &str) -> String {
    // Match versions with optional 'v' prefix and one or more dot-separated numeric components.
    // Examples matched: v1, v1.2, v1.2.3, 20230125.3, release-1
    // We use a heuristic: prefer matches with dots (multi-component versions) over
    // single numbers to avoid matching digits in file extensions (e.g., .tar.bz2)
    let regex = Regex::new(r"[vV]?(\d+(?:\.\d+)*)").unwrap();

    // Find all version-like matches, prefer ones with dots
    let all_matches: Vec<String> = regex
        .captures_iter(url)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .collect();

    // Prefer the rightmost match that contains a dot (multi-component version)
    // If no dotted version found, fall back to rightmost match overall
    all_matches
        .iter()
        .rev()
        .find(|v| v.contains('.'))
        .or_else(|| all_matches.last())
        .cloned()
        .unwrap_or_else(|| "unspecified".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cmake_arg() {
        let content = r#"
        FetchContent_Declare(
          fmt
          GIT_REPOSITORY https://github.com/fmtlib/fmt.git
          GIT_TAG        v9.1.0
        )
        "#;

        assert_eq!(
            extract_cmake_arg(content, "GIT_REPOSITORY"),
            Some("https://github.com/fmtlib/fmt.git".to_string())
        );
        assert_eq!(
            extract_cmake_arg(content, "GIT_TAG"),
            Some("v9.1.0".to_string())
        );
        assert_eq!(extract_cmake_arg(content, "NONEXISTENT"), None);
    }

    #[test]
    fn test_extract_cmake_arg_quoted_with_spaces() {
        let content = r#"
        ExternalProject_Add(
          myproject
          URL "https://example.com/files/project with spaces.tar.gz"
          SOURCE_DIR "path/to/source dir"
        )
        "#;

        assert_eq!(
            extract_cmake_arg(content, "URL"),
            Some("https://example.com/files/project with spaces.tar.gz".to_string())
        );
        assert_eq!(
            extract_cmake_arg(content, "SOURCE_DIR"),
            Some("path/to/source dir".to_string())
        );
    }

    #[test]
    fn test_extract_version_from_url() {
        // Single-number versions
        assert_eq!(extract_version_from_url("v5"), "5");
        assert_eq!(extract_version_from_url("release-1"), "1");
        // Multi-component versions
        assert_eq!(extract_version_from_url("v1.2.3"), "1.2.3");
        assert_eq!(extract_version_from_url("release-2.0.1"), "2.0.1");
        assert_eq!(
            extract_version_from_url(
                "https://github.com/owner/repo/archive/refs/tags/v3.1.0.tar.gz"
            ),
            "3.1.0"
        );
        // Edge case: multiple version-like numbers, should pick the rightmost
        assert_eq!(extract_version_from_url("v1.0-20230125.3"), "20230125.3");
        assert_eq!(extract_version_from_url("no-version-here"), "unspecified");
    }
}
