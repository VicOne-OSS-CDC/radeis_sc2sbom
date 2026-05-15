use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

/// Extract pkg-config package names from configure.ac PKG_CHECK_MODULES calls
///
/// Example:
/// ```text
/// PKG_CHECK_MODULES([GLIB], [glib-2.0 >= 2.50])
/// PKG_CHECK_MODULES([OPENSSL], [openssl >= 3.0])
/// ```
pub fn extract_pkgconfig_from_configure(
    path: &Path,
) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();

    // Pattern: PKG_CHECK_MODULES([VAR_NAME], [package >= version])
    let pkg_check_pattern =
        Regex::new(r"PKG_CHECK_MODULES\s*\(\s*\[?(\w+)\]?\s*,\s*\[?([^\]]+)\]?")?;

    for cap in pkg_check_pattern.captures_iter(&content) {
        let packages_str = &cap[2];

        // Parse package specifications (can have multiple packages)
        let packages = parse_pkg_spec(packages_str);

        for (name, version) in packages {
            let mut dep = Dependency {
                name,
                version,
                ecosystem: "pkg-config".to_string(),
                source: DependencySource::Manifest,
                source_file: Some(format_source_info("pkg-config", path, None, false)),
                is_dev: false,
                is_direct: true,
                ..Default::default()
            };
            if dep.license.is_none() {
                dep.license = crate::parsers::c::known_licenses::lookup(&dep.name);
            }
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}

/// Extract pkg-config package names from Makefile pkg-config invocations
///
/// Example:
/// ```text
/// CFLAGS = $(shell pkg-config --cflags openssl glib-2.0)
/// LIBS = $(shell pkg-config --libs zlib)
/// ```
pub fn extract_pkgconfig_from_makefile(
    path: &Path,
) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut seen = HashSet::new();

    // Pattern: pkg-config --flags package1 package2 ...
    // Match package names (alphanumeric, underscore, dot, hyphen, space) after the flag
    // Using non-greedy match and specific character class to avoid ReDoS
    let pkg_config_pattern =
        Regex::new(r"pkg-config\s+--[a-z-]+\s+([a-zA-Z0-9_. -]+?)(?:\s*\)|$)")?;

    for cap in pkg_config_pattern.captures_iter(&content) {
        let packages_str = &cap[1].trim();

        // Split by whitespace to get individual packages
        for package in packages_str.split_whitespace() {
            let package = package.trim();
            if package.is_empty() || seen.contains(package) {
                continue;
            }
            seen.insert(package.to_string());

            let mut dep = Dependency {
                name: package.to_string(),
                version: "unspecified".to_string(),
                ecosystem: "pkg-config".to_string(),
                source: DependencySource::Manifest,
                source_file: Some(format_source_info("pkg-config", path, None, false)),
                is_dev: false,
                is_direct: true,
                ..Default::default()
            };
            if dep.license.is_none() {
                dep.license = crate::parsers::c::known_licenses::lookup(&dep.name);
            }
            dependencies.push(dep);
        }
    }

    Ok(dependencies)
}

/// Parse package specification from PKG_CHECK_MODULES
/// Examples:
///   "glib-2.0 >= 2.50" -> ("glib-2.0", ">=2.50")
///   "openssl" -> ("openssl", "unspecified")
///   "glib-2.0 >= 2.50 gobject-2.0" -> [("glib-2.0", ">=2.50"), ("gobject-2.0", "unspecified")]
fn parse_pkg_spec(spec: &str) -> Vec<(String, String)> {
    let mut packages = Vec::new();
    let parts: Vec<&str> = spec.trim().split_whitespace().collect();

    let mut i = 0;
    while i < parts.len() {
        let name = parts[i];

        // Check if next token is a version constraint
        if i + 2 < parts.len() && is_version_operator(parts[i + 1]) {
            let operator = parts[i + 1];
            let version = parts[i + 2];
            packages.push((name.to_string(), format!("{}{}", operator, version)));
            i += 3;
        } else {
            packages.push((name.to_string(), "unspecified".to_string()));
            i += 1;
        }
    }

    packages
}

fn is_version_operator(s: &str) -> bool {
    matches!(s, ">=" | "<=" | "=" | ">" | "<" | "!=")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pkg_spec_with_version() {
        let result = parse_pkg_spec("glib-2.0 >= 2.50");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "glib-2.0");
        assert_eq!(result[0].1, ">=2.50");
    }

    #[test]
    fn test_parse_pkg_spec_without_version() {
        let result = parse_pkg_spec("openssl");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "openssl");
        assert_eq!(result[0].1, "unspecified");
    }

    #[test]
    fn test_parse_pkg_spec_multiple() {
        let result = parse_pkg_spec("glib-2.0 >= 2.50 gobject-2.0");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, "glib-2.0");
        assert_eq!(result[0].1, ">=2.50");
        assert_eq!(result[1].0, "gobject-2.0");
        assert_eq!(result[1].1, "unspecified");
    }
}
