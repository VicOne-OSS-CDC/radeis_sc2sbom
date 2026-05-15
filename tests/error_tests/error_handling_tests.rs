use radeis_sc2sbom::parsers::cargo::parse_cargo_toml;
use radeis_sc2sbom::parsers::npm::parse_package_json;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

#[test]
fn test_parse_invalid_json() {
    let content = r#"{ invalid json"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = parse_package_json(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_parse_invalid_toml() {
    let content = r#"[invalid toml"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let result = parse_cargo_toml(temp_file.path());
    assert!(result.is_err());
}

#[test]
fn test_parse_nonexistent_file() {
    let result = parse_package_json(Path::new("/nonexistent/file.json"));
    assert!(result.is_err());
}
