//! Ecosystem-Native Scope Extraction (v1.0.6)
//!
//! Extracts scope information directly from ecosystem-native formats.
//! This has the highest confidence (1.0) since it's explicit from the manifest.
//!
//! ## Supported Ecosystems
//!
//! - **Cargo**: [dev-dependencies], [build-dependencies] sections
//! - **NPM**: devDependencies in package.json
//! - **Maven**: <scope>test</scope>, <scope>provided</scope> in pom.xml
//! - **Gradle**: testImplementation, testCompileOnly, etc.
//! - **Python**: requirements-dev.txt, requirements-test.txt conventions

use crate::models::{Dependency, DependencyScope};

/// Extract scope from ecosystem-native formats
///
/// Returns (scope, confidence, reason) if scope can be determined from native format
pub fn extract_native_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    match dep.ecosystem.to_lowercase().as_str() {
        "npm" => extract_npm_scope(dep),
        "cargo" => extract_cargo_scope(dep),
        "pip" => extract_pip_scope(dep),
        "maven" => extract_maven_scope(dep),
        "gradle" => extract_gradle_scope(dep),
        _ => None,
    }
}

/// Extract scope from Cargo.toml sections
fn extract_cargo_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    if let Some(ref source) = dep.source_file {
        if source.contains("[dev-dependencies]") {
            return Some((
                DependencyScope::Development,
                1.0,
                "Cargo.toml [dev-dependencies]".to_string(),
            ));
        }
        if source.contains("[build-dependencies]") {
            return Some((
                DependencyScope::Build,
                1.0,
                "Cargo.toml [build-dependencies]".to_string(),
            ));
        }
    }
    None
}

/// Extract scope from package.json devDependencies
fn extract_npm_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    // Use the is_dev flag set by the npm parser when parsing devDependencies section
    if dep.is_dev {
        return Some((
            DependencyScope::Development,
            1.0,
            "package.json devDependencies".to_string(),
        ));
    }
    None
}

/// Extract scope from Maven pom.xml
fn extract_maven_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    if let Some(ref source) = dep.source_file {
        if source.contains("scope:test") {
            return Some((DependencyScope::Test, 1.0, "Maven test scope".to_string()));
        }
        if source.contains("scope:provided") {
            return Some((
                DependencyScope::Provided,
                1.0,
                "Maven provided scope".to_string(),
            ));
        }
        if source.contains("scope:runtime") {
            return Some((
                DependencyScope::Runtime,
                1.0,
                "Maven runtime scope".to_string(),
            ));
        }
        if source.contains("scope:compile") {
            return Some((
                DependencyScope::Runtime,
                1.0,
                "Maven compile scope".to_string(),
            ));
        }
    }
    None
}

/// Extract scope from Gradle dependencies
fn extract_gradle_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    if let Some(ref source) = dep.source_file {
        // Test dependencies
        if source.contains("testImplementation")
            || source.contains("testCompileOnly")
            || source.contains("testRuntimeOnly")
        {
            return Some((
                DependencyScope::Test,
                1.0,
                "Gradle test dependency".to_string(),
            ));
        }

        // Provided dependencies
        if source.contains("compileOnly") {
            return Some((
                DependencyScope::Provided,
                1.0,
                "Gradle compileOnly".to_string(),
            ));
        }
    }
    None
}

/// Extract scope from Python requirements files
fn extract_pip_scope(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    if let Some(ref source) = dep.source_file {
        // Development requirements
        if source.contains("requirements-dev.txt") || source.contains("dev-requirements.txt") {
            return Some((
                DependencyScope::Development,
                0.9,
                "requirements-dev.txt convention".to_string(),
            ));
        }

        // Test requirements
        if source.contains("requirements-test.txt") || source.contains("test-requirements.txt") {
            return Some((
                DependencyScope::Test,
                0.9,
                "requirements-test.txt convention".to_string(),
            ));
        }

        // Build requirements
        if source.contains("requirements-build.txt") || source.contains("build-requirements.txt") {
            return Some((
                DependencyScope::Build,
                0.9,
                "requirements-build.txt convention".to_string(),
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DependencySource;

    fn create_test_dep(source_file: String) -> Dependency {
        Dependency {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "CARGO".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            source_file: Some(source_file),
            ..Default::default()
        }
    }

    #[test]
    fn test_extract_cargo_dev_dependencies() {
        let dep = create_test_dep("Cargo.toml [dev-dependencies]".to_string());
        let result = extract_cargo_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Development);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_extract_cargo_build_dependencies() {
        let dep = create_test_dep("Cargo.toml [build-dependencies]".to_string());
        let result = extract_cargo_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Build);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_extract_npm_dev_dependencies() {
        let mut dep = create_test_dep("package.json".to_string());
        dep.ecosystem = "NPM".to_string();
        dep.is_dev = true; // Set by npm parser when parsing devDependencies section

        let result = extract_npm_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Development);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_extract_maven_test_scope() {
        let mut dep = create_test_dep("pom.xml".to_string());
        dep.ecosystem = "MAVEN".to_string();
        dep.source_file = Some("pom.xml scope:test".to_string());

        let result = extract_maven_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Test);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_extract_pip_dev_requirements() {
        let mut dep = create_test_dep("requirements-dev.txt".to_string());
        dep.ecosystem = "PIP".to_string();

        let result = extract_pip_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Development);
        assert_eq!(conf, 0.9);
    }

    #[test]
    fn test_extract_pip_test_requirements() {
        let mut dep = create_test_dep("requirements-test.txt".to_string());
        dep.ecosystem = "PIP".to_string();

        let result = extract_pip_scope(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Test);
        assert_eq!(conf, 0.9);
    }
}
