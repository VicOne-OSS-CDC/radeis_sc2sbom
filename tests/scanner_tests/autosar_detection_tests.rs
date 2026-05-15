use radeis_sc2sbom::cli::VendorMode;
use radeis_sc2sbom::scanner::detect_autosar;
use std::fs;
use tempfile::TempDir;

/// DET-01: a `.arxml` file anywhere in the tree triggers detection.
#[test]
fn test_detect_autosar_arxml_file() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let nested = temp_path.join("ecu/config");
    fs::create_dir_all(&nested).unwrap();
    fs::write(nested.join("system.arxml"), "<AUTOSAR/>").unwrap();

    assert!(
        detect_autosar(temp_path, &VendorMode::Skip, &[]),
        "expected is_autosar=true when .arxml file is present"
    );
}

/// DET-02: each canonical AUTOSAR directory name triggers detection.
#[test]
fn test_detect_autosar_directory_name() {
    for dir_name in &["BSW", "MCAL", "RTE", "AUTOSAR", "SWC"] {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();

        // Place the directory at depth 2 to confirm full-tree traversal.
        let nested = temp_path.join("project/src").join(dir_name);
        fs::create_dir_all(&nested).unwrap();

        assert!(
            detect_autosar(temp_path, &VendorMode::Skip, &[]),
            "expected is_autosar=true for directory name '{}'",
            dir_name
        );
    }
}

/// DET-03: AUTOSAR_VERSION or AR_VERSION in build files at root or depth 1 triggers detection.
#[test]
fn test_detect_autosar_build_file_variable() {
    // Case A: AR_VERSION in CMakeLists.txt at root
    {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        fs::write(
            temp_path.join("CMakeLists.txt"),
            "cmake_minimum_required(VERSION 3.10)\nset(AR_VERSION 4.0)\n",
        )
        .unwrap();
        assert!(
            detect_autosar(temp_path, &VendorMode::Skip, &[]),
            "expected is_autosar=true for AR_VERSION in root CMakeLists.txt"
        );
    }

    // Case B: AUTOSAR_VERSION in Makefile at depth 1
    {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        let sub = temp_path.join("src");
        fs::create_dir_all(&sub).unwrap();
        fs::write(sub.join("Makefile"), "AUTOSAR_VERSION := 4.4\nall:\n\techo ok\n").unwrap();
        assert!(
            detect_autosar(temp_path, &VendorMode::Skip, &[]),
            "expected is_autosar=true for AUTOSAR_VERSION in depth-1 Makefile"
        );
    }

    // Case C: AUTOSAR_VERSION in GNUmakefile at root
    {
        let temp_dir = TempDir::new().unwrap();
        let temp_path = temp_dir.path();
        fs::write(
            temp_path.join("GNUmakefile"),
            "AUTOSAR_VERSION = 22-11\n",
        )
        .unwrap();
        assert!(
            detect_autosar(temp_path, &VendorMode::Skip, &[]),
            "expected is_autosar=true for AUTOSAR_VERSION in root GNUmakefile"
        );
    }
}

/// Negative: plain C project with no AUTOSAR signals must NOT trigger detection.
#[test]
fn test_detect_autosar_plain_c_no_false_positive() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    let src = temp_path.join("src");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("main.c"), "int main(void) { return 0; }\n").unwrap();
    fs::write(src.join("header.h"), "#ifndef H\n#define H\n#endif\n").unwrap();
    fs::write(
        temp_path.join("CMakeLists.txt"),
        "cmake_minimum_required(VERSION 3.10)\nproject(plainc C)\nadd_executable(plainc src/main.c)\n",
    )
    .unwrap();

    assert!(
        !detect_autosar(temp_path, &VendorMode::Skip, &[]),
        "expected is_autosar=false for plain C project with no AUTOSAR signals"
    );
}

/// DET-03 boundary: substring like `MY_AR_VERSION_EXTRA` must NOT match standalone `AR_VERSION`.
#[test]
fn test_detect_autosar_no_substring_false_positive() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    fs::write(
        temp_path.join("CMakeLists.txt"),
        "set(MY_AR_VERSION_EXTRA 1.0)\nset(SOME_AUTOSAR_VERSION_HINT off)\n",
    )
    .unwrap();

    assert!(
        !detect_autosar(temp_path, &VendorMode::Skip, &[]),
        "expected is_autosar=false when only substring matches (MY_AR_VERSION_EXTRA, SOME_AUTOSAR_VERSION_HINT) are present"
    );
}

/// vendor_mode/excludes: a `BSW/` directory inside `node_modules/` must NOT trigger detection
/// when VendorMode::Skip is active (mirrors should_process_entry behavior).
#[test]
fn test_detect_autosar_respects_vendor_excludes() {
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // Plant a BSW directory deep inside a vendored subtree.
    let vendored_bsw = temp_path.join("node_modules/some-pkg/BSW");
    fs::create_dir_all(&vendored_bsw).unwrap();

    // Also plant a benign source file at the top so the tree isn't empty.
    fs::write(temp_path.join("README.md"), "plain project\n").unwrap();

    assert!(
        !detect_autosar(temp_path, &VendorMode::Skip, &[]),
        "expected is_autosar=false when BSW exists ONLY inside node_modules and VendorMode::Skip is active"
    );
}

/// Integration: verify detect_autosar fires on the real AUTOSAR-SOFTWARE-DEMO repo.
/// Skipped if the example_target_repos directory is not present.
#[test]
fn test_detect_autosar_real_repo_integration() {
    let repo_path = std::path::Path::new("example_target_repos/AUTOSAR-SOFTWARE-DEMO");
    if !repo_path.exists() {
        eprintln!("Skipping integration test: example_target_repos/AUTOSAR-SOFTWARE-DEMO not found");
        return;
    }
    assert!(
        detect_autosar(repo_path, &VendorMode::Skip, &[]),
        "expected is_autosar=true for AUTOSAR-SOFTWARE-DEMO (contains .arxml files)"
    );
}
