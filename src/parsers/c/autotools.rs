use super::pkgconfig_detector::extract_pkgconfig_from_configure;
use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Parse configure.ac or configure.in for Autotools dependencies
///
/// Extracts dependencies from:
/// - AC_CHECK_LIB(library, function)
/// - AC_SEARCH_LIBS(function, libraries)
/// - PKG_CHECK_MODULES (via pkgconfig_detector)
pub fn parse_configure_ac(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();

    // First, extract PKG_CHECK_MODULES dependencies
    if let Ok(pkg_deps) = extract_pkgconfig_from_configure(path) {
        for dep in pkg_deps {
            if seen.insert(dep.name.clone()) {
                dependencies.push(dep);
            }
        }
    }

    // Pattern 1: AC_CHECK_LIB(library, function)
    // Example: AC_CHECK_LIB([pthread], [pthread_create])
    let ac_check_lib_pattern = Regex::new(r"AC_CHECK_LIB\s*\(\s*\[?([a-zA-Z0-9_-]+)\]?\s*,")?;

    for cap in ac_check_lib_pattern.captures_iter(&content) {
        let lib_name = &cap[1];
        if seen.insert(lib_name.to_string()) {
            dependencies.push(Dependency {
                name: lib_name.to_string(),
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

    // Pattern 2: AC_SEARCH_LIBS(function, [lib1 lib2 lib3])
    // Example: AC_SEARCH_LIBS([sqrt], [m])
    // Use more specific pattern to avoid unbalanced bracket matching
    let ac_search_libs_pattern = Regex::new(
        r"AC_SEARCH_LIBS\s*\(\s*\[?(?:[a-zA-Z0-9_]+)\]?\s*,\s*\[?([a-zA-Z0-9_\s-]+)\]?\s*\)",
    )?;

    for cap in ac_search_libs_pattern.captures_iter(&content) {
        let libs_str = &cap[1];
        for lib in libs_str.split_whitespace() {
            let lib = lib.trim();
            if !lib.is_empty() && seen.insert(lib.to_string()) {
                dependencies.push(Dependency {
                    name: lib.to_string(),
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
    }

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_ac_check_lib() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "AC_CHECK_LIB([pthread], [pthread_create])").unwrap();
        writeln!(file, "AC_CHECK_LIB([m], [sqrt])").unwrap();

        let deps = parse_configure_ac(file.path()).unwrap();
        assert_eq!(deps.len(), 2);
        assert!(deps
            .iter()
            .any(|d| d.name == "pthread" && d.ecosystem == "autotools"));
        assert!(deps
            .iter()
            .any(|d| d.name == "m" && d.ecosystem == "autotools"));
    }

    #[test]
    fn test_parse_ac_search_libs() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "AC_SEARCH_LIBS([sqrt], [m])").unwrap();

        let deps = parse_configure_ac(file.path()).unwrap();
        assert!(deps.iter().any(|d| d.name == "m"));
    }

    #[test]
    fn test_parse_pkg_check_modules() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "PKG_CHECK_MODULES([GLIB], [glib-2.0 >= 2.50])").unwrap();

        let deps = parse_configure_ac(file.path()).unwrap();
        assert!(deps
            .iter()
            .any(|d| d.name == "glib-2.0" && d.ecosystem == "pkg-config"));
    }

    #[test]
    fn test_parse_mixed_dependencies() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "AC_CHECK_LIB([ssl], [SSL_new])").unwrap();
        writeln!(file, "PKG_CHECK_MODULES([OPENSSL], [openssl >= 3.0])").unwrap();

        let deps = parse_configure_ac(file.path()).unwrap();
        assert!(deps
            .iter()
            .any(|d| d.name == "ssl" && d.ecosystem == "autotools"));
        assert!(deps
            .iter()
            .any(|d| d.name == "openssl" && d.ecosystem == "pkg-config"));
    }
}
