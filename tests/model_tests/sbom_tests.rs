use radeis_sc2sbom::models::{Dependency, DependencySource, Sbom};
use std::path::PathBuf;

#[test]
fn test_sbom_struct() {
    let deps = vec![
        Dependency {
            name: "package1".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            scope: radeis_sc2sbom::models::DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        Dependency {
            name: "package2".to_string(),
            version: "2.0.0".to_string(),
            ecosystem: "cargo".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            scope: radeis_sc2sbom::models::DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
    ];

    let sbom = Sbom {
        project_path: PathBuf::from("/test/path"),
        generated_at: "2024-01-01T00:00:00Z".to_string(),
        dependencies: deps,
        ros_package: None,
        ros_packages: Vec::new(),
        scope_statistics: None,
    };

    assert_eq!(sbom.dependencies.len(), 2);
    assert_eq!(sbom.project_path, PathBuf::from("/test/path"));
}
