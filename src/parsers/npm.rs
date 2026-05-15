use crate::models::{Dependency, DependencyRelationship, DependencySource, LockFileData};
use anyhow::{Context, Result};
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Metadata extracted from npm package.json
#[derive(Debug, Clone, Default)]
pub struct NpmPackageMetadata {
    pub license: Option<String>,
    pub author: Option<String>,
    pub maintainers: Vec<String>,
    pub repository_url: Option<String>,
    pub homepage_url: Option<String>,
}

/// Extracts metadata from a package.json Value
/// Load package metadata from node_modules/package_name/package.json
/// Returns None if the file doesn't exist or can't be parsed
fn load_package_metadata_from_node_modules(
    package_name: &str,
    project_root: &Path,
) -> Option<NpmPackageMetadata> {
    // Try to find the package.json in node_modules
    let package_json_path = project_root
        .join("node_modules")
        .join(package_name)
        .join("package.json");

    if !package_json_path.exists() {
        return None;
    }

    // Read and parse the package.json
    let content = fs::read_to_string(&package_json_path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    Some(extract_npm_metadata(&json))
}

/// Fetch package metadata from npm registry API
/// This is used as a fallback when node_modules doesn't exist (v0.9.0)
/// Returns None if the network request fails or package doesn't exist
#[cfg(feature = "internal")]
fn fetch_package_metadata_from_registry(
    package_name: &str,
    version: &str,
) -> Option<NpmPackageMetadata> {
    // Skip if version is unknown/unspecified or if it's a version range
    if version == "unknown"
        || version == "unspecified"
        || version.contains("^")
        || version.contains("~")
        || version.contains(">")
        || version.contains("<")
    {
        return None;
    }

    // Build npm registry URL: https://registry.npmjs.org/{package}/{version}
    let url = format!("https://registry.npmjs.org/{}/{}", package_name, version);

    // Make HTTP request with timeout (reduced to 3s to fail faster)
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;

    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().ok()?;
    Some(extract_npm_metadata(&json))
}

use rayon::prelude::*;
/// Batch fetch metadata for multiple packages in parallel using rayon
/// Returns a HashMap of package_key -> metadata
use std::collections::HashMap;

#[cfg(feature = "internal")]
fn fetch_package_metadata_batch(
    packages: &[(String, String)],
) -> HashMap<String, NpmPackageMetadata> {
    packages
        .par_iter()
        .filter_map(|(name, version)| {
            // Create key for lookup
            let key = format!("{}@{}", name, version);

            // Try to fetch metadata
            fetch_package_metadata_from_registry(name, version).map(|metadata| (key, metadata))
        })
        .collect()
}

/// Hybrid metadata loading: try node_modules first, then registry API (v0.9.0)
/// This provides best-of-both-worlds: fast local lookup with network fallback
fn load_package_metadata_hybrid(
    package_name: &str,
    version: &str,
    project_root: &Path,
    enable_network: bool,
) -> NpmPackageMetadata {
    // Try node_modules first (fastest, no network needed)
    if let Some(metadata) = load_package_metadata_from_node_modules(package_name, project_root) {
        return metadata;
    }

    // Fall back to npm registry API if enabled
    #[cfg(feature = "internal")]
    if enable_network {
        if let Some(metadata) = fetch_package_metadata_from_registry(package_name, version) {
            return metadata;
        }
    }
    #[cfg(not(feature = "internal"))]
    let _ = enable_network;

    // Return empty metadata if both methods fail
    NpmPackageMetadata::default()
}

fn extract_npm_metadata(json: &serde_json::Value) -> NpmPackageMetadata {
    // Extract license (handle string, object, and array formats)
    let license = json
        .get("license")
        .and_then(|l| {
            if l.is_string() {
                l.as_str().map(|s| s.to_string())
            } else if l.is_object() {
                l.get("type")
                    .and_then(|t| t.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            }
        })
        .or_else(|| {
            // Fallback: check deprecated "licenses" array
            json.get("licenses")
                .and_then(|l| l.as_array())
                .and_then(|arr| arr.first())
                .and_then(|l| l.get("type"))
                .and_then(|t| t.as_str())
                .map(|s| s.to_string())
        });

    // Extract author (handle string and object formats)
    let author = json.get("author").and_then(|a| {
        if a.is_string() {
            // Simple string format: "Name <email> (url)"
            a.as_str().map(|s| s.to_string())
        } else if a.is_object() {
            // Object format: extract name
            a.get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    });

    // Extract maintainers
    let maintainers = json
        .get("maintainers")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    if m.is_string() {
                        m.as_str().map(|s| s.to_string())
                    } else if m.is_object() {
                        m.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default();

    // Extract repository URL
    let repository_url = json.get("repository").and_then(|r| {
        if r.is_string() {
            r.as_str().map(|s| s.to_string())
        } else if r.is_object() {
            r.get("url").and_then(|u| u.as_str()).map(|s| s.to_string())
        } else {
            None
        }
    });

    // Extract homepage
    let homepage_url = json
        .get("homepage")
        .and_then(|h| h.as_str())
        .map(|s| s.to_string());

    NpmPackageMetadata {
        license,
        author,
        maintainers,
        repository_url,
        homepage_url,
    }
}

pub fn parse_package_json(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read package.json at {:?}", path))?;

    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut dependencies = Vec::new();

    // Detect if this package.json is in node_modules (transitive dependency)
    let is_in_node_modules = path
        .components()
        .any(|comp| comp.as_os_str() == "node_modules");
    let is_direct = !is_in_node_modules;

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the javascript/packagejson extractor from {}",
        absolute_path.display()
    );

    // Determine project root for metadata extraction
    let project_root = if is_in_node_modules {
        // Find project root by going up until we find node_modules parent
        path.ancestors()
            .find(|p| p.file_name().map(|n| n == "node_modules").unwrap_or(false))
            .and_then(|nm| nm.parent())
            .unwrap_or(path.parent().unwrap_or(path))
    } else {
        path.parent().unwrap_or(path)
    };

    // Regular dependencies
    if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
        for (name, version) in deps {
            // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
            let version_str = version.as_str().unwrap_or("unknown");
            let metadata = load_package_metadata_hybrid(name, version_str, project_root, true);

            dependencies.push(Dependency {
                name: name.clone(),
                version: version_str.to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct,
                checksum_sha256: None,
                checksum_sha512: None,
                license: metadata.license,
                author: metadata.author,
                maintainers: if metadata.maintainers.is_empty() {
                    None
                } else {
                    Some(metadata.maintainers)
                },
                repository_url: metadata.repository_url,
                homepage_url: metadata.homepage_url,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    // Development dependencies
    if let Some(dev_deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
        for (name, version) in dev_deps {
            // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
            let version_str = version.as_str().unwrap_or("unknown");
            let metadata = load_package_metadata_hybrid(name, version_str, project_root, true);

            dependencies.push(Dependency {
                name: name.clone(),
                version: version_str.to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::Manifest,
                is_dev: true,
                is_direct,
                checksum_sha256: None,
                checksum_sha512: None,
                license: metadata.license,
                author: metadata.author,
                maintainers: if metadata.maintainers.is_empty() {
                    None
                } else {
                    Some(metadata.maintainers)
                },
                repository_url: metadata.repository_url,
                homepage_url: metadata.homepage_url,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    // Peer dependencies
    if let Some(peer_deps) = json.get("peerDependencies").and_then(|v| v.as_object()) {
        for (name, version) in peer_deps {
            // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
            let version_str = version.as_str().unwrap_or("unknown");
            let metadata = load_package_metadata_hybrid(name, version_str, project_root, true);

            dependencies.push(Dependency {
                name: name.clone(),
                version: version_str.to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct,
                checksum_sha256: None,
                checksum_sha512: None,
                license: metadata.license,
                author: metadata.author,
                maintainers: if metadata.maintainers.is_empty() {
                    None
                } else {
                    Some(metadata.maintainers)
                },
                repository_url: metadata.repository_url,
                homepage_url: metadata.homepage_url,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    // Optional dependencies
    if let Some(opt_deps) = json.get("optionalDependencies").and_then(|v| v.as_object()) {
        for (name, version) in opt_deps {
            // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
            let version_str = version.as_str().unwrap_or("unknown");
            let metadata = load_package_metadata_hybrid(name, version_str, project_root, true);

            dependencies.push(Dependency {
                name: name.clone(),
                version: version_str.to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct,
                checksum_sha256: None,
                checksum_sha512: None,
                license: metadata.license,
                author: metadata.author,
                maintainers: if metadata.maintainers.is_empty() {
                    None
                } else {
                    Some(metadata.maintainers)
                },
                repository_url: metadata.repository_url,
                homepage_url: metadata.homepage_url,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    Ok(dependencies)
}

pub fn parse_package_lock_json_with_relationships(path: &Path) -> Result<LockFileData> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read package-lock.json at {:?}", path))?;

    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut dependencies = Vec::new();
    let mut relationships = Vec::new();
    let mut seen_packages: HashSet<String> = HashSet::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the javascript/packagelockjson extractor from {}",
        absolute_path.display()
    );

    // v0.9.0: Determine project root for node_modules metadata lookup
    let project_root = path.parent().unwrap_or(path);

    // Parse packages from lockfileVersion 2 or 3
    if let Some(packages) = json.get("packages").and_then(|v| v.as_object()) {
        // v0.9.0 Performance: Two-pass approach for metadata extraction
        // Pass 1: Collect package info and try node_modules (fast local lookup)
        let mut package_info_list = Vec::new();
        #[cfg(feature = "internal")]
        let mut packages_needing_api = Vec::new();

        for (pkg_path, pkg_info) in packages {
            // Skip root package (empty string or "")
            if pkg_path.is_empty() {
                continue;
            }

            // Extract package name from the path (handle nested node_modules)
            let name = if let Some(idx) = pkg_path.rfind("node_modules/") {
                &pkg_path[idx + "node_modules/".len()..]
            } else {
                pkg_path.as_str()
            };
            let version = pkg_info
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            // Deduplicate by name@version to handle multiple versions
            let package_id = format!("{}@{}", name, version);
            if seen_packages.contains(&package_id) {
                continue; // Skip duplicate version
            }
            seen_packages.insert(package_id.clone());

            let is_dev = pkg_info
                .get("dev")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            // Determine if direct dependency
            let depth = pkg_path.matches("node_modules").count();
            let is_direct = depth == 1;

            // v0.9.0: Extract integrity hash (SHA-512 or SHA-256)
            let (checksum_sha256, checksum_sha512) = extract_npm_checksum(pkg_info);

            // Try to get metadata from node_modules first (fast)
            let metadata = load_package_metadata_from_node_modules(name, project_root);

            #[cfg(feature = "internal")]
            if metadata.is_none() {
                // No local metadata, will need API fetch
                packages_needing_api.push((name.to_string(), version.clone()));
            }

            package_info_list.push((
                name.to_string(),
                version.clone(),
                is_dev,
                is_direct,
                checksum_sha256,
                checksum_sha512,
                metadata,
                pkg_info.clone(),
            ));
        }

        // Pass 2: Batch fetch from API in parallel for packages without local metadata
        #[cfg(feature = "internal")]
        let api_metadata = if !packages_needing_api.is_empty() {
            eprintln!(
                "Fetching metadata for {} packages from npm registry (parallel)...",
                packages_needing_api.len()
            );
            fetch_package_metadata_batch(&packages_needing_api)
        } else {
            HashMap::new()
        };
        #[cfg(not(feature = "internal"))]
        let api_metadata: HashMap<String, NpmPackageMetadata> = HashMap::new();

        // Pass 3: Create dependencies with combined metadata
        for (
            name,
            version,
            is_dev,
            is_direct,
            checksum_sha256,
            checksum_sha512,
            local_metadata,
            pkg_info,
        ) in package_info_list
        {
            let package_key = format!("{}@{}", name, version);

            // Use local metadata if available, otherwise use API metadata
            let metadata = local_metadata
                .or_else(|| api_metadata.get(&package_key).cloned())
                .unwrap_or_default();

            dependencies.push(Dependency {
                name: name.clone(),
                version: version.clone(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev,
                is_direct,
                license: metadata.license,
                author: metadata.author,
                maintainers: if metadata.maintainers.is_empty() {
                    None
                } else {
                    Some(metadata.maintainers)
                },
                repository_url: metadata.repository_url,
                homepage_url: metadata.homepage_url,
                source_file: Some(source_info.clone()),
                checksum_sha256,
                checksum_sha512,
                ..Default::default()
            });

            // Extract dependencies field to build relationships
            if let Some(deps_obj) = pkg_info.get("dependencies").and_then(|v| v.as_object()) {
                let parent_id = format!("{}@{}", name, version);
                let child_names: Vec<String> = deps_obj.keys().map(|k| k.to_string()).collect();

                if !child_names.is_empty() {
                    relationships.push(DependencyRelationship {
                        parent_id,
                        child_names,
                    });
                }
            }
        }
    }
    // Fallback for lockfileVersion 1
    else if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
        extract_npm_dependencies_recursive(deps, &mut dependencies, true, project_root);
        // Note: lockfileVersion 1 doesn't provide easy relationship extraction.
        // Emit a warning so users know hierarchical dependency trees are unavailable.
        eprintln!(
            "Warning: package-lock.json at {:?} uses lockfileVersion 1; \
            dependency relationships (hierarchical trees) will not be available for this project.",
            path
        );
    }

    Ok(LockFileData {
        dependencies,
        relationships,
    })
}

pub fn extract_npm_dependencies_recursive(
    deps: &serde_json::Map<String, serde_json::Value>,
    result: &mut Vec<Dependency>,
    is_direct: bool,
    project_root: &Path,
) {
    // Note: source_file not tracked for lockfileVersion 1 (legacy format)
    // Modern package-lock.json uses parse_package_lock_json_with_relationships instead
    for (name, info) in deps {
        let version = info
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();
        let is_dev = info.get("dev").and_then(|v| v.as_bool()).unwrap_or(false);

        // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
        let metadata = load_package_metadata_hybrid(name, &version, project_root, true);

        result.push(Dependency {
            name: name.clone(),
            version,
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev,
            is_direct,
            checksum_sha256: None,
            checksum_sha512: None,
            license: metadata.license,
            author: metadata.author,
            maintainers: if metadata.maintainers.is_empty() {
                None
            } else {
                Some(metadata.maintainers)
            },
            repository_url: metadata.repository_url,
            homepage_url: metadata.homepage_url,
            source_file: None, // Legacy format doesn't track source
            ..Default::default()
        });

        // Recursively process nested dependencies
        if let Some(nested) = info.get("dependencies").and_then(|v| v.as_object()) {
            extract_npm_dependencies_recursive(nested, result, false, project_root);
        }
    }
}

pub fn parse_yarn_lock(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read yarn.lock at {:?}", path))?;

    let mut dependencies = Vec::new();
    let mut current_package: Option<String> = None;
    let mut current_version: Option<String> = None;

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the javascript/yarnlock extractor from {}",
        absolute_path.display()
    );

    // v0.9.0: Determine project root for node_modules metadata lookup
    let project_root = path.parent().unwrap_or(path);

    for line in content.lines() {
        let line = line.trim();

        // Package declaration line (e.g., "package@^1.0.0:", "package@npm:...", "\"pkg@^1\", \"pkg@^2\":")
        if !line.is_empty() && !line.starts_with('#') && line.ends_with(':') {
            // Save previous dependency if we have both name and version
            if let (Some(ref pkg), Some(ref ver)) = (&current_package, &current_version) {
                // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
                let metadata = load_package_metadata_hybrid(pkg, ver, project_root, true);

                dependencies.push(Dependency {
                    name: pkg.clone(),
                    version: ver.clone(),
                    ecosystem: "npm".to_string(),
                    source: DependencySource::LockFile,
                    is_dev: false,
                    is_direct: true, // Yarn lock doesn't distinguish well
                    checksum_sha256: None,
                    checksum_sha512: None,
                    license: metadata.license,
                    author: metadata.author,
                    maintainers: if metadata.maintainers.is_empty() {
                        None
                    } else {
                        Some(metadata.maintainers)
                    },
                    repository_url: metadata.repository_url,
                    homepage_url: metadata.homepage_url,
                    source_file: Some(source_info.clone()),
                    ..Default::default()
                });
                current_package = None;
                current_version = None;
            }

            // Extract package selectors from yarn.lock format (may be multiple, separated by commas)
            let pkg_spec = line.trim_end_matches(':').trim();

            // Take the first selector (e.g., from `"react@^16.13.1", react@^16.14.0` take `react@^16.13.1`)
            if let Some(first_selector) = pkg_spec.split(',').next() {
                let selector = first_selector.trim().trim_matches('"').trim_matches('\'');

                // Handle formats like:
                //   "package@version"
                //   "@scope/package@version"
                //   "package@npm:version"
                let name = if let Some(stripped) = selector.strip_prefix('@') {
                    // Scoped package: find the second '@' which separates name from version
                    if let Some(second_at_rel) = stripped.find('@') {
                        &selector[..1 + second_at_rel]
                    } else {
                        // Fallback: no second '@', treat whole selector as name
                        selector
                    }
                } else {
                    // Unscoped package: first '@' separates name from version
                    if let Some(first_at) = selector.find('@') {
                        &selector[..first_at]
                    } else {
                        // Fallback: no '@', treat whole selector as name
                        selector
                    }
                };

                if !name.is_empty() {
                    current_package = Some(name.to_string());
                }
            }
        } else if line.starts_with("version ") {
            // Extract version
            let version = line
                .trim_start_matches("version ")
                .trim_matches('"')
                .to_string();
            current_version = Some(version);
        }
    }

    // Don't forget the last dependency
    if let (Some(ref pkg), Some(ref ver)) = (&current_package, &current_version) {
        // v0.9.0: Hybrid metadata extraction (node_modules + npm registry API fallback)
        let metadata = load_package_metadata_hybrid(pkg, ver, project_root, true);

        dependencies.push(Dependency {
            name: pkg.clone(),
            version: ver.clone(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true, // Yarn lock doesn't distinguish well
            checksum_sha256: None,
            checksum_sha512: None,
            license: metadata.license,
            author: metadata.author,
            maintainers: if metadata.maintainers.is_empty() {
                None
            } else {
                Some(metadata.maintainers)
            },
            repository_url: metadata.repository_url,
            homepage_url: metadata.homepage_url,
            source_file: Some(source_info.clone()),
            ..Default::default()
        });
    }

    Ok(dependencies)
}

/// Extract checksum from npm package integrity field (v0.9.0)
/// npm uses subresource integrity format: "sha512-base64hash" or "sha256-base64hash"
fn extract_npm_checksum(pkg_info: &serde_json::Value) -> (Option<String>, Option<String>) {
    let integrity = pkg_info.get("integrity").and_then(|v| v.as_str());

    if let Some(integrity_str) = integrity {
        // Parse integrity format: "sha512-base64hash" or "sha256-base64hash"
        if let Some((algo, hash)) = integrity_str.split_once('-') {
            match algo {
                "sha512" => return (None, Some(hash.to_string())),
                "sha256" => return (Some(hash.to_string()), None),
                _ => {}
            }
        }
    }

    (None, None)
}
