use radeis_sc2sbom::cli::VendorMode;
use radeis_sc2sbom::scanner::{is_vendor_directory, resolve_component_dir, scan_directory};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_is_vendor_directory() {
    assert!(is_vendor_directory(Path::new("node_modules")));
    assert!(is_vendor_directory(Path::new("/path/to/node_modules")));
    assert!(is_vendor_directory(Path::new("vendor")));
    assert!(is_vendor_directory(Path::new(".venv")));
    assert!(is_vendor_directory(Path::new("__pycache__")));
    assert!(is_vendor_directory(Path::new("target")));

    assert!(!is_vendor_directory(Path::new("src")));
    assert!(!is_vendor_directory(Path::new("lib")));
    assert!(!is_vendor_directory(Path::new("test")));
}

#[test]
fn test_vendor_mode_skip() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create regular file
    let package_json = temp_path.join("package.json");
    fs::write(&package_json, r#"{"dependencies": {"express": "^4.17.1"}}"#).unwrap();

    // Create node_modules with package.json
    let node_modules = temp_path.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    let vendor_package = node_modules.join("package.json");
    fs::write(
        &vendor_package,
        r#"{"dependencies": {"lodash": "^4.17.21"}}"#,
    )
    .unwrap();

    let scan_context = scan_directory(
        temp_path,
        &VendorMode::Skip,
        &[],
        None,
        true,
        3,
        true,
        false,
        false,
        false,
        false,
        None,
        Some(0),
        false, // is_autosar
    )
    .unwrap();
    let deps = scan_context.dependencies;

    // Should only find express, not lodash from node_modules
    assert!(deps.iter().any(|d| d.name == "express"));
    assert!(!deps.iter().any(|d| d.name == "lodash"));
}

#[test]
fn test_vendor_mode_include() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create regular file
    let package_json = temp_path.join("package.json");
    fs::write(&package_json, r#"{"dependencies": {"express": "^4.17.1"}}"#).unwrap();

    // Create node_modules with package.json
    let node_modules = temp_path.join("node_modules");
    fs::create_dir(&node_modules).unwrap();
    let vendor_package = node_modules.join("package.json");
    fs::write(
        &vendor_package,
        r#"{"dependencies": {"lodash": "^4.17.21"}}"#,
    )
    .unwrap();

    let scan_context = scan_directory(
        temp_path,
        &VendorMode::Include,
        &[],
        None,
        true,
        3,
        true,
        false,
        false,
        false,
        false,
        None,
        None,
        false, // is_autosar
    )
    .unwrap();
    let deps = scan_context.dependencies;

    // Should find both express and lodash
    assert!(deps.iter().any(|d| d.name == "express"));
    assert!(deps.iter().any(|d| d.name == "lodash"));
}

#[test]
fn test_custom_exclusions() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create package.json in root
    let package_json = temp_path.join("package.json");
    fs::write(&package_json, r#"{"dependencies": {"express": "^4.17.1"}}"#).unwrap();

    // Create test directory with package.json
    let test_dir = temp_path.join("test");
    fs::create_dir(&test_dir).unwrap();
    let test_package = test_dir.join("package.json");
    fs::write(&test_package, r#"{"dependencies": {"mocha": "^9.0.0"}}"#).unwrap();

    let scan_context = scan_directory(
        temp_path,
        &VendorMode::Include,
        &["test".to_string()],
        None,
        true,
        3,
        true,
        false,
        false,
        false,
        false,
        None,
        None,
        false, // is_autosar
    )
    .unwrap();
    let deps = scan_context.dependencies;

    // Should find express but not mocha (excluded)
    assert!(deps.iter().any(|d| d.name == "express"));
    assert!(!deps.iter().any(|d| d.name == "mocha"));
}

#[cfg(test)]
mod component_dir_resolution {
    use super::*;
    use std::fs;

    #[test]
    fn test_component_with_matching_subdir_is_resolved() {
        let tmp = TempDir::new().unwrap();
        let subdir = tmp.path().join("brotlidec");
        fs::create_dir(&subdir).unwrap();

        let result = resolve_component_dir(tmp.path(), "brotlidec");
        assert_eq!(result, Some(subdir));
    }

    #[test]
    fn test_component_without_matching_subdir_returns_none() {
        let tmp = TempDir::new().unwrap();
        // No subdir created — external/system dep.

        let result = resolve_component_dir(tmp.path(), "nghttp2");
        assert!(result.is_none());
    }
}
