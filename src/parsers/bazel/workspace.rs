//! Bazel WORKSPACE file parser
//!
//! Extracts dependencies from Bazel WORKSPACE files using regex-based patterns.
//!
//! ## Extracted dependency types:
//! 1. `http_archive()` - HTTP tarball dependencies
//! 2. `git_repository()` - Git repository dependencies
//! 3. `local_repository()` - Local path dependencies (informational)

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;

lazy_static::lazy_static! {
    /// Regex to find http_archive function calls
    static ref HTTP_ARCHIVE_PATTERN: Regex = Regex::new(r#"\bhttp_archive\s*\("#).unwrap();

    /// Regex to find git_repository function calls
    static ref GIT_REPOSITORY_PATTERN: Regex = Regex::new(r#"\bgit_repository\s*\("#).unwrap();

    /// Regex to find local_repository function calls
    static ref LOCAL_REPOSITORY_PATTERN: Regex = Regex::new(r#"\blocal_repository\s*\("#).unwrap();

    /// Shared regex for extracting name parameter from Bazel declarations
    /// Used by http_archive, git_repository, and local_repository
    static ref NAME_PATTERN: Regex = Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap();

    /// Regex for strip_prefix in http_archive
    static ref STRIP_PREFIX: Regex = Regex::new(r#"strip_prefix\s*=\s*"([^"]+)""#).unwrap();

    /// Regex for urls in http_archive (captures the first URL from a single value or a list)
    /// Note: For multi-URL lists like `urls = ["url1", "url2"]`, this captures only "url1"
    /// This is intentional - version extraction typically works from any URL, and first is standard practice
    static ref HTTP_URLS: Regex = Regex::new(r#"urls?\s*=\s*(?:\[?\s*"([^"]+)")"#).unwrap();

    /// Regex for git remote
    static ref GIT_REMOTE: Regex = Regex::new(r#"remote\s*=\s*"([^"]+)""#).unwrap();

    /// Regex for git tag
    static ref GIT_TAG: Regex = Regex::new(r#"tag\s*=\s*"([^"]+)""#).unwrap();

    /// Regex for git commit
    static ref GIT_COMMIT: Regex = Regex::new(r#"commit\s*=\s*"([^"]+)""#).unwrap();

    /// Regex for local path
    static ref LOCAL_PATH: Regex = Regex::new(r#"path\s*=\s*"([^"]+)""#).unwrap();

    /// Regex to extract version from strings (supports single or multi-component versions)
    /// Matches: v1, 1, 1.0, 1.2.3, v1.2.3.4
    static ref VERSION_PATTERN: Regex = Regex::new(r#"v?(\d+(?:\.\d+)*)"#).unwrap();
}

/// Parse a Bazel WORKSPACE file and extract dependencies
///
/// # Arguments
/// * `path` - Path to the WORKSPACE or WORKSPACE.bazel file
///
/// # Returns
/// A vector of dependencies extracted from the file
///
/// # Note
/// Comment removal uses simple `#` detection and does not respect string literal boundaries.
/// Edge case: strings containing `#` (e.g., `"version#1"`) will be truncated, though this is rare in practice.
pub fn parse_workspace(path: &Path) -> Result<Vec<Dependency>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();

    // Remove comments to avoid matching commented-out dependencies
    let mut uncommented_content = String::new();
    for line in content.lines() {
        let line_no_comment = if let Some(pos) = line.find('#') {
            &line[..pos]
        } else {
            line
        };
        uncommented_content.push_str(line_no_comment);
        uncommented_content.push('\n');
    }

    // Extract http_archive dependencies
    dependencies.extend(extract_http_archives(
        &uncommented_content,
        path,
        &mut seen,
    )?);

    // Extract git_repository dependencies
    dependencies.extend(extract_git_repositories(
        &uncommented_content,
        path,
        &mut seen,
    )?);

    // Extract local_repository (informational only)
    dependencies.extend(extract_local_repositories(
        &uncommented_content,
        path,
        &mut seen,
    )?);

    Ok(dependencies)
}

/// Extract http_archive() declarations
fn extract_http_archives(
    content: &str,
    path: &Path,
    seen: &mut HashSet<String>,
) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    // Find all http_archive function calls using regex
    let mut start_indices = Vec::new();
    for mat in HTTP_ARCHIVE_PATTERN.find_iter(content) {
        start_indices.push(mat.start());
    }

    // Process each http_archive block
    for &start in &start_indices {
        // Find the end of this function call by counting parentheses
        let block_start = start + "http_archive".len();
        let remaining = &content[block_start..];
        // Skip past the opening '(' and any whitespace before it
        let paren_pos = remaining.find('(').unwrap_or(0) + 1;
        let after_paren = &remaining[paren_pos..];
        let block_end =
            crate::parsers::bazel::find_matching_paren(after_paren).unwrap_or(after_paren.len());
        let block = &after_paren[..block_end];
        // Extract name
        let name = match NAME_PATTERN.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => continue,
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        // Try to extract version from strip_prefix
        let mut version = String::from("unspecified");
        if let Some(cap) = STRIP_PREFIX.captures(block) {
            let strip_prefix = &cap[1];
            // Example: "googletest-1.14.0" -> "1.14.0"
            if let Some(ver_cap) = VERSION_PATTERN.captures(strip_prefix) {
                version = ver_cap[1].to_string();
            }
        }

        // If version not found in strip_prefix, try URL
        if version == "unspecified" {
            if let Some(cap) = HTTP_URLS.captures(block) {
                let url = &cap[1];
                if let Some(ver_cap) = VERSION_PATTERN.captures(url) {
                    version = ver_cap[1].to_string();
                }
            }
        }

        // Extract URL for repository_url field
        let repository_url = HTTP_URLS.captures(block).map(|cap| cap[1].to_string());

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "bazel".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format_source_info("bazel/workspace", path, None, false)),
            is_dev: false,
            is_direct: true,
            repository_url,
            ..Default::default()
        });
    }

    Ok(dependencies)
}

/// Extract git_repository() declarations
fn extract_git_repositories(
    content: &str,
    path: &Path,
    seen: &mut HashSet<String>,
) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    // Find all git_repository function calls using regex
    let mut start_indices = Vec::new();
    for mat in GIT_REPOSITORY_PATTERN.find_iter(content) {
        start_indices.push(mat.start());
    }

    // Process each git_repository block
    for &start in &start_indices {
        let block_start = start + "git_repository".len();
        let remaining = &content[block_start..];
        // Skip past the opening '(' and any whitespace before it
        let paren_pos = remaining.find('(').unwrap_or(0) + 1;
        let after_paren = &remaining[paren_pos..];
        let block_end =
            crate::parsers::bazel::find_matching_paren(after_paren).unwrap_or(after_paren.len());
        let block = &after_paren[..block_end];
        // Extract name
        let name = match NAME_PATTERN.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => continue,
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        // Extract version from tag or commit
        let mut version = String::from("unspecified");
        if let Some(cap) = GIT_TAG.captures(block) {
            version = cap[1].to_string();
        } else if let Some(cap) = GIT_COMMIT.captures(block) {
            // Use short commit SHA (first 7 chars)
            let commit = &cap[1];
            version = if commit.len() > 7 {
                commit[..7].to_string()
            } else {
                commit.to_string()
            };
        }

        // Extract remote URL
        let repository_url = GIT_REMOTE.captures(block).map(|cap| cap[1].to_string());

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "bazel".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format_source_info("bazel/workspace", path, None, false)),
            is_dev: false,
            is_direct: true,
            repository_url,
            ..Default::default()
        });
    }

    Ok(dependencies)
}

/// Extract local_repository() declarations (informational)
fn extract_local_repositories(
    content: &str,
    path: &Path,
    seen: &mut HashSet<String>,
) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    // Find all local_repository function calls using regex
    let mut start_indices = Vec::new();
    for mat in LOCAL_REPOSITORY_PATTERN.find_iter(content) {
        start_indices.push(mat.start());
    }

    // Process each local_repository block
    for &start in &start_indices {
        let block_start = start + "local_repository".len();
        let remaining = &content[block_start..];
        // Skip past the opening '(' and any whitespace before it
        let paren_pos = remaining.find('(').unwrap_or(0) + 1;
        let after_paren = &remaining[paren_pos..];
        let block_end =
            crate::parsers::bazel::find_matching_paren(after_paren).unwrap_or(after_paren.len());
        let block = &after_paren[..block_end];
        // Extract name
        let name = match NAME_PATTERN.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => continue,
        };

        if !seen.insert(name.clone()) {
            continue;
        }

        // Extract local path
        let local_path = LOCAL_PATH.captures(block).map(|cap| cap[1].to_string());

        let source_info = if let Some(ref p) = local_path {
            format!(
                "{} (local: {})",
                format_source_info("bazel/workspace", path, None, false),
                p
            )
        } else {
            format_source_info("bazel/workspace", path, None, false)
        };

        dependencies.push(Dependency {
            name,
            version: "local".to_string(),
            ecosystem: "bazel".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(source_info),
            is_dev: false,
            is_direct: true,
            ..Default::default()
        });
    }

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_http_archive() {
        let content = r#"
workspace(name = "myproject")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

http_archive(
    name = "com_google_absl",
    urls = ["https://github.com/abseil/abseil-cpp/archive/refs/tags/20230802.1.tar.gz"],
    strip_prefix = "abseil-cpp-20230802.1",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com_google_absl");
        assert_eq!(deps[0].version, "20230802.1");
        assert_eq!(deps[0].ecosystem, "bazel");
    }

    #[test]
    fn test_parse_git_repository() {
        let content = r#"
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

git_repository(
    name = "com_github_grpc_grpc",
    remote = "https://github.com/grpc/grpc.git",
    tag = "v1.58.0",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "com_github_grpc_grpc");
        assert_eq!(deps[0].version, "v1.58.0");
        assert!(deps[0].repository_url.is_some());
    }

    #[test]
    fn test_parse_git_repository_with_commit() {
        let content = r#"
git_repository(
    name = "my_dep",
    remote = "https://github.com/owner/repo.git",
    commit = "abc1234567890",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].version, "abc1234"); // Short commit SHA
    }

    #[test]
    fn test_parse_local_repository() {
        let content = r#"
local_repository(
    name = "my_local_dep",
    path = "../my-lib",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "my_local_dep");
        assert_eq!(deps[0].version, "local");
        assert!(deps[0].source_file.as_ref().unwrap().contains("../my-lib"));
    }

    #[test]
    fn test_parse_multiple_dependencies() {
        let content = r#"
workspace(name = "myproject")

http_archive(
    name = "com_google_absl",
    urls = ["https://github.com/abseil/abseil-cpp/archive/20230802.1.tar.gz"],
    strip_prefix = "abseil-cpp-20230802.1",
)

git_repository(
    name = "com_github_grpc_grpc",
    remote = "https://github.com/grpc/grpc.git",
    tag = "v1.58.0",
)

local_repository(
    name = "my_local",
    path = "../local",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 3);

        let absl = deps.iter().find(|d| d.name == "com_google_absl").unwrap();
        assert_eq!(absl.version, "20230802.1");

        let grpc = deps
            .iter()
            .find(|d| d.name == "com_github_grpc_grpc")
            .unwrap();
        assert_eq!(grpc.version, "v1.58.0");

        let local = deps.iter().find(|d| d.name == "my_local").unwrap();
        assert_eq!(local.version, "local");
    }

    #[test]
    fn test_parse_empty_workspace() {
        let content = "workspace(name = \"empty\")\n";

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert!(deps.is_empty());
    }

    #[test]
    fn test_version_extraction_from_url() {
        let content = r#"
http_archive(
    name = "googletest",
    urls = ["https://github.com/google/googletest/archive/v1.14.0.tar.gz"],
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps[0].version, "1.14.0");
    }

    #[test]
    fn test_empty_dependency_name_skipped() {
        let content = r#"
http_archive(
    name = "",
    urls = ["https://example.com/lib.tar.gz"],
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        // Empty name should be skipped
        assert!(deps.is_empty());
    }

    #[test]
    fn test_special_characters_in_strings() {
        let content = r#"
http_archive(
    name = "my-lib_v2.0",
    urls = ["https://example.com/my-lib@v2.0.tar.gz"],
    strip_prefix = "my-lib_v2.0",
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "my-lib_v2.0");
        assert_eq!(deps[0].version, "2.0");
    }

    #[test]
    fn test_malformed_syntax_graceful_degradation() {
        let content = r#"
http_archive(
    name = "good_dep",
    urls = ["https://example.com/good-1.0.tar.gz"],
)

http_archive(
    name = "malformed
    # Missing closing quote
)

http_archive(
    name = "another_good",
    urls = ["https://example.com/another-2.0.tar.gz"],
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        // Should extract valid dependencies, skip malformed ones
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "good_dep"));
        assert!(deps.iter().any(|d| d.name == "another_good"));
    }

    #[test]
    fn test_very_long_lines() {
        let long_url = "https://example.com/".to_string() + &"a".repeat(1000) + "/v1.0.tar.gz";
        let content = format!(
            r#"
http_archive(
    name = "long_url_dep",
    urls = ["{}"],
)
        "#,
            long_url
        );

        let file = create_temp_file(&content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "long_url_dep");
    }

    #[test]
    fn test_comments_at_various_positions() {
        let content = r#"
# Comment at start of file
workspace(name = "myproject") # Inline comment

# Comment before dependency
http_archive( # Comment after function name
    name = "dep1", # Comment after parameter
    urls = ["https://example.com/dep1-1.0.tar.gz"], # Comment after URL
) # Comment after closing paren

# Comment between dependencies
http_archive(
    name = "dep2",
    urls = ["https://example.com/dep2-2.0.tar.gz"],
)
# Comment at end of file
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "dep1"));
        assert!(deps.iter().any(|d| d.name == "dep2"));
    }

    #[test]
    fn test_multiple_hashes_in_same_line() {
        let content = r#"
http_archive(
    name = "dep1", # First comment # Second comment # Third comment
    urls = ["https://example.com/dep1-1.0.tar.gz"],
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep1");
    }

    #[test]
    fn test_hash_at_end_of_line() {
        let content = r#"
http_archive(
    name = "dep1",
    urls = ["https://example.com/dep1-1.0.tar.gz"],#
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep1");
    }

    #[test]
    fn test_nested_parentheses() {
        let content = r#"
http_archive(
    name = "dep1",
    urls = ["https://example.com/dep1.tar.gz"],
    patches = [
        "//patches:fix.patch",
    ],
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_workspace(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep1");
    }
}
