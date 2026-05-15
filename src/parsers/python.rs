use crate::models::{Dependency, DependencyRelationship, DependencySource, LockFileData};
use anyhow::{Context, Result};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::process::Command;

lazy_static::lazy_static! {
    /// Regex for parsing Python dependency specifications
    /// Matches: package, package==1.0, package>=1.0, package[extra]>=1.0, etc.
    static ref PYTHON_DEPENDENCY_REGEX: Regex =
        Regex::new(r"^([a-zA-Z0-9_\-\.]+)(\[[^\]]+\])?(.*?)$").unwrap();
}

/// Extract version from Python version specifier
/// Only strips exact pin operators (== or =) to preserve constraint semantics
/// For non-exact specs, returns "unspecified" since the actual version is unknown
/// Example: "==2.6.0" -> "2.6.0", ">=1.0" -> "unspecified", "*" -> "unspecified"
fn strip_version_operator(version_spec: &str) -> String {
    let trimmed = version_spec.trim();

    // Handle empty or unspecified
    if trimmed.is_empty() || trimmed == "*" {
        return "unspecified".to_string();
    }

    // Only strip exact pin operators (== or =)
    // For other operators (>=, <=, ~=, !=, >, <), return unspecified
    // since they represent constraints, not actual versions
    if let Some(version) = trimmed.strip_prefix("==") {
        return version.trim().to_string();
    }
    if let Some(version) = trimmed.strip_prefix('=') {
        // Handle single = (less common, but valid)
        if !version.starts_with('=') {
            return version.trim().to_string();
        }
    }

    // For non-exact version specs (>=, <=, ~=, !=, >, <, or compound specs),
    // return unspecified since we don't know the actual installed version
    "unspecified".to_string()
}

pub fn parse_requirements_txt(path: &Path) -> Result<Vec<Dependency>> {
    let content = fs::read_to_string(path)
        .context(format!("Failed to read requirements.txt at {:?}", path))?;

    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/requirements extractor from {}",
        absolute_path.display()
    );

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Remove inline comments
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        if let Some(caps) = PYTHON_DEPENDENCY_REGEX.captures(line) {
            let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let extras = caps.get(2).map_or("", |m| m.as_str());
            let version_spec = caps.get(3).map_or("", |m| m.as_str()).trim();

            // Strip version operators (==, >=, <=, ~=, !=, >, <) to get clean version number
            let version = if version_spec.is_empty() {
                "unspecified".to_string()
            } else {
                strip_version_operator(version_spec)
            };

            // v0.9.0: Hybrid metadata extraction (PyPI API) - do this before moving name
            let metadata = load_python_metadata_hybrid(&name, &version, true);

            // Combine name with extras if present
            let full_name = if !extras.is_empty() {
                format!("{}{}", name, extras)
            } else {
                name
            };

            dependencies.push(Dependency {
                name: full_name,
                version,
                ecosystem: "pip".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct: true,
                license: metadata.license,
                author: metadata.author,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        } else {
            // Fallback: store the entire line as name if regex doesn't match
            // v0.9.0: Try to extract metadata even for fallback cases
            let metadata = load_python_metadata_hybrid(line, "unspecified", true);

            dependencies.push(Dependency {
                name: line.to_string(),
                version: "unspecified".to_string(),
                ecosystem: "pip".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct: true,
                license: metadata.license,
                author: metadata.author,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });
        }
    }

    Ok(dependencies)
}

/// Resolve transitive dependencies for a requirements.txt file using pip
///
/// This function uses `pip install --dry-run --report` to resolve the full
/// dependency tree including transitive dependencies.
///
/// # Arguments
/// * `path` - Path to the requirements.txt file
/// * `enable_network` - Whether to enable network requests for resolution
///
/// # Returns
/// * LockFileData with direct and transitive dependencies, plus relationships
pub fn resolve_requirements_txt_transitive(
    path: &Path,
    enable_network: bool,
) -> Result<LockFileData> {
    // First parse direct dependencies
    let direct_deps = parse_requirements_txt(path)?;
    let direct_names: HashSet<String> = direct_deps
        .iter()
        .map(|d| normalize_python_package_name(&d.name))
        .collect();

    if !enable_network {
        // Without network, we can only return direct dependencies
        return Ok(LockFileData {
            dependencies: direct_deps,
            relationships: Vec::new(),
        });
    }

    // Check if pip is available
    let pip_check = Command::new("pip").args(["--version"]).output();

    if pip_check.is_err() || !pip_check.unwrap().status.success() {
        eprintln!("Warning: pip not available, skipping transitive dependency resolution");
        return Ok(LockFileData {
            dependencies: direct_deps,
            relationships: Vec::new(),
        });
    }

    // Use the original requirements file directly to preserve constraint semantics
    // This avoids reconstructing version specs from normalized dependencies,
    // which could change semantics (e.g., ">=2,<3" becoming "==2")
    let requirements_path = path.to_str().unwrap_or("");

    // Run pip install --dry-run --report to resolve dependencies
    // The --report flag (pip 22.2+) outputs JSON with resolved packages
    // Use the original requirements.txt to preserve exact version constraints
    let output = Command::new("pip")
        .args([
            "install",
            "--dry-run",
            "--report",
            "-",
            "--quiet",
            "-r",
            requirements_path,
        ])
        .output();

    let output = match output {
        Ok(o) => o,
        Err(e) => {
            eprintln!(
                "Warning: Failed to run pip for transitive resolution: {}",
                e
            );
            return Ok(LockFileData {
                dependencies: direct_deps,
                relationships: Vec::new(),
            });
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: pip resolution failed: {}", stderr);
        return Ok(LockFileData {
            dependencies: direct_deps,
            relationships: Vec::new(),
        });
    }

    // Parse the JSON report from stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Warning: Failed to parse pip report JSON: {}", e);
            return Ok(LockFileData {
                dependencies: direct_deps,
                relationships: Vec::new(),
            });
        }
    };

    // Extract resolved packages from the report
    let mut all_dependencies = Vec::new();
    let mut relationships = Vec::new();
    let mut seen_packages: HashSet<String> = HashSet::new();

    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    if let Some(install_array) = report.get("install").and_then(|i| i.as_array()) {
        for item in install_array {
            let metadata = item.get("metadata").unwrap_or(item);

            let name = match metadata.get("name").and_then(|n| n.as_str()) {
                Some(n) => n,
                None => continue,
            };

            let version = metadata
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unspecified");

            let normalized_name = normalize_python_package_name(name);

            // Skip if already seen
            if seen_packages.contains(&normalized_name) {
                continue;
            }
            seen_packages.insert(normalized_name.clone());

            let is_direct = direct_names.contains(&normalized_name);

            let source_info = if is_direct {
                format!(
                    "Identified by the python/requirements extractor from {}",
                    absolute_path.display()
                )
            } else {
                format!(
                    "Identified by pip transitive resolution from {}",
                    absolute_path.display()
                )
            };

            all_dependencies.push(Dependency {
                name: name.to_string(),
                version: version.to_string(),
                ecosystem: "pip".to_string(),
                source: if is_direct {
                    DependencySource::Manifest
                } else {
                    DependencySource::LockFile // Mark transitive as lockfile-resolved
                },
                is_dev: false,
                is_direct,
                source_file: Some(source_info),
                ..Default::default()
            });

            // Build relationships: extract requires_dist from metadata
            if let Some(requires) = metadata.get("requires_dist").and_then(|r| r.as_array()) {
                let child_names: Vec<String> = requires
                    .iter()
                    .filter_map(|r| r.as_str())
                    .filter_map(|req| {
                        // Parse requirement string like "packaging>=20.0" -> "packaging"
                        req.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                            .next()
                            .map(|s| s.to_string())
                    })
                    .collect();

                if !child_names.is_empty() {
                    relationships.push(DependencyRelationship {
                        parent_id: format!("{}@{}", name, version),
                        child_names,
                    });
                }
            }
        }
    }

    // If pip resolution didn't return packages, fall back to direct deps
    if all_dependencies.is_empty() {
        return Ok(LockFileData {
            dependencies: direct_deps,
            relationships: Vec::new(),
        });
    }

    Ok(LockFileData {
        dependencies: all_dependencies,
        relationships,
    })
}

pub fn normalize_python_package_name(name: &str) -> String {
    let re = Regex::new(r"[-_.]+").unwrap();
    re.replace_all(&name.to_lowercase(), "_").to_string()
}

/// Metadata extracted from Python setup.py or pyproject.toml
#[derive(Debug, Clone, Default)]
pub struct PythonPackageMetadata {
    pub license: Option<String>,
    pub author: Option<String>,
}

/// Fetch package metadata from PyPI API (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_python_metadata_from_pypi(
    package_name: &str,
    version: &str,
) -> Option<PythonPackageMetadata> {
    if version == "unknown"
        || version == "unspecified"
        || version.contains(">")
        || version.contains("<")
        || version.contains("=")
    {
        return None;
    }

    let url = format!("https://pypi.org/pypi/{}/{}/json", package_name, version);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().ok()?;
    let info = json.get("info")?;

    let license = info
        .get("license")
        .and_then(|l| l.as_str())
        .map(|s| s.to_string());
    let author = info
        .get("author")
        .and_then(|a| a.as_str())
        .map(|s| s.to_string());

    Some(PythonPackageMetadata { license, author })
}

/// Hybrid metadata loading: local setup.py first, then PyPI API (v0.9.0)
fn load_python_metadata_hybrid(
    package_name: &str,
    version: &str,
    enable_network: bool,
) -> PythonPackageMetadata {
    #[cfg(feature = "internal")]
    if enable_network {
        if let Some(metadata) = fetch_python_metadata_from_pypi(package_name, version) {
            return metadata;
        }
    }
    #[cfg(not(feature = "internal"))]
    let _ = (package_name, version, enable_network);
    PythonPackageMetadata {
        license: None,
        author: None,
    }
}

/// Batch fetch Python metadata for multiple packages in parallel using rayon (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_python_metadata_batch(
    packages: &[(String, String)],
) -> HashMap<String, PythonPackageMetadata> {
    packages
        .par_iter()
        .filter_map(|(name, version)| {
            let key = format!("{}@{}", name, version);
            fetch_python_metadata_from_pypi(name, version).map(|metadata| (key, metadata))
        })
        .collect()
}

/// Parse Python setup.py for ROS package dependencies
/// Returns (package_version, dependencies)
pub fn parse_setup_py(path: &Path) -> Result<(Option<String>, Vec<Dependency>)> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read setup.py at {:?}", path))?;

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/setuppy extractor from {}",
        absolute_path.display()
    );

    // Extract version: version='0.40.4' or version="0.40.4"
    let version_regex = Regex::new(r#"version\s*=\s*['"]([^'"]+)['"]"#).unwrap();
    let package_version = version_regex
        .captures(&content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string());

    let mut dependencies = Vec::new();

    // Extract install_requires=['pkg1', 'pkg2>=1.0', ...] or install_requires=(...)
    // Use DOTALL (?s) and non-greedy match to support multiline lists and both [] / ()
    let install_requires_regex =
        Regex::new(r#"(?s)install_requires\s*=\s*[\[\(]\s*(?P<list>.*?)\s*[\]\)]"#).unwrap();

    if let Some(caps) = install_requires_regex.captures(&content) {
        if let Some(deps_str) = caps.name("list") {
            parse_python_dependency_list(
                deps_str.as_str(),
                &mut dependencies,
                false,
                Some(&source_info),
            );
        }
    }

    // Extract extras_require entries for dev-related extras like 'test', 'dev', 'docs', etc.
    // We treat common dev-related extras as dev dependencies
    // Use DOTALL (?s) and non-greedy match to support multiline extras
    let extras_regex =
        Regex::new(r#"(?s)['"](?P<extra>[^'"]+)['"]\s*:\s*\[\s*(?P<list>.*?)\s*\]"#).unwrap();

    for caps in extras_regex.captures_iter(&content) {
        let extra_name = caps.name("extra").map(|m| m.as_str()).unwrap_or_default();
        // Only treat known dev-related extras as dev dependencies
        let is_dev_extra = matches!(
            extra_name,
            "test" | "tests" | "testing" | "dev" | "development" | "doc" | "docs"
        );

        if is_dev_extra {
            if let Some(deps_str) = caps.name("list") {
                parse_python_dependency_list(
                    deps_str.as_str(),
                    &mut dependencies,
                    true,
                    Some(&source_info),
                );
            }
        }
    }

    Ok((package_version, dependencies))
}

/// Helper to parse Python list of dependencies like ['pkg1', 'pkg2>=1.0']
pub fn parse_python_dependency_list(
    list_content: &str,
    dependencies: &mut Vec<Dependency>,
    is_dev: bool,
    source_info: Option<&str>,
) {
    let dep_regex = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();
    let spec_regex = Regex::new(r"^([a-zA-Z0-9_\-\.]+)(\[[^\]]+\])?(.*?)$").unwrap();

    for caps in dep_regex.captures_iter(list_content) {
        if let Some(dep_str) = caps.get(1) {
            let dep_str = dep_str.as_str();

            if let Some(spec_caps) = spec_regex.captures(dep_str) {
                let name = spec_caps.get(1).map_or("", |m| m.as_str()).to_string();
                let extras = spec_caps.get(2).map_or("", |m| m.as_str());
                let version_spec = spec_caps.get(3).map_or("", |m| m.as_str()).trim();

                let version = if version_spec.is_empty() {
                    "unspecified".to_string()
                } else {
                    version_spec.to_string()
                };

                // v0.9.0: Hybrid metadata extraction (PyPI API) - do this before moving name
                let metadata = load_python_metadata_hybrid(&name, &version, true);

                let full_name = if !extras.is_empty() {
                    format!("{}{}", name, extras)
                } else {
                    name
                };

                dependencies.push(Dependency {
                    name: full_name,
                    version,
                    ecosystem: "pip".to_string(),
                    source: DependencySource::Manifest,
                    is_dev,
                    is_direct: true,
                    license: metadata.license,
                    author: metadata.author,
                    source_file: source_info.map(|s| s.to_string()),
                    ..Default::default()
                });
            }
        }
    }
}

/// Parses poetry.lock with relationship data for hierarchical dependency trees.
/// Extracts parent-child relationships from the [package.dependencies] table in each package entry.
///
/// # Returns
/// LockFileData containing:
/// - dependencies: All packages from poetry.lock
/// - relationships: Parent→child mappings from dependencies tables
///
/// Parameters:
/// - path: Path to poetry.lock file
/// - enable_network: If true, fetch metadata from PyPI API. If false, skip network calls.
pub fn parse_poetry_lock_with_relationships(
    path: &Path,
    enable_network: bool,
) -> Result<LockFileData> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read poetry.lock at {:?}", path))?;

    let toml_value: toml::Value = toml::from_str(&content)?;
    let mut dependencies = Vec::new();
    let mut relationships = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/poetry extractor from {}",
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

                let category = package
                    .get("category")
                    .and_then(|v| v.as_str())
                    .unwrap_or("main");
                let is_dev = category == "dev";

                // v0.9.3: Extract SHA256 checksum from files section
                let checksum = package
                    .get("files")
                    .and_then(|f| f.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|file| file.get("hash"))
                    .and_then(|h| h.as_str())
                    .and_then(|hash_str| {
                        // Poetry format: "sha256:abc123..."
                        if hash_str.starts_with("sha256:") {
                            Some(hash_str.trim_start_matches("sha256:").to_string())
                        } else {
                            None
                        }
                    });

                // Collect for batch API fetch
                #[cfg(feature = "internal")]
                packages_needing_api.push((name.to_string(), version.clone()));

                package_info_list.push((
                    name.to_string(),
                    version,
                    is_dev,
                    checksum,
                    package.clone(),
                ));
            }
        }

        // PASS 2: Parallel batch fetch from PyPI API (v0.9.0) (only if network enabled)
        #[cfg(feature = "internal")]
        let api_metadata = if enable_network && !packages_needing_api.is_empty() {
            eprintln!(
                "Fetching metadata for {} Python packages from PyPI (parallel)...",
                packages_needing_api.len()
            );
            fetch_python_metadata_batch(&packages_needing_api)
        } else {
            HashMap::new()
        };
        #[cfg(not(feature = "internal"))]
        let api_metadata: HashMap<String, PythonPackageMetadata> = {
            let _ = enable_network;
            HashMap::new()
        };

        // PASS 3: Create dependencies with metadata
        for (name, version, is_dev, checksum, package) in package_info_list {
            let package_key = format!("{}@{}", name, version);
            let metadata =
                api_metadata
                    .get(&package_key)
                    .cloned()
                    .unwrap_or(PythonPackageMetadata {
                        license: None,
                        author: None,
                    });

            dependencies.push(Dependency {
                name: name.clone(),
                version: version.clone(),
                ecosystem: "pip".to_string(),
                source: DependencySource::LockFile,
                is_dev,
                is_direct: true, // Will be corrected by DependencyGraph::build_from_deps_with_relationships()
                checksum_sha256: checksum,
                license: metadata.license,
                author: metadata.author,
                source_file: Some(source_info.clone()),
                ..Default::default()
            });

            // Extract dependencies table to build relationships
            // Poetry format: [package.dependencies]
            // certifi = ">=2017.4.17"
            // charset-normalizer = ">=2,<4"
            if let Some(deps_table) = package.get("dependencies").and_then(|v| v.as_table()) {
                let parent_id = format!("{}@{}", name, version);
                let child_names: Vec<String> = deps_table.keys().map(|k| k.to_string()).collect();

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

// ============================================================================
// v0.9.3: Pipfile/Pipfile.lock Support
// ============================================================================

use serde::Deserialize;

/// Pipfile.lock data structures for deserialization
#[derive(Debug, Deserialize)]
struct PipfileLock {
    default: HashMap<String, PipfilePackage>,
    #[serde(default)]
    develop: Option<HashMap<String, PipfilePackage>>,
}

#[derive(Debug, Deserialize)]
struct PipfilePackage {
    version: String, // "==1.12.0"
    #[serde(default)]
    hashes: Vec<String>, // ["sha256:abc...", "sha256:def..."]
    index: Option<String>, // "pypi" if direct dependency
}

/// Parse Pipfile.lock and extract all dependencies with SHA256 checksums
///
/// Parameters:
/// - path: Path to Pipfile.lock file
/// - enable_network: If true, fetch metadata from PyPI API. If false, skip network calls.
///
/// Returns:
/// - dependencies: All packages from Pipfile.lock (default + develop sections)
/// - relationships: Empty vector (Pipfile.lock doesn't include dependency relationships)
///
/// Note: Unlike poetry.lock, Pipfile.lock does NOT include explicit dependency relationships.
/// The `index` field indicates direct dependencies (those listed in Pipfile).
pub fn parse_pipfile_lock_with_relationships(
    path: &Path,
    enable_network: bool,
) -> Result<LockFileData> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read Pipfile.lock at {:?}", path))?;

    let lock: PipfileLock =
        serde_json::from_str(&content).context("Failed to parse Pipfile.lock JSON")?;

    let mut all_packages = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/pipfilelock extractor from {}",
        absolute_path.display()
    );

    // Process default packages (production dependencies)
    for (name, pkg) in lock.default {
        let version = pkg.version.trim_start_matches("==").to_string();
        let is_direct = pkg.index.is_some();
        let checksum = extract_first_sha256(&pkg.hashes);

        all_packages.push((name, version, false, is_direct, checksum));
    }

    // Process develop packages (development dependencies)
    if let Some(dev_packages) = lock.develop {
        for (name, pkg) in dev_packages {
            let version = pkg.version.trim_start_matches("==").to_string();
            let is_direct = pkg.index.is_some();
            let checksum = extract_first_sha256(&pkg.hashes);

            all_packages.push((name, version, true, is_direct, checksum));
        }
    }

    // Prepare for batch metadata fetch
    #[cfg(feature = "internal")]
    let packages_needing_api: Vec<(String, String)> = all_packages
        .iter()
        .map(|(name, version, _, _, _)| (name.clone(), version.clone()))
        .collect();

    // PASS 2: Parallel batch fetch from PyPI API (only if network enabled)
    #[cfg(feature = "internal")]
    let api_metadata = if enable_network && !packages_needing_api.is_empty() {
        eprintln!(
            "Fetching metadata for {} Python packages from PyPI (parallel)...",
            packages_needing_api.len()
        );
        fetch_python_metadata_batch(&packages_needing_api)
    } else {
        HashMap::new()
    };
    #[cfg(not(feature = "internal"))]
    let api_metadata: HashMap<String, PythonPackageMetadata> = {
        let _ = enable_network;
        HashMap::new()
    };

    // PASS 3: Build dependencies with metadata
    let dependencies: Vec<Dependency> = all_packages
        .into_iter()
        .map(|(name, version, is_dev, is_direct, checksum)| {
            let package_key = format!("{}@{}", name, version);
            let metadata =
                api_metadata
                    .get(&package_key)
                    .cloned()
                    .unwrap_or(PythonPackageMetadata {
                        license: None,
                        author: None,
                    });

            Dependency {
                name: name.clone(),
                version,
                ecosystem: "pip".to_string(),
                source: DependencySource::LockFile,
                is_dev,
                is_direct,
                checksum_sha256: checksum,
                license: metadata.license,
                author: metadata.author,
                source_file: Some(source_info.clone()),
                ..Default::default()
            }
        })
        .collect();

    // Pipfile.lock doesn't include dependency relationships, so return empty vector
    Ok(LockFileData {
        dependencies,
        relationships: Vec::new(),
    })
}

/// Extract the first SHA256 hash from a list of hashes
fn extract_first_sha256(hashes: &[String]) -> Option<String> {
    hashes
        .iter()
        .find(|h| h.starts_with("sha256:"))
        .map(|h| h.trim_start_matches("sha256:").to_string())
}

/// Parse Pipfile (manifest file) and extract direct dependencies
///
/// Pipfile format (TOML):
/// ```toml
/// [packages]
/// jinja2 = "*"
/// requests = ">=2.28.0"
///
/// [dev-packages]
/// pytest = "^7.0"
/// ```
///
/// Parameters:
/// - path: Path to Pipfile file
/// - enable_network: Currently unused, but kept for consistency with other parsers
pub fn parse_pipfile(path: &Path, _enable_network: bool) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read Pipfile at {:?}", path))?;

    let pipfile: toml::Value = toml::from_str(&content).context("Failed to parse Pipfile TOML")?;

    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/pipfile extractor from {}",
        absolute_path.display()
    );

    // Parse [packages] section (production dependencies)
    if let Some(packages) = pipfile.get("packages").and_then(|v| v.as_table()) {
        for (name, spec) in packages {
            let version = parse_pipfile_version_spec(spec);
            dependencies.push(create_python_manifest_dependency(
                name,
                version,
                false,
                &source_info,
            ));
        }
    }

    // Parse [dev-packages] section (development dependencies)
    if let Some(dev_packages) = pipfile.get("dev-packages").and_then(|v| v.as_table()) {
        for (name, spec) in dev_packages {
            let version = parse_pipfile_version_spec(spec);
            dependencies.push(create_python_manifest_dependency(
                name,
                version,
                true,
                &source_info,
            ));
        }
    }

    Ok(dependencies)
}

/// Parse version specification from Pipfile
fn parse_pipfile_version_spec(spec: &toml::Value) -> String {
    match spec.as_str() {
        Some("*") => "unspecified".to_string(),
        Some(v) => v.trim_start_matches("==").to_string(),
        None => {
            // Complex version object: {"version": ">=1.0", "extras": [...]}
            if let Some(version) = spec.get("version").and_then(|v| v.as_str()) {
                if version == "*" {
                    "unspecified".to_string()
                } else {
                    version.trim_start_matches("==").to_string()
                }
            } else {
                "unspecified".to_string()
            }
        }
    }
}

/// Create a Dependency from Python manifest entry (Pipfile, pyproject.toml, etc.)
fn create_python_manifest_dependency(
    name: &str,
    version: String,
    is_dev: bool,
    source_info: &str,
) -> Dependency {
    Dependency {
        name: name.to_string(),
        version,
        ecosystem: "pip".to_string(),
        source: DependencySource::Manifest,
        is_dev,
        is_direct: true,
        source_file: Some(source_info.to_string()),
        ..Default::default()
    }
}

// ============================================================================
// v0.9.3: pyproject.toml Support (PEP 517/518)
// ============================================================================

/// Parse pyproject.toml and extract dependencies
///
/// Supports multiple formats:
/// - PEP 621 ([project] section) - modern standard
/// - Poetry ([tool.poetry] section)
/// - PDM ([tool.pdm] section)
///
/// pyproject.toml is a manifest file, not a lock file, so versions may be ranges.
///
/// Parameters:
/// - path: Path to pyproject.toml file
/// - enable_network: Currently unused, but kept for consistency with other parsers
pub fn parse_pyproject_toml(path: &Path, _enable_network: bool) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read pyproject.toml at {:?}", path))?;

    let pyproject: toml::Value =
        toml::from_str(&content).context("Failed to parse pyproject.toml TOML")?;

    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the python/pyproject extractor from {}",
        absolute_path.display()
    );

    // Priority order: Poetry > PEP 621 > PDM (select only one format to avoid double-counting)
    // Check for Poetry format first ([tool.poetry] section)
    if let Some(poetry) = pyproject.get("tool").and_then(|t| t.get("poetry")) {
        dependencies.extend(parse_poetry_pyproject_dependencies(poetry, &source_info));
    }
    // If no Poetry, try PEP 621 format ([project] section)
    else if let Some(project) = pyproject.get("project") {
        dependencies.extend(parse_pep621_dependencies(project, &source_info));
    }
    // If neither, try PDM format ([tool.pdm] section)
    else if let Some(pdm) = pyproject.get("tool").and_then(|t| t.get("pdm")) {
        dependencies.extend(parse_pdm_dependencies(pdm, &source_info));
    }

    Ok(dependencies)
}

/// Parse PEP 621 format dependencies from [project] section
fn parse_pep621_dependencies(project: &toml::Value, source_info: &str) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // Parse [project.dependencies]
    if let Some(dependencies) = project.get("dependencies").and_then(|v| v.as_array()) {
        for dep_spec in dependencies {
            if let Some(spec_str) = dep_spec.as_str() {
                if let Some((name, version)) = parse_dependency_spec(spec_str) {
                    deps.push(create_python_manifest_dependency(
                        &name,
                        version,
                        false,
                        source_info,
                    ));
                }
            }
        }
    }

    // Parse [project.optional-dependencies]
    if let Some(optional) = project
        .get("optional-dependencies")
        .and_then(|v| v.as_table())
    {
        for (group, deps_array) in optional {
            let is_dev =
                group == "dev" || group == "test" || group == "tests" || group == "testing";

            if let Some(deps_list) = deps_array.as_array() {
                for dep_spec in deps_list {
                    if let Some(spec_str) = dep_spec.as_str() {
                        if let Some((name, version)) = parse_dependency_spec(spec_str) {
                            deps.push(create_python_manifest_dependency(
                                &name,
                                version,
                                is_dev,
                                source_info,
                            ));
                        }
                    }
                }
            }
        }
    }

    deps
}

/// Parse Poetry format dependencies from [tool.poetry] section
fn parse_poetry_pyproject_dependencies(poetry: &toml::Value, source_info: &str) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // Parse [tool.poetry.dependencies]
    if let Some(dependencies) = poetry.get("dependencies").and_then(|v| v.as_table()) {
        for (name, spec) in dependencies {
            // Skip python version constraint
            if name == "python" {
                continue;
            }

            let version = parse_poetry_version_spec(spec);
            deps.push(create_python_manifest_dependency(
                name,
                version,
                false,
                source_info,
            ));
        }
    }

    // Parse [tool.poetry.dev-dependencies]
    if let Some(dev_deps) = poetry.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (name, spec) in dev_deps {
            let version = parse_poetry_version_spec(spec);
            deps.push(create_python_manifest_dependency(
                name,
                version,
                true,
                source_info,
            ));
        }
    }

    // Parse [tool.poetry.group.*.dependencies] (Poetry 1.2+ format)
    if let Some(groups) = poetry.get("group").and_then(|v| v.as_table()) {
        for (group_name, group) in groups {
            let is_dev = group_name == "dev" || group_name == "test";

            if let Some(dependencies) = group.get("dependencies").and_then(|v| v.as_table()) {
                for (name, spec) in dependencies {
                    if name == "python" {
                        continue;
                    }

                    let version = parse_poetry_version_spec(spec);
                    deps.push(create_python_manifest_dependency(
                        name,
                        version,
                        is_dev,
                        source_info,
                    ));
                }
            }
        }
    }

    deps
}

/// Parse PDM format dependencies from [tool.pdm] section
fn parse_pdm_dependencies(pdm: &toml::Value, source_info: &str) -> Vec<Dependency> {
    let mut deps = Vec::new();

    // Parse [tool.pdm.dependencies]
    if let Some(dependencies) = pdm.get("dependencies").and_then(|v| v.as_array()) {
        for dep_spec in dependencies {
            if let Some(spec_str) = dep_spec.as_str() {
                if let Some((name, version)) = parse_dependency_spec(spec_str) {
                    deps.push(create_python_manifest_dependency(
                        &name,
                        version,
                        false,
                        source_info,
                    ));
                }
            }
        }
    }

    // Parse [tool.pdm.dev-dependencies] (groups like test, lint, etc.)
    if let Some(dev_deps) = pdm.get("dev-dependencies").and_then(|v| v.as_table()) {
        for (_group_name, deps_array) in dev_deps {
            if let Some(deps_list) = deps_array.as_array() {
                for dep_spec in deps_list {
                    if let Some(spec_str) = dep_spec.as_str() {
                        if let Some((name, version)) = parse_dependency_spec(spec_str) {
                            deps.push(create_python_manifest_dependency(
                                &name,
                                version,
                                true,
                                source_info,
                            ));
                        }
                    }
                }
            }
        }
    }

    deps
}

/// Parse dependency specification like "requests>=2.28.0", "click>=8.0,<9.0", or "pydantic[email]>=2.0"
fn parse_dependency_spec(spec: &str) -> Option<(String, String)> {
    if let Some(caps) = PYTHON_DEPENDENCY_REGEX.captures(spec) {
        // Extract package name (group 1) and extras (group 2) if present
        let base_name = caps.get(1)?.as_str();
        let extras = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let name = format!("{}{}", base_name, extras);

        let version_spec = caps
            .get(3)
            .map(|m| m.as_str().trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "unspecified".to_string());

        Some((name, version_spec))
    } else {
        None
    }
}

/// Parse Poetry version specification (supports caret, tilde, etc.)
fn parse_poetry_version_spec(spec: &toml::Value) -> String {
    match spec.as_str() {
        Some("*") => "unspecified".to_string(),
        Some(v) => {
            // Poetry uses caret (^) and tilde (~) for version ranges
            // ^1.2.3 means >=1.2.3,<2.0.0
            // ~1.2.3 means >=1.2.3,<1.3.0
            // We keep the original spec as-is
            v.to_string()
        }
        None => {
            // Complex version object: {version = ">=1.0", optional = true}
            if let Some(version) = spec.get("version").and_then(|v| v.as_str()) {
                version.to_string()
            } else {
                "unspecified".to_string()
            }
        }
    }
}
