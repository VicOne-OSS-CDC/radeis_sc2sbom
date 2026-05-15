//! End-to-end integration tests for AUTOSAR classification (Phase 7, OUT-01).
//!
//! These tests exercise the full pipeline:
//!   detect_autosar (Phase 6) -> gate -> BswConfig::load_bundled ->
//!   classify_autosar_components (Phase 7).
//!
//! Synthetic source trees are built in TempDir; Dependency vectors are
//! constructed in-test to isolate this layer from upstream parsers.

use radeis_sc2sbom::classifier::autosar::{classify_autosar_components, BswConfig};
use radeis_sc2sbom::cli::VendorMode;
use radeis_sc2sbom::models::{Dependency, DependencyScope, DependencySource};
use radeis_sc2sbom::scanner::detect_autosar;
use std::fs;
use tempfile::TempDir;

/// Construct a minimal Dependency with every field named explicitly.
/// Plan 07-01 added autosar_metadata: None to the struct; this helper
/// must keep that field present.
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

/// Replicates the main.rs gate so the integration test exercises the
/// real production pathway in-process.
fn run_pipeline(temp_path: &std::path::Path, deps: &mut Vec<Dependency>) {
    let is_autosar = detect_autosar(temp_path, &VendorMode::Skip, &[]);
    if is_autosar {
        let cfg = BswConfig::load_bundled();
        classify_autosar_components(deps, &cfg);
    }
}

/// OUT-01 positive: AUTOSAR signal + BSW-named dep -> dep is classified
/// to the autosar ecosystem with layer/platform metadata.
#[test]
fn test_autosar_e2e_positive() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Plant a .arxml file so detect_autosar returns true.
    let nested = temp_path.join("ecu/config");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("system.arxml"), "<AUTOSAR/>").unwrap();

    // Simulate parsers having discovered an NvM component in this tree.
    let mut deps = vec![make_dep("NvM", "c")];

    run_pipeline(temp_path, &mut deps);

    assert_eq!(
        deps[0].ecosystem, "autosar",
        "OUT-01: NvM in an AUTOSAR project must land in the autosar ecosystem bucket"
    );
    let meta = deps[0]
        .autosar_metadata
        .as_ref()
        .expect("autosar_metadata must be Some after end-to-end classification");
    assert_eq!(meta.module_name, "NvM");
    assert_eq!(meta.layer, "BSW-Memory");
    assert_eq!(meta.platform, "Classic");
}

/// Pitfall 3 / D-02: non-AUTOSAR project with a coincidental NvM-named
/// dependency must NOT be reclassified — the is_autosar gate prevents
/// false positives on plain C trees.
#[test]
fn test_autosar_e2e_negative_gating() {
    let temp = TempDir::new().unwrap();
    let temp_path = temp.path();

    // Plain C project — no .arxml, no BSW directory, no AR_VERSION.
    let src = temp_path.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(
        temp_path.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(plainc C)\n",
    )
    .unwrap();

    // A hypothetical C library named NvM that is NOT AUTOSAR.
    let mut deps = vec![make_dep("NvM", "c")];

    run_pipeline(temp_path, &mut deps);

    // is_autosar=false, so classify_autosar_components must NOT have run.
    assert_eq!(
        deps[0].ecosystem, "c",
        "Pitfall 3: non-AUTOSAR project must not reclassify components"
    );
    assert!(
        deps[0].autosar_metadata.is_none(),
        "Pitfall 3: autosar_metadata must remain None when is_autosar=false"
    );
}
