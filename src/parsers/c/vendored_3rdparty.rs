//! Vendored 3rd-party Library Scanner (v1.0.6)
//!
//! Detects vendored libraries in `3rd_party/` or `3rdparty/` directories
//! by scanning for CMakeLists.txt files with VERSION declarations.
//!
//! # Detection Strategy
//!
//! 1. Find all `3rd_party/` or `3rdparty/` directories recursively
//! 2. For each subdirectory, check for CMakeLists.txt
//! 3. Extract VERSION from CMakeLists.txt (PROJECT VERSION syntax)
//! 4. Library name is the directory name
//!
//! # Examples
//!
//! - `lib/quark/3rd_party/rapidjson/CMakeLists.txt` with `set(LIB_MAJOR_VERSION "1")`
//!   → Detected as `rapidjson @ 1.1.0`
//!
//! - `lib/atom_utils/3rd_party/paho-embedded-c/library.properties` with `version=1.0.0`
//!   → Detected as `paho.mqtt.embedded-c @ 1.0.0`

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use crate::util::warn_on_walkdir_err;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

lazy_static::lazy_static! {
    /// Regex for CMake PROJECT VERSION syntax
    /// Examples:
    /// - project(rapidjson VERSION 1.1.0)
    /// - PROJECT(minizip VERSION 1.2.0 LANGUAGES C)
    /// - project(paho-embedded-c VERSION 1.0.0)
    /// - project("my-lib" VERSION 2.0.0)
    static ref CMAKE_PROJECT_VERSION: Regex =
        Regex::new(r#"(?i)project\s*\(\s*(?:"[^"]+"|[^\s)]+)\s+VERSION\s+([0-9]+(?:\.[0-9]+){0,3})"#).unwrap();

    /// Regex for CMake set() version patterns - suffix style
    /// Examples: set(LIB_MAJOR_VERSION "1")
    static ref CMAKE_SET_VERSION_SUFFIX: Regex =
        Regex::new(r#"set\s*\(\s*\w+_(MAJOR|MINOR|PATCH)_VERSION\s+"?([0-9]+)"?\s*\)"#).unwrap();

    /// Regex for CMake set() version patterns - infix style
    /// Examples: set(PAHO_VERSION_MAJOR 1)
    static ref CMAKE_SET_VERSION_INFIX: Regex =
        Regex::new(r#"set\s*\(\s*\w+_VERSION_(MAJOR|MINOR|PATCH)\s+"?([0-9]+)"?\s*\)"#).unwrap();

    /// Regex for library.properties version (Arduino/embedded libraries)
    /// Example: version=1.0.0
    static ref PROPERTIES_VERSION: Regex =
        Regex::new(r"(?m)^version\s*=\s*([0-9]+(?:\.[0-9]+){0,3})").unwrap();
}

/// Scan repository for vendored 3rd-party libraries
///
/// # Arguments
/// * `repo_root` - Root directory of the repository
/// * `excludes` - Directory name patterns to skip (same list as `--exclude` CLI flag)
///
/// # Returns
/// * Vector of detected vendored library dependencies
pub fn scan_vendored_3rdparty(
    repo_root: &Path,
    excludes: &[String],
) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let mut dependencies = Vec::new();

    // Common directories that should never be traversed (build artifacts, package caches, VCS)
    const SKIP_DIRS: &[&str] = &[
        "node_modules",
        "target",
        "build",
        "dist",
        ".git",
        ".svn",
        ".hg",
        "__pycache__",
        ".tox",
        ".mypy_cache",
        ".pytest_cache",
    ];

    // Find all 3rd_party or 3rdparty directories
    for entry in WalkDir::new(repo_root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            if let Some(name) = e.path().file_name().and_then(|n| n.to_str()) {
                // Respect --exclude patterns
                if excludes.iter().any(|pattern| name == pattern) {
                    return false;
                }
                // Skip common heavy directories (build artifacts, VCS, package caches)
                if SKIP_DIRS.contains(&name) {
                    return false;
                }
            }
            true
        })
        .filter_map(warn_on_walkdir_err)
    {
        let path = entry.path();

        // Check if this is a 3rd_party directory
        if !path.is_dir() {
            continue;
        }

        let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if dir_name != "3rd_party" && dir_name != "3rdparty" {
            continue;
        }

        // Scan immediate subdirectories of 3rd_party
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.filter_map(|e| e.ok()) {
                let lib_path = entry.path();
                if lib_path.is_dir() {
                    if let Some(dep) = detect_vendored_library(&lib_path) {
                        dependencies.push(dep);
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

/// Detect a single vendored library in a directory
///
/// # Detection Methods (in order of preference)
/// 1. CMakeLists.txt with PROJECT(... VERSION ...)
/// 2. CMakeLists.txt with set(LIB_*_VERSION ...)
/// 3. library.properties with version=...
/// 4. Fallback: Detect as "unspecified" if directory looks like a library
///
/// # Arguments
/// * `lib_dir` - Directory containing the vendored library
///
/// # Returns
/// * Some(Dependency) if library detected, None if directory should be ignored
fn detect_vendored_library(lib_dir: &Path) -> Option<Dependency> {
    let lib_name = lib_dir.file_name()?.to_str()?.to_string();

    // Skip non-library directories
    if should_skip_directory(&lib_name) {
        return None;
    }

    // Normalize library name for known patterns
    let normalized_name = normalize_vendored_lib_name(&lib_name);

    // Try CMakeLists.txt first
    let cmake_path = lib_dir.join("CMakeLists.txt");
    if cmake_path.exists() {
        if let Ok(content) = fs::read_to_string(&cmake_path) {
            // Try PROJECT VERSION syntax
            if let Some(cap) = CMAKE_PROJECT_VERSION.captures(&content) {
                let version = cap[1].to_string();
                return Some(create_dependency(normalized_name, version, cmake_path));
            }

            // Try set(LIB_*_VERSION) syntax
            if let Some(version) = extract_lib_version_from_cmake(&content) {
                return Some(create_dependency(normalized_name, version, cmake_path));
            }
            // CMakeLists.txt exists but no version - still a library
            return Some(create_dependency(
                normalized_name,
                "unspecified".to_string(),
                cmake_path,
            ));
        }
    }

    // Try library.properties (Arduino/embedded style)
    let properties_path = lib_dir.join("library.properties");
    if properties_path.exists() {
        if let Ok(content) = fs::read_to_string(&properties_path) {
            if let Some(cap) = PROPERTIES_VERSION.captures(&content) {
                let version = cap[1].to_string();
                return Some(create_dependency(normalized_name, version, properties_path));
            }
        }
    }

    // Check if directory contains typical library files (headers, sources)
    if has_library_files(lib_dir) {
        return Some(create_dependency(
            normalized_name,
            "unspecified".to_string(),
            lib_dir.to_path_buf(),
        ));
    }

    None
}

/// Extract version from CMake set(*_VERSION_*) variables
/// Handles patterns like:
/// - set(PAHO_VERSION_MAJOR 1) - infix pattern
/// - set(LIB_MAJOR_VERSION "1") - suffix pattern
fn extract_lib_version_from_cmake(content: &str) -> Option<String> {
    let mut major = None;
    let mut minor = None;
    let mut patch = None;

    // Try suffix pattern first: set(LIB_MAJOR_VERSION "1")
    for cap in CMAKE_SET_VERSION_SUFFIX.captures_iter(content) {
        let var_type = &cap[1];
        let value = &cap[2];

        match var_type {
            "MAJOR" => major = Some(value.to_string()),
            "MINOR" => minor = Some(value.to_string()),
            "PATCH" => patch = Some(value.to_string()),
            _ => {}
        }
    }

    // Try infix pattern if suffix didn't work: set(PAHO_VERSION_MAJOR 1)
    if major.is_none() {
        for cap in CMAKE_SET_VERSION_INFIX.captures_iter(content) {
            let var_type = &cap[1];
            let value = &cap[2];

            match var_type {
                "MAJOR" => major = Some(value.to_string()),
                "MINOR" => minor = Some(value.to_string()),
                "PATCH" => patch = Some(value.to_string()),
                _ => {}
            }
        }
    }

    // Construct version string
    match (major, minor, patch) {
        (Some(maj), Some(min), Some(pat)) => Some(format!("{}.{}.{}", maj, min, pat)),
        (Some(maj), Some(min), None) => Some(format!("{}.{}", maj, min)),
        (Some(maj), None, None) => Some(maj),
        _ => None,
    }
}

/// Check if directory should be skipped (not a vendored library we want to track)
fn should_skip_directory(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    // Skip test frameworks and benchmarks
    if matches!(
        name_lower.as_str(),
        "test"
            | "tests"
            | "testing"
            | "googletest"
            | "gtest"
            | "catch2"
            | "doctest"
            | "benchmark"
            | "benchmarks"
            | "example"
            | "examples"
            | "samples"
            | "doc"
            | "docs"
            | "documentation"
            | "build"
            | ".git"
            | ".svn"
    ) {
        return true;
    }

    // Skip test/benchmark directories by word-boundary patterns to avoid false positives
    // (e.g., "latest", "contest", "testament" must not be skipped)
    let is_test_dir = name_lower == "test"
        || name_lower == "tests"
        || name_lower == "testing"
        || name_lower.starts_with("test_")
        || name_lower.starts_with("test-")
        || name_lower.starts_with("tests_")
        || name_lower.starts_with("tests-")
        || name_lower.starts_with("testing_")
        || name_lower.starts_with("testing-")
        || name_lower.ends_with("_test")
        || name_lower.ends_with("-test")
        || name_lower.ends_with("_tests")
        || name_lower.ends_with("-tests");
    let is_bench_dir = name_lower == "benchmark"
        || name_lower == "benchmarks"
        || name_lower.starts_with("benchmark_")
        || name_lower.starts_with("benchmark-")
        || name_lower.ends_with("_benchmark")
        || name_lower.ends_with("-benchmark")
        || name_lower.ends_with("_benchmarks")
        || name_lower.ends_with("-benchmarks");
    if is_test_dir || is_bench_dir {
        return true;
    }

    false
}

/// Check if directory contains library source files
fn has_library_files(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                // Check for C/C++ source or header files
                if matches!(ext, "c" | "cpp" | "cxx" | "cc" | "h" | "hpp" | "hxx") {
                    return true;
                }
            }
        }
    }
    false
}

/// Normalize vendored library names to match expected SBOM names
fn normalize_vendored_lib_name(name: &str) -> String {
    match name {
        "paho-embedded-c" | "paho.mqtt.embedded-c" | "paho_mqtt_embedded_c" => {
            "paho.mqtt.embedded-c".to_string()
        }
        "minizip" | "minizip-ng" => "minizip".to_string(),
        _ => name.to_string(),
    }
}

/// Create Dependency object for vendored library
fn create_dependency(name: String, version: String, source_path: PathBuf) -> Dependency {
    Dependency {
        name,
        version,
        ecosystem: "VENDORED".to_string(),
        source: DependencySource::Manifest,
        is_dev: false,
        is_direct: true,
        source_file: Some(format_source_info(
            "vendored-3rdparty",
            &source_path,
            None,
            false,
        )),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmake_project_version() {
        let content = "project(rapidjson VERSION 1.1.0 LANGUAGES CXX)";
        let cap = CMAKE_PROJECT_VERSION.captures(content).unwrap();
        assert_eq!(&cap[1], "1.1.0");
    }

    #[test]
    fn test_cmake_set_version() {
        let content = r#"
set(LIB_MAJOR_VERSION "1")
set(LIB_MINOR_VERSION "1")
set(LIB_PATCH_VERSION "0")
        "#;
        let version = extract_lib_version_from_cmake(content).unwrap();
        assert_eq!(version, "1.1.0");
    }

    #[test]
    fn test_properties_version() {
        let content = "version=1.0.0\nname=paho";
        let cap = PROPERTIES_VERSION.captures(content).unwrap();
        assert_eq!(&cap[1], "1.0.0");
    }

    #[test]
    fn test_normalize_paho_name() {
        assert_eq!(
            normalize_vendored_lib_name("paho-embedded-c"),
            "paho.mqtt.embedded-c"
        );
    }
}
