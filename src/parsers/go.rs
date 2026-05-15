use crate::models::{Dependency, DependencySource};
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn parse_go_mod(path: &Path) -> Result<Vec<Dependency>> {
    let content =
        fs::read_to_string(path).context(format!("Failed to read go.mod at {:?}", path))?;

    let mut dependencies = Vec::new();
    let mut in_require_block = false;

    // v0.8.0: Create source tracking string
    let absolute_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let source_info = format!(
        "Identified by the go/gomod extractor from {}",
        absolute_path.display()
    );

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("require (") {
            in_require_block = true;
            continue;
        }

        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }

        if line.starts_with("require ") || in_require_block {
            let parts: Vec<&str> = line
                .trim_start_matches("require ")
                .split_whitespace()
                .collect();

            if parts.len() >= 2 {
                dependencies.push(Dependency {
                    name: parts[0].to_string(),
                    version: parts[1].to_string(),
                    ecosystem: "go".to_string(),
                    source: DependencySource::Manifest,
                    is_dev: false,
                    is_direct: true,
                    source_file: Some(source_info.clone()),
                    ..Default::default()
                });
            }
        }
    }

    Ok(dependencies)
}
