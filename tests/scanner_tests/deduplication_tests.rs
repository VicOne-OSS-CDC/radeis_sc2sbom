use radeis_sc2sbom::models::{Dependency, DependencySource};
use radeis_sc2sbom::parsers::deduplicate_dependencies;

#[test]
fn test_deduplicate_dependencies() {
    let deps = vec![
        Dependency {
            name: "express".to_string(),
            version: "^4.17.1".to_string(),
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
            name: "express".to_string(),
            version: "4.18.2".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
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
            name: "lodash".to_string(),
            version: "4.17.21".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
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

    let deduped = deduplicate_dependencies(deps);

    assert_eq!(deduped.len(), 2);

    let express = deduped.iter().find(|d| d.name == "express").unwrap();
    // Should prefer lock file version
    assert_eq!(express.version, "4.18.2");
    assert!(matches!(express.source, DependencySource::LockFile));
}

#[test]
fn test_deduplicate_with_import_scan_priority() {
    let deps = vec![
        Dependency {
            name: "express".to_string(),
            version: "detected".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::ImportScan,
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
            name: "express".to_string(),
            version: "^4.17.1".to_string(),
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
    ];

    let result = deduplicate_dependencies(deps);

    // Should keep manifest version (higher priority than ImportScan)
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].version, "^4.17.1");
    assert!(matches!(result[0].source, DependencySource::Manifest));
}

fn make_dep_with_eco(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
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
    }
}

#[test]
fn test_pkgconfig_wins_over_system_for_same_lib() {
    let deps = vec![
        make_dep_with_eco("SDL2", "unspecified", "system"),
        make_dep_with_eco("sdl2", "2.0.12", "pkg-config"),
    ];
    let deduped = deduplicate_dependencies(deps);
    assert_eq!(deduped.len(), 1, "Should deduplicate to 1 entry");
    assert_eq!(deduped[0].version, "2.0.12");
    assert_eq!(deduped[0].ecosystem, "pkg-config");
}

#[test]
fn test_system_kept_when_no_pkgconfig_match() {
    let deps = vec![
        make_dep_with_eco("pthread", "unspecified", "system"),
        make_dep_with_eco("dl", "unspecified", "system"),
    ];
    let deduped = deduplicate_dependencies(deps);
    assert_eq!(deduped.len(), 2, "Both system libs should be kept");
}
