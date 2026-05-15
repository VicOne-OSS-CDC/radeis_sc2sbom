use radeis_sc2sbom::classifier;
use radeis_sc2sbom::models::{Dependency, DependencyScope, DependencySource};

#[test]
fn test_scope_filtering_runtime_only() {
    let deps = vec![
        create_test_dep("express", "npm", DependencyScope::Runtime),
        create_test_dep("jest", "npm", DependencyScope::Test),
        create_test_dep("webpack", "npm", DependencyScope::Build),
    ];

    let filtered: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| d.scope == DependencyScope::Runtime)
        .collect();

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].name, "express");
}

#[test]
fn test_scope_filtering_production_mode() {
    let deps = vec![
        create_test_dep("express", "npm", DependencyScope::Runtime),
        create_test_dep("optional-lib", "npm", DependencyScope::Optional),
        create_test_dep("jest", "npm", DependencyScope::Test),
        create_test_dep("webpack", "npm", DependencyScope::Build),
    ];

    // Production mode: Runtime + Optional
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    assert_eq!(filtered.len(), 2);
    assert!(filtered.iter().any(|d| d.name == "express"));
    assert!(filtered.iter().any(|d| d.name == "optional-lib"));
}

#[test]
fn test_classifier_preserves_scope() {
    let deps = vec![
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new("django".to_string(), "4.0.0".to_string(), "PIP".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // pytest should be classified as Test
    let pytest = classified.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(pytest.scope, DependencyScope::Test);
    assert!(pytest.scope_confidence > 0.8);

    // cmake should be classified as Build
    let cmake = classified.iter().find(|d| d.name == "cmake").unwrap();
    assert_eq!(cmake.scope, DependencyScope::Build);
    assert!(cmake.scope_confidence > 0.8);

    // django should be classified as Runtime
    let django = classified.iter().find(|d| d.name == "django").unwrap();
    assert_eq!(django.scope, DependencyScope::Runtime);
}

#[test]
fn test_scope_filter_multiple_ecosystems() {
    let deps = vec![
        create_test_dep("express", "npm", DependencyScope::Runtime),
        create_test_dep("serde", "cargo", DependencyScope::Runtime),
        create_test_dep("pytest", "PIP", DependencyScope::Test),
        create_test_dep("cargo-test", "cargo", DependencyScope::Test),
    ];

    let test_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| d.scope == DependencyScope::Test)
        .collect();

    assert_eq!(test_deps.len(), 2);
    assert!(test_deps
        .iter()
        .any(|d| d.name == "pytest" && d.ecosystem == "PIP"));
    assert!(test_deps
        .iter()
        .any(|d| d.name == "cargo-test" && d.ecosystem == "cargo"));
}

fn create_test_dep(name: &str, ecosystem: &str, scope: DependencyScope) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: "1.0.0".to_string(),
        ecosystem: ecosystem.to_string(),
        source: DependencySource::Manifest,
        is_dev: false,
        is_direct: true,
        license: None,
        author: None,
        maintainers: None,
        repository_url: None,
        homepage_url: None,
        source_file: None,
        checksum_sha256: None,
        checksum_sha512: None,
        scope,
        scope_confidence: 0.9,
        scope_reason: "Test".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }
}
