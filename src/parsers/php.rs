use crate::models::{Dependency, DependencySource};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata extracted from Packagist API
#[derive(Debug, Clone, Default)]
pub struct PhpPackageMetadata {
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub repository_url: Option<String>,
    pub homepage_url: Option<String>,
}

/// Fetch package metadata from Packagist API (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_php_metadata_from_packagist(
    package_name: &str,
    version: &str,
) -> Option<PhpPackageMetadata> {
    // Skip if version is unknown/unspecified or contains operators
    if version == "unknown"
        || version == "unspecified"
        || version.contains(">")
        || version.contains("<")
        || version.contains("^")
        || version.contains("~")
        || version.contains("*")
    {
        return None;
    }

    // Packagist API: https://packagist.org/packages/{vendor}/{package}.json
    let url = format!("https://repo.packagist.org/p2/{}.json", package_name);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().ok()?;

    // Packagist v2 API format: {"packages": {"vendor/package": [versions...]}}
    let packages = json.get("packages")?.as_object()?;
    let package_versions = packages.get(package_name)?.as_array()?;

    // Find matching version or use latest
    let version_data = package_versions
        .iter()
        .find(|v| v.get("version").and_then(|ver| ver.as_str()) == Some(version))
        .or_else(|| package_versions.first())?;

    let license = version_data.get("license").and_then(|l| {
        if l.is_string() {
            l.as_str().map(|s| s.to_string())
        } else if l.is_array() {
            l.as_array()
                .and_then(|arr| arr.first())
                .and_then(|first| first.as_str())
                .map(|s| s.to_string())
        } else {
            None
        }
    });

    let authors = version_data
        .get("authors")
        .and_then(|a| a.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|author| {
                    author
                        .get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();

    let repository_url = version_data
        .get("source")
        .and_then(|s| s.get("url"))
        .and_then(|u| u.as_str())
        .map(|s| s.to_string());

    let homepage_url = version_data
        .get("homepage")
        .and_then(|h| h.as_str())
        .map(|s| s.to_string());

    Some(PhpPackageMetadata {
        license,
        authors,
        repository_url,
        homepage_url,
    })
}

/// Batch fetch PHP metadata for multiple packages in parallel using rayon (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_php_metadata_batch(packages: &[(String, String)]) -> HashMap<String, PhpPackageMetadata> {
    packages
        .par_iter()
        .filter_map(|(name, version)| {
            let key = format!("{}@{}", name, version);
            fetch_php_metadata_from_packagist(name, version).map(|metadata| (key, metadata))
        })
        .collect()
}

pub fn parse_composer_json(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read composer.json at {:?}", path))?;

    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the php/composer extractor from {}",
        absolute_path.display()
    );

    // PASS 1: Collect all dependencies (require + require-dev) - v0.9.0 parallel optimization
    let mut package_info_list = Vec::new();
    #[cfg(feature = "internal")]
    let mut packages_needing_api = Vec::new();

    // Collect regular dependencies
    if let Some(deps) = json.get("require").and_then(|v| v.as_object()) {
        for (name, version) in deps {
            if name != "php" && !name.starts_with("ext-") {
                let version_str = version.as_str().unwrap_or("unknown");
                #[cfg(feature = "internal")]
                packages_needing_api.push((name.clone(), version_str.to_string()));
                package_info_list.push((name.clone(), version_str.to_string(), false));
            }
        }
    }

    // Collect dev dependencies
    if let Some(dev_deps) = json.get("require-dev").and_then(|v| v.as_object()) {
        for (name, version) in dev_deps {
            if name != "php" && !name.starts_with("ext-") {
                let version_str = version.as_str().unwrap_or("unknown");
                #[cfg(feature = "internal")]
                packages_needing_api.push((name.clone(), version_str.to_string()));
                package_info_list.push((name.clone(), version_str.to_string(), true));
            }
        }
    }

    // PASS 2: Parallel batch fetch from Packagist API (v0.9.0)
    #[cfg(feature = "internal")]
    let api_metadata = if !packages_needing_api.is_empty() {
        eprintln!(
            "Fetching metadata for {} PHP packages from Packagist (parallel)...",
            packages_needing_api.len()
        );
        fetch_php_metadata_batch(&packages_needing_api)
    } else {
        HashMap::new()
    };
    #[cfg(not(feature = "internal"))]
    let api_metadata: HashMap<String, PhpPackageMetadata> = HashMap::new();

    // PASS 3: Create dependencies with metadata
    for (name, version, is_dev) in package_info_list {
        let package_key = format!("{}@{}", name, version);
        let metadata = api_metadata.get(&package_key).cloned().unwrap_or_default();

        let author = metadata.authors.first().map(|s| s.to_string());

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "composer".to_string(),
            source: DependencySource::Manifest,
            is_dev,
            is_direct: true,
            license: metadata.license,
            author,
            maintainers: if metadata.authors.len() > 1 {
                Some(metadata.authors[1..].to_vec())
            } else {
                None
            },
            repository_url: metadata.repository_url,
            homepage_url: metadata.homepage_url,
            source_file: Some(source_info.clone()),
            ..Default::default()
        });
    }

    Ok(dependencies)
}
