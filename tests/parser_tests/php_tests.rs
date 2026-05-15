use radeis_sc2sbom::parsers::php::parse_composer_json;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_parse_composer_json() {
    let content = r#"{
        "name": "myapp/project",
        "require": {
            "php": ">=7.4",
            "symfony/console": "^5.3",
            "guzzlehttp/guzzle": "~6.0"
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_composer_json(temp_file.path()).unwrap();

    // Should have 2 dependencies (php is filtered out)
    assert_eq!(deps.len(), 2);

    let symfony = deps.iter().find(|d| d.name == "symfony/console").unwrap();
    assert_eq!(symfony.version, "^5.3");
    assert_eq!(symfony.ecosystem, "composer");

    let guzzle = deps.iter().find(|d| d.name == "guzzlehttp/guzzle").unwrap();
    assert_eq!(guzzle.version, "~6.0");
}

#[test]
fn test_parse_composer_json_require_dev() {
    let content = r#"{
        "name": "myapp/project",
        "require": {
            "php": ">=7.4",
            "symfony/console": "^5.3",
            "guzzlehttp/guzzle": "~6.0"
        },
        "require-dev": {
            "phpunit/phpunit": "^9.5",
            "symfony/var-dumper": "^5.3"
        }
    }"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_composer_json(temp_file.path()).unwrap();

    // Should have 4 dependencies (php is filtered out)
    assert_eq!(deps.len(), 4);

    let symfony = deps.iter().find(|d| d.name == "symfony/console").unwrap();
    assert_eq!(symfony.ecosystem, "composer");

    let phpunit = deps.iter().find(|d| d.name == "phpunit/phpunit").unwrap();
    assert_eq!(phpunit.ecosystem, "composer");
    assert_eq!(phpunit.is_dev, true);
    assert_eq!(phpunit.version, "^9.5");
}
// Source tracking tests

#[test]
fn test_php_source_tracking_composer() {
    let content = r#"{
    "name": "test/project",
    "require": {
        "symfony/console": "^5.3",
        "guzzlehttp/guzzle": "~6.0"
    },
    "require-dev": {
        "phpunit/phpunit": "^9.5"
    }
}"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_composer_json(temp_file.path()).unwrap();

    assert!(!deps.is_empty());
    let source = deps[0].source_file.as_ref().unwrap();
    assert!(source.starts_with("Identified by the php/composer extractor from"));
    assert!(source.contains(temp_file.path().to_str().unwrap()));
}
