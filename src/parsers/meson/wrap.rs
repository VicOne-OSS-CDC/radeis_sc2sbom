//! Meson .wrap file parser
//!
//! Parses WrapDB subproject definition files.
//!
//! ## Supported formats:
//! - `[wrap-file]`: Tarball-based subprojects
//! - `[wrap-git]`: Git-based subprojects

use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;

lazy_static::lazy_static! {
    /// Regex to extract version from URLs (supports single or multi-component versions)
    /// Matches patterns like: zlib-1.tar.gz, zlib-1.2.13.tar.gz, fmt-9.1.0.zip, lib_v2.zip
    static ref VERSION_IN_URL: Regex = Regex::new(r#"[-_]v?(\d+(?:\.\d+)*)"#).unwrap();
}

#[derive(Debug, PartialEq)]
enum WrapType {
    File,
    Git,
}

/// Parse a single .wrap file and extract dependency information
///
/// # Arguments
/// * `path` - Path to the .wrap file
///
/// # Returns
/// A single Dependency extracted from the wrap file
pub fn parse_wrap_file(path: &Path) -> Result<Dependency> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Err(anyhow::anyhow!("Failed to read wrap file"));
        }
    };

    // Determine wrap type
    let wrap_type = if content.contains("[wrap-file]") {
        WrapType::File
    } else if content.contains("[wrap-git]") {
        WrapType::Git
    } else {
        eprintln!("Warning: Unknown wrap type in {}", path.display());
        return Err(anyhow::anyhow!("Unknown wrap type"));
    };

    // Extract package name from filename
    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    match wrap_type {
        WrapType::File => parse_wrap_file_type(&content, &name, path),
        WrapType::Git => parse_wrap_git_type(&content, &name, path),
    }
}

/// Parse [wrap-file] section
fn parse_wrap_file_type(content: &str, name: &str, path: &Path) -> Result<Dependency> {
    let mut url = String::new();
    let mut version = String::from("unspecified");
    let mut directory = String::new();

    // Parse INI-like format
    let mut in_wrap_section = false;
    for line in content.lines() {
        let line = line.trim();

        if line == "[wrap-file]" {
            in_wrap_section = true;
            continue;
        }

        if line.starts_with('[') {
            in_wrap_section = false;
            continue;
        }

        if !in_wrap_section || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            // Strip inline comments from value
            let value = if let Some(pos) = value.find('#') {
                value[..pos].trim()
            } else {
                value.trim()
            };

            match key {
                "source_url" => url = value.to_string(),
                "directory" => directory = value.to_string(),
                _ => {}
            }
        }
    }

    // Try to extract version from directory name or URL
    if !directory.is_empty() {
        if let Some(cap) = VERSION_IN_URL.captures(&directory) {
            version = cap[1].to_string();
        }
    }

    if version == "unspecified" && !url.is_empty() {
        if let Some(cap) = VERSION_IN_URL.captures(&url) {
            version = cap[1].to_string();
        }
    }

    Ok(Dependency {
        name: name.to_string(),
        version,
        ecosystem: "meson-wrap".to_string(),
        source: DependencySource::Manifest,
        source_file: Some(format_source_info("meson-wrap", path, None, false)),
        is_dev: false,
        is_direct: true,
        repository_url: if !url.is_empty() { Some(url) } else { None },
        ..Default::default()
    })
}

/// Parse [wrap-git] section
fn parse_wrap_git_type(content: &str, name: &str, path: &Path) -> Result<Dependency> {
    let mut url = String::new();
    let mut revision = String::from("unspecified");

    // Parse INI-like format
    let mut in_wrap_section = false;
    for line in content.lines() {
        let line = line.trim();

        if line == "[wrap-git]" {
            in_wrap_section = true;
            continue;
        }

        if line.starts_with('[') {
            in_wrap_section = false;
            continue;
        }

        if !in_wrap_section || line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            // Strip inline comments from value
            let value = if let Some(pos) = value.find('#') {
                value[..pos].trim()
            } else {
                value.trim()
            };

            match key {
                "url" => url = value.to_string(),
                "revision" => revision = value.to_string(),
                _ => {}
            }
        }
    }

    Ok(Dependency {
        name: name.to_string(),
        version: revision,
        ecosystem: "meson-wrap".to_string(),
        source: DependencySource::Manifest,
        source_file: Some(format_source_info("meson-wrap", path, None, false)),
        is_dev: false,
        is_direct: true,
        repository_url: if !url.is_empty() { Some(url) } else { None },
        ..Default::default()
    })
}

/// Parse all .wrap files in a subprojects directory
///
/// # Arguments
/// * `subprojects_dir` - Path to the subprojects directory
///
/// # Returns
/// A vector of dependencies extracted from all .wrap files
pub fn parse_all_wraps(subprojects_dir: &Path) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();

    if !subprojects_dir.exists() || !subprojects_dir.is_dir() {
        return Ok(dependencies);
    }

    let entries = match fs::read_dir(subprojects_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!(
                "Warning: Failed to read {}: {}",
                subprojects_dir.display(),
                e
            );
            return Ok(dependencies);
        }
    };

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("wrap") {
                if let Ok(dep) = parse_wrap_file(&path) {
                    dependencies.push(dep);
                }
            }
        }
    }

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_wrap(content: &str, name: &str) -> NamedTempFile {
        let file = NamedTempFile::with_suffix(&format!("_{}.wrap", name)).unwrap();
        let mut file_handle = file.reopen().unwrap();
        file_handle.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_wrap_file_basic() {
        let content = r#"
[wrap-file]
directory = zlib-1.2.13
source_url = https://zlib.net/fossils/zlib-1.2.13.tar.gz
source_filename = zlib-1.2.13.tar.gz
source_hash = b3a24de97a8fdbc835b9833169501030b8977031bcb54b3b3ac13740f846ab30

[provide]
zlib = zlib_dep
        "#;

        let file = create_temp_wrap(content, "zlib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert!(dep.name.contains("zlib"));
        assert_eq!(dep.version, "1.2.13");
        assert_eq!(dep.ecosystem, "meson-wrap");
        assert!(dep.repository_url.is_some());
    }

    #[test]
    fn test_parse_wrap_git_basic() {
        let content = r#"
[wrap-git]
url = https://github.com/fmtlib/fmt.git
revision = 9.1.0
depth = 1

[provide]
fmt = fmt_dep
        "#;

        let file = create_temp_wrap(content, "fmt");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert!(dep.name.contains("fmt"));
        assert_eq!(dep.version, "9.1.0");
        assert_eq!(dep.ecosystem, "meson-wrap");
        assert!(dep.repository_url.is_some());
        assert!(dep.repository_url.unwrap().contains("github.com"));
    }

    #[test]
    fn test_version_extraction_from_url() {
        let content = r#"
[wrap-file]
source_url = https://example.com/mylib-2.5.1.tar.gz
        "#;

        let file = create_temp_wrap(content, "mylib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "2.5.1");
    }

    #[test]
    fn test_version_extraction_from_directory() {
        let content = r#"
[wrap-file]
directory = boost-1.82.0
source_url = https://example.com/boost.tar.gz
        "#;

        let file = create_temp_wrap(content, "boost");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "1.82.0");
    }

    #[test]
    fn test_wrap_file_with_comments() {
        let content = r#"
# This is a comment
[wrap-file]
# Another comment
directory = zlib-1.2.13
source_url = https://zlib.net/zlib-1.2.13.tar.gz
        "#;

        let file = create_temp_wrap(content, "zlib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "1.2.13");
    }

    #[test]
    fn test_comments_at_various_positions() {
        let content = r#"
# Comment at start
[wrap-file]
# Comment in section
directory = zlib-1.2.13 # Inline comment after value
# Comment between lines
source_url = https://zlib.net/zlib-1.2.13.tar.gz # Another inline comment
# Comment at end
        "#;

        let file = create_temp_wrap(content, "zlib");
        let dep = parse_wrap_file(file.path()).unwrap();

        // Inline comments should be stripped from values
        assert_eq!(dep.version, "1.2.13");
    }

    #[test]
    fn test_multiple_hashes_in_same_line() {
        let content = r#"
[wrap-file]
directory = zlib-1.2.13 # First # Second # Third
source_url = https://zlib.net/zlib-1.2.13.tar.gz
        "#;

        let file = create_temp_wrap(content, "zlib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "1.2.13");
    }

    #[test]
    fn test_hash_at_end_of_line() {
        let content = r#"
[wrap-file]
directory = zlib-1.2.13#
source_url = https://zlib.net/zlib-1.2.13.tar.gz
        "#;

        let file = create_temp_wrap(content, "zlib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "1.2.13");
    }

    #[test]
    fn test_empty_values_graceful_handling() {
        let content = r#"
[wrap-file]
directory =
source_url =
        "#;

        let file = create_temp_wrap(content, "mylib");
        let dep = parse_wrap_file(file.path()).unwrap();

        // Should handle empty values gracefully
        assert!(dep.name.contains("mylib"));
        assert_eq!(dep.version, "unspecified");
    }

    #[test]
    fn test_special_characters_in_values() {
        let content = r#"
[wrap-file]
directory = my-lib_v2.0+patch
source_url = https://example.com/my-lib@v2.0.tar.gz
        "#;

        let file = create_temp_wrap(content, "my-lib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "2.0");
    }

    #[test]
    fn test_very_long_lines() {
        let long_url = "https://example.com/".to_string() + &"a".repeat(1000) + "/v1.0.tar.gz";
        let content = format!(
            r#"
[wrap-file]
directory = mylib-1.0
source_url = {}
        "#,
            long_url
        );

        let file = create_temp_wrap(&content, "mylib");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "1.0");
        assert!(dep.repository_url.is_some());
    }

    #[test]
    fn test_malformed_section_header() {
        let content = r#"
[wrap-file
directory = mylib-1.0
source_url = https://example.com/mylib-1.0.tar.gz
        "#;

        let file = create_temp_wrap(content, "mylib");
        let result = parse_wrap_file(file.path());

        // Should fail gracefully with unknown wrap type
        assert!(result.is_err());
    }

    #[test]
    fn test_version_with_leading_v() {
        let content = r#"
[wrap-git]
url = https://github.com/fmtlib/fmt.git
revision = v9.1.0
        "#;

        let file = create_temp_wrap(content, "fmt");
        let dep = parse_wrap_file(file.path()).unwrap();

        assert_eq!(dep.version, "v9.1.0");
    }
}
