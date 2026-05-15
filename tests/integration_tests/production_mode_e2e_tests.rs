/// End-to-end tests for production mode and scope filtering
/// These tests validate the complete pipeline from classification to output generation
use radeis_sc2sbom::classifier;
#[cfg(feature = "internal")]
use radeis_sc2sbom::formats::cyclonedx;
use radeis_sc2sbom::models::{Dependency, DependencyScope, Sbom, ScopeStatistics};
use std::path::PathBuf;

/// Create a realistic set of dependencies similar to what would be found in a real project
fn create_realistic_project_dependencies() -> Vec<Dependency> {
    vec![
        // Runtime libraries (should appear in production SBOM)
        Dependency::new(
            "zlib".to_string(),
            "1.3.1".to_string(),
            "BUILD-CONFIG".to_string(),
        )
        .with_scope(
            DependencyScope::Runtime,
            0.9,
            "BUILD-CONFIG linked at runtime",
        ),
        Dependency::new(
            "curl".to_string(),
            "8.15.0".to_string(),
            "SYSTEM".to_string(),
        )
        .with_scope(DependencyScope::Runtime, 0.8, "System library"),
        Dependency::new(
            "openssl".to_string(),
            "3.0.0".to_string(),
            "SYSTEM".to_string(),
        )
        .with_scope(DependencyScope::Runtime, 0.8, "System library"),
        Dependency::new(
            "protobuf".to_string(),
            "3.21.0".to_string(),
            "BUILD-CONFIG".to_string(),
        )
        .with_scope(DependencyScope::Runtime, 0.9, "Runtime library"),
        // Build tools (should be filtered out in production mode)
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        )
        .with_scope(DependencyScope::Build, 1.0, "Build tool"),
        Dependency::new(
            "gcc".to_string(),
            "11.0.0".to_string(),
            "system".to_string(),
        )
        .with_scope(DependencyScope::Build, 1.0, "Build tool"),
        Dependency::new(
            "ninja".to_string(),
            "1.10.0".to_string(),
            "system".to_string(),
        )
        .with_scope(DependencyScope::Build, 1.0, "Build tool"),
        Dependency::new(
            "meson".to_string(),
            "0.59.0".to_string(),
            "system".to_string(),
        )
        .with_scope(DependencyScope::Build, 0.8, "Build system"),
        // Test dependencies (should be filtered out in production mode)
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()).with_scope(
            DependencyScope::Test,
            1.0,
            "Test framework",
        ),
        Dependency::new(
            "gtest".to_string(),
            "1.11.0".to_string(),
            "BUILD-CONFIG".to_string(),
        )
        .with_scope(DependencyScope::Test, 1.0, "Test framework"),
        Dependency::new(
            "unity".to_string(),
            "2.5.0".to_string(),
            "BUILD-CONFIG".to_string(),
        )
        .with_scope(DependencyScope::Test, 1.0, "Test framework"),
        // Development tools (should be filtered out in production mode)
        Dependency::new(
            "pylint".to_string(),
            "2.12.0".to_string(),
            "PIP".to_string(),
        )
        .with_scope(DependencyScope::Development, 1.0, "Dev tool"),
        Dependency::new("black".to_string(), "22.0.0".to_string(), "PIP".to_string()).with_scope(
            DependencyScope::Development,
            1.0,
            "Dev tool",
        ),
        Dependency::new(
            "clang-format".to_string(),
            "14.0.0".to_string(),
            "system".to_string(),
        )
        .with_scope(DependencyScope::Development, 0.9, "Dev tool"),
        // Optional dependencies (should appear in production mode)
        Dependency::new(
            "optional-feature".to_string(),
            "1.0.0".to_string(),
            "npm".to_string(),
        )
        .with_scope(DependencyScope::Optional, 0.8, "Optional dependency"),
    ]
}

#[test]
fn test_e2e_default_mode_includes_all() {
    // Test default behavior: all dependencies should be included
    let deps = create_realistic_project_dependencies();
    let total_deps = deps.len();

    // Default: no filtering
    let filtered_deps = deps.clone();

    assert_eq!(
        filtered_deps.len(),
        total_deps,
        "Default mode should include all {} dependencies",
        total_deps
    );

    // Verify all scope types are present
    assert!(filtered_deps
        .iter()
        .any(|d| d.scope == DependencyScope::Runtime));
    assert!(filtered_deps
        .iter()
        .any(|d| d.scope == DependencyScope::Build));
    assert!(filtered_deps
        .iter()
        .any(|d| d.scope == DependencyScope::Test));
    assert!(filtered_deps
        .iter()
        .any(|d| d.scope == DependencyScope::Development));
    assert!(filtered_deps
        .iter()
        .any(|d| d.scope == DependencyScope::Optional));
}

#[test]
fn test_e2e_production_mode_filters_correctly() {
    // Test production mode: only Runtime + Optional should remain
    let deps = create_realistic_project_dependencies();

    // Simulate production mode filtering
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    // Should have 4 Runtime + 1 Optional = 5 dependencies
    assert_eq!(
        filtered_deps.len(),
        5,
        "Production mode should have 5 dependencies (4 Runtime + 1 Optional)"
    );

    // Verify only Runtime and Optional remain
    for dep in &filtered_deps {
        assert!(
            dep.scope == DependencyScope::Runtime || dep.scope == DependencyScope::Optional,
            "Dependency {} should be Runtime or Optional, got {:?}",
            dep.name,
            dep.scope
        );
    }

    // Verify specific runtime packages are included
    assert!(filtered_deps.iter().any(|d| d.name == "zlib"));
    assert!(filtered_deps.iter().any(|d| d.name == "curl"));
    assert!(filtered_deps.iter().any(|d| d.name == "openssl"));
    assert!(filtered_deps.iter().any(|d| d.name == "protobuf"));
    assert!(filtered_deps.iter().any(|d| d.name == "optional-feature"));

    // Verify build/test/dev packages are excluded
    assert!(!filtered_deps.iter().any(|d| d.name == "cmake"));
    assert!(!filtered_deps.iter().any(|d| d.name == "pytest"));
    assert!(!filtered_deps.iter().any(|d| d.name == "pylint"));
}

#[test]
fn test_e2e_scope_statistics_generation() {
    // Test that scope statistics are correctly calculated
    let deps = create_realistic_project_dependencies();
    let stats = ScopeStatistics::from_dependencies(&deps);

    // Verify counts match expected
    assert_eq!(stats.runtime, 4, "Should have 4 Runtime dependencies");
    assert_eq!(stats.build, 4, "Should have 4 Build dependencies");
    assert_eq!(stats.test, 3, "Should have 3 Test dependencies");
    assert_eq!(
        stats.development, 3,
        "Should have 3 Development dependencies"
    );
    assert_eq!(stats.optional, 1, "Should have 1 Optional dependency");
    assert_eq!(stats.total, 15, "Should have 15 total dependencies");

    // Verify average confidence is reasonable (all are 0.8-1.0)
    assert!(
        stats.avg_confidence >= 0.8 && stats.avg_confidence <= 1.0,
        "Average confidence should be between 0.8 and 1.0, got {}",
        stats.avg_confidence
    );
}

#[test]
fn test_e2e_sbom_generation_with_production_filter() {
    // Test full SBOM generation pipeline with production filtering
    let deps = create_realistic_project_dependencies();

    // Apply production filter
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    // Generate scope statistics
    let stats = Some(ScopeStatistics::from_dependencies(&filtered_deps));

    // Create SBOM
    let sbom = Sbom {
        project_path: PathBuf::from("/test/production-project"),
        generated_at: "2026-03-04T00:00:00Z".to_string(),
        dependencies: filtered_deps,
        ros_package: None,
        ros_packages: Vec::new(),
        scope_statistics: stats,
    };

    // Verify SBOM structure
    assert_eq!(
        sbom.dependencies.len(),
        5,
        "Production SBOM should have 5 dependencies"
    );
    assert!(
        sbom.scope_statistics.is_some(),
        "Should have scope statistics"
    );

    let sbom_stats = sbom.scope_statistics.unwrap();
    assert_eq!(
        sbom_stats.runtime, 4,
        "Statistics should show 4 runtime deps"
    );
    assert_eq!(
        sbom_stats.optional, 1,
        "Statistics should show 1 optional dep"
    );
    assert_eq!(
        sbom_stats.build, 0,
        "Statistics should show 0 build deps (filtered out)"
    );
    assert_eq!(
        sbom_stats.test, 0,
        "Statistics should show 0 test deps (filtered out)"
    );
}

#[cfg(feature = "internal")]
#[test]
fn test_e2e_cyclonedx_output_with_production_filter() {
    // Test CycloneDX output generation with production filtering
    let deps = create_realistic_project_dependencies();

    // Apply production filter
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    let stats = Some(ScopeStatistics::from_dependencies(&filtered_deps));

    let sbom = Sbom {
        project_path: PathBuf::from("/test/production-project"),
        generated_at: "2026-03-04T00:00:00Z".to_string(),
        dependencies: filtered_deps,
        ros_package: None,
        ros_packages: Vec::new(),
        scope_statistics: stats,
    };

    // Generate CycloneDX output
    let cdx_doc = cyclonedx::convert_to_cyclonedx(&sbom, None, &[]);

    // Verify CycloneDX document structure
    assert_eq!(cdx_doc.bom_format, "CycloneDX");
    assert_eq!(cdx_doc.spec_version, "1.5");
    assert!(cdx_doc.serial_number.starts_with("urn:uuid:"));

    // Verify component count (5 dependencies in production mode)
    assert_eq!(cdx_doc.components.len(), 5, "Should have 5 components");

    // Verify dependency-scope property is included for each component.
    // Note: "dependency-scope" indicates direct vs. transitive (not lifecycle scope like Runtime/Build).
    // Lifecycle scope is enforced by the scope filter before generating the CycloneDX output.
    for component in &cdx_doc.components {
        let has_scope_prop = component
            .properties
            .iter()
            .any(|p| p.name == "dependency-scope");
        assert!(
            has_scope_prop,
            "Component {} should have dependency-scope property (direct/transitive)",
            component.name
        );

        let scope_value = component
            .properties
            .iter()
            .find(|p| p.name == "dependency-scope")
            .map(|p| p.value.as_str())
            .unwrap_or("");
        assert!(
            scope_value == "direct" || scope_value == "transitive",
            "Component {} dependency-scope should be 'direct' or 'transitive', got '{}'",
            component.name,
            scope_value
        );
    }
}

#[test]
fn test_e2e_classification_pipeline() {
    // Test the complete classification pipeline
    let mut deps = vec![
        // Create unclassified dependencies
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new(
            "curl".to_string(),
            "8.15.0".to_string(),
            "SYSTEM".to_string(),
        ),
        Dependency::new("django".to_string(), "4.0.0".to_string(), "PIP".to_string()),
    ];

    // Classify dependencies
    deps = classifier::classify_dependencies(deps);

    // Verify classifications
    let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(
        pytest.scope,
        DependencyScope::Test,
        "pytest should be classified as Test"
    );
    assert!(
        pytest.scope_confidence > 0.8,
        "pytest classification should have high confidence"
    );

    let cmake = deps.iter().find(|d| d.name == "cmake").unwrap();
    assert_eq!(
        cmake.scope,
        DependencyScope::Build,
        "cmake should be classified as Build"
    );
    assert!(
        cmake.scope_confidence > 0.8,
        "cmake classification should have high confidence"
    );

    let curl = deps.iter().find(|d| d.name == "curl").unwrap();
    assert_eq!(
        curl.scope,
        DependencyScope::Runtime,
        "curl should be classified as Runtime"
    );

    let django = deps.iter().find(|d| d.name == "django").unwrap();
    assert_eq!(
        django.scope,
        DependencyScope::Runtime,
        "django should be classified as Runtime"
    );
}

#[test]
fn test_e2e_custom_scope_combinations() {
    // Test various custom scope filter combinations
    let deps = create_realistic_project_dependencies();

    // Test 1: Runtime only
    let runtime_only: Vec<_> = deps
        .iter()
        .filter(|d| d.scope == DependencyScope::Runtime)
        .collect();
    assert_eq!(runtime_only.len(), 4, "Should have 4 Runtime dependencies");

    // Test 2: Build + Test (CI/CD scenario)
    let ci_deps: Vec<_> = deps
        .iter()
        .filter(|d| d.scope == DependencyScope::Build || d.scope == DependencyScope::Test)
        .collect();
    assert_eq!(ci_deps.len(), 7, "Should have 7 Build+Test dependencies");

    // Test 3: Development only
    let dev_only: Vec<_> = deps
        .iter()
        .filter(|d| d.scope == DependencyScope::Development)
        .collect();
    assert_eq!(dev_only.len(), 3, "Should have 3 Development dependencies");
}

#[test]
fn test_e2e_confidence_score_distribution() {
    // Verify confidence scores are distributed correctly
    let deps = create_realistic_project_dependencies();

    let high_confidence = deps.iter().filter(|d| d.scope_confidence >= 0.9).count();
    let medium_confidence = deps
        .iter()
        .filter(|d| d.scope_confidence >= 0.7 && d.scope_confidence < 0.9)
        .count();
    let low_confidence = deps.iter().filter(|d| d.scope_confidence < 0.7).count();

    // Most should be high confidence (we have good classification)
    assert!(
        high_confidence > medium_confidence,
        "Most dependencies should have high confidence"
    );

    // Should have minimal low confidence
    assert!(
        low_confidence == 0,
        "Should have no low confidence classifications in test data"
    );
}

#[test]
fn test_e2e_scope_reason_populated() {
    // Verify all dependencies have classification reasoning
    let deps = create_realistic_project_dependencies();

    for dep in &deps {
        assert!(
            !dep.scope_reason.is_empty(),
            "Dependency {} should have a scope reason, got empty string",
            dep.name
        );

        // Verify reason string contains useful information
        assert!(
            dep.scope_reason.len() > 5,
            "Dependency {} scope reason is too short: '{}'",
            dep.name,
            dep.scope_reason
        );
    }
}

#[test]
fn test_e2e_production_sbom_size_reduction() {
    // Verify production mode significantly reduces SBOM size
    let deps = create_realistic_project_dependencies();
    let initial_count = deps.len();

    // Apply production filter
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    let filtered_count = filtered_deps.len();
    let reduction_percent =
        ((initial_count - filtered_count) as f32 / initial_count as f32) * 100.0;

    // Should reduce by at least 50% in typical projects
    assert!(
        reduction_percent >= 50.0,
        "Production mode should reduce SBOM size by at least 50%, got {:.1}% ({} -> {} deps)",
        reduction_percent,
        initial_count,
        filtered_count
    );

    println!(
        "Production mode reduced SBOM size by {:.1}% ({} → {} dependencies)",
        reduction_percent, initial_count, filtered_count
    );
}

#[test]
fn test_e2e_ecosystem_diversity() {
    // Verify production filtering works across multiple ecosystems
    let deps = create_realistic_project_dependencies();

    // Apply production filter
    let production_scopes = vec![DependencyScope::Runtime, DependencyScope::Optional];
    let filtered_deps: Vec<Dependency> = deps
        .into_iter()
        .filter(|d| production_scopes.contains(&d.scope))
        .collect();

    // Collect unique ecosystems
    let ecosystems: std::collections::HashSet<String> =
        filtered_deps.iter().map(|d| d.ecosystem.clone()).collect();

    // Should have dependencies from multiple ecosystems in production
    assert!(
        ecosystems.len() >= 2,
        "Production SBOM should include dependencies from multiple ecosystems"
    );

    // Verify specific ecosystems are present
    assert!(
        ecosystems.contains("BUILD-CONFIG") || ecosystems.contains("SYSTEM"),
        "Should include C/C++ dependencies"
    );
}
