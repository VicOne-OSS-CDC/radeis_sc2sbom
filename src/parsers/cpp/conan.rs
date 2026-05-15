//! Conan lock file parser (conan.lock)
//!
//! Parses Conan 2.x lock files to extract dependency information with exact versions.

use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
struct ConanLock {
    #[serde(default)]
    requires: Vec<String>,
    #[serde(default)]
    build_requires: Vec<String>,
    #[serde(default)]
    tool_requires: Vec<String>,
    #[serde(default)]
    test_requires: Vec<String>,
}

/// Parse Conan lock file (conan.lock)
///
/// Format: `name/version#revision%timestamp`
/// Example: `zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805.054666`
pub fn parse_conan_lock(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let lock: ConanLock = match serde_json::from_str(&content) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Warning: Malformed conan.lock at {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();

    // Parse runtime dependencies
    for require in &lock.requires {
        match parse_conan_reference(require, path, false) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan dependency '{}' in {}: {}",
                require,
                path.display(),
                e
            ),
        }
    }

    // Parse build dependencies (mark as dev)
    for build_req in lock.build_requires.iter().chain(lock.tool_requires.iter()) {
        match parse_conan_reference(build_req, path, true) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan build dependency '{}' in {}: {}",
                build_req,
                path.display(),
                e
            ),
        }
    }

    // Parse test dependencies (mark as dev)
    for test_req in &lock.test_requires {
        match parse_conan_reference(test_req, path, true) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan test dependency '{}' in {}: {}",
                test_req,
                path.display(),
                e
            ),
        }
    }

    Ok(dependencies)
}

/// Parse Conan reference string
///
/// Format: `name/version#revision%timestamp` or `name/version`
/// Example: `zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805`
fn parse_conan_reference(
    reference: &str,
    source_path: &Path,
    is_dev: bool,
) -> Result<Dependency, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = reference.split('/').collect();
    if parts.len() < 2 {
        return Err(format!("Invalid Conan reference format: {}", reference).into());
    }

    let name = parts[0].to_string();

    // Extract version and revision
    // Format: version#revision%timestamp
    let version_part = parts[1];
    let version = version_part
        .split('#')
        .next()
        .ok_or("Missing version")?
        .to_string();

    // Extract recipe revision (after # and before %)
    let checksum = version_part
        .split('#')
        .nth(1)
        .and_then(|r| r.split('%').next())
        .map(String::from);

    Ok(Dependency {
        name,
        version,
        ecosystem: "conan".to_string(),
        source: DependencySource::LockFile,
        is_dev,
        is_direct: true,
        source_file: Some(format_source_info("conan/lock", source_path, None, false)),
        checksum_sha256: checksum,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_lock(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn test_parse_conan_lock_basic() {
        let lock_content = r#"{
            "version": "0.5",
            "requires": [
                "zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805"
            ],
            "build_requires": [
                "cmake/3.27.0"
            ],
            "tool_requires": [],
            "test_requires": [],
            "python_requires": []
        }"#;

        let temp_file = create_temp_lock(lock_content);
        let deps = parse_conan_lock(temp_file.path()).unwrap();

        assert_eq!(deps.len(), 2);

        // Check runtime dependency
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "1.2.13");
        assert_eq!(deps[0].ecosystem, "conan");
        assert!(!deps[0].is_dev);
        assert!(deps[0]
            .checksum_sha256
            .as_ref()
            .unwrap()
            .starts_with("416618"));

        // Check build dependency
        assert_eq!(deps[1].name, "cmake");
        assert!(deps[1].is_dev);
    }

    #[test]
    fn test_parse_conan_lock_with_tool_requires() {
        let lock_content = r#"{
            "version": "0.5",
            "requires": ["zlib/1.2.13"],
            "build_requires": [],
            "tool_requires": ["cmake/3.27.0"],
            "test_requires": ["gtest/1.14.0"],
            "python_requires": []
        }"#;

        let temp_file = create_temp_lock(lock_content);
        let deps = parse_conan_lock(temp_file.path()).unwrap();

        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "zlib");
        assert!(!deps[0].is_dev);

        // tool_requires should be dev dependencies
        assert_eq!(deps[1].name, "cmake");
        assert!(deps[1].is_dev);

        // test_requires should be dev dependencies
        assert_eq!(deps[2].name, "gtest");
        assert!(deps[2].is_dev);
    }

    #[test]
    fn test_parse_conan_reference_simple() {
        let dep = parse_conan_reference("zlib/1.2.13", Path::new("conan.lock"), false).unwrap();
        assert_eq!(dep.name, "zlib");
        assert_eq!(dep.version, "1.2.13");
        assert_eq!(dep.ecosystem, "conan");
        assert!(dep.checksum_sha256.is_none());
    }

    #[test]
    fn test_parse_conan_reference_with_revision() {
        let dep = parse_conan_reference(
            "zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805",
            Path::new("conan.lock"),
            false,
        )
        .unwrap();
        assert_eq!(dep.name, "zlib");
        assert_eq!(dep.version, "1.2.13");
        assert_eq!(
            dep.checksum_sha256.as_ref().unwrap(),
            "416618fa04d433c6bd94279ed2e93638"
        );
    }

    #[test]
    fn test_parse_malformed_lock() {
        let lock_content = r#"{"version": "0.5", "requires": []}"#;
        let temp_file = create_temp_lock(lock_content);
        let deps = parse_conan_lock(temp_file.path()).unwrap();
        assert_eq!(deps.len(), 0);
    }
}
