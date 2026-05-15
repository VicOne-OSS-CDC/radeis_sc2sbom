use radeis_sc2sbom::parsers::java::parse_pom_xml;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_parse_pom_xml() {
    let content = r#"<project>
        <modelVersion>4.0.0</modelVersion>
        <groupId>com.example</groupId>
        <artifactId>myapp</artifactId>
    </project>"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_pom_xml(temp_file.path()).unwrap();

    // Currently returns empty as it's basic detection only
    assert_eq!(deps.len(), 0);
}
