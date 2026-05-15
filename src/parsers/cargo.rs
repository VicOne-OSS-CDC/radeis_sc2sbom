use crate::models::{Dependency, DependencyRelationship, DependencySource, LockFileData};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata extracted from Cargo.toml [package] section
#[derive(Debug, Clone, Default)]
pub struct CargoPackageMetadata {
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub homepage: Option<String>,
}

/// Fetch package metadata from crates.io API
/// Returns None if the network request fails or package doesn't exist (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_cargo_metadata_from_registry(
    package_name: &str,
    version: &str,
) -> Option<CargoPackageMetadata> {
    // Skip if version is unknown/unspecified
    if version == "unknown" || version == "unspecified" {
        return None;
    }

    // Build crates.io API URL
    let url = format!(
        "https://crates.io/api/v1/crates/{}/{}",
        package_name, version
    );

    // Make HTTP request with timeout
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .user_agent("radeis_sc2sbom/0.9.0") // crates.io requires user agent
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;

    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().ok()?;

    // Extract from crates.io API response format
    let version_data = json.get("version")?;

    let license = version_data
        .get("license")
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());

    // Note: crates.io deprecated authors field in favor of crate owners
    // Try to get it anyway for backward compatibility
    let authors = version_data
        .get("authors")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let repository = version_data
        .get("repository")
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    let homepage = version_data
        .get("homepage")
        .and_then(|h| h.as_str())
        .map(|s| s.to_string());

    Some(CargoPackageMetadata {
        license,
        authors,
        repository,
        homepage,
    })
}

/// Hybrid metadata loading for Cargo: local Cargo.toml first, then crates.io API (v0.9.0)
#[cfg(feature = "internal")]
fn load_cargo_metadata_hybrid(
    package_name: &str,
    version: &str,
    enable_network: bool,
) -> CargoPackageMetadata {
    // For Cargo, we don't have local package metadata like npm's node_modules
    // Go directly to registry API if enabled
    if enable_network {
        if let Some(metadata) = fetch_cargo_metadata_from_registry(package_name, version) {
            return metadata;
        }
    }

    CargoPackageMetadata::default()
}

/// Batch fetch Cargo metadata for multiple packages in parallel using rayon (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_cargo_metadata_batch(
    packages: &[(String, String)],
) -> HashMap<String, CargoPackageMetadata> {
    packages
        .par_iter()
        .filter_map(|(name, version)| {
            let key = format!("{}@{}", name, version);
            fetch_cargo_metadata_from_registry(name, version).map(|metadata| (key, metadata))
        })
        .collect()
}

pub fn parse_cargo_toml(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read Cargo.toml at {:?}", path))?;

    let toml_value: toml::Value = toml::from_str(&content)?;
    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the rust/cargo extractor from {}",
        absolute_path.display()
    );

    if let Some(deps) = toml_value.get("dependencies").and_then(|v| v.as_table()) {
        for (name, value) in deps {
            let version = match value {
                toml::Value::String(v) => v.clone(),
                toml::Value::Table(t) => t
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string(),
                _ => "unknown".to_string(),
            };

            // v0.9.0: Hybrid metadata extraction (crates.io API, internal builds only)
            #[cfg(feature = "internal")]
            let metadata = load_cargo_metadata_hybrid(name, &version, true);
            #[cfg(not(feature = "internal"))]
            let metadata = CargoPackageMetadata::default();
            let author = metadata.authors.first().map(|s| s.to_string());

            dependencies.push(Dependency {
                name: name.clone(),
                version,
                ecosystem: "cargo".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct: true,
                license: metadata.license,
                author,
                maintainers: if metadata.authors.len() > 1 {
                    Some(metadata.authors[1..].to_vec())
                } else {
                    None
                },
                repository_url: metadata.repository,
                homepage_url: metadata.homepage,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    Ok(dependencies)
}

/// Parses Cargo.lock with relationship data for hierarchical dependency trees.
/// Extracts parent-child relationships from the dependencies array in each package entry.
///
/// # Returns
/// LockFileData containing:
/// - dependencies: All packages from Cargo.lock
/// - relationships: Parent→child mappings from dependencies arrays
pub fn parse_cargo_lock_with_relationships(path: &Path) -> Result<LockFileData> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read Cargo.lock at {:?}", path))?;

    let toml_value: toml::Value = toml::from_str(&content)?;
    let mut dependencies = Vec::new();
    let mut relationships = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the rust/cargolock extractor from {}",
        absolute_path.display()
    );

    if let Some(packages) = toml_value.get("package").and_then(|v| v.as_array()) {
        // PASS 1: Collect package info (v0.9.0 parallel optimization)
        let mut package_info_list = Vec::new();
        #[cfg(feature = "internal")]
        let mut packages_needing_api = Vec::new();

        for package in packages {
            if let Some(name) = package.get("name").and_then(|v| v.as_str()) {
                let version = package
                    .get("version")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Collect for batch API fetch
                #[cfg(feature = "internal")]
                packages_needing_api.push((name.to_string(), version.clone()));

                package_info_list.push((name.to_string(), version, package.clone()));
            }
        }

        // PASS 2: Parallel batch fetch from crates.io API (v0.9.0)
        #[cfg(feature = "internal")]
        let api_metadata = if !packages_needing_api.is_empty() {
            eprintln!(
                "Fetching metadata for {} Cargo packages from crates.io (parallel)...",
                packages_needing_api.len()
            );
            fetch_cargo_metadata_batch(&packages_needing_api)
        } else {
            HashMap::new()
        };
        #[cfg(not(feature = "internal"))]
        let api_metadata: HashMap<String, CargoPackageMetadata> = HashMap::new();

        // PASS 3: Create dependencies with metadata
        for (name, version, package) in package_info_list {
            let package_key = format!("{}@{}", name, version);
            let metadata = api_metadata.get(&package_key).cloned().unwrap_or_default();

            let author = metadata.authors.first().map(|s| s.to_string());

            dependencies.push(Dependency {
                name: name.clone(),
                version: version.clone(),
                ecosystem: "cargo".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true, // Will be corrected by DependencyGraph::build_from_deps_with_relationships()
                license: metadata.license,
                author,
                maintainers: if metadata.authors.len() > 1 {
                    Some(metadata.authors[1..].to_vec())
                } else {
                    None
                },
                repository_url: metadata.repository,
                homepage_url: metadata.homepage,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });

            // Extract dependencies array to build relationships
            if let Some(deps_array) = package.get("dependencies").and_then(|v| v.as_array()) {
                let parent_id = format!("{}@{}", name, version);
                let child_names: Vec<String> = deps_array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|dep_str| {
                        // Cargo.lock dependencies format: Just package names (e.g., "tokio", "serde")
                        // The version is not included in the dependencies array, only the name.
                        // The name_to_id HashMap (built during graph construction) will resolve
                        // "tokio" to the full ID "tokio@1.35.0" for graph lookups.
                        dep_str.to_string()
                    })
                    .collect();

                if !child_names.is_empty() {
                    relationships.push(DependencyRelationship {
                        parent_id,
                        child_names,
                    });
                }
            }
        }
    }

    Ok(LockFileData {
        dependencies,
        relationships,
    })
}
