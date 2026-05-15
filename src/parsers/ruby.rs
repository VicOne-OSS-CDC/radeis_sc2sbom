use crate::models::{Dependency, DependencySource};
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Metadata extracted from RubyGems API
#[derive(Debug, Clone, Default)]
pub struct RubyPackageMetadata {
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub homepage_url: Option<String>,
    pub source_code_url: Option<String>,
}

/// Fetch package metadata from RubyGems API (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_ruby_metadata_from_rubygems(
    package_name: &str,
    version: &str,
) -> Option<RubyPackageMetadata> {
    // Skip if version is unknown/unspecified or contains operators
    if version == "unknown"
        || version == "unspecified"
        || version.contains(">")
        || version.contains("<")
        || version.contains("~")
        || version.contains("=")
    {
        return None;
    }

    // RubyGems API: https://rubygems.org/api/v2/rubygems/{gem}/versions/{version}.json
    let url = format!(
        "https://rubygems.org/api/v2/rubygems/{}/versions/{}.json",
        package_name, version
    );

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .build()
        .ok()?;

    let response = client.get(&url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }

    let json: serde_json::Value = response.json().ok()?;

    let license = json
        .get("licenses")
        .and_then(|l| l.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            json.get("license")
                .and_then(|l| l.as_str())
                .map(|s| s.to_string())
        });

    let authors = json
        .get("authors")
        .and_then(|a| a.as_str())
        .map(|s| vec![s.to_string()])
        .unwrap_or_default();

    let homepage_url = json
        .get("homepage_uri")
        .and_then(|h| h.as_str())
        .map(|s| s.to_string());

    let source_code_url = json
        .get("source_code_uri")
        .and_then(|s| s.as_str())
        .map(|s| s.to_string());

    Some(RubyPackageMetadata {
        license,
        authors,
        homepage_url,
        source_code_url,
    })
}

/// Batch fetch Ruby metadata for multiple gems in parallel using rayon (v0.9.0)
#[cfg(feature = "internal")]
fn fetch_ruby_metadata_batch(gems: &[(String, String)]) -> HashMap<String, RubyPackageMetadata> {
    gems.par_iter()
        .filter_map(|(name, version)| {
            let key = format!("{}@{}", name, version);
            fetch_ruby_metadata_from_rubygems(name, version).map(|metadata| (key, metadata))
        })
        .collect()
}

pub fn parse_gemfile(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read Gemfile at {:?}", path))?;

    let mut dependencies = Vec::new();

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the ruby/gemfile extractor from {}",
        absolute_path.display()
    );

    // PASS 1: Collect gems (v0.9.0 parallel optimization)
    let mut gem_info_list = Vec::new();
    #[cfg(feature = "internal")]
    let mut gems_needing_api = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("gem ") {
            let parts: Vec<&str> = line.trim_start_matches("gem ").split(',').collect();

            if !parts.is_empty() {
                let name = parts[0].trim().trim_matches(|c| c == '"' || c == '\'');
                let version = if parts.len() > 1 {
                    parts[1]
                        .trim()
                        .trim_matches(|c| c == '"' || c == '\'')
                        .trim()
                } else {
                    "unspecified"
                };

                // Collect for batch API fetch
                #[cfg(feature = "internal")]
                gems_needing_api.push((name.to_string(), version.to_string()));
                gem_info_list.push((name.to_string(), version.to_string()));
            }
        }
    }

    // PASS 2: Parallel batch fetch from RubyGems API (v0.9.0)
    #[cfg(feature = "internal")]
    let api_metadata = if !gems_needing_api.is_empty() {
        eprintln!(
            "Fetching metadata for {} Ruby gems from RubyGems (parallel)...",
            gems_needing_api.len()
        );
        fetch_ruby_metadata_batch(&gems_needing_api)
    } else {
        HashMap::new()
    };
    #[cfg(not(feature = "internal"))]
    let api_metadata: HashMap<String, RubyPackageMetadata> = HashMap::new();

    // PASS 3: Create dependencies with metadata
    for (name, version) in gem_info_list {
        let package_key = format!("{}@{}", name, version);
        let metadata = api_metadata.get(&package_key).cloned().unwrap_or_default();

        let author = metadata.authors.first().map(|s| s.to_string());

        dependencies.push(Dependency {
            name,
            version,
            ecosystem: "rubygems".to_string(),
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
            repository_url: metadata.source_code_url,
            homepage_url: metadata.homepage_url,
            source_file: Some(source_info.clone()),
            checksum_sha256: None,
            checksum_sha512: None,
            scope: crate::models::DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        });
    }

    Ok(dependencies)
}
