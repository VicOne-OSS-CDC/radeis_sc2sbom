use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

lazy_static::lazy_static! {
    /// Regex for extracting -l flags from Makefile.am
    /// Pattern: -l followed by library name (letters, numbers, underscores, hyphens, dots, plus)
    static ref LIB_FLAG_PATTERN: Regex = Regex::new(r"-l([a-zA-Z0-9_+.-]+)").unwrap();
}

/// Parse Makefile.am for library dependencies
///
/// Extracts -l flags from LDADD, LIBADD, and similar variables
/// Example:
/// ```text
/// myapp_LDADD = -lssl -lcrypto -lpthread
/// libfoo_la_LIBADD = -lz
/// ```
pub fn parse_makefile_am(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();

    // Extract library flags from various *_LDADD and *_LIBADD variables
    let lib_flags = extract_lib_flags(&content);

    for lib in lib_flags {
        if seen.insert(lib.clone()) {
            dependencies.push(Dependency {
                name: lib,
                version: "unspecified".to_string(),
                ecosystem: "autotools".to_string(),
                source: DependencySource::Manifest,
                source_file: Some(format_source_info("autotools", path, None, false)),
                is_dev: false,
                is_direct: true,
                ..Default::default()
            });
        }
    }

    Ok(dependencies)
}

/// Extract library names from -l flags
/// Pattern: -l{name}
fn extract_lib_flags(content: &str) -> Vec<String> {
    let mut libs = Vec::new();

    for cap in LIB_FLAG_PATTERN.captures_iter(content) {
        libs.push(cap[1].to_string());
    }

    libs
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_lib_flags() {
        let content = "myapp_LDADD = -lssl -lcrypto -lpthread";
        let libs = extract_lib_flags(content);
        assert_eq!(libs.len(), 3);
        assert!(libs.contains(&"ssl".to_string()));
        assert!(libs.contains(&"crypto".to_string()));
        assert!(libs.contains(&"pthread".to_string()));
    }

    #[test]
    fn test_parse_makefile_am() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "myapp_LDADD = -lssl -lcrypto").unwrap();
        writeln!(file, "libfoo_la_LIBADD = -lz").unwrap();

        let deps = parse_makefile_am(file.path()).unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps
            .iter()
            .any(|d| d.name == "ssl" && d.ecosystem == "autotools"));
        assert!(deps.iter().any(|d| d.name == "z"));
    }

    #[test]
    fn test_parse_makefile_am_deduplication() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "myapp_LDADD = -lssl -lssl").unwrap();

        let deps = parse_makefile_am(file.path()).unwrap();
        assert_eq!(deps.len(), 1);
    }
}
