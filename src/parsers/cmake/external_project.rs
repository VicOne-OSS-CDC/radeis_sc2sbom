//! ExternalProject_Add parser for CMake
//!
//! Parses ExternalProject_Add blocks to extract dependency information.
//! ExternalProject is the legacy CMake way to include external projects.

use super::utils::{extract_cmake_arg, extract_version_from_url};
use crate::models::dependency::{Dependency, DependencySource};
use regex::Regex;
use std::path::Path;

/// Find the index of the closing parenthesis that matches the opening parenthesis at `open_paren_index`.
///
/// This performs a scan that:
/// - Tracks nested parentheses using a depth counter, starting at 1 (for the opening paren).
/// - Ignores parentheses that appear inside double-quoted strings.
/// - Ignores parentheses that appear inside `#` line comments.
fn find_matching_paren(content: &str, open_paren_index: usize) -> Option<usize> {
    let mut depth: usize = 1; // Start at 1 since we're already at the opening paren
    let mut in_string = false;
    let mut escape = false;
    let mut in_comment = false;

    // Iterate over characters AFTER the opening parenthesis
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
            if escape {
                // Current character is escaped; skip its special meaning
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => {
                in_string = true;
            }
            '#' => {
                in_comment = true;
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

/// Parse ExternalProject_Add blocks from CMake content
///
/// Example CMake:
/// ```cmake
/// ExternalProject_Add(
///   googletest
///   GIT_REPOSITORY https://github.com/google/googletest.git
///   GIT_TAG        release-1.12.1
///   CMAKE_ARGS     -DCMAKE_INSTALL_PREFIX=${CMAKE_BINARY_DIR}/install
/// )
/// ```
pub fn parse_external_project(
    content: &str,
    source_path: &Path,
) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let mut dependencies = Vec::new();

    // Regex to locate the start of ExternalProject_Add blocks and capture the project name.
    // (?is) = case insensitive + dot matches newlines
    // Group 1: the opening parenthesis '('
    // Group 2: the project name
    let pattern = r"(?is)ExternalProject_Add\s*(\()\s*(\w+)";
    let regex = Regex::new(pattern)?;

    for captures in regex.captures_iter(content) {
        let open_paren_match = captures
            .get(1)
            .expect("Expected opening parenthesis capture for ExternalProject_Add");
        let name_match = captures
            .get(2)
            .expect("Expected project name capture for ExternalProject_Add");

        let name = name_match.as_str().trim().to_string();

        // Find the matching closing parenthesis for this ExternalProject_Add call.
        let open_paren_index = open_paren_match.start();
        let closing_paren_index = match find_matching_paren(content, open_paren_index) {
            Some(idx) => idx,
            None => {
                eprintln!(
                    "Warning: Skipping {} - unable to find matching ')' for ExternalProject_Add",
                    name
                );
                continue;
            }
        };

        // The body starts right after the project name and ends before the matching ')'.
        let body_start = name_match.end();
        if body_start > closing_paren_index {
            eprintln!(
                "Warning: Skipping {} - invalid ExternalProject_Add block range",
                name
            );
            continue;
        }
        let body = &content[body_start..closing_paren_index];

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
        // Skip dependencies with only local configurations
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
    fn test_parse_external_project_git() {
        let cmake_content = r#"
        ExternalProject_Add(
          googletest
          GIT_REPOSITORY https://github.com/google/googletest.git
          GIT_TAG        release-1.12.1
          CMAKE_ARGS     -DCMAKE_INSTALL_PREFIX=${CMAKE_BINARY_DIR}/install
        )
        "#;

        let deps = parse_external_project(cmake_content, Path::new("CMakeLists.txt")).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "googletest");
        assert_eq!(deps[0].version, "release-1.12.1");
        assert_eq!(
            deps[0].repository_url,
            Some("https://github.com/google/googletest.git".to_string())
        );
        assert_eq!(deps[0].ecosystem, "cmake");
    }

    #[test]
    fn test_parse_external_project_url() {
        let cmake_content = r#"
        ExternalProject_Add(
          zlib
          URL https://zlib.net/zlib-1.2.13.tar.gz
          URL_HASH SHA256=abc123
        )
        "#;

        let deps = parse_external_project(cmake_content, Path::new("CMakeLists.txt")).unwrap();
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "1.2.13");
        assert_eq!(deps[0].checksum_sha256, Some("abc123".to_string()));
    }
}
