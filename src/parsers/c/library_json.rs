use crate::models::{Dependency, DependencyScope, DependencySource};
use crate::parsers::format_source_info;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
struct LibraryJson {
    name: Option<String>,
    version: Option<String>,
    repository: Option<LibraryRepository>,
    license: Option<String>,
}

#[derive(Deserialize)]
struct LibraryRepository {
    url: Option<String>,
}

pub fn parse_library_json(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(path)?;
    if let Some(dep) = parse_library_json_content(&content, path)? {
        Ok(vec![dep])
    } else {
        Ok(vec![])
    }
}

pub fn parse_library_json_content(
    content: &str,
    path: &Path,
) -> Result<Option<Dependency>, Box<dyn std::error::Error>> {
    let lib: LibraryJson = serde_json::from_str(content)?;
    let name = match lib.name {
        Some(n) if !n.is_empty() => n,
        _ => return Ok(None),
    };
    let version = lib
        .version
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unspecified".to_string());
    let repo_url = lib.repository.and_then(|r| r.url);

    Ok(Some(Dependency {
        name,
        version,
        ecosystem: "vendored".to_string(),
        source: DependencySource::Manifest,
        source_file: Some(format_source_info("library_json", path, None, false)),
        is_dev: false,
        is_direct: true,
        license: lib.license,
        repository_url: repo_url,
        scope: DependencyScope::Runtime,
        scope_confidence: 0.5,
        scope_reason: "Vendored library described by library.json".to_string(),
        ..Default::default()
    }))
}
