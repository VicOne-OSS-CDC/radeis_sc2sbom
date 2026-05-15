use radeis_sc2sbom::classifier::autosar::{classify_autosar_components, BswConfig};
use radeis_sc2sbom::models::{Dependency, DependencyScope, DependencySource};
use std::fs;
use tempfile::TempDir;

/// Construct a minimal Dependency for classification tests.
/// Every field of Dependency must be named explicitly because the struct
/// has no `..Default::default()` shorthand at construction sites.
fn make_dep(name: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: "unspecified".to_string(),
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
        scope_reason: String::new(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }
}

// ---------------------------------------------------------------
// CLS-01 (matching), CLS-02 (layer), CLS-03 (platform)
// ---------------------------------------------------------------

#[test]
fn test_nvm_classifies_as_bsw_memory_classic() {
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("NvM", "c")];
    classify_autosar_components(&mut deps, &config);

    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "autosar", "ecosystem must be 'autosar'");
    let meta = dep
        .autosar_metadata
        .as_ref()
        .expect("autosar_metadata must be Some after match");
    assert_eq!(meta.module_name, "NvM", "module_name preserves canonical casing");
    assert_eq!(meta.layer, "BSW-Memory", "layer must be BSW-Memory");
    assert_eq!(meta.platform, "Classic", "platform must be Classic");
}

#[test]
fn test_case_insensitive_match() {
    // CLS-01 / D-01: NvM, nvm, NVM all match the same entry.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("nvm", "c")];
    classify_autosar_components(&mut deps, &config);

    assert_eq!(deps[0].ecosystem, "autosar");
    let meta = deps[0].autosar_metadata.as_ref().unwrap();
    // module_name preserves canonical casing from the YAML config, NOT the
    // input case — so even though we passed "nvm", we get "NvM".
    assert_eq!(meta.module_name, "NvM");
}

#[test]
fn test_unknown_component_unchanged() {
    // D-02: components that do not match are left entirely unchanged.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("zlib", "c")];
    classify_autosar_components(&mut deps, &config);

    assert_eq!(deps[0].ecosystem, "c", "ecosystem must stay unchanged");
    assert!(
        deps[0].autosar_metadata.is_none(),
        "autosar_metadata must stay None"
    );
}

#[test]
fn test_substring_no_match() {
    // D-01: exact match only — NvM_Stub must NOT match NvM.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("NvM_Stub", "c")];
    classify_autosar_components(&mut deps, &config);

    assert_eq!(deps[0].ecosystem, "c");
    assert!(deps[0].autosar_metadata.is_none());
}

#[test]
fn test_mcal_layer_classic() {
    // CLS-02: Wdg classifies as MCAL/Classic.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("Wdg", "c")];
    classify_autosar_components(&mut deps, &config);

    let meta = deps[0].autosar_metadata.as_ref().unwrap();
    assert_eq!(meta.layer, "MCAL");
    assert_eq!(meta.platform, "Classic");
}

#[test]
fn test_communication_layer_classic() {
    // CLS-02: PduR classifies as BSW-Communication/Classic.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("PduR", "c")];
    classify_autosar_components(&mut deps, &config);

    let meta = deps[0].autosar_metadata.as_ref().unwrap();
    assert_eq!(meta.layer, "BSW-Communication");
    assert_eq!(meta.platform, "Classic");
}

#[test]
fn test_adaptive_platform() {
    // CLS-03 / D-06: ara_com is Adaptive.
    let config = BswConfig::load_bundled();
    let mut deps = vec![make_dep("ara_com", "c")];
    classify_autosar_components(&mut deps, &config);

    let meta = deps[0].autosar_metadata.as_ref().unwrap();
    assert_eq!(meta.platform, "Adaptive");
    assert_eq!(meta.layer, "BSW-Communication");
}

#[test]
fn test_load_from_file_override() {
    // D-03: --bsw-config override loads a user YAML; bundled entries are
    // NOT merged in — the override config replaces the bundled list.
    let tmp = TempDir::new().unwrap();
    let yaml_path = tmp.path().join("custom.yaml");
    fs::write(
        &yaml_path,
        "MyCustomModule:\n  layer: MCAL\n  platform: Classic\n",
    )
    .unwrap();

    let config = BswConfig::load_from_file(&yaml_path).expect("override yaml must parse");

    let mut deps = vec![
        make_dep("MyCustomModule", "c"),
        make_dep("NvM", "c"), // present in bundled config but NOT in override
    ];
    classify_autosar_components(&mut deps, &config);

    // Custom module matches via override.
    assert_eq!(deps[0].ecosystem, "autosar");
    let meta = deps[0].autosar_metadata.as_ref().unwrap();
    assert_eq!(meta.module_name, "MyCustomModule");
    assert_eq!(meta.layer, "MCAL");

    // NvM does NOT match because the override config does not contain it.
    assert_eq!(
        deps[1].ecosystem, "c",
        "override config replaces bundled list, NvM must not match"
    );
    assert!(deps[1].autosar_metadata.is_none());
}
