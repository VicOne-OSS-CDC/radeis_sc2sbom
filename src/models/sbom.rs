use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

use super::dependency::{Dependency, DependencyScope};

#[derive(Debug, Clone, Serialize)]
pub struct RosPackageMetadata {
    pub name: String,
    pub version: String,
    pub source_file: PathBuf,
    // v0.8.0: Additional metadata fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub maintainers: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub authors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RosPackageWithDeps {
    pub metadata: RosPackageMetadata,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Serialize)]
pub struct ScopeStatistics {
    pub runtime: usize,
    pub build: usize,
    pub test: usize,
    pub development: usize,
    pub optional: usize,
    pub provided: usize,
    pub total: usize,
    /// Average confidence score across all classified dependencies
    pub avg_confidence: f32,
}

impl ScopeStatistics {
    pub fn from_dependencies(dependencies: &[Dependency]) -> Self {
        let mut stats: HashMap<DependencyScope, usize> = HashMap::new();
        let mut total_confidence = 0.0;

        for dep in dependencies {
            *stats.entry(dep.scope.clone()).or_insert(0) += 1;
            total_confidence += dep.scope_confidence;
        }

        let total = dependencies.len();
        let avg_confidence = if total > 0 {
            total_confidence / total as f32
        } else {
            0.0
        };

        ScopeStatistics {
            runtime: *stats.get(&DependencyScope::Runtime).unwrap_or(&0),
            build: *stats.get(&DependencyScope::Build).unwrap_or(&0),
            test: *stats.get(&DependencyScope::Test).unwrap_or(&0),
            development: *stats.get(&DependencyScope::Development).unwrap_or(&0),
            optional: *stats.get(&DependencyScope::Optional).unwrap_or(&0),
            provided: *stats.get(&DependencyScope::Provided).unwrap_or(&0),
            total,
            avg_confidence,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Sbom {
    pub project_path: PathBuf,
    pub generated_at: String,
    pub dependencies: Vec<Dependency>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ros_package: Option<RosPackageMetadata>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub ros_packages: Vec<RosPackageWithDeps>,
    /// v1.0.6: Scope classification statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope_statistics: Option<ScopeStatistics>,
}
