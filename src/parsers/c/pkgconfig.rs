use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use std::fs;
use std::path::Path;

/// Check if a line in a .pc file is a variable definition
/// Variable definitions have '=' before any ':', and don't start with uppercase
/// Examples: prefix=/usr, exec_prefix=${prefix}
/// Non-variables: Name: OpenSSL, URL: https://example.com/?param=value
fn is_pc_variable_definition(line: &str) -> bool {
    let has_equals_before_colon = if let Some(eq_pos) = line.find('=') {
        match line.find(':') {
            Some(colon_pos) => eq_pos < colon_pos,
            None => true,
        }
    } else {
        false
    };

    has_equals_before_colon && !line.chars().next().map_or(false, |c| c.is_uppercase())
}

/// Parse a .pc (pkg-config) file
///
/// .pc files follow an INI-like format with key-value pairs and sections.
/// Example:
/// ```text
/// Name: OpenSSL
/// Version: 3.0.2
/// Description: Secure Sockets Layer and cryptography libraries
/// Requires: libcrypto libssl
/// ```
pub fn parse_pc_file(path: &Path) -> Result<Dependency, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Err(Box::new(e));
        }
    };

    let mut name = String::new();
    let mut version = String::from("unspecified");
    let mut license: Option<String> = None;
    // Note: Description field is not currently stored in Dependency struct
    // Could be added in future if needed for metadata enrichment

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Skip variable definitions (prefix=..., exec_prefix=...)
        if is_pc_variable_definition(line) {
            continue;
        }

        // Parse key-value pairs
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "Name" => name = value.to_string(),
                "Version" => version = value.to_string(),
                "License" => license = Some(value.to_string()),
                // Description is parsed but not stored (no field in Dependency struct)
                "Description" => {} // Intentionally ignored for now
                _ => {}
            }
        }
    }

    if name.is_empty() {
        // Use filename as fallback (remove .pc extension)
        name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
    }

    Ok(Dependency {
        name,
        version,
        ecosystem: "pkg-config".to_string(),
        source: DependencySource::Manifest,
        source_file: Some(format_source_info("pkg-config", path, None, false)),
        is_dev: false,
        is_direct: true,
        license,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_basic_pc_file() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name: OpenSSL").unwrap();
        writeln!(file, "Version: 3.0.2").unwrap();
        writeln!(file, "Description: Secure Sockets Layer").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert_eq!(dep.name, "OpenSSL");
        assert_eq!(dep.version, "3.0.2");
        assert_eq!(dep.ecosystem, "pkg-config");
        assert_eq!(dep.source, DependencySource::Manifest);
    }

    #[test]
    fn test_parse_pc_file_with_variables() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "prefix=/usr").unwrap();
        writeln!(file, "exec_prefix=${{prefix}}").unwrap();
        writeln!(file, "Name: zlib").unwrap();
        writeln!(file, "Version: 1.2.13").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert_eq!(dep.name, "zlib");
        assert_eq!(dep.version, "1.2.13");
    }

    #[test]
    fn test_parse_pc_file_no_version() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name: pthread").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert_eq!(dep.name, "pthread");
        assert_eq!(dep.version, "unspecified");
    }

    #[test]
    fn test_parse_pc_file_no_name_uses_filename() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Version: 1.0.0").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert!(!dep.name.is_empty());
        assert_eq!(dep.version, "1.0.0");
    }

    #[test]
    fn test_parse_pc_file_with_license() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name: mylib").unwrap();
        writeln!(file, "Version: 1.0.0").unwrap();
        writeln!(file, "License: MIT").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert_eq!(dep.name, "mylib");
        assert_eq!(dep.license, Some("MIT".to_string()));
    }

    #[test]
    fn test_parse_pc_file_with_url_containing_equals() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "Name: mylib").unwrap();
        writeln!(file, "Version: 2.0.0").unwrap();
        writeln!(file, "URL: https://example.com/?param=value").unwrap();

        let dep = parse_pc_file(file.path()).unwrap();
        assert_eq!(dep.name, "mylib");
        assert_eq!(dep.version, "2.0.0");
        // URL is not currently extracted, but shouldn't cause parsing to fail
    }
}
