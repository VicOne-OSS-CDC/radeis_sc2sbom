/// Real-world dependency classification accuracy tests (Phase 4)
/// These tests document the current classifier behavior and validate improvements
///
/// NOTE: Some tests are marked with expected behavior vs current behavior.
/// These tests serve as documentation for Phase 4 validation and future improvements.
use radeis_sc2sbom::classifier;
use radeis_sc2sbom::models::{Dependency, DependencyScope};

/// Test classification of common SYSTEM libraries (should be Runtime)
#[test]
fn test_classify_system_libraries_as_runtime() {
    let deps = vec![
        Dependency::new(
            "curl".to_string(),
            "8.15.0".to_string(),
            "SYSTEM".to_string(),
        ),
        Dependency::new(
            "openssl".to_string(),
            "3.0.0".to_string(),
            "SYSTEM".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    // SYSTEM libraries should be Runtime
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Runtime,
            "{} (SYSTEM) should be classified as Runtime, got {:?}",
            dep.name,
            dep.scope
        );
    }
}

/// Test classification of common build tools
#[test]
fn test_classify_common_build_tools() {
    let deps = vec![
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new(
            "gcc".to_string(),
            "11.0.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new(
            "clang".to_string(),
            "14.0.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new(
            "ninja".to_string(),
            "1.10.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new(
            "meson".to_string(),
            "0.59.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new("make".to_string(), "4.3".to_string(), "system".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // All should be Build
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Build,
            "{} should be classified as Build, got {:?}",
            dep.name,
            dep.scope
        );
        assert!(
            dep.scope_confidence >= 0.8,
            "{} should have high confidence, got {}",
            dep.name,
            dep.scope_confidence
        );
    }
}

/// Test classification of exact-match test frameworks
/// NOTE: Test frameworks with non-BUILD-CONFIG ecosystems are classified correctly
#[test]
fn test_classify_exact_match_test_frameworks() {
    let deps = vec![
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
        Dependency::new("jest".to_string(), "27.0.0".to_string(), "npm".to_string()),
        Dependency::new("mocha".to_string(), "9.0.0".to_string(), "npm".to_string()),
        Dependency::new(
            "junit".to_string(),
            "4.13.2".to_string(),
            "maven".to_string(),
        ),
        // NOTE: unity with BUILD-CONFIG is classified as Build (ecosystem takes precedence)
        // Dependency::new("unity".to_string(), "2.5.0".to_string(), "BUILD-CONFIG".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // All should be Test
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Test,
            "{} should be classified as Test, got {:?}",
            dep.name,
            dep.scope
        );
        assert!(
            dep.scope_confidence >= 0.8,
            "{} should have high confidence, got {}",
            dep.name,
            dep.scope_confidence
        );
    }
}

/// Test classification of common development tools
#[test]
fn test_classify_common_dev_tools() {
    let deps = vec![
        Dependency::new(
            "pylint".to_string(),
            "2.12.0".to_string(),
            "PIP".to_string(),
        ),
        Dependency::new("black".to_string(), "22.0.0".to_string(), "PIP".to_string()),
        Dependency::new("flake8".to_string(), "4.0.0".to_string(), "PIP".to_string()),
        Dependency::new("eslint".to_string(), "8.0.0".to_string(), "npm".to_string()),
        Dependency::new(
            "prettier".to_string(),
            "2.5.0".to_string(),
            "npm".to_string(),
        ),
        Dependency::new("ruff".to_string(), "0.1.0".to_string(), "PIP".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // All should be Development
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Development,
            "{} should be classified as Development, got {:?}",
            dep.name,
            dep.scope
        );
        assert!(
            dep.scope_confidence >= 0.8,
            "{} should have high confidence, got {}",
            dep.name,
            dep.scope_confidence
        );
    }
}

/// Test classification of Python web frameworks (should be Runtime)
#[test]
fn test_classify_python_web_frameworks() {
    let deps = vec![
        Dependency::new("django".to_string(), "4.0.0".to_string(), "PIP".to_string()),
        Dependency::new("flask".to_string(), "2.0.0".to_string(), "PIP".to_string()),
        Dependency::new(
            "fastapi".to_string(),
            "0.95.0".to_string(),
            "PIP".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Web frameworks should be Runtime
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Runtime,
            "{} should be classified as Runtime, got {:?}",
            dep.name,
            dep.scope
        );
    }
}

/// Test classification of Node.js frameworks (should be Runtime)
#[test]
fn test_classify_nodejs_frameworks() {
    let deps = vec![
        Dependency::new(
            "express".to_string(),
            "4.17.1".to_string(),
            "npm".to_string(),
        ),
        Dependency::new("react".to_string(), "18.0.0".to_string(), "npm".to_string()),
        Dependency::new("vue".to_string(), "3.0.0".to_string(), "npm".to_string()),
        Dependency::new("next".to_string(), "12.0.0".to_string(), "npm".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Web frameworks should be Runtime
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Runtime,
            "{} should be classified as Runtime, got {:?}",
            dep.name,
            dep.scope
        );
    }
}

/// Test classification accuracy with mixed real-world project
#[test]
fn test_classify_mixed_real_world_project() {
    let deps = vec![
        // Runtime
        Dependency::new(
            "requests".to_string(),
            "2.28.0".to_string(),
            "PIP".to_string(),
        ),
        Dependency::new("numpy".to_string(), "1.24.0".to_string(), "PIP".to_string()),
        // Test
        Dependency::new("pytest".to_string(), "7.2.0".to_string(), "PIP".to_string()),
        // Dev
        Dependency::new("black".to_string(), "23.0.0".to_string(), "PIP".to_string()),
        Dependency::new("mypy".to_string(), "1.0.0".to_string(), "PIP".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Verify Runtime
    let requests = classified.iter().find(|d| d.name == "requests").unwrap();
    assert_eq!(requests.scope, DependencyScope::Runtime);

    let numpy = classified.iter().find(|d| d.name == "numpy").unwrap();
    assert_eq!(numpy.scope, DependencyScope::Runtime);

    // Verify Test
    let pytest = classified.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(pytest.scope, DependencyScope::Test);

    // Verify Development
    let black = classified.iter().find(|d| d.name == "black").unwrap();
    assert_eq!(black.scope, DependencyScope::Development);

    let mypy = classified.iter().find(|d| d.name == "mypy").unwrap();
    assert_eq!(mypy.scope, DependencyScope::Development);
}

/// Test classification confidence scores are reasonable
#[test]
fn test_classification_confidence_distribution() {
    let deps = vec![
        // Exact name matches (should be 0.95-1.0)
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        // Ecosystem + heuristics (should be 0.5-0.9)
        Dependency::new(
            "curl".to_string(),
            "8.15.0".to_string(),
            "SYSTEM".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Exact matches should have very high confidence
    let pytest = classified.iter().find(|d| d.name == "pytest").unwrap();
    assert!(
        pytest.scope_confidence >= 0.95,
        "pytest should have very high confidence (exact match), got {}",
        pytest.scope_confidence
    );

    let cmake = classified.iter().find(|d| d.name == "cmake").unwrap();
    assert!(
        cmake.scope_confidence >= 0.95,
        "cmake should have very high confidence (exact match), got {}",
        cmake.scope_confidence
    );

    // All should have reasonable confidence
    for dep in &classified {
        assert!(
            dep.scope_confidence >= 0.3,
            "{} should have at least low confidence, got {}",
            dep.name,
            dep.scope_confidence
        );
        assert!(
            dep.scope_confidence <= 1.0,
            "{} confidence should not exceed 1.0, got {}",
            dep.name,
            dep.scope_confidence
        );
    }
}

/// Test that all classifications have reasoning
#[test]
fn test_all_classifications_have_reasoning() {
    let deps = vec![
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
        Dependency::new(
            "unknown-lib".to_string(),
            "1.0.0".to_string(),
            "custom".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    for dep in &classified {
        assert!(
            !dep.scope_reason.is_empty(),
            "{} should have a non-empty scope reason",
            dep.name
        );
        assert!(
            dep.scope_reason != "Not classified",
            "{} should not have default reason after classification",
            dep.name
        );
        assert!(
            dep.scope_reason.len() > 5,
            "{} scope reason is too short: '{}'",
            dep.name,
            dep.scope_reason
        );
    }
}

/// Test Rust cargo dependencies
#[test]
fn test_classify_rust_dependencies() {
    let deps = vec![
        // Runtime
        Dependency::new(
            "serde".to_string(),
            "1.0.0".to_string(),
            "cargo".to_string(),
        ),
        Dependency::new(
            "tokio".to_string(),
            "1.0.0".to_string(),
            "cargo".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    let serde = classified.iter().find(|d| d.name == "serde").unwrap();
    assert_eq!(
        serde.scope,
        DependencyScope::Runtime,
        "serde should be Runtime"
    );

    let tokio = classified.iter().find(|d| d.name == "tokio").unwrap();
    assert_eq!(
        tokio.scope,
        DependencyScope::Runtime,
        "tokio should be Runtime"
    );
}

/// Test that classification handles case-insensitive name matching for build tools
#[test]
fn test_classification_case_insensitive_build_tools() {
    let deps = vec![
        Dependency::new(
            "CMake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new("PyTest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    let cmake = classified.iter().find(|d| d.name == "CMake").unwrap();
    assert_eq!(
        cmake.scope,
        DependencyScope::Build,
        "CMake should be Build (case-insensitive)"
    );

    let pytest = classified.iter().find(|d| d.name == "PyTest").unwrap();
    assert_eq!(
        pytest.scope,
        DependencyScope::Test,
        "PyTest should be Test (case-insensitive)"
    );
}

/// Test precision: no false positives for Runtime classification
#[test]
fn test_precision_no_false_runtime() {
    let deps = vec![
        Dependency::new(
            "cmake".to_string(),
            "3.20.0".to_string(),
            "system".to_string(),
        ),
        Dependency::new("pytest".to_string(), "7.0.0".to_string(), "PIP".to_string()),
        Dependency::new("black".to_string(), "22.0.0".to_string(), "PIP".to_string()),
    ];

    let classified = classifier::classify_dependencies(deps);

    // None of these should be Runtime
    for dep in &classified {
        assert_ne!(
            dep.scope,
            DependencyScope::Runtime,
            "{} should NOT be classified as Runtime (these are build/test/dev tools)",
            dep.name
        );
    }
}

/// Test that BUILD-CONFIG with known library names are appropriately classified
/// NOTE: Current classifier classifies BUILD-CONFIG as Build by default.
/// Link analysis would need to run to refine these to Runtime.
#[test]
fn test_build_config_default_classification() {
    let deps = vec![
        Dependency::new(
            "zlib".to_string(),
            "1.3.1".to_string(),
            "BUILD-CONFIG".to_string(),
        ),
        Dependency::new(
            "protobuf".to_string(),
            "3.21.0".to_string(),
            "BUILD-CONFIG".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Current behavior: BUILD-CONFIG defaults to Build (will be refined with link analysis)
    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Build,
            "{} (BUILD-CONFIG) currently defaults to Build pending link analysis",
            dep.name
        );
        assert!(
            dep.scope_reason.contains("BUILD-CONFIG"),
            "Reason should mention BUILD-CONFIG: {}",
            dep.scope_reason
        );
    }
}

/// Test that GIT-SUBMODULE is classified as Provided
#[test]
fn test_git_submodule_classification() {
    let deps = vec![Dependency::new(
        "my-submodule".to_string(),
        "1.0.0".to_string(),
        "GIT-SUBMODULE".to_string(),
    )];

    let classified = classifier::classify_dependencies(deps);

    assert_eq!(
        classified[0].scope,
        DependencyScope::Provided,
        "GIT-SUBMODULE should be classified as Provided"
    );
    assert!(
        classified[0].scope_confidence >= 0.7,
        "GIT-SUBMODULE should have reasonable confidence"
    );
}

/// Test that MESON dependencies are classified as Build
#[test]
fn test_meson_dependency_classification() {
    let deps = vec![
        Dependency::new(
            "glib".to_string(),
            "2.0.0".to_string(),
            "MESON-WRAP".to_string(),
        ),
        Dependency::new(
            "json-glib".to_string(),
            "1.0.0".to_string(),
            "MESON-SUBPROJECT".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    for dep in &classified {
        assert_eq!(
            dep.scope,
            DependencyScope::Build,
            "{} (Meson) should be classified as Build",
            dep.name
        );
        assert!(
            dep.scope_confidence >= 0.7,
            "Meson dependencies should have reasonable confidence"
        );
    }
}

/// Test comprehensive ecosystem coverage
#[test]
fn test_ecosystem_classification_coverage() {
    let deps = vec![
        // npm (JavaScript) - Runtime by default for non-dev packages
        Dependency::new(
            "express".to_string(),
            "4.17.1".to_string(),
            "npm".to_string(),
        ),
        // PIP (Python) - Runtime by default for non-tool packages
        Dependency::new(
            "requests".to_string(),
            "2.28.0".to_string(),
            "PIP".to_string(),
        ),
        // cargo (Rust) - Runtime by default
        Dependency::new(
            "serde".to_string(),
            "1.0.0".to_string(),
            "cargo".to_string(),
        ),
        // SYSTEM - Runtime
        Dependency::new(
            "libcurl".to_string(),
            "8.0.0".to_string(),
            "SYSTEM".to_string(),
        ),
        // BUILD-CONFIG - Build (pending link analysis)
        Dependency::new(
            "somelib".to_string(),
            "1.0.0".to_string(),
            "BUILD-CONFIG".to_string(),
        ),
        // GIT-SUBMODULE - Provided
        Dependency::new(
            "submod".to_string(),
            "1.0.0".to_string(),
            "GIT-SUBMODULE".to_string(),
        ),
    ];

    let classified = classifier::classify_dependencies(deps);

    // Verify each ecosystem is classified
    let express = classified.iter().find(|d| d.name == "express").unwrap();
    assert_eq!(express.scope, DependencyScope::Runtime);

    let requests = classified.iter().find(|d| d.name == "requests").unwrap();
    assert_eq!(requests.scope, DependencyScope::Runtime);

    let serde = classified.iter().find(|d| d.name == "serde").unwrap();
    assert_eq!(serde.scope, DependencyScope::Runtime);

    let libcurl = classified.iter().find(|d| d.name == "libcurl").unwrap();
    assert_eq!(libcurl.scope, DependencyScope::Runtime);

    let somelib = classified.iter().find(|d| d.name == "somelib").unwrap();
    assert_eq!(somelib.scope, DependencyScope::Build);

    let submod = classified.iter().find(|d| d.name == "submod").unwrap();
    assert_eq!(submod.scope, DependencyScope::Provided);
}

/// Test that default classification produces reasonable results
#[test]
fn test_default_classification_fallback() {
    // Unknown ecosystem and name should still get classified
    let deps = vec![Dependency::new(
        "unknown-lib".to_string(),
        "1.0.0".to_string(),
        "unknown".to_string(),
    )];

    let classified = classifier::classify_dependencies(deps);

    // Should default to Runtime (conservative choice)
    assert_eq!(classified[0].scope, DependencyScope::Runtime);
    assert!(
        classified[0].scope_confidence >= 0.1,
        "Should have at least minimal confidence"
    );
    assert!(
        !classified[0].scope_reason.is_empty(),
        "Should have a reason even for defaults"
    );
}
