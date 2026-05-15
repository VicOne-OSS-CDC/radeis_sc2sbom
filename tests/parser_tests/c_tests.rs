use radeis_sc2sbom::parsers::c::library_json::parse_library_json_content;
use radeis_sc2sbom::parsers::c::mk_file::{normalize_library_name, scan_mk_files_for_versions};
use radeis_sc2sbom::parsers::c::so_scanner::find_so_files;
use radeis_sc2sbom::parsers::c::{
    parse_configure_ac, parse_makefile, parse_makefile_am, parse_pc_file,
};
use std::fs;
use std::path::Path;

#[test]
fn test_parse_pc_file() {
    let fixture_path = Path::new("tests/fixtures/c/openssl.pc");
    let dep = parse_pc_file(fixture_path).expect("Failed to parse .pc file");

    assert_eq!(dep.name, "OpenSSL");
    assert_eq!(dep.version, "3.0.2");
    assert_eq!(dep.ecosystem, "pkg-config");
    assert!(dep.is_direct);
}

#[test]
fn test_parse_configure_ac() {
    let fixture_path = Path::new("tests/fixtures/c/configure.ac");
    let deps = parse_configure_ac(fixture_path).expect("Failed to parse configure.ac");

    // Should find dependencies from AC_CHECK_LIB and PKG_CHECK_MODULES
    assert!(!deps.is_empty());

    // Check for AC_CHECK_LIB dependencies (autotools ecosystem)
    let ssl = deps.iter().find(|d| d.name == "ssl");
    assert!(ssl.is_some(), "ssl dependency should be found");
    if let Some(ssl) = ssl {
        assert_eq!(ssl.ecosystem, "autotools");
    }

    let pthread = deps.iter().find(|d| d.name == "pthread");
    assert!(pthread.is_some(), "pthread dependency should be found");
    if let Some(pthread) = pthread {
        assert_eq!(pthread.ecosystem, "autotools");
    }

    // Check for PKG_CHECK_MODULES dependencies (pkg-config ecosystem)
    let glib = deps.iter().find(|d| d.name == "glib-2.0");
    assert!(glib.is_some(), "glib-2.0 dependency should be found");
    if let Some(glib) = glib {
        assert_eq!(glib.ecosystem, "pkg-config");
    }

    let openssl = deps.iter().find(|d| d.name == "openssl");
    assert!(openssl.is_some(), "openssl dependency should be found");
    if let Some(openssl) = openssl {
        assert_eq!(openssl.ecosystem, "pkg-config");
    }
}

#[test]
fn test_parse_makefile_am() {
    let fixture_path = Path::new("tests/fixtures/c/Makefile.am");
    let deps = parse_makefile_am(fixture_path).expect("Failed to parse Makefile.am");

    assert!(!deps.is_empty());

    // Check that dependencies have correct attributes
    for dep in &deps {
        assert_eq!(dep.ecosystem, "autotools");
        assert!(dep.is_direct);
    }
}

#[test]
fn test_parse_makefile() {
    let fixture_path = Path::new("tests/fixtures/c/Makefile");
    let scan_root = Path::new("tests/fixtures/c");
    let deps = parse_makefile(fixture_path, false, false, None, scan_root)
        .expect("Failed to parse Makefile");

    assert!(!deps.is_empty());

    // Should extract -l flags (system libraries)
    let pthread = deps.iter().find(|d| d.name == "pthread");
    assert!(pthread.is_some(), "pthread dependency should be found");
    if let Some(pthread) = pthread {
        assert_eq!(pthread.ecosystem, "system");
    }

    // Should also extract pkg-config packages
    let openssl = deps.iter().find(|d| d.name == "openssl");
    assert!(
        openssl.is_some(),
        "openssl (from pkg-config) should be found"
    );
    if let Some(openssl) = openssl {
        assert_eq!(openssl.ecosystem, "pkg-config");
    }
}

#[test]
fn test_parse_pc_file_with_url() {
    // Test that URL fields with '=' in query parameters don't break parsing
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().expect("Failed to create temporary .pc file");
    // Minimal .pc content with a URL containing '=' in the query parameters
    writeln!(
        file,
        "Name: mylib\nVersion: 2.0.0\nDescription: Test library\nURL: https://example.com/download?token=a=b\nLibs: -lmylib"
    )
    .expect("Failed to write to temporary .pc file");

    let dep = parse_pc_file(file.path()).expect("Failed to parse .pc file with URL");
    assert_eq!(dep.name, "mylib");
    assert_eq!(dep.version, "2.0.0");
}

#[test]
fn test_pkgconfig_detection_no_redos() {
    // Test that pkg-config pattern doesn't cause ReDoS with malicious input
    use radeis_sc2sbom::parsers::c::extract_pkgconfig_from_makefile;
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().unwrap();
    // Create input that could trigger ReDoS with nested quantifiers
    let malicious_input = "pkg-config --cflags ".to_string() + &"a ".repeat(100);
    writeln!(file, "{}", malicious_input).unwrap();

    // This should complete quickly without hanging
    let deps =
        extract_pkgconfig_from_makefile(file.path()).expect("Should handle input gracefully");
    // Just verify it doesn't hang - the actual dependencies found don't matter
    assert!(deps.len() <= 100);
}

#[test]
fn test_autotools_detection_in_parent_dir() {
    // This test verifies that Autotools detection checks parent directories
    // The actual scanner.rs logic is tested in integration tests
    // Here we just verify the parsers work independently
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut file = NamedTempFile::new().expect("Failed to create temporary Makefile");
    writeln!(file, "LDFLAGS = -lpthread -lm").expect("Failed to write to Makefile");

    let scan_root = file.path().parent().unwrap();
    let _deps = parse_makefile(file.path(), false, false, None, scan_root)
        .expect("Failed to parse subdirectory Makefile");
    // Makefile parsing should work regardless of Autotools presence
}

// Issue #5: Test extended library name normalization
#[test]
fn test_extended_library_name_normalization() {
    let candidates = normalize_library_name("pthread");
    assert!(candidates.contains(&"pthreads".to_string()));

    let candidates = normalize_library_name("m");
    assert!(candidates.contains(&"libm".to_string()));

    let candidates = normalize_library_name("dl");
    assert!(candidates.contains(&"libdl".to_string()));

    let candidates = normalize_library_name("rt");
    assert!(candidates.contains(&"librt".to_string()));

    let candidates = normalize_library_name("jpeg");
    assert!(candidates.contains(&"libjpeg".to_string()));

    let candidates = normalize_library_name("png");
    assert!(candidates.contains(&"libpng".to_string()));
}

// Issue #7: Test .so symlink deduplication
#[test]
fn test_so_symlink_deduplication() {
    use tempfile::tempdir;

    // Create a temp directory with .so files
    let temp_dir = tempdir().unwrap();
    let lib_dir = temp_dir.path().join("lib");
    fs::create_dir(&lib_dir).unwrap();

    // Create actual file
    let actual_file = lib_dir.join("libcurl.so.4.8.0");
    fs::write(&actual_file, b"dummy").unwrap();

    // Create symlinks (if supported on the platform)
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let _ = symlink("libcurl.so.4.8.0", lib_dir.join("libcurl.so.4"));
        let _ = symlink("libcurl.so.4", lib_dir.join("libcurl.so"));
    }

    let so_files = find_so_files(temp_dir.path());

    // Should only find one entry (the actual file), not three (with symlinks)
    let curl_files: Vec<_> = so_files
        .iter()
        .filter(|p| p.file_name().unwrap().to_str().unwrap().contains("curl"))
        .collect();

    #[cfg(unix)]
    {
        // On Unix systems with symlinks, should deduplicate to 1 file
        assert_eq!(
            curl_files.len(),
            1,
            "Should deduplicate symlinks to single canonical file"
        );
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems, just verify no crash
        assert!(
            !curl_files.is_empty(),
            "Should find at least the actual file"
        );
    }
}

#[test]
fn test_build_tool_variables_excluded() {
    use radeis_sc2sbom::parsers::c::mk_file::parse_mk_content;

    let content = r#"
MAKE_VERSION ?= 4.3
CMAKE_VERSION := 3.25.0
GCC_VERSION = 11.3.0
PYTHON_VERSION = 3.11
CURL_VERSION ?= 8.15.0
OPENSSL_VERSION := 3.0.0
    "#;

    let mk_versions = parse_mk_content(content).unwrap();

    // All VERSION variables should be extracted (filtering happens in parse_mk_files_as_dependencies)
    assert_eq!(mk_versions.versions.len(), 6);
    assert!(mk_versions.versions.contains_key("MAKE_VERSION"));
    assert!(mk_versions.versions.contains_key("CMAKE_VERSION"));
    assert!(mk_versions.versions.contains_key("GCC_VERSION"));
    assert!(mk_versions.versions.contains_key("PYTHON_VERSION"));
    assert!(mk_versions.versions.contains_key("CURL_VERSION"));
    assert!(mk_versions.versions.contains_key("OPENSSL_VERSION"));

    // Test that parse_mk_files_as_dependencies filters out build tools
    // (This would require creating temp files and calling parse_mk_files_as_dependencies)
}

// Comment #5: Test inline comments in VERSION variables
#[test]
fn test_mk_inline_comments() {
    use radeis_sc2sbom::parsers::c::mk_file::parse_mk_content;

    let content = r#"
CURL_VERSION ?= 8.15.0 # latest stable
OPENSSL_VERSION := 3.0.0 # LTS version
ZLIB_VERSION = 1.3.1# no space before comment
LIBSSH2_VERSION ?= 1.11.0 # with # multiple # hashes
    "#;

    let mk_versions = parse_mk_content(content).unwrap();

    // Verify that inline comments are properly stripped
    assert_eq!(mk_versions.versions.len(), 4);
    assert_eq!(
        mk_versions.versions.get("CURL_VERSION"),
        Some(&"8.15.0".to_string())
    );
    assert_eq!(
        mk_versions.versions.get("OPENSSL_VERSION"),
        Some(&"3.0.0".to_string())
    );
    assert_eq!(
        mk_versions.versions.get("ZLIB_VERSION"),
        Some(&"1.3.1".to_string())
    );
    assert_eq!(
        mk_versions.versions.get("LIBSSH2_VERSION"),
        Some(&"1.11.0".to_string())
    );
}

#[test]
fn test_mode1_mode2_deduplication() {
    use radeis_sc2sbom::models::{Dependency, DependencySource};
    use radeis_sc2sbom::parsers::deduplicate_dependencies;

    // Simulate Mode 1 detecting curl@8.15.0 (ecosystem: "system")
    let mode1_dep = Dependency {
        name: "curl".to_string(),
        version: "8.15.0".to_string(),
        ecosystem: "system".to_string(),
        source: DependencySource::Manifest,
        source_file: Some("Makefile".to_string()),
        is_dev: false,
        is_direct: true,
        license: None,
        author: None,
        maintainers: None,
        repository_url: None,
        homepage_url: None,
        checksum_sha256: None,
        checksum_sha512: None,
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    };

    // Simulate Mode 2 detecting curl@8.15.0 (ecosystem: "BUILD-CONFIG")
    let mode2_dep = Dependency {
        name: "curl".to_string(),
        version: "8.15.0".to_string(),
        ecosystem: "BUILD-CONFIG".to_string(),
        source: DependencySource::Manifest,
        source_file: Some("curl.mk".to_string()),
        is_dev: false,
        is_direct: true,
        license: None,
        author: None,
        maintainers: None,
        repository_url: None,
        homepage_url: None,
        checksum_sha256: None,
        checksum_sha512: None,
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    };

    let deps = vec![mode1_dep, mode2_dep];
    let deduplicated = deduplicate_dependencies(deps);

    // Should only have one curl@8.15.0 entry (system preferred over BUILD-CONFIG)
    assert_eq!(deduplicated.len(), 1);
    assert_eq!(deduplicated[0].name, "curl");
    assert_eq!(deduplicated[0].version, "8.15.0");
    assert_eq!(deduplicated[0].ecosystem, "system");
}

#[test]
fn test_multiple_mk_files_same_library() {
    use tempfile::tempdir;

    let temp_dir = tempdir().unwrap();
    let dir1 = temp_dir.path().join("toolchains/3rd_party");
    let dir2 = temp_dir.path().join("other");
    fs::create_dir_all(&dir1).unwrap();
    fs::create_dir_all(&dir2).unwrap();

    // Create curl.mk in two different directories with different versions
    fs::write(dir1.join("curl.mk"), "CURL_VERSION ?= 8.15.0\n").unwrap();
    fs::write(dir2.join("curl.mk"), "CURL_VERSION ?= 7.88.0\n").unwrap();

    // Scan should find both, but deduplication in HashMap will keep one
    let versions = scan_mk_files_for_versions(temp_dir.path(), None).unwrap();

    // Should have exactly one curl version (last one wins in HashMap)
    assert!(versions.contains_key("curl"));
    // The version could be either one depending on directory traversal order
    assert!(
        versions["curl"] == "8.15.0" || versions["curl"] == "7.88.0",
        "Got unexpected version: {}",
        versions["curl"]
    );
}

#[test]
fn test_parse_library_json_full() {
    let content = r#"{"name":"lv_drivers","version":"7.11.0","repository":{"type":"git","url":"https://github.com/littlevgl/lv_drivers.git"},"license":"MIT"}"#;
    let dep = parse_library_json_content(content, Path::new("lv_drivers/library.json"))
        .unwrap()
        .unwrap();
    assert_eq!(dep.name, "lv_drivers");
    assert_eq!(dep.version, "7.11.0");
    assert_eq!(dep.ecosystem, "vendored");
    assert_eq!(
        dep.repository_url,
        Some("https://github.com/littlevgl/lv_drivers.git".to_string())
    );
    assert_eq!(dep.license, Some("MIT".to_string()));
}

#[test]
fn test_parse_library_json_missing_version_uses_unspecified() {
    let content = r#"{"name":"mylib","keywords":["c","embedded"]}"#;
    let dep = parse_library_json_content(content, Path::new("mylib/library.json"))
        .unwrap()
        .unwrap();
    assert_eq!(dep.name, "mylib");
    assert_eq!(dep.version, "unspecified");
}

#[test]
fn test_parse_library_json_missing_name_returns_none() {
    let content = r#"{"version":"1.0.0"}"#;
    let result = parse_library_json_content(content, Path::new("lib/library.json")).unwrap();
    assert!(result.is_none(), "Missing name should return None");
}
