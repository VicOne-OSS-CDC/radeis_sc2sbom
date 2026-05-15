#[cfg(feature = "internal")]
use radeis_sc2sbom::formats::cyclonedx::{
    convert_to_cyclonedx, create_cyclonedx_metadata, create_dependency_component,
};
use radeis_sc2sbom::models::{
    AIModelMetadata, AutosarMetadata, Dependency, DependencyScope, DependencySource,
    RosPackageMetadata, RosPackageWithDeps, Sbom, SubModelInfo,
};
use serde_json;
use std::path::PathBuf;

// ---- Test helpers ----

fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
        source: DependencySource::LockFile,
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
        scope: DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }
}

fn make_sbom(deps: Vec<Dependency>) -> Sbom {
    Sbom {
        project_path: std::path::PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: deps,
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    }
}

// ---- End test helpers ----

#[test]
fn test_convert_to_cyclonedx_basic() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test/project"),
        generated_at: "2026-01-21T00:00:00Z".to_string(),
        dependencies: vec![
            Dependency {
                name: "express".to_string(),
                version: "4.17.1".to_string(),
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
            },
            Dependency {
                name: "jest".to_string(),
                version: "27.0.0".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: true,
                is_direct: false,
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
        ],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let cdx_doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    // Verify document structure
    assert_eq!(cdx_doc.bom_format, "CycloneDX");
    assert_eq!(cdx_doc.spec_version, "1.5");
    assert_eq!(cdx_doc.version, 1);
    assert!(cdx_doc.serial_number.starts_with("urn:uuid:"));

    // Verify metadata
    assert_eq!(cdx_doc.metadata.timestamp, "2026-01-21T00:00:00Z");
    assert_eq!(cdx_doc.metadata.tools.components.len(), 1);
    assert_eq!(cdx_doc.metadata.tools.components[0].name, "radeis_sc2sbom");
    assert!(cdx_doc.metadata.component.is_some());

    // Verify components
    assert_eq!(cdx_doc.components.len(), 2);

    let express = cdx_doc
        .components
        .iter()
        .find(|c| c.name == "express")
        .unwrap();
    assert_eq!(express.component_type, "library");
    assert_eq!(express.version, Some("4.17.1".to_string()));
    assert!(express.purl.as_ref().unwrap().contains("pkg:npm/express"));
    assert_eq!(express.properties.len(), 2); // source + scope (not dev)

    let jest = cdx_doc
        .components
        .iter()
        .find(|c| c.name == "jest")
        .unwrap();
    assert_eq!(jest.component_type, "library");
    assert_eq!(jest.version, Some("27.0.0".to_string()));
    assert_eq!(jest.properties.len(), 3); // dev + source + scope
    let dev_prop = jest
        .properties
        .iter()
        .find(|p| p.name == "dev-dependency")
        .unwrap();
    assert_eq!(dev_prop.value, "true");
}

#[test]
fn test_convert_to_cyclonedx_ros_packages() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test/ros-project"),
        generated_at: "2026-01-21T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![RosPackageWithDeps {
            metadata: RosPackageMetadata {
                name: "ros2_pkg".to_string(),
                version: "1.0.0".to_string(),
                source_file: PathBuf::from("/test/package.xml"),
                license: None,
                maintainers: vec![],
                authors: vec![],
                description: None,
            },
            dependencies: vec![
                Dependency {
                    name: "ament_cmake".to_string(),
                    version: "unspecified".to_string(),
                    ecosystem: "ros".to_string(),
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
                },
                Dependency {
                    name: "pytest".to_string(),
                    version: "6.0".to_string(),
                    ecosystem: "pip".to_string(),
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
                    scope: radeis_sc2sbom::models::DependencyScope::default(),
                    scope_confidence: 0.0,
                    scope_reason: "Not classified".to_string(),
                    ai_model_metadata: None,
                    autosar_metadata: None,
                    ..Default::default()
                },
            ],
        }],
        scope_statistics: None,
    };

    let cdx_doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    // Verify components: 1 ROS package + 2 dependencies
    assert_eq!(cdx_doc.components.len(), 3);

    // Verify ROS package component
    let ros_component = cdx_doc
        .components
        .iter()
        .find(|c| c.name == "ros2_pkg")
        .unwrap();
    assert_eq!(ros_component.component_type, "application");
    assert_eq!(ros_component.version, Some("1.0.0".to_string()));
    assert!(ros_component
        .purl
        .as_ref()
        .unwrap()
        .contains("pkg:ros/ros2_pkg"));

    // Verify dependencies relationship: 1 ROS package entry + 1 root entry (C2 fix)
    assert_eq!(cdx_doc.dependencies.len(), 2);
    let dep_rel = cdx_doc
        .dependencies
        .iter()
        .find(|d| d.reference == "ros-package-1")
        .unwrap();
    assert_eq!(dep_rel.depends_on.len(), 2);
}

#[test]
fn test_cyclonedx_metadata_structure() {
    let metadata = create_cyclonedx_metadata(
        "2026-01-21T00:00:00Z",
        "test-project",
        "project-test-project",
    );

    assert_eq!(metadata.timestamp, "2026-01-21T00:00:00Z");
    assert_eq!(metadata.tools.components.len(), 1);
    assert_eq!(metadata.tools.components[0].component_type, "application");
    assert_eq!(metadata.tools.components[0].name, "radeis_sc2sbom");
    assert_eq!(
        metadata.tools.components[0].version,
        env!("CARGO_PKG_VERSION")
    );

    assert!(metadata.component.is_some());
    let comp = metadata.component.unwrap();
    assert_eq!(comp.component_type, "application");
    assert_eq!(comp.name, "test-project");
    assert_eq!(comp.bom_ref, "project-test-project");
}

#[test]
fn test_cyclonedx_component_properties() {
    let dep = Dependency {
        name: "test-lib".to_string(),
        version: "1.0.0".to_string(),
        ecosystem: "npm".to_string(),
        source: DependencySource::LockFile,
        is_dev: true,
        is_direct: false,
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

    let component = create_dependency_component(&dep, "dep-npm-1", None);

    // Verify properties
    assert_eq!(component.properties.len(), 3);

    let dev_prop = component
        .properties
        .iter()
        .find(|p| p.name == "dev-dependency")
        .unwrap();
    assert_eq!(dev_prop.value, "true");

    let source_prop = component
        .properties
        .iter()
        .find(|p| p.name == "dependency-source")
        .unwrap();
    assert_eq!(source_prop.value, "lock-file");

    let scope_prop = component
        .properties
        .iter()
        .find(|p| p.name == "dependency-scope")
        .unwrap();
    assert_eq!(scope_prop.value, "transitive");
}

#[test]
fn test_cyclonedx_uuid_generation() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test/project"),
        generated_at: "2026-01-21T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let cdx_doc1 = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);
    let cdx_doc2 = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    // Verify UUID format
    assert!(cdx_doc1.serial_number.starts_with("urn:uuid:"));
    assert!(cdx_doc2.serial_number.starts_with("urn:uuid:"));

    // UUIDs should be different (v4 random)
    assert_ne!(cdx_doc1.serial_number, cdx_doc2.serial_number);
}

// v1.0.8 SBOM spec compliance: version field must be None for sentinel values

#[test]
fn test_cyclonedx_component_detected_version_is_none() {
    // utime with "detected" version — component.version must be None
    // NOTE: for a single-dep SBOM, components[0] is the dependency (root is in metadata.component)
    let dep = make_dep("utime", "detected", "pip");
    let doc = convert_to_cyclonedx(&make_sbom(vec![dep]), None, #[cfg(feature = "internal")] &[]);
    assert!(
        doc.components[0].version.is_none(),
        "detected version must produce None, got: {:?}",
        doc.components[0].version
    );
}

#[test]
fn test_cyclonedx_component_real_version_is_some() {
    let dep = make_dep("sdl2", "2.0.12", "pkg-config");
    let doc = convert_to_cyclonedx(&make_sbom(vec![dep]), None, #[cfg(feature = "internal")] &[]);
    assert_eq!(doc.components[0].version, Some("2.0.12".to_string()));
}

// v1.0.8 SBOM spec compliance: metadata.tools must use non-deprecated CycloneDX 1.5 format

#[test]
fn test_cyclonedx_metadata_tools_new_format() {
    let doc = convert_to_cyclonedx(&make_sbom(vec![]), None, #[cfg(feature = "internal")] &[]);
    let json = serde_json::to_value(&doc).unwrap();
    assert!(
        json["metadata"]["tools"]["components"].is_array(),
        "metadata.tools must have components array"
    );
    assert!(
        !json["metadata"]["tools"].is_array(),
        "metadata.tools must not be a flat array (deprecated format)"
    );
    let tool = &json["metadata"]["tools"]["components"][0];
    assert_eq!(tool["type"], "application");
    assert_eq!(tool["name"], "radeis_sc2sbom");
}

// Post-scan review: C1 — CycloneDX MUST NOT duplicate components across ROS packages

#[test]
fn test_ros_shared_dep_not_duplicated_in_cyclonedx() {
    let shared_dep = make_dep("openssl", "3.0.2", "pkg-config");

    let sbom = Sbom {
        project_path: std::path::PathBuf::from("/test/ros-project"),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![
            RosPackageWithDeps {
                metadata: RosPackageMetadata {
                    name: "pkg_a".to_string(),
                    version: "1.0.0".to_string(),
                    source_file: std::path::PathBuf::from("/test/pkg_a/package.xml"),
                    license: None,
                    maintainers: vec![],
                    authors: vec![],
                    description: None,
                },
                dependencies: vec![shared_dep.clone()],
            },
            RosPackageWithDeps {
                metadata: RosPackageMetadata {
                    name: "pkg_b".to_string(),
                    version: "1.0.0".to_string(),
                    source_file: std::path::PathBuf::from("/test/pkg_b/package.xml"),
                    license: None,
                    maintainers: vec![],
                    authors: vec![],
                    description: None,
                },
                dependencies: vec![shared_dep.clone()],
            },
        ],
        scope_statistics: None,
    };

    let doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    // openssl should appear exactly once, not once per ROS package
    let openssl_count = doc
        .components
        .iter()
        .filter(|c| c.name == "openssl")
        .count();
    assert_eq!(
        openssl_count, 1,
        "openssl shared by 2 ROS packages must appear exactly once in components, got {}",
        openssl_count
    );
}

// Post-scan review: C2 — root metadata.component bom-ref must appear in dependencies array

#[test]
fn test_cyclonedx_root_appears_in_dependencies() {
    let sbom = Sbom {
        project_path: std::path::PathBuf::from("/test/project"),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        dependencies: vec![make_dep("openssl", "3.0.2", "pkg-config")],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    let root_bom_ref = doc
        .metadata
        .component
        .as_ref()
        .map(|c| c.bom_ref.clone())
        .expect("metadata.component must exist");

    let root_in_deps = doc.dependencies.iter().any(|d| d.reference == root_bom_ref);
    assert!(
        root_in_deps,
        "root bom-ref '{}' must appear in dependencies array",
        root_bom_ref
    );
}

// ── v1.0.13: Sub-model nested components ────────────────────────────────────

#[test]
fn test_cyclonedx_nested_sub_model_components() {
    let mut dep = make_dep("gemma4-test", "unknown", "safetensors");
    dep.ai_model_metadata = Some(AIModelMetadata {
        architecture: Some("Gemma4ForConditionalGeneration".to_string()),
        sub_models: vec![
            SubModelInfo {
                model_type: Some("gemma4_text".to_string()),
                modality: "text".to_string(),
                num_hidden_layers: Some(35),
                hidden_size: Some(1536),
                num_attention_heads: Some(8),
                vocab_size: Some(262144),
                ..Default::default()
            },
            SubModelInfo {
                model_type: Some("gemma4_vision".to_string()),
                modality: "vision".to_string(),
                num_hidden_layers: Some(16),
                hidden_size: Some(768),
                patch_size: Some(16),
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    let sbom = make_sbom(vec![dep]);
    let doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);

    // Find the parent AI model component
    let parent = doc
        .components
        .iter()
        .find(|c| c.name == "gemma4-test")
        .expect("parent component must exist");

    // Verify nested components
    assert_eq!(parent.components.len(), 2, "should have 2 nested sub-model components");

    let text_sub = &parent.components[0];
    assert_eq!(text_sub.name, "gemma4_text");
    assert_eq!(text_sub.component_type, "machine-learning-model");
    let text_props: std::collections::HashMap<&str, &str> = text_sub
        .properties
        .iter()
        .map(|p| (p.name.as_str(), p.value.as_str()))
        .collect();
    assert_eq!(text_props.get("radeis:ai:sub_model:modality"), Some(&"text"));
    assert_eq!(text_props.get("radeis:ai:sub_model:num_hidden_layers"), Some(&"35"));
    assert_eq!(text_props.get("radeis:ai:sub_model:hidden_size"), Some(&"1536"));
    assert_eq!(text_props.get("radeis:ai:sub_model:vocab_size"), Some(&"262144"));

    let vision_sub = &parent.components[1];
    assert_eq!(vision_sub.name, "gemma4_vision");
    let vision_props: std::collections::HashMap<&str, &str> = vision_sub
        .properties
        .iter()
        .map(|p| (p.name.as_str(), p.value.as_str()))
        .collect();
    assert_eq!(vision_props.get("radeis:ai:sub_model:modality"), Some(&"vision"));
    assert_eq!(vision_props.get("radeis:ai:sub_model:patch_size"), Some(&"16"));

    // Verify nested sub-model components do NOT appear in top-level components
    assert!(
        !doc.components.iter().any(|c| c.name == "gemma4_text" || c.name == "gemma4_vision"),
        "sub-model components must NOT appear in top-level components array"
    );
}

#[test]
fn test_autosar_cyclonedx_properties() {
    use radeis_sc2sbom::models::AutosarMetadata;

    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(AutosarMetadata {
        module_name: "NvM".to_string(),
        layer: "BSW-Memory".to_string(),
        platform: "Classic".to_string(),
    });

    let component = create_dependency_component(&dep, "test-bom-ref", None);

    let layer_prop = component
        .properties
        .iter()
        .find(|p| p.name == "autosar:layer")
        .expect("autosar:layer property must be present");
    assert_eq!(layer_prop.value, "BSW-Memory");

    let platform_prop = component
        .properties
        .iter()
        .find(|p| p.name == "autosar:platform")
        .expect("autosar:platform property must be present");
    assert_eq!(platform_prop.value, "Classic");
}

// ---- Phase 8 (v1.0.15): autosar:supplier tests ----

fn make_autosar_metadata() -> AutosarMetadata {
    AutosarMetadata {
        module_name: "NvM".to_string(),
        layer: "BSW-Memory".to_string(),
        platform: "Classic".to_string(),
    }
}

#[test]
fn test_autosar_supplier_mapped() {
    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(make_autosar_metadata());
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let cdx = convert_to_cyclonedx(&sbom, Some(&resolver), #[cfg(feature = "internal")] &[]);
    let nvm_component = cdx.components.iter().find(|c| c.name == "NvM").unwrap();
    let prop = nvm_component
        .properties
        .iter()
        .find(|p| p.name == "autosar:supplier")
        .expect("autosar:supplier property must be emitted");
    assert_eq!(prop.value, "Vector Informatik");
}

#[test]
fn test_autosar_supplier_noassertion_no_config() {
    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(make_autosar_metadata());
    let sbom = make_sbom(vec![dep]);

    let cdx = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);
    let prop = cdx
        .components
        .iter()
        .find(|c| c.name == "NvM")
        .unwrap()
        .properties
        .iter()
        .find(|p| p.name == "autosar:supplier")
        .unwrap();
    assert_eq!(prop.value, "NOASSERTION");
}

#[test]
fn test_autosar_supplier_noassertion_no_entry() {
    let mut dep = make_dep("Com", "unspecified", "autosar");
    dep.autosar_metadata = Some(AutosarMetadata {
        module_name: "Com".to_string(),
        layer: "BSW-Communication".to_string(),
        platform: "Classic".to_string(),
    });
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let cdx = convert_to_cyclonedx(&sbom, Some(&resolver), #[cfg(feature = "internal")] &[]);
    let prop = cdx
        .components
        .iter()
        .find(|c| c.name == "Com")
        .unwrap()
        .properties
        .iter()
        .find(|p| p.name == "autosar:supplier")
        .unwrap();
    assert_eq!(prop.value, "NOASSERTION");
}

#[test]
fn test_non_autosar_no_supplier() {
    let dep = make_dep("openssl", "3.0.0", "cargo"); // autosar_metadata defaults to None
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("openssl".to_string(), "Should Not Appear".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let cdx = convert_to_cyclonedx(&sbom, Some(&resolver), #[cfg(feature = "internal")] &[]);
    let comp = cdx.components.iter().find(|c| c.name == "openssl").unwrap();
    assert!(
        comp.properties.iter().all(|p| p.name != "autosar:supplier"),
        "non-AUTOSAR component must not carry autosar:supplier property"
    );
}
