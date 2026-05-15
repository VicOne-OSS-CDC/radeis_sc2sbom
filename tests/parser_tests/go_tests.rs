use radeis_sc2sbom::parsers::go::parse_go_mod;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_parse_go_mod_inline_require() {
    let content = r#"module example.com/myapp

go 1.18

require github.com/gin-gonic/gin v1.7.0
require github.com/stretchr/testify v1.7.0
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_go_mod(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 2);

    let gin = deps
        .iter()
        .find(|d| d.name == "github.com/gin-gonic/gin")
        .unwrap();
    assert_eq!(gin.version, "v1.7.0");
    assert_eq!(gin.ecosystem, "go");
}

#[test]
fn test_parse_go_mod_block_require() {
    let content = r#"module example.com/myapp

go 1.18

require (
    github.com/gin-gonic/gin v1.7.0
    github.com/stretchr/testify v1.7.0
    golang.org/x/crypto v0.0.0-20210711020723-a769d52b0f97
)
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_go_mod(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 3);

    let crypto = deps
        .iter()
        .find(|d| d.name == "golang.org/x/crypto")
        .unwrap();
    assert_eq!(crypto.version, "v0.0.0-20210711020723-a769d52b0f97");
}
// Source tracking tests

#[test]
fn test_go_source_tracking_mod() {
    let content = r#"module github.com/example/project

go 1.19

require (
    github.com/gin-gonic/gin v1.9.0
    github.com/stretchr/testify v1.8.0
)
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_go_mod(temp_file.path()).unwrap();

    assert!(!deps.is_empty());
    let source = deps[0].source_file.as_ref().unwrap();
    assert!(source.starts_with("Identified by the go/gomod extractor from"));
    assert!(source.contains(temp_file.path().to_str().unwrap()));
}
