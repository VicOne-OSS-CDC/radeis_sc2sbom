//! meson.build file parser
//!
//! Extracts dependencies from Meson build files using regex-based patterns.
//!
//! ## Extracted dependency types:
//! 1. `dependency('name')` - pkg-config dependencies
//! 2. `cc.find_library('name')` - System libraries
//! 3. `subproject('name')` - Meson subprojects

use anyhow::Result;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;

lazy_static::lazy_static! {
    /// Regex for dependency() calls
    /// Matches: dependency('glib-2.0', version : '>= 2.50')
    /// Note: This pattern only captures the dependency name and version string.
    /// Additional parameters like `required:` and `modules:` are not extracted in v1.0.4.
    static ref DEP_PATTERN: Regex = Regex::new(
        r#"dependency\s*\(\s*['"]([^'"]+)['"](?:[^)]*version\s*:\s*['"]([^'"]+)['"])?"#
    ).unwrap();

    /// Regex for cc.find_library() calls
    /// Matches: cc.find_library('m', required : true)
    static ref FIND_LIB_PATTERN: Regex = Regex::new(
        r#"find_library\s*\(\s*['"]([^'"]+)['"]\s*(?:,\s*required\s*:\s*(true|false))?"#
    ).unwrap();

    /// Regex for subproject() calls
    /// Matches: subproject('libfoo')
    static ref SUBPROJECT_PATTERN: Regex = Regex::new(
        r#"subproject\s*\(\s*['"]([^'"]+)['"]"#
    ).unwrap();
}

/// Parse a meson.build file and extract dependencies
///
/// # Arguments
/// * `path` - Path to the meson.build file
///
/// # Returns
/// A vector of dependencies extracted from the file
///
/// # Example
/// ```ignore
/// let deps = parse_meson_build(Path::new("meson.build"))?;
/// for dep in deps {
///     println!("{}: {}", dep.name, dep.version);
/// }
/// ```
///
/// # Note
/// Comment removal uses simple `#` detection and does not respect string literal boundaries.
/// Edge case: strings containing `#` will be truncated, though this is rare in practice.
/// Deduplication uses prefixed keys ("lib:", "subproject:") to avoid collisions between dependency types.
pub fn parse_meson_build(path: &Path) -> Result<Vec<Dependency>> {
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

    // Extract dependency() calls (pkg-config packages)
    for cap in DEP_PATTERN.captures_iter(&uncommented_content) {
        let name = cap[1].to_string();

        // Skip if already processed
        if !seen.insert(name.clone()) {
            continue;
        }

        let version = cap
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unspecified".to_string());

        dependencies.push(Dependency {
            name: name.clone(),
            version,
            ecosystem: "meson".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format_source_info("meson", path, None, false)),
            is_dev: false,
            is_direct: true,
            ..Default::default()
        });
    }

    // Extract cc.find_library() calls (system libraries)
    for cap in FIND_LIB_PATTERN.captures_iter(&uncommented_content) {
        let name = cap[1].to_string();

        // Skip if already processed
        if !seen.insert(format!("lib:{}", name)) {
            continue;
        }

        dependencies.push(Dependency {
            name: name.clone(),
            version: "unspecified".to_string(),
            ecosystem: "system".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format!(
                "{} (find_library)",
                format_source_info("meson", path, None, false)
            )),
            is_dev: false,
            is_direct: true,
            ..Default::default()
        });
    }

    // Extract subproject() calls
    // Note: Actual subproject resolution is handled by wrap parser
    for cap in SUBPROJECT_PATTERN.captures_iter(&uncommented_content) {
        let name = cap[1].to_string();

        // Skip if already processed
        if !seen.insert(format!("subproject:{}", name)) {
            continue;
        }

        // Add as informational dependency
        // The actual subproject details will be parsed from .wrap files
        dependencies.push(Dependency {
            name: name.clone(),
            version: "subproject".to_string(),
            ecosystem: "meson-subproject".to_string(),
            source: DependencySource::Manifest,
            source_file: Some(format!(
                "{} (subproject reference)",
                format_source_info("meson", path, None, false)
            )),
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
    fn test_parse_basic_dependency() {
        let content = r#"
project('myproject', 'c')
glib_dep = dependency('glib-2.0', version : '>= 2.50')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "glib-2.0");
        assert_eq!(deps[0].version, ">= 2.50");
        assert_eq!(deps[0].ecosystem, "meson");
    }

    #[test]
    fn test_parse_dependency_without_version() {
        let content = r#"
zlib_dep = dependency('zlib')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "unspecified");
    }

    #[test]
    fn test_parse_find_library() {
        let content = r#"
cc = meson.get_compiler('c')
math_dep = cc.find_library('m', required : true)
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "m");
        assert_eq!(deps[0].ecosystem, "system");
        assert!(deps[0]
            .source_file
            .as_ref()
            .unwrap()
            .contains("find_library"));
    }

    #[test]
    fn test_parse_subproject() {
        let content = r#"
libfoo_proj = subproject('libfoo')
libfoo_dep = libfoo_proj.get_variable('libfoo_dep')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "libfoo");
        assert_eq!(deps[0].ecosystem, "meson-subproject");
    }

    #[test]
    fn test_parse_multiple_dependencies() {
        let content = r#"
project('myproject', 'c')

glib_dep = dependency('glib-2.0', version : '>= 2.50')
openssl_dep = dependency('openssl', version : '>= 1.1')
zlib_dep = dependency('zlib')

cc = meson.get_compiler('c')
math_dep = cc.find_library('m', required : true)
pthread_dep = cc.find_library('pthread')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 5);

        // Check dependencies
        let glib = deps.iter().find(|d| d.name == "glib-2.0").unwrap();
        assert_eq!(glib.version, ">= 2.50");

        let openssl = deps.iter().find(|d| d.name == "openssl").unwrap();
        assert_eq!(openssl.version, ">= 1.1");

        // Check system libraries
        let math = deps.iter().find(|d| d.name == "m").unwrap();
        assert_eq!(math.ecosystem, "system");
    }

    #[test]
    fn test_parse_duplicate_dependencies() {
        let content = r#"
glib_dep = dependency('glib-2.0')
glib_dep2 = dependency('glib-2.0')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        // Should be deduplicated
        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "glib-2.0");
    }

    #[test]
    fn test_parse_empty_file() {
        let content = "project('empty', 'c')\n";

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
project('myproject', 'c')

# This is a commented-out dependency
# zlib_dep = dependency('zlib')

glib_dep = dependency('glib-2.0', version : '>= 2.50')  # Inline comment
openssl_dep = dependency('openssl')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        // Should only find glib-2.0 and openssl, not zlib (commented out)
        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "glib-2.0"));
        assert!(deps.iter().any(|d| d.name == "openssl"));
        assert!(!deps.iter().any(|d| d.name == "zlib"));
    }

    #[test]
    fn test_empty_dependency_name_skipped() {
        let content = r#"
dependency('')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        // Empty name should be skipped by the regex
        assert!(deps.is_empty());
    }

    #[test]
    fn test_special_characters_in_dependency_names() {
        let content = r#"
glib_dep = dependency('glib-2.0', version : '>= 2.50')
gtk_dep = dependency('gtk+-3.0')
lib_dep = dependency('my_lib')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 3);
        assert!(deps.iter().any(|d| d.name == "glib-2.0"));
        assert!(deps.iter().any(|d| d.name == "gtk+-3.0"));
        assert!(deps.iter().any(|d| d.name == "my_lib"));
    }

    #[test]
    fn test_malformed_syntax_graceful_degradation() {
        let content = r#"
dependency('good1', version : '>= 1.0')
dependency( # Missing name parameter
dependency('good2')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        // Should extract valid dependencies, skip malformed ones
        // Regex-based parsing will naturally skip incomplete dependency() calls
        assert!(deps.iter().any(|d| d.name == "good1"));
        assert!(deps.iter().any(|d| d.name == "good2"));
    }

    #[test]
    fn test_very_long_lines() {
        let long_name = "a".repeat(500);
        let content = format!(
            r#"
dependency('{}')
        "#,
            long_name
        );

        let file = create_temp_file(&content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, long_name);
    }

    #[test]
    fn test_comments_at_various_positions() {
        let content = r#"
# Comment at start
project('myproject', 'c') # Inline comment

# Comment before dependency
dependency('dep1') # Comment after
# Comment between
dependency('dep2')
# Comment at end
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 2);
        assert!(deps.iter().any(|d| d.name == "dep1"));
        assert!(deps.iter().any(|d| d.name == "dep2"));
    }

    #[test]
    fn test_multiple_hashes_in_same_line() {
        let content = r#"
dependency('dep1') # First # Second # Third
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep1");
    }

    #[test]
    fn test_hash_at_end_of_line() {
        let content = r#"
dependency('dep1')#
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "dep1");
    }

    #[test]
    fn test_deduplication_across_different_types() {
        let content = r#"
dependency('mylib')
cc = meson.get_compiler('c')
mylib_sys = cc.find_library('mylib')
subproject('mylib')
        "#;

        let file = create_temp_file(content);
        let deps = parse_meson_build(file.path()).unwrap();

        // All three should be present because they use different prefixes in deduplication
        assert_eq!(deps.len(), 3);

        // Check that we have one from each type
        assert!(deps.iter().any(|d| d.ecosystem == "meson"));
        assert!(deps.iter().any(|d| d.ecosystem == "system"));
        assert!(deps.iter().any(|d| d.ecosystem == "meson-subproject"));
    }
}
