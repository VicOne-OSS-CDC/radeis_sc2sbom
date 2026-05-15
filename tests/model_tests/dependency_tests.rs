use radeis_sc2sbom::models::{
    Dependency, DependencyGraph, DependencyRelationship, DependencySource,
};

#[test]
fn test_dependency_struct() {
    let dep = Dependency {
        name: "test-package".to_string(),
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
    };

    assert_eq!(dep.name, "test-package");
    assert_eq!(dep.version, "1.0.0");
    assert_eq!(dep.ecosystem, "npm");
    assert!(!dep.is_dev);
    assert!(dep.is_direct);
}

#[test]
fn test_correct_direct_flags() {
    let deps = vec![
        Dependency {
            name: "express".to_string(),
            version: "4.18.2".to_string(),
            is_direct: true,
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
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
            name: "body-parser".to_string(),
            version: "1.20.1".to_string(),
            is_direct: true, // Initially marked as direct (WRONG - it's a child of express)
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
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
            name: "bytes".to_string(),
            version: "3.1.2".to_string(),
            is_direct: true, // Initially marked as direct (WRONG - it's a child of body-parser)
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
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

    let rels = vec![
        DependencyRelationship {
            parent_id: "express@4.18.2".to_string(),
            child_names: vec!["body-parser".to_string()],
        },
        DependencyRelationship {
            parent_id: "body-parser@1.20.1".to_string(),
            child_names: vec!["bytes".to_string()],
        },
    ];

    let graph = DependencyGraph::build_from_deps_with_relationships(&deps, &rels);

    // Verify express is still marked as direct (it's a root)
    let express_node = graph.get_node("express@4.18.2").unwrap();
    assert!(
        express_node.dependency.is_direct,
        "express should be marked as direct"
    );

    // Verify body-parser is now marked as transitive (it's a child of express)
    let bp_node = graph.get_node("body-parser@1.20.1").unwrap();
    assert!(
        !bp_node.dependency.is_direct,
        "body-parser should be marked as transitive, not direct"
    );

    // Verify bytes is now marked as transitive (it's a child of body-parser)
    let bytes_node = graph.get_node("bytes@3.1.2").unwrap();
    assert!(
        !bytes_node.dependency.is_direct,
        "bytes should be marked as transitive, not direct"
    );
}
