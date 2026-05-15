use std::process::Command;

/// Integration test: validates the generated SPDX JSON against the official
/// pyspdxtools validator (from the `spdx-tools` Python package).
///
/// Requires `pyspdxtools` to be installed (`pip install spdx-tools`).
/// If not found in PATH the test is skipped with a warning so that
/// developers without a Python environment still get a clean `cargo test`.
#[test]
fn test_spdx_output_passes_pyspdxtools_validation() {
    // Graceful skip if pyspdxtools is not installed
    if Command::new("pyspdxtools")
        .arg("--version")
        .output()
        .is_err()
    {
        eprintln!("SKIP: pyspdxtools not found in PATH — install with: pip install spdx-tools");
        return;
    }

    // Generate SPDX JSON by scanning the rclcpp example repo
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");
    let binary = env!("CARGO_BIN_EXE_radeis_sc2sbom");

    let scan_status = Command::new(binary)
        .args([
            "--path",
            "example_target_repos/rclcpp",
            "--output",
            output_dir.path().to_str().unwrap(),
            "--format",
            "all",
        ])
        .status()
        .expect("failed to run radeis_sc2sbom binary");

    assert!(scan_status.success(), "SBOM binary scan failed");

    // Run pyspdxtools validator on the generated file
    let spdx_file = output_dir.path().join("rclcpp_spdx.json");
    assert!(spdx_file.exists(), "rclcpp_spdx.json was not generated");

    let result = Command::new("pyspdxtools")
        .args(["-i", spdx_file.to_str().unwrap()])
        .output()
        .expect("failed to run pyspdxtools");

    assert!(
        result.status.success(),
        "pyspdxtools validation failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr),
    );
}
