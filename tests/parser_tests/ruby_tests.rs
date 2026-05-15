use radeis_sc2sbom::parsers::ruby::parse_gemfile;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_parse_gemfile() {
    let content = r#"source 'https://rubygems.org'

gem 'rails', '~> 6.1.0'
gem 'pg', '>= 1.0'
gem 'puma'
gem "devise", "4.8.0"
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gemfile(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 4);

    let rails = deps.iter().find(|d| d.name == "rails").unwrap();
    assert_eq!(rails.version, "~> 6.1.0");
    assert_eq!(rails.ecosystem, "rubygems");

    let puma = deps.iter().find(|d| d.name == "puma").unwrap();
    assert_eq!(puma.version, "unspecified");

    let devise = deps.iter().find(|d| d.name == "devise").unwrap();
    assert_eq!(devise.version, "4.8.0");
}

#[test]
fn test_parse_gemfile_preserve_operators() {
    let content = r#"source 'https://rubygems.org'

gem 'rails', '~> 6.1.0'
gem 'pg', '>= 1.0'
gem 'puma'
gem "devise", "4.8.0"
gem 'nokogiri', '~> 1.13.0'
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gemfile(temp_file.path()).unwrap();

    assert_eq!(deps.len(), 5);

    let rails = deps.iter().find(|d| d.name == "rails").unwrap();
    assert_eq!(rails.version, "~> 6.1.0");
    assert_eq!(rails.ecosystem, "rubygems");

    let pg = deps.iter().find(|d| d.name == "pg").unwrap();
    assert_eq!(pg.version, ">= 1.0");

    let devise = deps.iter().find(|d| d.name == "devise").unwrap();
    assert_eq!(devise.version, "4.8.0");
}
// Source tracking tests

#[test]
fn test_ruby_source_tracking_gemfile() {
    let content = r#"
source 'https://rubygems.org'

gem 'rails', '~> 6.0'
gem 'pg', '>= 1.1'
"#;

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(content.as_bytes()).unwrap();
    temp_file.flush().unwrap();

    let deps = parse_gemfile(temp_file.path()).unwrap();

    assert!(!deps.is_empty());
    let source = deps[0].source_file.as_ref().unwrap();
    assert!(source.starts_with("Identified by the ruby/gemfile extractor from"));
    assert!(source.contains(temp_file.path().to_str().unwrap()));
}
