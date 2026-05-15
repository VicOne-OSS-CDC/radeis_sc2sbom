use crate::models::{Dependency, DependencySource, RosPackageMetadata};
use crate::parsers::python::{normalize_python_package_name, parse_setup_py};
use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Normalize ROS license format to SPDX identifiers
fn normalize_ros_license_to_spdx(license: &str) -> String {
    match license.trim() {
        "Apache License 2.0" | "Apache 2.0" | "Apache License, Version 2.0" => {
            "Apache-2.0".to_string()
        }
        "BSD" => "BSD-3-Clause".to_string(),
        "MIT" => "MIT".to_string(),
        "GPLv3" | "GPL-3.0" => "GPL-3.0-only".to_string(),
        "GPLv2" | "GPL-2.0" => "GPL-2.0-only".to_string(),
        "LGPLv3" | "LGPL-3.0" => "LGPL-3.0-only".to_string(),
        "LGPLv2" | "LGPL-2.0" | "LGPLv2.1" | "LGPL-2.1" => "LGPL-2.1-only".to_string(),
        other => other.to_string(), // Pass through if unknown
    }
}

/// Parse ROS/ROS2 package.xml and optionally adjacent setup.py
/// Returns (package_metadata, dependencies)
pub fn parse_ros_package(
    package_xml_path: &Path,
) -> Result<(Option<RosPackageMetadata>, Vec<Dependency>)> {
    let content = fs::read_to_string(package_xml_path).context(format!(
        "Failed to read package.xml at {:?}",
        package_xml_path
    ))?;

    // v0.8.0: Create source tracking string
    let absolute_path = package_xml_path
        .canonicalize()
        .unwrap_or_else(|_| package_xml_path.to_path_buf());
    let source_info = format!(
        "Identified by the ros/packagexml extractor from {}",
        absolute_path.display()
    );

    let mut reader = Reader::from_str(&content);
    reader.trim_text(true);
    let mut buf = Vec::new();
    let mut dependencies = Vec::new();
    let mut current_tag = String::new();

    // Track package name and version
    let mut package_name: Option<String> = None;
    let mut package_version: Option<String> = None;
    // v0.8.0: Track metadata fields
    let mut license: Option<String> = None;
    let mut maintainers: Vec<String> = Vec::new();
    let mut authors: Vec<String> = Vec::new();
    let mut description: Option<String> = None;

    // v0.8.0: Track attributes for maintainer/author tags (email attribute)
    let mut current_email: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                // Track which dependency tag we're in
                current_tag = String::from_utf8_lossy(e.name().as_ref()).to_string();

                // Extract email attribute from maintainer/author tags
                current_email = None;
                if current_tag == "maintainer" || current_tag == "author" {
                    for attr in e.attributes() {
                        if let Ok(attr) = attr {
                            if attr.key.as_ref() == b"email" {
                                current_email = attr.unescape_value().ok().map(|s| s.to_string());
                            }
                        }
                    }
                }
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().unwrap_or_default().trim().to_string();

                if text.is_empty() {
                    continue;
                }

                // Extract package name and version
                match current_tag.as_str() {
                    "name" => {
                        package_name = Some(text.clone());
                        continue;
                    }
                    "version" => {
                        package_version = Some(text.clone());
                        continue;
                    }
                    "license" => {
                        // Normalize ROS license format to SPDX
                        license = Some(normalize_ros_license_to_spdx(&text));
                        continue;
                    }
                    "description" => {
                        description = Some(text.clone());
                        continue;
                    }
                    "maintainer" => {
                        // Format: "Name <email>" if email present, else just "Name"
                        let formatted = if let Some(ref email) = current_email {
                            format!("{} <{}>", text, email)
                        } else {
                            text.clone()
                        };
                        maintainers.push(formatted);
                        continue;
                    }
                    "author" => {
                        // Format: "Name <email>" if email present, else just "Name"
                        let formatted = if let Some(ref email) = current_email {
                            format!("{} <{}>", text, email)
                        } else {
                            text.clone()
                        };
                        authors.push(formatted);
                        continue;
                    }
                    _ => {}
                }

                // Determine if this is a dependency tag and whether it's a dev dependency
                let is_dev = match current_tag.as_str() {
                    "test_depend" => true,
                    "exec_depend"
                    | "build_depend"
                    | "buildtool_depend"
                    | "depend"
                    | "build_export_depend" => false,
                    _ => {
                        // Not a dependency tag, skip
                        continue;
                    }
                };

                // Create dependency entry
                dependencies.push(Dependency {
                    name: text,
                    version: "unspecified".to_string(),
                    ecosystem: "ros".to_string(),
                    source: DependencySource::Manifest,
                    is_dev,
                    is_direct: true,
                    source_file: Some(source_info.clone()),
                    ..Default::default()
                });
            }
            Ok(Event::End(_)) => {
                // Clear current tag when we exit an element
                current_tag.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(anyhow::anyhow!(
                    "XML parsing error at position {} in {:?}: {}",
                    reader.buffer_position(),
                    package_xml_path,
                    e
                ));
            }
            _ => (),
        }
        buf.clear();
    }

    // Try to parse adjacent setup.py
    if let Some(parent_dir) = package_xml_path.parent() {
        let setup_py_path = parent_dir.join("setup.py");

        if setup_py_path.exists() {
            if let Ok((setup_version, setup_deps)) = parse_setup_py(&setup_py_path) {
                // Use setup.py version if package.xml doesn't have one
                if package_version.is_none() {
                    package_version = setup_version;
                }

                // Enrich dependency versions
                for setup_dep in setup_deps {
                    // Normalize names for matching using PEP-503 compliant normalization
                    if let Some(ros_dep) = dependencies.iter_mut().find(|d| {
                        normalize_python_package_name(&d.name)
                            == normalize_python_package_name(&setup_dep.name)
                    }) {
                        // Enrich version if setup.py has one
                        if setup_dep.version != "unspecified" {
                            ros_dep.version = setup_dep.version.clone();
                        }
                    } else {
                        // Add Python-only dependencies
                        dependencies.push(setup_dep);
                    }
                }
            }
        }
    }

    // Create metadata if we have name and version
    let metadata = if let (Some(name), Some(version)) = (package_name, package_version) {
        Some(RosPackageMetadata {
            name,
            version,
            source_file: package_xml_path.to_path_buf(),
            license,
            maintainers,
            authors,
            description,
        })
    } else {
        None
    };

    Ok((metadata, dependencies))
}

// ============================================================================
// v0.9.1: ROS/rosdep Version Resolution Support
// ============================================================================

/// ROS distribution information
#[derive(Debug, Clone)]
struct RosPackageInfo {
    version: String,
    repository_url: Option<String>,
}

/// rosdistro database containing package version information
#[derive(Debug, Clone)]
struct RosDistroDatabase {
    repositories: HashMap<String, RosPackageInfo>,
}

lazy_static::lazy_static! {
    static ref ROSDISTRO_CACHE: Arc<Mutex<HashMap<String, RosDistroDatabase>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

/// Detect ROS distribution from CLI override, environment variable, or default
///
/// Priority order:
/// 1. CLI flag (--ros-distro)
/// 2. ROS_DISTRO environment variable
/// 3. Default to "jazzy" (latest stable ROS 2)
fn detect_ros_distribution(cli_override: Option<&str>) -> Option<String> {
    // First priority: CLI override
    if let Some(distro) = cli_override {
        if !distro.is_empty() {
            return Some(distro.to_string());
        }
    }

    // Second priority: ROS_DISTRO environment variable
    if let Ok(distro) = std::env::var("ROS_DISTRO") {
        if !distro.is_empty() {
            return Some(distro);
        }
    }

    // Default: latest stable ROS 2 distribution
    // TODO: Add version-based inference in future
    Some("jazzy".to_string())
}

/// Fetch and parse rosdistro distribution YAML from GitHub
#[cfg(feature = "internal")]
fn fetch_rosdistro_database(distro: &str) -> Option<RosDistroDatabase> {
    let url = format!(
        "https://raw.githubusercontent.com/ros/rosdistro/master/{}/distribution.yaml",
        distro
    );

    // HTTP request with 10-second timeout for rosdistro fetch
    // (3 seconds was too short for reliable network fetch)
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;
    let yaml_text = response.text().ok()?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&yaml_text).ok()?;

    // Extract repositories
    let repos = yaml.get("repositories")?;
    let mut database = RosDistroDatabase {
        repositories: HashMap::new(),
    };

    // Parse each repository entry
    if let serde_yaml::Value::Mapping(repos_map) = repos {
        for (name, info) in repos_map {
            if let (serde_yaml::Value::String(pkg_name), serde_yaml::Value::Mapping(pkg_info)) =
                (name, info)
            {
                // Try release.version first, then source.version
                let version = pkg_info
                    .get(&serde_yaml::Value::String("release".to_string()))
                    .and_then(|r| r.get(&serde_yaml::Value::String("version".to_string())))
                    .or_else(|| {
                        pkg_info
                            .get(&serde_yaml::Value::String("source".to_string()))
                            .and_then(|s| s.get(&serde_yaml::Value::String("version".to_string())))
                    });

                // Extract repository URL from source.url
                let repository_url = pkg_info
                    .get(&serde_yaml::Value::String("source".to_string()))
                    .and_then(|s| s.get(&serde_yaml::Value::String("url".to_string())))
                    .and_then(|u| {
                        if let serde_yaml::Value::String(url) = u {
                            Some(url.clone())
                        } else {
                            None
                        }
                    });

                if let Some(serde_yaml::Value::String(ver)) = version {
                    // Strip Debian release suffix (e.g., "3.3.13-1" → "3.3.13")
                    let clean_ver = ver.split('-').next().unwrap_or(ver).to_string();

                    database.repositories.insert(
                        pkg_name.clone(),
                        RosPackageInfo {
                            version: clean_ver,
                            repository_url,
                        },
                    );
                }
            }
        }
    }

    Some(database)
}

/// Get cached rosdistro database or fetch and cache it
#[cfg(feature = "internal")]
fn get_or_fetch_rosdistro_database(distro: &str) -> Option<RosDistroDatabase> {
    // Check cache first
    {
        if let Ok(cache) = ROSDISTRO_CACHE.lock() {
            if let Some(db) = cache.get(distro) {
                return Some(db.clone());
            }
        }
    }

    // Fetch from network
    let db = fetch_rosdistro_database(distro)?;

    // Store in cache
    {
        if let Ok(mut cache) = ROSDISTRO_CACHE.lock() {
            cache.insert(distro.to_string(), db.clone());
        }
    }

    Some(db)
}

/// Resolve ROS package name to system package name variants
fn resolve_package_name_variants(package_name: &str, distro: &str) -> Vec<String> {
    vec![
        package_name.to_string(),                              // rclpy
        format!("python3-{}", package_name),                   // python3-rclpy
        format!("ros-{}-{}", distro, package_name),            // ros-jazzy-rclpy
        package_name.replace("_", "-"),                        // ament-index-python
        format!("python3-{}", package_name.replace("_", "-")), // python3-ament-index-python
    ]
}

/// Look up package information in rosdistro database
/// Returns (version, repository_url) if found
fn lookup_package_info(
    package_name: &str,
    database: &RosDistroDatabase,
    distro: &str,
) -> Option<(String, Option<String>)> {
    // Try all name variants
    let variants = resolve_package_name_variants(package_name, distro);

    for variant in variants {
        if let Some(info) = database.repositories.get(&variant) {
            return Some((info.version.clone(), info.repository_url.clone()));
        }
    }

    None
}

/// Resolve versions for all ROS dependencies in parallel using rosdistro database
///
/// # Arguments
/// * `dependencies` - Mutable slice of dependencies to resolve
/// * `ros_distro_override` - Optional CLI override for ROS distribution
pub fn resolve_ros_dependency_versions(
    dependencies: &mut [Dependency],
    ros_distro_override: Option<&str>,
) {
    // Detect ROS distribution (CLI override > env var > default)
    let ros_distro = match detect_ros_distribution(ros_distro_override) {
        Some(distro) => distro,
        None => return, // No distribution detected, skip resolution
    };

    // Fetch rosdistro database (with caching)
    #[cfg(not(feature = "internal"))]
    return; // Network fetch not available in public build
    #[cfg(feature = "internal")]
    let database = match get_or_fetch_rosdistro_database(&ros_distro) {
        Some(db) => db,
        None => return, // Failed to fetch database, skip resolution
    };

    // Parallel update using rayon (matching existing metadata enrichment pattern)
    #[cfg(feature = "internal")]
    {
        use rayon::prelude::*;

        dependencies.par_iter_mut().for_each(|dep| {
            // Only try to resolve ROS ecosystem packages with unspecified/detected versions
            if dep.ecosystem == "ros" && (dep.version == "unspecified" || dep.version == "detected") {
                if let Some((version, repository_url)) =
                    lookup_package_info(&dep.name, &database, &ros_distro)
                {
                    dep.version = version;
                    // Populate repository_url if available
                    if repository_url.is_some() {
                        dep.repository_url = repository_url;
                    }
                }
            }
        });
    }
}
