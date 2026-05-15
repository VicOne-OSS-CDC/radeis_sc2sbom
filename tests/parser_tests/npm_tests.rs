use radeis_sc2sbom::parsers::npm::{
    parse_package_json, parse_package_lock_json_with_relationships, parse_yarn_lock,
};
use std::io::Write;
use tempfile::NamedTempFile;

// v0.8.0 Source Tracking Tests

#[test]
fn test_npm_source_tracking_package_json() {
    let content = r#"{"dependencies": {"axios": "^1.0.0"}}"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_package_json(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 1);

    let source = deps[0].source_file.as_ref().unwrap();
    assert!(
        source.starts_with("Identified by the javascript/packagejson extractor from"),
        "Source tracking missing or incorrect: {:?}",
        source
    );
    assert!(source.contains(temp_file.path().to_str().unwrap()));
}

#[test]
fn test_npm_source_tracking_package_lock() {
    let content = r#"{
        "lockfileVersion": 2,
        "packages": {
            "node_modules/axios": {
                "version": "1.0.0"
            }
        }
    }"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let lock_data = parse_package_lock_json_with_relationships(temp_file.path()).unwrap();
    assert!(!lock_data.dependencies.is_empty());

    let source = lock_data.dependencies[0].source_file.as_ref().unwrap();
    assert!(
        source.starts_with("Identified by the javascript/packagelockjson extractor from"),
        "Source tracking missing or incorrect: {:?}",
        source
    );
}

#[test]
fn test_npm_source_tracking_yarn_lock() {
    let content = "axios@^1.0.0:\n  version \"1.0.0\"";
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_yarn_lock(temp_file.path()).unwrap();
    assert!(!deps.is_empty());

    let source = deps[0].source_file.as_ref().unwrap();
    assert!(
        source.starts_with("Identified by the javascript/yarnlock extractor from"),
        "Source tracking missing or incorrect: {:?}",
        source
    );
}

// Existing npm tests (migrated from main.rs)

#[test]
fn test_parse_package_json_with_dependencies() {
    let content = r#"{
        "dependencies": {
            "express": "^4.17.1",
            "axios": "^0.21.1"
        },
        "devDependencies": {
            "jest": "^27.0.0"
        }
    }"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_package_json(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 3);

    // Check regular dependencies
    let express = deps.iter().find(|d| d.name == "express").unwrap();
    assert_eq!(express.version, "^4.17.1");
    assert!(!express.is_dev);

    // Check dev dependencies
    let jest = deps.iter().find(|d| d.name == "jest").unwrap();
    assert_eq!(jest.version, "^27.0.0");
    assert!(jest.is_dev);
}

#[test]
fn test_parse_package_json_empty_dependencies() {
    let content = r#"{}"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_package_json(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 0);
}

#[test]
fn test_parse_package_json_peer_and_optional_dependencies() {
    let content = r#"{
        "peerDependencies": {
            "react": "^17.0.0"
        },
        "optionalDependencies": {
            "fsevents": "^2.3.2"
        }
    }"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_package_json(temp_file.path()).unwrap();
    assert_eq!(deps.len(), 2);

    let react = deps.iter().find(|d| d.name == "react").unwrap();
    assert!(!react.is_dev);

    let fsevents = deps.iter().find(|d| d.name == "fsevents").unwrap();
    assert!(!fsevents.is_dev);
}

#[test]
fn test_parse_yarn_lock_with_scoped_packages() {
    let content = r#"
"@babel/core@^7.12.0":
  version "7.12.3"
  resolved "https://registry.yarnpkg.com/@babel/core/-/core-7.12.3.tgz"

axios@^1.0.0:
  version "1.0.0"
  resolved "https://registry.yarnpkg.com/axios/-/axios-1.0.0.tgz"
"#;
    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_yarn_lock(temp_file.path()).unwrap();
    assert!(deps.len() >= 2);

    let babel = deps.iter().find(|d| d.name == "@babel/core").unwrap();
    assert_eq!(babel.version, "7.12.3");

    let axios = deps.iter().find(|d| d.name == "axios").unwrap();
    assert_eq!(axios.version, "1.0.0");
}
