//! Dependency Scope Classification Module (v1.0.6)
//!
//! Automatically classifies dependencies into scopes (Runtime/Build/Test/Development/Optional/Provided)
//! using multiple strategies:
//!
//! ## Classification Pipeline (Priority Order)
//!
//! 0. **Ecosystem-Native Extraction** (confidence: 1.0)
//!    - Cargo.toml [dev-dependencies], [build-dependencies]
//!    - package.json devDependencies
//!    - Maven pom.xml <scope>
//!    - Python requirements-dev.txt, requirements-test.txt
//!
//! 1. **Ecosystem-Based Rules** (confidence: 0.6-0.9)
//!    - PIP → usually left unclassified here; most PIP deps fall through to
//!      name/directory heuristics, with only strong signals mapped (e.g., to Development)
//!    - MESON-WRAP → Build
//!    - GIT-SUBMODULE → Provided
//!    - BUILD-CONFIG → Build (refined later with link analysis)
//!    - SYSTEM → typically not directly scoped here; many SYSTEM deps rely on
//!      name/directory heuristics and link analysis to determine Runtime vs Build
//!
//! 2. **Name-Based Heuristics** (confidence: 0.7-1.0)
//!    - Test frameworks: unity, gtest, pytest, junit → Test
//!    - Build tools: cmake, meson, ninja, gcc → Build
//!    - Dev tools: eslint, prettier, black → Development
//!    - Runtime libs: zlib, openssl, curl → Runtime
//!
//! 3. **Directory-Based Analysis** (confidence: 0.6-0.7)
//!    - /test/, /tests/ → Test
//!    - /3rd_party/, /toolchains/ → Build
//!    - /scripts/, /tools/ → Development
//!
//! 4. **Link Analysis Refinement** (confidence: 0.9)
//!    - BUILD-CONFIG packages linked by SYSTEM → Runtime
//!    - Enhanced library name normalization
//!
//! 5. **Default Fallback** (confidence: 0.3)
//!    - Unmatched → Runtime (conservative)

pub mod autosar;
pub mod ecosystem;
pub mod rules;

use crate::models::{Dependency, DependencyScope};

/// Classify all dependencies using the multi-strategy pipeline
///
/// # Arguments
/// * `deps` - Vector of dependencies to classify
///
/// # Returns
/// * Vector of dependencies with scope, confidence, and reasoning
pub fn classify_dependencies(deps: Vec<Dependency>) -> Vec<Dependency> {
    deps.into_iter()
        .map(|dep| classify_single_dependency(dep))
        .collect()
}

/// Classify a single dependency
fn classify_single_dependency(dep: Dependency) -> Dependency {
    // Step -1: Respect pre-existing high-confidence scope from parser,
    // including parser-assigned Runtime/default scope values.
    // (e.g., Gradle parser sets scope with confidence 1.0 based on configuration type)
    if dep.scope_confidence >= 1.0 {
        return dep;
    }

    // Step 0: Ecosystem-native scope extraction (HIGHEST confidence)
    if let Some((scope, conf, reason)) = ecosystem::extract_native_scope(&dep) {
        return assign_scope(dep, scope, conf, reason);
    }

    // Step 1: Ecosystem-based rules (high confidence)
    if let Some((scope, conf, reason)) = rules::classify_by_ecosystem(&dep) {
        return assign_scope(dep, scope, conf, reason);
    }

    // Step 2: Name-based heuristics (medium-high confidence)
    if let Some((scope, conf, reason)) = rules::classify_by_name(&dep) {
        return assign_scope(dep, scope, conf, reason);
    }

    // Step 3: Directory-based (medium confidence)
    if let Some((scope, conf, reason)) = rules::classify_by_directory(&dep) {
        return assign_scope(dep, scope, conf, reason);
    }

    // Step 4: Default to Runtime (low confidence)
    assign_scope(dep, DependencyScope::Runtime, 0.3, "Default (no match)")
}

/// Assign scope, confidence, and reason to a dependency
fn assign_scope(
    mut dep: Dependency,
    scope: DependencyScope,
    confidence: f32,
    reason: impl Into<String>,
) -> Dependency {
    dep.scope = scope;
    dep.scope_confidence = confidence;
    dep.scope_reason = reason.into();
    dep
}

/// Refine BUILD-CONFIG classification using link analysis
///
/// Upgrades BUILD-CONFIG packages to Runtime if they are linked by SYSTEM libraries
pub fn refine_build_config_classification(deps: &mut Vec<Dependency>) {
    use std::collections::HashSet;

    // Collect all SYSTEM runtime libraries with enhanced normalization
    let system_libs: HashSet<String> = deps
        .iter()
        .filter(|d| d.ecosystem.eq_ignore_ascii_case("system"))
        .flat_map(|d| rules::normalize_lib_name_enhanced(&d.name))
        .collect();

    // Upgrade BUILD-CONFIG packages that are actually linked
    for dep in deps.iter_mut() {
        if dep.ecosystem == "BUILD-CONFIG" && dep.scope == DependencyScope::Build {
            let candidates = rules::normalize_lib_name_enhanced(&dep.name);

            if candidates.iter().any(|c| system_libs.contains(c)) {
                dep.scope = DependencyScope::Runtime;
                dep.scope_confidence = 0.9;
                dep.scope_reason =
                    "BUILD-CONFIG linked at runtime (matched SYSTEM lib)".to_string();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DependencySource;

    fn create_test_dep(name: &str, ecosystem: &str, source_file: Option<String>) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            ecosystem: ecosystem.to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            source_file,
            ..Default::default()
        }
    }

    #[test]
    fn test_classify_test_framework() {
        let dep = create_test_dep("pytest", "PIP", None);
        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Test);
        assert!(classified.scope_confidence >= 0.9);
    }

    #[test]
    fn test_classify_build_tool() {
        let dep = create_test_dep("cmake", "system", None);
        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Build);
        assert_eq!(classified.scope_confidence, 1.0);
    }

    #[test]
    fn test_classify_runtime_library() {
        let dep = create_test_dep("zlib", "system", None);
        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Runtime);
    }

    #[test]
    fn test_classify_cargo_dev_dependency() {
        let dep = create_test_dep(
            "serde_test",
            "CARGO",
            Some("Cargo.toml [dev-dependencies]".to_string()),
        );
        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Development);
        assert_eq!(classified.scope_confidence, 1.0);
    }

    #[test]
    fn test_refine_build_config_linked() {
        let mut deps = vec![
            create_test_dep("zlib", "BUILD-CONFIG", None),
            create_test_dep("z", "system", None),
        ];

        // Classify first
        deps = classify_dependencies(deps);

        // zlib should be Build initially
        assert_eq!(
            deps.iter().find(|d| d.name == "zlib").unwrap().scope,
            DependencyScope::Build
        );

        // Refine with link analysis
        refine_build_config_classification(&mut deps);

        // zlib should now be Runtime (linked by system library 'z')
        assert_eq!(
            deps.iter().find(|d| d.name == "zlib").unwrap().scope,
            DependencyScope::Runtime
        );
    }

    #[test]
    fn test_classifier_preserves_high_confidence_parser_scope() {
        // Gradle parser sets scope with confidence 1.0 — classifier should not overwrite
        let mut dep = create_test_dep(
            "org.springframework.boot:spring-boot-starter-test",
            "maven",
            Some("java/gradle config:testImplementation".to_string()),
        );
        dep.scope = DependencyScope::Test;
        dep.scope_confidence = 1.0;
        dep.scope_reason = "Gradle testImplementation configuration".to_string();

        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Test);
        assert_eq!(classified.scope_confidence, 1.0);
    }

    #[test]
    fn test_classifier_preserves_high_confidence_runtime_scope() {
        // Gradle implementation config → Runtime at confidence 1.0, should be preserved
        let mut dep = create_test_dep(
            "org.springframework.boot:spring-boot-starter-web",
            "maven",
            Some("java/gradle config:implementation".to_string()),
        );
        dep.scope = DependencyScope::Runtime;
        dep.scope_confidence = 1.0;
        dep.scope_reason = "Gradle implementation configuration".to_string();

        let classified = classify_single_dependency(dep);
        assert_eq!(classified.scope, DependencyScope::Runtime);
        assert_eq!(classified.scope_confidence, 1.0);
        assert!(classified.scope_reason.contains("Gradle"));
    }

    #[test]
    fn test_classifier_still_classifies_default_scope_deps() {
        // A dep with default scope (Runtime, confidence 0.0) should still be classified
        let dep = create_test_dep("some-unknown-lib", "maven", None);
        let classified = classify_single_dependency(dep);
        // Unknown dep falls through to default Runtime at low confidence
        assert_eq!(classified.scope, DependencyScope::Runtime);
        assert_eq!(classified.scope_confidence, 0.3);
    }
}
