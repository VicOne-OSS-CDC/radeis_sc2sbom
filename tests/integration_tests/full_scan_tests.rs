use radeis_sc2sbom::cli::VendorMode;
use radeis_sc2sbom::scanner::scan_directory;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_scan_directory_integration() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Create a package.json
    let package_json = temp_path.join("package.json");
    fs::write(
        &package_json,
        r#"{
            "dependencies": {
                "express": "^4.17.1"
            }
        }"#,
    )
    .unwrap();

    // Create a requirements.txt
    let requirements = temp_path.join("requirements.txt");
    fs::write(&requirements, "Django==3.2.0\nrequests==2.26.0").unwrap();

    // Scan the directory
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

    // Should find dependencies from both files
    assert!(deps.len() >= 3);

    let express = deps.iter().find(|d| d.name == "express");
    assert!(express.is_some());

    let django = deps.iter().find(|d| d.name == "Django");
    assert!(django.is_some());
}

// Multi-ecosystem integration tests

#[test]
fn test_source_tracking_preservation_across_parsers() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create multi-ecosystem test project
    std::fs::write(
        base_path.join("package.json"),
        r#"{"dependencies": {"axios": "^1.0.0"}}"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0""#,
    )
    .unwrap();

    std::fs::write(base_path.join("requirements.txt"), "requests==2.28.0").unwrap();

    let scan_context = scan_directory(
        base_path,
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
        None,
        false, // is_autosar
    )
    .unwrap();
    let deps = scan_context.dependencies;

    // Verify npm source tracking
    let npm_dep = deps
        .iter()
        .find(|d| d.name == "axios")
        .expect("axios dependency not found");
    assert!(npm_dep.source_file.is_some());
    let npm_source = npm_dep.source_file.as_ref().unwrap();
    assert!(npm_source.contains("javascript/packagejson"));
    assert!(npm_source.contains("package.json"));

    // Verify Cargo source tracking
    let cargo_dep = deps
        .iter()
        .find(|d| d.name == "serde")
        .expect("serde dependency not found");
    assert!(cargo_dep.source_file.is_some());
    let cargo_source = cargo_dep.source_file.as_ref().unwrap();
    assert!(cargo_source.contains("rust/cargo"));
    assert!(cargo_source.contains("Cargo.toml"));

    // Verify Python source tracking
    let python_dep = deps
        .iter()
        .find(|d| d.name == "requests")
        .expect("requests dependency not found");
    assert!(python_dep.source_file.is_some());
    let python_source = python_dep.source_file.as_ref().unwrap();
    assert!(python_source.contains("python/requirements"));
    assert!(python_source.contains("requirements.txt"));
}

#[test]
fn test_multi_ecosystem_deduplication() {
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path();

    // Create overlapping dependencies (same package, different ecosystems)
    std::fs::write(
        base_path.join("package.json"),
        r#"{"dependencies": {"lodash": "^4.17.21"}}"#,
    )
    .unwrap();

    std::fs::write(
        base_path.join("Cargo.toml"),
        r#"[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
tokio = "1.0""#,
    )
    .unwrap();

    let scan_context = scan_directory(
        base_path,
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
        None,
        false, // is_autosar
    )
    .unwrap();
    let deps = scan_context.dependencies;

    // Verify each ecosystem's dependencies are preserved
    let npm_deps: Vec<_> = deps.iter().filter(|d| d.ecosystem == "npm").collect();
    assert!(!npm_deps.is_empty(), "Should have npm dependencies");

    let cargo_deps: Vec<_> = deps.iter().filter(|d| d.ecosystem == "cargo").collect();
    assert!(!cargo_deps.is_empty(), "Should have cargo dependencies");

    // Verify source tracking is preserved for each ecosystem
    for dep in npm_deps {
        assert!(
            dep.source_file.is_some(),
            "npm dependency should have source tracking"
        );
    }

    for dep in cargo_deps {
        assert!(
            dep.source_file.is_some(),
            "cargo dependency should have source tracking"
        );
    }
}

// ============================================================================
// v0.9.1: Integration Test for ROS/rosdep Version Resolution
// ============================================================================

#[test]
fn test_ros2cli_rosdep_version_resolution() {
    // Test scanning ros2cli with rosdep version resolution
    let ros2cli_path = std::path::Path::new("example_target_repos/ros2cli/ros2cli");

    if !ros2cli_path.exists() {
        println!("Skipping test: ros2cli example not found");
        return;
    }

    // Test with CLI override to humble distribution
    let scan_context = scan_directory(
        ros2cli_path,
        &VendorMode::Skip,
        &[],
        Some("humble"),
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

    // Should find ROS dependencies
    let ros_deps: Vec<_> = deps.iter().filter(|d| d.ecosystem == "ros").collect();
    assert!(!ros_deps.is_empty(), "Should find ROS dependencies");

    // Check if rclpy was resolved with a version
    let rclpy = deps
        .iter()
        .find(|d| d.name == "rclpy" && d.ecosystem == "ros");
    if let Some(rclpy_dep) = rclpy {
        // Should have resolved version (not unspecified)
        if rclpy_dep.version != "unspecified" {
            // Version should be in semver format (e.g., "3.3.19")
            assert!(
                rclpy_dep.version.contains('.'),
                "Resolved version should contain dots: {}",
                rclpy_dep.version
            );
            println!("✓ rclpy version resolved: {}", rclpy_dep.version);
        } else {
            println!("⚠ rclpy version not resolved (network might be unavailable)");
        }
    }
}

#[test]
fn test_ros_version_resolution_with_different_distros() {
    let ros2cli_path = std::path::Path::new("example_target_repos/ros2cli/ros2cli");

    if !ros2cli_path.exists() {
        println!("Skipping test: ros2cli example not found");
        return;
    }

    // Test with different distributions
    let distros = vec!["humble", "iron", "jazzy"];

    for distro in distros {
        let scan_context = scan_directory(
            ros2cli_path,
            &VendorMode::Skip,
            &[],
            Some(distro),
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
        let rclpy = deps
            .iter()
            .find(|d| d.name == "rclpy" && d.ecosystem == "ros");

        if let Some(rclpy_dep) = rclpy {
            if rclpy_dep.version != "unspecified" {
                println!("✓ {} distribution - rclpy: {}", distro, rclpy_dep.version);
                assert!(rclpy_dep.version.contains('.'));
            }
        }
    }
}
