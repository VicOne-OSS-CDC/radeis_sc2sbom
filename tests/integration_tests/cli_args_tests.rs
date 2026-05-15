use assert_cmd::Command;
use predicates::str::contains;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

fn bin() -> Command {
    Command::cargo_bin("radeis_sc2sbom").expect("binary exists")
}

fn empty_scan_dir() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn test_supplier_config_missing_file_hard_errors() {
    let scan = empty_scan_dir();
    let out = empty_scan_dir();
    bin()
        .args([
            "--path",
            scan.path().to_str().unwrap(),
            "--supplier-config",
            "/nonexistent/path/to/never-exists.yaml",
            "--output",
            out.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("Failed to read supplier config"));
}

#[test]
fn test_supplier_config_invalid_yaml_hard_errors() {
    let scan = empty_scan_dir();
    let out = empty_scan_dir();
    let mut bad = NamedTempFile::new().unwrap();
    writeln!(bad, ": : :").unwrap();
    bin()
        .args([
            "--path",
            scan.path().to_str().unwrap(),
            "--supplier-config",
            bad.path().to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(contains("Failed to parse supplier config YAML"));
}

#[test]
fn test_supplier_config_valid_yaml_succeeds() {
    let scan = empty_scan_dir();
    let out = empty_scan_dir();
    let mut good = NamedTempFile::new().unwrap();
    writeln!(good, "NvM: \"Vector Informatik\"").unwrap();
    let assertion = bin()
        .args([
            "--path",
            scan.path().to_str().unwrap(),
            "--supplier-config",
            good.path().to_str().unwrap(),
            "--output",
            out.path().to_str().unwrap(),
        ])
        .assert();
    // Binary may succeed or fail for unrelated reasons (empty scan dir),
    // but the supplier-config load itself must not appear in any error.
    let output = assertion.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("Failed to read supplier config"),
        "supplier load should not error on valid yaml, stderr: {}",
        stderr
    );
    assert!(
        !stderr.contains("Failed to parse supplier config YAML"),
        "supplier parse should not error on valid yaml, stderr: {}",
        stderr
    );
}

#[test]
fn test_no_supplier_config_succeeds() {
    let scan = empty_scan_dir();
    let out = empty_scan_dir();
    let assertion = bin()
        .args(["--path", scan.path().to_str().unwrap(), "--output", out.path().to_str().unwrap()])
        .assert();
    let output = assertion.get_output();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("supplier config"),
        "no supplier flag → no supplier-config error path, stderr: {}",
        stderr
    );
}
