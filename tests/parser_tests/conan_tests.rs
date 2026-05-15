use radeis_sc2sbom::parsers::cpp::{parse_conan_lock, parse_conanfile_py, parse_conanfile_txt};
use std::path::Path;

#[test]
fn test_parse_conan_lock() {
    let fixture_path = Path::new("tests/fixtures/cpp/conan.lock");
    let deps = parse_conan_lock(fixture_path).expect("Failed to parse conan.lock");

    // Should have 3 runtime + 1 build + 1 tool + 1 test = 6 dependencies
    assert_eq!(deps.len(), 6);

    // Check runtime dependencies
    let zlib = deps
        .iter()
        .find(|d| d.name == "zlib")
        .expect("zlib not found");
    assert_eq!(zlib.version, "1.2.13");
    assert_eq!(zlib.ecosystem, "conan");
    assert!(!zlib.is_dev);
    assert!(zlib.checksum_sha256.as_ref().unwrap().starts_with("416618"));

    let openssl = deps
        .iter()
        .find(|d| d.name == "openssl")
        .expect("openssl not found");
    assert_eq!(openssl.version, "3.1.2");
    assert!(!openssl.is_dev);

    let boost = deps
        .iter()
        .find(|d| d.name == "boost")
        .expect("boost not found");
    assert_eq!(boost.version, "1.82.0");
    assert!(!boost.is_dev);

    // Check build dependency
    let cmake = deps
        .iter()
        .find(|d| d.name == "cmake")
        .expect("cmake not found");
    assert_eq!(cmake.version, "3.27.0");
    assert!(cmake.is_dev, "cmake should be marked as dev dependency");

    // Check tool_requires dependency
    let ninja = deps
        .iter()
        .find(|d| d.name == "ninja")
        .expect("ninja not found");
    assert_eq!(ninja.version, "1.11.1");
    assert!(ninja.is_dev, "ninja should be marked as dev dependency");

    // Check test_requires dependency
    let gtest = deps
        .iter()
        .find(|d| d.name == "gtest")
        .expect("gtest not found");
    assert_eq!(gtest.version, "1.14.0");
    assert!(gtest.is_dev, "gtest should be marked as dev dependency");
}

#[test]
fn test_parse_conanfile_txt() {
    let fixture_path = Path::new("tests/fixtures/cpp/conanfile.txt");
    let deps = parse_conanfile_txt(fixture_path).expect("Failed to parse conanfile.txt");

    // Should have 3 requires + 1 build + 1 tool + 1 test = 6 dependencies
    assert!(
        deps.len() >= 6,
        "Expected at least 6 dependencies, got {}",
        deps.len()
    );

    // Check runtime dependencies
    let zlib = deps
        .iter()
        .find(|d| d.name == "zlib")
        .expect("zlib not found");
    assert_eq!(zlib.version, "1.2.13");
    assert_eq!(zlib.ecosystem, "conan");
    assert!(!zlib.is_dev);

    let openssl = deps
        .iter()
        .find(|d| d.name == "openssl")
        .expect("openssl not found");
    assert_eq!(openssl.version, ">=3.0");
    assert!(!openssl.is_dev);

    let boost = deps
        .iter()
        .find(|d| d.name == "boost")
        .expect("boost not found");
    assert_eq!(boost.version, "1.82.0");
    assert!(!boost.is_dev);

    // Check build_requires
    let cmake = deps
        .iter()
        .find(|d| d.name == "cmake")
        .expect("cmake not found");
    assert!(cmake.is_dev);

    // Check tool_requires
    let ninja = deps
        .iter()
        .find(|d| d.name == "ninja")
        .expect("ninja not found");
    assert!(ninja.is_dev);

    // Check test_requires
    let gtest = deps
        .iter()
        .find(|d| d.name == "gtest")
        .expect("gtest not found");
    assert!(gtest.is_dev);
}

#[test]
fn test_parse_conanfile_py() {
    let fixture_path = Path::new("tests/fixtures/cpp/conanfile.py");
    let deps = parse_conanfile_py(fixture_path).expect("Failed to parse conanfile.py");

    // Should have at least: zlib, openssl, boost (via self.requires), cmake, ninja, gtest, doxygen
    assert!(
        deps.len() >= 7,
        "Expected at least 7 dependencies, got {}",
        deps.len()
    );

    // Check list-format dependencies
    assert!(deps
        .iter()
        .any(|d| d.name == "zlib" && d.version == "1.2.13" && !d.is_dev));
    assert!(deps
        .iter()
        .any(|d| d.name == "openssl" && d.version == "3.1.2" && !d.is_dev));

    // Check method-call dependencies
    assert!(deps
        .iter()
        .any(|d| d.name == "boost" && d.version == "1.82.0" && !d.is_dev));

    // Check build dependencies
    assert!(deps.iter().any(|d| d.name == "cmake" && d.is_dev));
    assert!(deps.iter().any(|d| d.name == "doxygen" && d.is_dev));

    // Check tool dependencies
    assert!(deps.iter().any(|d| d.name == "ninja" && d.is_dev));

    // Check test dependencies
    assert!(deps.iter().any(|d| d.name == "gtest" && d.is_dev));
}

#[test]
fn test_conan_lock_with_simple_references() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let lock_content = r#"{
        "version": "0.5",
        "requires": ["zlib/1.2.13", "openssl/3.1.2"],
        "build_requires": [],
        "tool_requires": [],
        "test_requires": [],
        "python_requires": []
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", lock_content).unwrap();

    let deps = parse_conan_lock(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().all(|d| d.ecosystem == "conan"));
    assert!(deps.iter().all(|d| d.checksum_sha256.is_none()));
}

#[test]
fn test_conanfile_txt_version_ranges() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let content = r#"
[requires]
zlib/[>=1.2]
openssl/[>3.0 <4.0]
boost/[~=1.82]
fmt/[^10.0]

[options]
openssl:shared=True
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = parse_conanfile_txt(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 4);

    assert!(deps
        .iter()
        .any(|d| d.name == "zlib" && d.version == ">=1.2"));
    assert!(deps
        .iter()
        .any(|d| d.name == "openssl" && d.version == ">3.0 <4.0"));
    assert!(deps
        .iter()
        .any(|d| d.name == "boost" && d.version == "~=1.82"));
    assert!(deps.iter().any(|d| d.name == "fmt" && d.version == "^10.0"));
}

#[test]
fn test_conanfile_py_mixed_formats() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let content = r#"
from conan import ConanFile

class TestConan(ConanFile):
    requires = ["zlib/1.2.13"]

    def requirements(self):
        self.requires("openssl/3.1.2")
        self.requires("boost/1.82.0")
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = parse_conanfile_py(temp_file.path()).unwrap();
    assert!(deps.len() >= 3);
    assert!(deps.iter().any(|d| d.name == "zlib"));
    assert!(deps.iter().any(|d| d.name == "openssl"));
    assert!(deps.iter().any(|d| d.name == "boost"));
}

#[test]
fn test_conan_purl_format() {
    use radeis_sc2sbom::formats::spdx::create_package_url;
    use radeis_sc2sbom::models::dependency::{Dependency, DependencySource};

    let dep = Dependency {
        name: "zlib".to_string(),
        version: "1.2.13".to_string(),
        ecosystem: "conan".to_string(),
        source: DependencySource::LockFile,
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
    };

    let purl = create_package_url(&dep);
    assert_eq!(purl, "pkg:conan/zlib@1.2.13");
}

#[test]
fn test_conanfile_txt_with_user_channel() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let content = r#"
[requires]
mylib/1.0.0@user/stable
otherlib/2.3.4@company/testing
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = parse_conanfile_txt(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 2);

    // User/channel should be stripped from version
    assert!(deps
        .iter()
        .any(|d| d.name == "mylib" && d.version == "1.0.0"));
    assert!(deps
        .iter()
        .any(|d| d.name == "otherlib" && d.version == "2.3.4"));
}

#[test]
fn test_malformed_conan_lock() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let lock_content = r#"{"version": "0.5", "invalid_json"}"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", lock_content).unwrap();

    // Should return empty vec with warning, not error
    let deps = parse_conan_lock(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_empty_conanfile_txt() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let content = r#"
[options]
openssl:shared=True
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "{}", content).unwrap();

    let deps = parse_conanfile_txt(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 0);
}
