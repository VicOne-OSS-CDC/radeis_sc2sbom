//! End-to-end test for --supplier-config with --format cyclonedx-json (Phase 8, v1.0.15).
//!
//! Verifies that a real binary invocation with --supplier-config produces
//! CycloneDX JSON where AUTOSAR components carry the autosar:supplier property
//! with the mapped value.
//!
//! Scope (locked per plan): cyclonedx-json only. SPDX and --format all are
//! covered at unit-test level by Tests 1 and 2 in Plan 08-02b.

use assert_cmd::Command;
use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn bin() -> Command {
    Command::cargo_bin("radeis_sc2sbom").expect("binary exists")
}

/// Create a minimal AUTOSAR scan directory.
///
/// Structure:
///   <dir>/
///     BSW/               ← triggers detect_autosar (DET-02: BSW dir)
///     Makefile           ← references -lNvM so the makefile parser discovers
///                          component "NvM"; classify_autosar_components then
///                          classifies it (ecosystem="autosar", autosar_metadata=Some)
fn make_autosar_scan_dir() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    // DET-02: BSW directory signals AUTOSAR project.
    let bsw = dir.path().join("BSW");
    fs::create_dir_all(&bsw).unwrap();
    // A stub source file inside BSW so the dir is non-empty.
    fs::write(bsw.join("NvM_Cfg.c"), "// stub\n").unwrap();
    // Makefile with -lNvM so the makefile parser produces a "NvM" component
    // in the system ecosystem. classify_autosar_components then upgrades it.
    fs::write(
        dir.path().join("Makefile"),
        "LDFLAGS = -lNvM\nall:\n\t@echo done\n",
    )
    .unwrap();
    dir
}

/// Write a supplier config YAML mapping NvM -> "Vector Informatik".
fn make_supplier_config() -> NamedTempFile {
    let mut f = NamedTempFile::new().unwrap();
    writeln!(f, "NvM: \"Vector Informatik\"").unwrap();
    f
}

/// e2e: binary with --supplier-config emits autosar:supplier in cyclonedx-json stdout.
///
/// Pipeline exercised:
///   detect_autosar -> classify_autosar_components -> convert_to_cyclonedx
///   -> print_cyclonedx_json (stdout)
///
/// Verification: parse the JSON from stdout and assert the autosar:supplier
/// property exists on the NvM component with the mapped value.
#[test]
fn e2e_autosar_supplier_in_cyclonedx() {
    let scan = make_autosar_scan_dir();
    let cfg = make_supplier_config();

    // --format cyclonedx-json writes to --output dir (file-based output).
    let out_dir = tempfile::tempdir().unwrap();
    let output = bin()
        .args([
            "--path",
            scan.path().to_str().unwrap(),
            "--supplier-config",
            cfg.path().to_str().unwrap(),
            "--format",
            "cyclonedx-json",
            "--output",
            out_dir.path().to_str().unwrap(),
        ])
        .output()
        .expect("binary must run");

    assert!(
        output.status.success(),
        "binary must exit 0; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Binary writes {project_name}_cyclonedx.json to the output dir.
    let json_file = std::fs::read_dir(out_dir.path())
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.file_name().to_string_lossy().ends_with("_cyclonedx.json"))
        .expect("cyclonedx JSON file must exist in output dir");
    let json_str = std::fs::read_to_string(json_file.path()).expect("must read JSON file");
    let json: serde_json::Value =
        serde_json::from_str(&json_str).expect("output file must contain valid JSON");

    let components = json["components"]
        .as_array()
        .expect("CycloneDX output must have a components array");

    let nvm = components
        .iter()
        .find(|c| c["name"] == "NvM")
        .expect("NvM component must appear in cyclonedx output");

    let props = nvm["properties"]
        .as_array()
        .expect("NvM component must have a properties array");

    let supplier_prop = props
        .iter()
        .find(|p| p["name"] == "autosar:supplier")
        .expect("autosar:supplier property must be emitted on AUTOSAR component");

    assert_eq!(
        supplier_prop["value"], "Vector Informatik",
        "autosar:supplier value must equal the mapped string from --supplier-config"
    );
}
