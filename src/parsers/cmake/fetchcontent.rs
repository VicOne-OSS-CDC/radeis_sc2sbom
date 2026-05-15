//! FetchContent_Declare parser for CMake
//!
//! Parses FetchContent_Declare blocks to extract dependency information.
//! FetchContent is the modern CMake (3.11+) way to fetch external dependencies.

use super::utils::{extract_cmake_arg, extract_version_from_url};
use crate::models::dependency::{Dependency, DependencySource};
use regex::Regex;
use std::path::Path;

/// Find the matching closing parenthesis, handling nested parentheses, strings, and comments
fn find_matching_paren(content: &str, open_paren_index: usize) -> Option<usize> {
    let mut depth = 1usize;
    let mut in_string = false;
    let mut string_delim: char = '\0';
    let mut in_comment = false;

    let mut iter = content
        .char_indices()
        .skip_while(|(idx, _)| *idx <= open_paren_index);
    while let Some((idx, ch)) = iter.next() {
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }

        if in_string {
            if ch == '\\' {
                // Skip the next character to handle simple escaping.
                iter.next();
                continue;
            }
            if ch == string_delim {
                in_string = false;
            }
            continue;
        }

        match ch {
            '#' => {
                in_comment = true;
            }
            '"' | '\'' => {
                in_string = true;
                string_delim = ch;
            }
            '(' => {
                depth += 1;
            }
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }

    None
}

/// Parse FetchContent_Declare blocks from CMake content
///
/// Example CMake:
/// ```cmake
/// FetchContent_Declare(
///   json
///   GIT_REPOSITORY https://github.com/nlohmann/json.git
///   GIT_TAG        v3.11.2
/// )
/// ```
pub fn parse_fetchcontent(
    content: &str,
    source_path: &Path,
) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let mut dependencies = Vec::new();

    // Regex to find the start of FetchContent_Declare blocks and capture the name.
    // (?is) = case insensitive + dot matches newlines
    let pattern = r"(?is)FetchContent_Declare\s*\(\s*(\w+)";
    let regex = Regex::new(pattern)?;

    for captures in regex.captures_iter(content) {
        let name_match = captures.get(1).unwrap();
        let name = name_match.as_str().trim().to_string();

        // Determine the position of the opening parenthesis for this block.
        let full_match = captures.get(0).unwrap();
        let match_start = full_match.start();
        let match_end = full_match.end();
        let prefix = &content[match_start..match_end];
        let rel_open_paren = match prefix.rfind('(') {
            Some(pos) => pos,
            None => continue,
        };
        let open_paren_index = match_start + rel_open_paren;

        // Scan forward to find the matching closing parenthesis
        let close_paren_index = match find_matching_paren(content, open_paren_index) {
            Some(idx) => idx,
            None => {
                eprintln!(
                    "Warning: Skipping {} - unable to find matching ')' for FetchContent_Declare",
                    name
                );
                continue;
            }
        };

        // Body starts after the name; skip any whitespace.
        let mut body_start = name_match.end();
        while let Some(ch) = content[body_start..].chars().next() {
            if ch.is_whitespace() {
                body_start += ch.len_utf8();
            } else {
                break;
            }
        }
        let body_end = close_paren_index;
        if body_start > body_end || body_end > content.len() {
            eprintln!(
                "Warning: Skipping {} - invalid FetchContent_Declare block range",
                name
            );
            continue;
        }
        let body = &content[body_start..body_end];

        let mut dep = Dependency {
            name: name.clone(),
            version: String::new(),
            ecosystem: "cmake".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            source_file: Some(source_path.to_string_lossy().to_string()), // v1.0.6: Use actual path for directory-based classification
            ..Default::default()
        };

        // Extract GIT_REPOSITORY
        if let Some(repo) = extract_cmake_arg(body, "GIT_REPOSITORY") {
            // Check for CMake variables (cannot resolve statically)
            if repo.contains("${") {
                eprintln!(
                    "Warning: Skipping {} - CMake variable in GIT_REPOSITORY: {}",
                    name, repo
                );
                continue;
            }
            dep.repository_url = Some(repo);
        }

        // Extract GIT_TAG (version)
        if let Some(tag) = extract_cmake_arg(body, "GIT_TAG") {
            if tag.contains("${") {
                eprintln!(
                    "Warning: Skipping {} - CMake variable in GIT_TAG: {}",
                    name, tag
                );
                continue;
            }
            dep.version = tag;
        }

        // Extract URL (alternative to GIT_REPOSITORY)
        if let Some(url) = extract_cmake_arg(body, "URL") {
            if url.contains("${") {
                eprintln!(
                    "Warning: Skipping {} - CMake variable in URL: {}",
                    name, url
                );
                continue;
            }
            dep.repository_url = Some(url.clone());

            // Try to extract version from URL path if GIT_TAG not present
            if dep.version.is_empty() {
                dep.version = extract_version_from_url(&url);
            }
        }

        // Extract URL_HASH (checksum)
        if let Some(hash) = extract_cmake_arg(body, "URL_HASH") {
            // Format: SHA256=abc123...
            if let Some((alg, value)) = hash.split_once('=') {
                if alg.trim().eq_ignore_ascii_case("SHA256") {
                    dep.checksum_sha256 = Some(value.trim().to_string());
                }
            }
        }

        // Set default version if still empty
        if dep.version.is_empty() {
            dep.version = "unspecified".to_string();
        }

        // Only add if we have a resolvable source (URL or Git repository)
        // Skip dependencies with only SOURCE_DIR or FIND_PACKAGE_ARGS
        if dep.repository_url.is_some() {
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_fetchcontent_git() {
        let cmake_content = r#"
        FetchContent_Declare(
          json
          GIT_REPOSITORY https://github.com/nlohmann/json.git
          GIT_TAG        v3.11.2
        )
        "#;

        let deps = parse_fetchcontent(cmake_content, Path::new("CMakeLists.txt")).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "json");
        assert_eq!(deps[0].version, "v3.11.2");
        assert_eq!(
            deps[0].repository_url,
            Some("https://github.com/nlohmann/json.git".to_string())
        );
        assert_eq!(deps[0].ecosystem, "cmake");
    }

    #[test]
    fn test_parse_fetchcontent_url() {
        let cmake_content = r#"
        FetchContent_Declare(
          fmt
          URL https://github.com/fmtlib/fmt/releases/download/9.1.0/fmt-9.1.0.zip
          URL_HASH SHA256=abc123def456
        )
        "#;

        let deps = parse_fetchcontent(cmake_content, Path::new("CMakeLists.txt")).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "fmt");
        assert_eq!(deps[0].version, "9.1.0");
        assert_eq!(deps[0].checksum_sha256, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_skip_cmake_variables() {
        let cmake_content = r#"
        set(JSON_VERSION "v3.11.2")
        FetchContent_Declare(
          json
          GIT_REPOSITORY https://github.com/nlohmann/json.git
          GIT_TAG        ${JSON_VERSION}
        )
        "#;

        let deps = parse_fetchcontent(cmake_content, Path::new("CMakeLists.txt")).unwrap();
        // Should skip due to CMake variable
        assert_eq!(deps.len(), 0);
    }
}
