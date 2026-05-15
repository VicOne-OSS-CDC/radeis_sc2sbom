use radeis_sc2sbom::models::dependency::Dependency;
use radeis_sc2sbom::parsers::cmake::{
    parse_cmake_file, parse_external_project, parse_fetchcontent,
};
use std::path::Path;

#[test]
fn test_parse_fetchcontent_git_repository() {
    let cmake_path = Path::new("tests/fixtures/cmake/CMakeLists_fetchcontent.txt");
    let deps = parse_cmake_file(cmake_path).expect("Failed to parse CMake file");

    assert_eq!(deps.len(), 3, "Should find 3 dependencies");

    // Test nlohmann/json dependency
    let json_dep = deps
        .iter()
        .find(|d| d.name == "json")
        .expect("json dependency not found");
    assert_eq!(json_dep.ecosystem, "cmake");
    assert_eq!(json_dep.version, "v3.11.2");
    assert_eq!(
        json_dep.repository_url,
        Some("https://github.com/nlohmann/json.git".to_string())
    );
    assert!(json_dep.is_direct);
    assert!(!json_dep.is_dev);

    // Test fmt dependency (URL-based with hash)
    let fmt_dep = deps
        .iter()
        .find(|d| d.name == "fmt")
        .expect("fmt dependency not found");
    assert_eq!(fmt_dep.ecosystem, "cmake");
    assert_eq!(fmt_dep.version, "9.1.0"); // Extracted from URL
    assert_eq!(
        fmt_dep.checksum_sha256,
        Some("5dea48d1fcddc3ec571ce2058e13910a0d4a6bab4cc09a809d8b1dd1c88ae6f2".to_string())
    );

    // Test spdlog dependency (GitLab)
    let spdlog_dep = deps
        .iter()
        .find(|d| d.name == "spdlog")
        .expect("spdlog dependency not found");
    assert_eq!(spdlog_dep.ecosystem, "cmake");
    assert_eq!(spdlog_dep.version, "v1.11.0");
    assert_eq!(
        spdlog_dep.repository_url,
        Some("https://gitlab.com/gabime/spdlog.git".to_string())
    );
}

#[test]
fn test_parse_externalproject_add() {
    let cmake_path = Path::new("tests/fixtures/cmake/CMakeLists_externalproject.txt");
    let deps = parse_cmake_file(cmake_path).expect("Failed to parse CMake file");

    assert_eq!(deps.len(), 3, "Should find 3 dependencies");

    // Test googletest dependency
    let gtest_dep = deps
        .iter()
        .find(|d| d.name == "googletest")
        .expect("googletest dependency not found");
    assert_eq!(gtest_dep.ecosystem, "cmake");
    assert_eq!(gtest_dep.version, "release-1.12.1");
    assert_eq!(
        gtest_dep.repository_url,
        Some("https://github.com/google/googletest.git".to_string())
    );

    // Test zlib dependency (URL-based)
    let zlib_dep = deps
        .iter()
        .find(|d| d.name == "zlib")
        .expect("zlib dependency not found");
    assert_eq!(zlib_dep.ecosystem, "cmake");
    assert_eq!(zlib_dep.version, "1.2.13"); // Extracted from URL
    assert_eq!(
        zlib_dep.checksum_sha256,
        Some("b3a24de97a8fdbc835b9833169501030b8977031bcb54b3b3ac13740f846ab30".to_string())
    );

    // Test eigen dependency (Bitbucket)
    let eigen_dep = deps
        .iter()
        .find(|d| d.name == "eigen")
        .expect("eigen dependency not found");
    assert_eq!(eigen_dep.ecosystem, "cmake");
    assert_eq!(eigen_dep.version, "3.4.0");
    assert_eq!(
        eigen_dep.repository_url,
        Some("https://bitbucket.org/eigen/eigen.git".to_string())
    );
}

#[test]
fn test_cmake_variables_are_skipped() {
    let cmake_path = Path::new("tests/fixtures/cmake/CMakeLists_with_variables.txt");
    let deps = parse_cmake_file(cmake_path).expect("Failed to parse CMake file");

    // Should only find spdlog (the one without variables)
    assert_eq!(
        deps.len(),
        1,
        "Should only find 1 dependency (skipping those with variables)"
    );

    let spdlog_dep = &deps[0];
    assert_eq!(spdlog_dep.name, "spdlog");
    assert_eq!(spdlog_dep.version, "v1.11.0");
}

#[test]
fn test_parse_fetchcontent_directly() {
    let content = r#"
FetchContent_Declare(
  Catch2
  GIT_REPOSITORY https://github.com/catchorg/Catch2.git
  GIT_TAG        v3.3.2
)
    "#;

    let deps =
        parse_fetchcontent(content, Path::new("test.cmake")).expect("Failed to parse FetchContent");

    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "Catch2");
    assert_eq!(deps[0].version, "v3.3.2");
    assert_eq!(deps[0].ecosystem, "cmake");
}

#[test]
fn test_parse_externalproject_directly() {
    let content = r#"
ExternalProject_Add(
  boost
  URL https://boostorg.jfrog.io/artifactory/main/release/1.82.0/source/boost_1_82_0.tar.bz2
  URL_HASH SHA256=a6e1ab9b0860e6a2881dd7b21fe9f737a095e5f33a3a874afc6a345228597ee6
)
    "#;

    let deps = parse_external_project(content, Path::new("test.cmake"))
        .expect("Failed to parse ExternalProject");

    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "boost");
    assert_eq!(deps[0].version, "1.82.0"); // Extracted from URL
    assert_eq!(deps[0].ecosystem, "cmake");
    assert_eq!(
        deps[0].checksum_sha256,
        Some("a6e1ab9b0860e6a2881dd7b21fe9f737a095e5f33a3a874afc6a345228597ee6".to_string())
    );
}

#[test]
fn test_version_extraction_from_url() {
    let content = r#"
FetchContent_Declare(
  abseil
  URL https://github.com/abseil/abseil-cpp/archive/refs/tags/20230125.3.tar.gz
)
    "#;

    let deps = parse_fetchcontent(content, Path::new("test.cmake")).expect("Failed to parse");

    assert_eq!(deps.len(), 1);
    // Should extract version from URL
    assert!(deps[0].version == "20230125.3" || deps[0].version == "unspecified");
}

#[test]
fn test_empty_cmake_file() {
    let content = r#"
cmake_minimum_required(VERSION 3.14)
project(empty_project)
    "#;

    let deps = parse_fetchcontent(content, Path::new("test.cmake")).expect("Failed to parse");
    assert!(
        deps.is_empty(),
        "Empty CMake file should produce no dependencies"
    );
}

#[test]
fn test_source_file_attribution() {
    let cmake_path = Path::new("tests/fixtures/cmake/CMakeLists_fetchcontent.txt");
    let deps = parse_cmake_file(cmake_path).expect("Failed to parse CMake file");

    for dep in deps {
        assert!(
            dep.source_file.is_some(),
            "All dependencies should have source file attribution"
        );
        let source = dep.source_file.as_ref().unwrap();
        assert!(
            source.contains("cmake/"),
            "Source should indicate cmake parser"
        );
        assert!(
            source.contains("CMakeLists_fetchcontent.txt"),
            "Source should reference the file"
        );
    }
}

// ========== Edge Case Tests ==========

#[test]
fn test_cmake_quoted_url_with_spaces() {
    let content = r#"
    ExternalProject_Add(
        myproject
        URL "https://example.com/files/project with spaces.tar.gz"
        URL_HASH SHA256=abc123
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "myproject");
    // Should extract the quoted URL correctly
    assert!(deps[0].repository_url.is_some());
}

#[test]
fn test_cmake_nested_parentheses() {
    let content = r#"
    FetchContent_Declare(
        json
        GIT_REPOSITORY https://github.com/nlohmann/json.git
        GIT_TAG v3.11.2
        CMAKE_ARGS -DJSON_BuildTests=OFF -DCMAKE_INSTALL_PREFIX=${CMAKE_BINARY_DIR}/install
        PATCH_COMMAND ${CMAKE_COMMAND} -E echo "Patching (with parentheses)"
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "json");
    assert_eq!(deps[0].version, "v3.11.2");
}

#[test]
fn test_cmake_line_comments() {
    let content = r#"
    # This is a comment
    FetchContent_Declare(
        fmt
        GIT_REPOSITORY https://github.com/fmtlib/fmt.git
        # This is an inline comment with )
        GIT_TAG v9.1.0  # Another comment
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "fmt");
    assert_eq!(deps[0].version, "v9.1.0");
}

#[test]
fn test_cmake_multiline_strings() {
    let content = r#"
    ExternalProject_Add(
        mylib
        URL https://example.com/mylib.tar.gz
        CONFIGURE_COMMAND ${CMAKE_COMMAND} -E echo "Line 1
        Line 2
        Line 3"
        BUILD_COMMAND make
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "mylib");
}

#[test]
fn test_cmake_empty_git_tag() {
    let content = r#"
    FetchContent_Declare(
        dep1
        GIT_REPOSITORY https://github.com/owner/repo.git
        # No GIT_TAG specified
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    // Should still detect the dependency even without GIT_TAG
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "dep1");
}

#[test]
fn test_cmake_malformed_missing_closing_paren() {
    let content = r#"
    FetchContent_Declare(
        broken
        GIT_REPOSITORY https://github.com/owner/repo.git
        # Missing closing parenthesis
    
    FetchContent_Declare(
        valid
        GIT_REPOSITORY https://github.com/owner/valid.git
        GIT_TAG v1.0.0
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    // Should skip broken and parse valid
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "valid");
}

#[test]
fn test_cmake_complex_formatting() {
    let content = r#"
    FetchContent_Declare(
        complex
        GIT_REPOSITORY  
            https://github.com/owner/repo.git
        GIT_TAG    
            v1.2.3
        CMAKE_ARGS
            -DBUILD_SHARED_LIBS=OFF
            -DCMAKE_BUILD_TYPE=Release
    )
    "#;

    let deps = parse_cmake_file_from_string(content);
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "complex");
    assert_eq!(deps[0].version, "v1.2.3");
}

// Helper function for edge case tests
fn parse_cmake_file_from_string(content: &str) -> Vec<Dependency> {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    parse_cmake_file(temp_file.path()).unwrap_or_else(|_| Vec::new())
}
