//! Bazel MODULE.bazel parser (Bazel 6.0+ bzlmod)
//!
//! Extracts dependencies from MODULE.bazel files (new dependency format in Bazel 6.0+).
//!
//! ## Extracted dependency types:
//! - `bazel_dep()` - Module dependencies with explicit versions

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;

lazy_static::lazy_static! {
    /// Regex for bazel_dep() calls (supports multi-line and flexible parameter order)
    /// Matches: bazel_dep(name = "abseil-cpp", version = "20230802.1")
    /// Also matches: bazel_dep(version = "1.0", name = "foo")
    /// And multi-line variants
    static ref BAZEL_DEP_NAME: Regex = Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap();
    static ref BAZEL_DEP_VERSION: Regex = Regex::new(r#"version\s*=\s*"([^"]+)""#).unwrap();
    static ref BAZEL_DEP_CALL: Regex = Regex::new(r#"bazel_dep\s*\("#).unwrap();
}

/// Parse a MODULE.bazel file and extract dependencies
///
/// # Arguments
/// * `path` - Path to the MODULE.bazel file
///
/// # Returns
/// A vector of dependencies extracted from the file
///
/// # Example
/// ```ignore
/// let deps = parse_module_bazel(Path::new("MODULE.bazel"))?;
/// for dep in deps {
///     println!("{}: {}", dep.name, dep.version);
/// }
/// ```
///
/// # Note
/// Comment removal uses simple `#` detection and does not respect string literal boundaries.
/// Edge case: strings containing `#` will be truncated, though this is rare in practice.
pub fn parse_module_bazel(path: &Path) -> Result<Vec<Dependency>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();

    // Remove comments and build a single string so we can match multi-line bazel_dep() calls
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

    // Find all bazel_dep() calls and extract name/version from each block
    let mut start_indices = Vec::new();
    for mat in BAZEL_DEP_CALL.find_iter(&uncommented_content) {
        start_indices.push(mat.start());
    }

    for &start in &start_indices {
        let block_start = start + "bazel_dep".len();
        let remaining = &uncommented_content[block_start..];
        // Skip past the opening '(' and any whitespace before it
        let paren_pos = remaining.find('(').unwrap_or(0) + 1;
        let after_paren = &remaining[paren_pos..];
        let block_end =
            crate::parsers::bazel::find_matching_paren(after_paren).unwrap_or(after_paren.len());
        let block = &after_paren[..block_end];

        // Extract name and version from this bazel_dep block (order-independent)
        let name = match BAZEL_DEP_NAME.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => continue, // Skip if no name found
        };

        let version = match BAZEL_DEP_VERSION.captures(block) {
            Some(cap) => cap[1].to_string(),
            None => continue, // Skip if no version found
        };

        // Skip if already processed
        if !seen.insert(name.clone()) {
            continue;
        }

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "bazel-bzlmod".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format_source_info("bazel/bzlmod", path, None, false)),
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
    fn test_parse_basic_bazel_dep() {
        let content = r#"
module(name = "myproject")

bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "googletest", version = "1.14.0")
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert_eq!(deps.len(), 2);

        let absl = deps.iter().find(|d| d.name == "abseil-cpp").unwrap();
        assert_eq!(absl.version, "20230802.1");
        assert_eq!(absl.ecosystem, "bazel-bzlmod");

        let gtest = deps.iter().find(|d| d.name == "googletest").unwrap();
        assert_eq!(gtest.version, "1.14.0");
    }

    #[test]
    fn test_parse_bazel_dep_with_repo_name() {
        let content = r#"
bazel_dep(name = "grpc", version = "1.58.0", repo_name = "com_github_grpc_grpc")
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "grpc");
        assert_eq!(deps[0].version, "1.58.0");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# This is a comment
bazel_dep(name = "abseil-cpp", version = "20230802.1")  # Inline comment
# Another comment
bazel_dep(name = "googletest", version = "1.14.0")
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert_eq!(deps.len(), 2);
    }

    #[test]
    fn test_parse_duplicate_deps() {
        let content = r#"
bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "abseil-cpp", version = "20230802.1")
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        // Should be deduplicated
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn test_parse_empty_module() {
        let content = r#"
module(name = "empty")
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_multiline_bazel_dep() {
        let content = r#"
module(name = "myproject")

bazel_dep(
    name = "abseil-cpp",
    version = "20230802.1"
)

bazel_dep(
    name = "googletest",
    version = "1.14.0",
    repo_name = "com_google_googletest"
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert_eq!(deps.len(), 2);

        let absl = deps.iter().find(|d| d.name == "abseil-cpp").unwrap();
        assert_eq!(absl.version, "20230802.1");

        let gtest = deps.iter().find(|d| d.name == "googletest").unwrap();
        assert_eq!(gtest.version, "1.14.0");
    }

    #[test]
    fn test_parse_flexible_parameter_order() {
        let content = r#"
module(name = "myproject")

# Normal order: name then version
bazel_dep(name = "abseil-cpp", version = "20230802.1")

# Reversed order: version then name
bazel_dep(version = "1.14.0", name = "googletest")

# Multi-line with reversed order
bazel_dep(
    version = "1.58.0",
    name = "grpc"
)
        "#;

        let file = create_temp_file(content);
        let deps = parse_module_bazel(file.path()).unwrap();

        assert_eq!(deps.len(), 3);

        let absl = deps.iter().find(|d| d.name == "abseil-cpp").unwrap();
        assert_eq!(absl.version, "20230802.1");

        let gtest = deps.iter().find(|d| d.name == "googletest").unwrap();
        assert_eq!(gtest.version, "1.14.0");

        let grpc = deps.iter().find(|d| d.name == "grpc").unwrap();
        assert_eq!(grpc.version, "1.58.0");
    }
}
