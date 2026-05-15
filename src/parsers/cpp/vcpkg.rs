//! vcpkg.json manifest parser
//!
//! Parses vcpkg.json manifest files to extract dependencies.
//!
//! ## Supported formats:
//! - Simple string dependencies: `"zlib"`
//! - Object dependencies with version constraints: `{ "name": "openssl", "version>=": "3.0" }`
//! - Features: `{ "name": "boost", "features": ["filesystem", "system"] }`
//! - Overrides section for pinning versions
//!
//! ## Version constraint formats:
//! - `version>=`: Minimum version (e.g., "3.0")
//! - `version>`: Greater than version
//! - `version=`: Exact version
//! - `version-semver`: Semantic version
//! - `version-date`: Date-based version (e.g., "2023-01-15")
//! - `port-version`: Port revision number

use anyhow::Result;
use serde::Deserialize;
use std::fs;
use std::path::Path;

use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;

/// vcpkg.json manifest structure
#[derive(Debug, Deserialize)]
struct VcpkgManifest {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<String>,
    #[serde(default)]
    dependencies: Vec<VcpkgDep>,
    #[serde(default)]
    overrides: Vec<VcpkgOverride>,
}

/// A vcpkg dependency can be either a simple string or an object with metadata
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum VcpkgDep {
    /// Simple string format: "zlib"
    String(String),
    /// Object format with version constraints and features
    Object {
        name: String,
        #[serde(rename = "version>=")]
        version_gte: Option<String>,
        #[serde(rename = "version>")]
        version_gt: Option<String>,
        #[serde(rename = "version=")]
        version_eq: Option<String>,
        #[serde(rename = "version-semver")]
        version_semver: Option<String>,
        #[serde(rename = "version-date")]
        version_date: Option<String>,
        #[serde(rename = "port-version")]
        port_version: Option<u32>,
        #[serde(default)]
        features: Option<Vec<String>>,
        /// Platform filter (e.g., "windows", "linux")
        #[serde(default)]
        #[allow(dead_code)]
        platform: Option<String>,
        /// Whether this is a host dependency (build tool)
        #[serde(default)]
        host: Option<bool>,
    },
}

/// Version override for pinning specific package versions
#[derive(Debug, Deserialize)]
struct VcpkgOverride {
    name: String,
    version: Option<String>,
    #[serde(rename = "version-semver")]
    version_semver: Option<String>,
    #[serde(rename = "version-date")]
    version_date: Option<String>,
    #[serde(rename = "port-version")]
    port_version: Option<u32>,
}

impl VcpkgDep {
    /// Extract the dependency name
    fn name(&self) -> &str {
        match self {
            VcpkgDep::String(name) => name,
            VcpkgDep::Object { name, .. } => name,
        }
    }

    /// Extract the version string from various version fields
    /// Priority: exact > semver > date > gte > gt > port-version
    fn extract_version(&self) -> String {
        match self {
            VcpkgDep::String(_) => "unspecified".to_string(),
            VcpkgDep::Object {
                version_gte,
                version_gt,
                version_eq,
                version_semver,
                version_date,
                port_version,
                ..
            } => {
                // Priority: exact > semver > date > gte > gt > port-version
                if let Some(v) = version_eq {
                    return v.clone();
                }
                if let Some(v) = version_semver {
                    return v.clone();
                }
                if let Some(v) = version_date {
                    return v.clone();
                }
                if let Some(v) = version_gte {
                    return format!(">={}", v);
                }
                if let Some(v) = version_gt {
                    return format!(">{}", v);
                }
                if let Some(pv) = port_version {
                    return format!("port-version:{}", pv);
                }
                "unspecified".to_string()
            }
        }
    }

    /// Extract features as a formatted string for source_file metadata
    fn features_string(&self) -> Option<String> {
        match self {
            VcpkgDep::String(_) => None,
            VcpkgDep::Object { features, .. } => features.as_ref().and_then(|f| {
                if f.is_empty() {
                    None
                } else {
                    Some(format!(" [{}]", f.join(", ")))
                }
            }),
        }
    }

    /// Check if this is a host (build-time) dependency
    fn is_host(&self) -> bool {
        match self {
            VcpkgDep::String(_) => false,
            VcpkgDep::Object { host, .. } => host.unwrap_or(false),
        }
    }
}

impl VcpkgOverride {
    /// Extract the override version
    fn version(&self) -> String {
        if let Some(v) = &self.version {
            return v.clone();
        }
        if let Some(v) = &self.version_semver {
            return v.clone();
        }
        if let Some(v) = &self.version_date {
            return v.clone();
        }
        if let Some(pv) = self.port_version {
            return format!("port-version:{}", pv);
        }
        "unspecified".to_string()
    }
}

/// Parse a vcpkg.json manifest file and extract dependencies
///
/// # Arguments
/// * `path` - Path to the vcpkg.json file
///
/// # Returns
/// A vector of dependencies extracted from the manifest
///
/// # Example
/// ```ignore
/// let deps = parse_vcpkg_json(Path::new("vcpkg.json"))?;
/// for dep in deps {
///     println!("{}: {}", dep.name, dep.version);
/// }
/// ```
pub fn parse_vcpkg_json(path: &Path) -> Result<Vec<Dependency>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let manifest: VcpkgManifest = match serde_json::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: Malformed vcpkg.json at {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();

    // Build override map for version pinning
    let mut override_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for override_dep in &manifest.overrides {
        override_map.insert(override_dep.name.clone(), override_dep.version());
    }

    // Parse each dependency
    for dep in &manifest.dependencies {
        let name = dep.name().to_string();

        // Apply override if exists, otherwise use declared version
        let version = if let Some(override_version) = override_map.get(&name) {
            override_version.clone()
        } else {
            dep.extract_version()
        };

        // Build source_file with optional features suffix
        let feature_suffix = dep.features_string().unwrap_or_default();
        let source_info = format_source_info("vcpkg", path, None, false);
        let source_file = if feature_suffix.is_empty() {
            source_info
        } else {
            format!("{}{}", source_info, feature_suffix)
        };

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "vcpkg".to_string(),
            source: DependencySource::Manifest,
            is_dev: dep.is_host(), // Host dependencies are build-time only
            is_direct: true,
            source_file: Some(source_file),
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
    fn test_parse_simple_string_deps() {
        let content = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": ["zlib", "openssl", "boost"]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 3);
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "unspecified");
        assert_eq!(deps[0].ecosystem, "vcpkg");
        assert_eq!(deps[1].name, "openssl");
        assert_eq!(deps[2].name, "boost");
    }

    #[test]
    fn test_parse_object_deps_with_version_gte() {
        let content = r#"{
            "name": "test-project",
            "dependencies": [
                { "name": "openssl", "version>=": "3.0" }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "openssl");
        assert_eq!(deps[0].version, ">=3.0");
    }

    #[test]
    fn test_parse_object_deps_with_semver() {
        let content = r#"{
            "name": "test-project",
            "dependencies": [
                { "name": "boost", "version-semver": "1.82.0" }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "boost");
        assert_eq!(deps[0].version, "1.82.0");
    }

    #[test]
    fn test_parse_object_deps_with_date() {
        let content = r#"{
            "name": "test-project",
            "dependencies": [
                { "name": "fmt", "version-date": "2023-01-15" }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "fmt");
        assert_eq!(deps[0].version, "2023-01-15");
    }

    #[test]
    fn test_parse_deps_with_features() {
        let content = r#"{
            "name": "test-project",
            "dependencies": [
                { "name": "boost", "features": ["filesystem", "system"] }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "boost");
        // Features should be in source_file
        assert!(deps[0]
            .source_file
            .as_ref()
            .unwrap()
            .contains("[filesystem, system]"));
    }

    #[test]
    fn test_parse_deps_with_overrides() {
        let content = r#"{
            "name": "test-project",
            "dependencies": ["zlib"],
            "overrides": [
                { "name": "zlib", "version": "1.2.13" }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "1.2.13"); // Override applied
    }

    #[test]
    fn test_parse_host_deps() {
        let content = r#"{
            "name": "test-project",
            "dependencies": [
                { "name": "cmake", "host": true }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "cmake");
        assert!(deps[0].is_dev); // Host deps are dev deps
    }

    #[test]
    fn test_parse_empty_manifest() {
        let content = r#"{
            "name": "empty-project",
            "version": "1.0.0",
            "dependencies": []
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert!(deps.is_empty());
    }

    #[test]
    fn test_parse_mixed_deps() {
        let content = r#"{
            "name": "test-project",
            "version": "1.0.0",
            "dependencies": [
                "zlib",
                { "name": "openssl", "version>=": "3.0" },
                { "name": "boost", "version-semver": "1.82.0", "features": ["filesystem"] },
                { "name": "fmt", "version-date": "2023-01-15" }
            ],
            "overrides": [
                { "name": "zlib", "version": "1.2.13" }
            ]
        }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        assert_eq!(deps.len(), 4);

        // zlib should have override version
        assert_eq!(deps[0].name, "zlib");
        assert_eq!(deps[0].version, "1.2.13");

        // openssl with version>=
        assert_eq!(deps[1].name, "openssl");
        assert_eq!(deps[1].version, ">=3.0");

        // boost with version-semver and features
        assert_eq!(deps[2].name, "boost");
        assert_eq!(deps[2].version, "1.82.0");
        assert!(deps[2]
            .source_file
            .as_ref()
            .unwrap()
            .contains("[filesystem]"));

        // fmt with version-date
        assert_eq!(deps[3].name, "fmt");
        assert_eq!(deps[3].version, "2023-01-15");
    }

    #[test]
    fn test_parse_malformed_json() {
        let content = r#"{ invalid json }"#;

        let file = create_temp_file(content);
        let deps = parse_vcpkg_json(file.path()).unwrap();

        // Should return empty vec on error, not panic
        assert!(deps.is_empty());
    }
}
