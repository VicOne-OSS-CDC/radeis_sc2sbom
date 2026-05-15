use radeis_sc2sbom::parsers::ros::parse_ros_package;
use std::fs::File;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

#[test]
fn test_parse_ros_package_basic() {
    let content = r#"<?xml version="1.0"?>
<package format="2">
  <name>test_pkg</name>
  <version>1.0.0</version>
  <exec_depend>rclpy</exec_depend>
  <exec_depend>ros2cli</exec_depend>
  <test_depend>pytest</test_depend>
  <test_depend>ament_flake8</test_depend>
</package>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let (metadata, deps) = parse_ros_package(temp_file.path()).unwrap();

    // Check metadata
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.name, "test_pkg");
    assert_eq!(metadata.version, "1.0.0");

    assert_eq!(deps.len(), 4);

    let rclpy = deps.iter().find(|d| d.name == "rclpy").unwrap();
    assert_eq!(rclpy.ecosystem, "ros");
    assert_eq!(rclpy.version, "unspecified");
    assert_eq!(rclpy.is_dev, false);
    assert_eq!(rclpy.is_direct, true);

    let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(pytest.is_dev, true);
}

#[test]
fn test_parse_ros_package_all_types() {
    let content = r#"<?xml version="1.0"?>
<package format="3">
  <name>test_pkg</name>
  <exec_depend>runtime_dep</exec_depend>
  <build_depend>build_dep</build_depend>
  <buildtool_depend>cmake</buildtool_depend>
  <test_depend>test_dep</test_depend>
  <depend>generic_dep</depend>
  <build_export_depend>export_dep</build_export_depend>
</package>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let (_metadata, deps) = parse_ros_package(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 6);

    // Only test_depend should be marked as dev dependency
    let dev_deps: Vec<_> = deps.iter().filter(|d| d.is_dev).collect();
    assert_eq!(dev_deps.len(), 1);
    assert_eq!(dev_deps[0].name, "test_dep");

    // All should be ecosystem "ros"
    assert!(deps.iter().all(|d| d.ecosystem == "ros"));
}

#[test]
fn test_parse_ros_package_no_dependencies() {
    let content = r#"<?xml version="1.0"?>
<package format="2">
  <name>empty_pkg</name>
  <version>1.0.0</version>
  <description>Package with no dependencies</description>
</package>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let (metadata, deps) = parse_ros_package(temp_file.path()).unwrap();
    assert!(metadata.is_some());
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_parse_ros_package_with_setup_py() {
    let temp_dir = TempDir::new().unwrap();

    // Create package.xml
    let package_xml_path = temp_dir.path().join("package.xml");
    let package_xml_content = r#"<?xml version="1.0"?>
<package format="2">
  <name>test_ros_pkg</name>
  <version>1.0.0</version>
  <exec_depend>rclpy</exec_depend>
  <exec_depend>ros2cli</exec_depend>
  <test_depend>pytest</test_depend>
</package>"#;

    let mut package_xml_file = File::create(&package_xml_path).unwrap();
    package_xml_file
        .write_all(package_xml_content.as_bytes())
        .unwrap();

    // Create setup.py in same directory
    let setup_py_path = temp_dir.path().join("setup.py");
    let setup_py_content = r#"from setuptools import setup

setup(
    name='test_ros_pkg',
    version='1.0.0',
    install_requires=['ros2cli>=0.40', 'rclpy>=2.0'],
    extras_require={'test': ['pytest>=6.0']},
)
"#;

    let mut setup_py_file = File::create(&setup_py_path).unwrap();
    setup_py_file
        .write_all(setup_py_content.as_bytes())
        .unwrap();

    let (metadata, deps) = parse_ros_package(&package_xml_path).unwrap();

    // Check metadata
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.name, "test_ros_pkg");
    assert_eq!(metadata.version, "1.0.0");

    // Check dependencies - versions enriched from setup.py
    assert_eq!(deps.len(), 3);

    let rclpy = deps.iter().find(|d| d.name == "rclpy").unwrap();
    assert_eq!(rclpy.ecosystem, "ros");
    assert_eq!(rclpy.version, ">=2.0"); // Enriched!
    assert_eq!(rclpy.is_dev, false);

    let pytest = deps.iter().find(|d| d.name == "pytest").unwrap();
    assert_eq!(pytest.version, ">=6.0"); // Enriched!
    assert_eq!(pytest.is_dev, true);
}

#[test]
fn test_parse_ros_package_without_setup_py() {
    let content = r#"<?xml version="1.0"?>
<package format="3">
  <name>cmake_pkg</name>
  <version>2.0.0</version>
  <buildtool_depend>cmake</buildtool_depend>
</package>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let (metadata, deps) = parse_ros_package(temp_file.path()).unwrap();

    // Check metadata extracted
    assert!(metadata.is_some());
    let metadata = metadata.unwrap();
    assert_eq!(metadata.name, "cmake_pkg");
    assert_eq!(metadata.version, "2.0.0");

    // Check dependencies not enriched (no setup.py)
    assert_eq!(deps.len(), 1);
    let cmake = deps.iter().find(|d| d.name == "cmake").unwrap();
    assert_eq!(cmake.version, "unspecified");
}
// Source tracking tests

#[test]
fn test_ros_source_tracking_package_xml() {
    let content = r#"<?xml version="1.0"?>
<package format="3">
  <name>test_package</name>
  <version>1.0.0</version>
  <description>Test package</description>
  <maintainer email="test@example.com">Test User</maintainer>
  <license>Apache-2.0</license>
  
  <depend>ament_cmake</depend>
  <test_depend>ament_lint</test_depend>
</package>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let (_, deps) = parse_ros_package(temp_file.path()).unwrap();

    assert!(!deps.is_empty());
    let source = deps[0].source_file.as_ref().unwrap();
    assert!(source.starts_with("Identified by the ros/packagexml extractor from"));
    assert!(source.contains(temp_file.path().to_str().unwrap()));
}

// ============================================================================
// v0.9.1: Unit Tests for rosdep Version Resolution
// ============================================================================

#[cfg(feature = "internal")]
#[test]
fn test_resolve_ros_dependency_versions_with_cli_override() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    let mut deps = vec![Dependency {
        name: "rclpy".to_string(),
        version: "unspecified".to_string(),
        ecosystem: "ros".to_string(),
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    // Test with CLI override to "humble"
    resolve_ros_dependency_versions(&mut deps, Some("humble"));

    // Should have resolved version
    assert_ne!(deps[0].version, "unspecified");
    // humble should have a different version than jazzy
    // We can't hard-code the exact version as it changes, but it should be resolved
    assert!(deps[0].version.contains('.'));
}

#[cfg(feature = "internal")]
#[test]
fn test_resolve_ros_dependency_versions_default() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    let mut deps = vec![Dependency {
        name: "rclpy".to_string(),
        version: "unspecified".to_string(),
        ecosystem: "ros".to_string(),
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    // Test with no override (should default to jazzy)
    resolve_ros_dependency_versions(&mut deps, None);

    // Should have resolved version
    assert_ne!(deps[0].version, "unspecified");
    assert!(deps[0].version.contains('.'));
}

#[test]
fn test_resolve_ros_dependency_versions_non_ros_packages() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    let mut deps = vec![Dependency {
        name: "numpy".to_string(),
        version: "unspecified".to_string(),
        ecosystem: "pip".to_string(), // Not ROS ecosystem
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    // Should not modify non-ROS packages
    resolve_ros_dependency_versions(&mut deps, Some("humble"));

    // Version should still be unspecified for non-ROS packages
    assert_eq!(deps[0].version, "unspecified");
}

#[test]
fn test_resolve_ros_dependency_versions_already_specified() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    let mut deps = vec![Dependency {
        name: "rclpy".to_string(),
        version: "1.0.0".to_string(), // Already has version
        ecosystem: "ros".to_string(),
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    // Should not modify packages that already have versions
    resolve_ros_dependency_versions(&mut deps, Some("humble"));

    // Version should remain unchanged
    assert_eq!(deps[0].version, "1.0.0");
}

#[test]
fn test_resolve_ros_dependency_versions_package_name_variants() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    // Test various package name formats
    let mut deps = vec![Dependency {
        name: "python3-argcomplete".to_string(), // python3- prefix
        version: "unspecified".to_string(),
        ecosystem: "ros".to_string(),
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    resolve_ros_dependency_versions(&mut deps, Some("humble"));

    // May or may not resolve depending on rosdistro database
    // But should not crash
    assert!(deps[0].version == "unspecified" || deps[0].version.contains('.'));
}

#[cfg(feature = "internal")]
#[test]
fn test_resolve_ros_dependency_versions_with_repository_url() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::ros::resolve_ros_dependency_versions;

    // Test that repository_url is populated from rosdistro
    let mut deps = vec![Dependency {
        name: "rclpy".to_string(),
        version: "unspecified".to_string(),
        ecosystem: "ros".to_string(),
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
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }];

    resolve_ros_dependency_versions(&mut deps, Some("jazzy"));

    // Check version is resolved
    assert_ne!(deps[0].version, "unspecified");
    assert!(deps[0].version.contains('.'));

    // Check repository URL is populated (v0.9.1 enhancement)
    assert!(deps[0].repository_url.is_some());
    let repo_url = deps[0].repository_url.as_ref().unwrap();
    assert!(repo_url.contains("github.com") || repo_url.contains("git"));
    assert!(repo_url.contains("rclpy"));
}
