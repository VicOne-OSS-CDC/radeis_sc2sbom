use radeis_sc2sbom::models::{
    Dependency, DependencyScope, DependencySource, Sbom, ScopeStatistics,
};
use std::path::PathBuf;

/// Helper function to create test dependencies with different scopes
fn create_test_dependencies() -> Vec<Dependency> {
    vec![
        // Runtime dependencies
        Dependency {
            name: "zlib".to_string(),
            version: "1.3.1".to_string(),
            ecosystem: "BUILD-CONFIG".to_string(),
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
            scope: DependencyScope::Runtime,
            scope_confidence: 0.9,
            scope_reason: "BUILD-CONFIG linked at runtime".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        Dependency {
            name: "curl".to_string(),
            version: "8.15.0".to_string(),
            ecosystem: "SYSTEM".to_string(),
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
            scope: DependencyScope::Runtime,
            scope_confidence: 0.8,
            scope_reason: "System library".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        // Build dependencies
        Dependency {
            name: "cmake".to_string(),
            version: "3.20.0".to_string(),
            ecosystem: "system".to_string(),
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
            scope: DependencyScope::Build,
            scope_confidence: 1.0,
            scope_reason: "Build tool".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        Dependency {
            name: "gcc".to_string(),
            version: "11.0.0".to_string(),
            ecosystem: "system".to_string(),
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
            scope: DependencyScope::Build,
            scope_confidence: 1.0,
            scope_reason: "Build tool".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        // Test dependencies
        Dependency {
            name: "pytest".to_string(),
            version: "7.0.0".to_string(),
            ecosystem: "PIP".to_string(),
            source: DependencySource::Manifest,
            is_dev: true,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            scope: DependencyScope::Test,
            scope_confidence: 1.0,
            scope_reason: "Test framework".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        Dependency {
            name: "gtest".to_string(),
            version: "1.11.0".to_string(),
            ecosystem: "BUILD-CONFIG".to_string(),
            source: DependencySource::Manifest,
            is_dev: true,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            scope: DependencyScope::Test,
            scope_confidence: 1.0,
            scope_reason: "Test framework".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        // Development dependencies
        Dependency {
            name: "pylint".to_string(),
            version: "2.12.0".to_string(),
            ecosystem: "PIP".to_string(),
            source: DependencySource::Manifest,
            is_dev: true,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            scope: DependencyScope::Development,
            scope_confidence: 1.0,
            scope_reason: "Dev tool".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        // Optional dependencies
        Dependency {
            name: "optional-lib".to_string(),
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
            scope: DependencyScope::Optional,
            scope_confidence: 0.8,
            scope_reason: "Optional dependency".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
    ]
}

/// Apply scope filtering logic (mimics main.rs filtering)
fn apply_scope_filter(
    deps: Vec<Dependency>,
    filters: Option<Vec<DependencyScope>>,
) -> Vec<Dependency> {
    if let Some(scope_filters) = filters {
        deps.into_iter()
            .filter(|dep| scope_filters.contains(&dep.scope))
            .collect()
    } else {
        deps
    }
}

#[test]
fn test_default_behavior_no_filtering() {
    // Default behavior: no filtering, all packages included (backwards compatible)
    let deps = create_test_dependencies();
    let initial_count = deps.len();

    let filtered = apply_scope_filter(deps, None);

    assert_eq!(
        filtered.len(),
        initial_count,
        "Default behavior should include all packages"
    );
    assert_eq!(filtered.len(), 8, "Should have all 8 test dependencies");
}

#[test]
fn test_production_mode_filters_runtime_and_optional() {
    // Production mode: Runtime + Optional only
    let deps = create_test_dependencies();

    // Simulate production mode: --production flag creates Runtime + Optional filters
    let production_filters = Some(vec![DependencyScope::Runtime, DependencyScope::Optional]);
    let filtered = apply_scope_filter(deps, production_filters);

    // Should have 2 Runtime + 1 Optional = 3 packages
    assert_eq!(
        filtered.len(),
        3,
        "Production mode should filter to 3 packages"
    );

    // Verify only Runtime and Optional packages remain
    assert!(
        filtered
            .iter()
            .any(|d| d.name == "zlib" && d.scope == DependencyScope::Runtime),
        "Should include zlib (Runtime)"
    );
    assert!(
        filtered
            .iter()
            .any(|d| d.name == "curl" && d.scope == DependencyScope::Runtime),
        "Should include curl (Runtime)"
    );
    assert!(
        filtered
            .iter()
            .any(|d| d.name == "optional-lib" && d.scope == DependencyScope::Optional),
        "Should include optional-lib (Optional)"
    );

    // Verify excluded packages
    assert!(
        !filtered.iter().any(|d| d.name == "cmake"),
        "Should exclude cmake (Build)"
    );
    assert!(
        !filtered.iter().any(|d| d.name == "pytest"),
        "Should exclude pytest (Test)"
    );
    assert!(
        !filtered.iter().any(|d| d.name == "pylint"),
        "Should exclude pylint (Development)"
    );
}

#[test]
fn test_custom_scope_filter_single_scope() {
    // Filter for Test scope only
    let deps = create_test_dependencies();
    let test_filter = Some(vec![DependencyScope::Test]);
    let filtered = apply_scope_filter(deps, test_filter);

    // Should have 2 Test packages (pytest, gtest)
    assert_eq!(filtered.len(), 2, "Should filter to 2 Test packages");
    assert!(filtered.iter().any(|d| d.name == "pytest"));
    assert!(filtered.iter().any(|d| d.name == "gtest"));
}

#[test]
fn test_custom_scope_filter_multiple_scopes() {
    // Filter for Runtime + Build scopes
    let deps = create_test_dependencies();
    let filters = Some(vec![DependencyScope::Runtime, DependencyScope::Build]);
    let filtered = apply_scope_filter(deps, filters);

    // Should have 2 Runtime + 2 Build = 4 packages
    assert_eq!(filtered.len(), 4, "Should filter to 4 packages");

    // Verify included packages
    assert!(filtered.iter().any(|d| d.name == "zlib"));
    assert!(filtered.iter().any(|d| d.name == "curl"));
    assert!(filtered.iter().any(|d| d.name == "cmake"));
    assert!(filtered.iter().any(|d| d.name == "gcc"));

    // Verify excluded packages
    assert!(!filtered.iter().any(|d| d.name == "pytest"));
    assert!(!filtered.iter().any(|d| d.name == "pylint"));
}

#[test]
fn test_scope_statistics_calculation() {
    let deps = create_test_dependencies();
    let stats = ScopeStatistics::from_dependencies(&deps);

    // Verify counts
    assert_eq!(stats.runtime, 2, "Should have 2 Runtime packages");
    assert_eq!(stats.build, 2, "Should have 2 Build packages");
    assert_eq!(stats.test, 2, "Should have 2 Test packages");
    assert_eq!(stats.development, 1, "Should have 1 Development package");
    assert_eq!(stats.optional, 1, "Should have 1 Optional package");
    assert_eq!(stats.provided, 0, "Should have 0 Provided packages");
    assert_eq!(stats.total, 8, "Should have 8 total packages");

    // Verify average confidence
    // All confidences: 0.9, 0.8, 1.0, 1.0, 1.0, 1.0, 1.0, 0.8 = 7.5 / 8 = 0.9375
    assert!(
        (stats.avg_confidence - 0.9375).abs() < 0.01,
        "Average confidence should be ~0.9375, got {}",
        stats.avg_confidence
    );
}

#[test]
fn test_scope_statistics_empty_dependencies() {
    let deps: Vec<Dependency> = vec![];
    let stats = ScopeStatistics::from_dependencies(&deps);

    assert_eq!(stats.total, 0);
    assert_eq!(stats.avg_confidence, 0.0);
}

#[test]
fn test_sbom_with_scope_statistics() {
    let deps = create_test_dependencies();
    let scope_stats = Some(ScopeStatistics::from_dependencies(&deps));

    let sbom = Sbom {
        project_path: PathBuf::from("/test/project"),
        generated_at: "2026-03-04T00:00:00Z".to_string(),
        dependencies: deps,
        ros_package: None,
        ros_packages: Vec::new(),
        scope_statistics: scope_stats,
    };

    assert!(sbom.scope_statistics.is_some());
    let stats = sbom.scope_statistics.unwrap();
    assert_eq!(stats.total, 8);
    assert_eq!(stats.runtime, 2);
}

#[test]
fn test_filter_runtime_only() {
    let deps = create_test_dependencies();
    let runtime_filter = Some(vec![DependencyScope::Runtime]);
    let filtered = apply_scope_filter(deps, runtime_filter);

    assert_eq!(filtered.len(), 2, "Should have 2 Runtime packages");
    assert!(filtered.iter().all(|d| d.scope == DependencyScope::Runtime));
}

#[test]
fn test_filter_build_and_test() {
    let deps = create_test_dependencies();
    let filters = Some(vec![DependencyScope::Build, DependencyScope::Test]);
    let filtered = apply_scope_filter(deps, filters);

    assert_eq!(
        filtered.len(),
        4,
        "Should have 2 Build + 2 Test = 4 packages"
    );
    assert!(filtered
        .iter()
        .all(|d| { d.scope == DependencyScope::Build || d.scope == DependencyScope::Test }));
}

#[test]
fn test_filter_development_only() {
    let deps = create_test_dependencies();
    let dev_filter = Some(vec![DependencyScope::Development]);
    let filtered = apply_scope_filter(deps, dev_filter);

    assert_eq!(filtered.len(), 1, "Should have 1 Development package");
    assert_eq!(filtered[0].name, "pylint");
    assert_eq!(filtered[0].scope, DependencyScope::Development);
}

#[test]
fn test_no_matching_scope() {
    let deps = create_test_dependencies();
    // No Provided packages in test data
    let provided_filter = Some(vec![DependencyScope::Provided]);
    let filtered = apply_scope_filter(deps, provided_filter);

    assert_eq!(filtered.len(), 0, "Should have no Provided packages");
}

#[test]
fn test_scope_confidence_values() {
    let deps = create_test_dependencies();

    // Check that all dependencies have reasonable confidence scores
    for dep in &deps {
        assert!(
            dep.scope_confidence >= 0.0 && dep.scope_confidence <= 1.0,
            "Confidence score for {} should be between 0.0 and 1.0, got {}",
            dep.name,
            dep.scope_confidence
        );
        assert!(
            !dep.scope_reason.is_empty(),
            "Scope reason for {} should not be empty",
            dep.name
        );
    }

    // Check high-confidence classifications
    let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(
        pytest.scope_confidence, 1.0,
        "pytest should have 100% confidence"
    );

    let cmake = deps.iter().find(|d| d.name == "cmake").unwrap();
    assert_eq!(
        cmake.scope_confidence, 1.0,
        "cmake should have 100% confidence"
    );
}

#[test]
fn test_scope_statistics_percentages() {
    let deps = create_test_dependencies();
    let stats = ScopeStatistics::from_dependencies(&deps);

    // Calculate percentages
    let runtime_pct = (stats.runtime as f32 / stats.total as f32) * 100.0;
    let build_pct = (stats.build as f32 / stats.total as f32) * 100.0;
    let test_pct = (stats.test as f32 / stats.total as f32) * 100.0;

    // 2/8 = 25%
    assert!((runtime_pct - 25.0).abs() < 0.1, "Runtime should be ~25%");
    // 2/8 = 25%
    assert!((build_pct - 25.0).abs() < 0.1, "Build should be ~25%");
    // 2/8 = 25%
    assert!((test_pct - 25.0).abs() < 0.1, "Test should be ~25%");
}

#[test]
fn test_mixed_confidence_average() {
    // Create dependencies with varying confidence scores
    let deps = vec![
        Dependency {
            name: "high-conf".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "test".to_string(),
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
            scope: DependencyScope::Runtime,
            scope_confidence: 1.0,
            scope_reason: "High confidence".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
        Dependency {
            name: "low-conf".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "test".to_string(),
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
            scope: DependencyScope::Runtime,
            scope_confidence: 0.5,
            scope_reason: "Low confidence".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        },
    ];

    let stats = ScopeStatistics::from_dependencies(&deps);

    // Average should be (1.0 + 0.5) / 2 = 0.75
    assert!(
        (stats.avg_confidence - 0.75).abs() < 0.01,
        "Average should be 0.75"
    );
}
