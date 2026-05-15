//! Conan manifest parsers (conanfile.txt, conanfile.py)
//!
//! Parses Conan manifest files to extract dependency requirements.

use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use regex::Regex;
use std::fs;
use std::path::Path;

/// Parse Conan manifest file (conanfile.txt)
///
/// Format (INI-like):
/// ```ini
/// [requires]
/// zlib/1.2.13
/// openssl/[>=3.0]
///
/// [build_requires]
/// cmake/3.27.0
/// ```
pub fn parse_conanfile_txt(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();
    let mut current_section: Option<String> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Detect section headers: [requires], [build_requires], [tool_requires], etc.
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].to_string());
            continue;
        }

        // Parse dependencies based on current section
        match current_section.as_deref() {
            Some("requires") => match parse_conan_dependency_spec(line, path, false) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan requirement '{}' in {}: {}",
                    line,
                    path.display(),
                    e
                ),
            },
            Some("build_requires") | Some("tool_requires") => {
                match parse_conan_dependency_spec(line, path, true) {
                    Ok(dep) => dependencies.push(dep),
                    Err(e) => eprintln!(
                        "Warning: Failed to parse Conan build requirement '{}' in {}: {}",
                        line,
                        path.display(),
                        e
                    ),
                }
            }
            Some("test_requires") => match parse_conan_dependency_spec(line, path, true) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan test requirement '{}' in {}: {}",
                    line,
                    path.display(),
                    e
                ),
            },
            _ => {
                // Ignore other sections like [options], [generators]
            }
        }
    }

    Ok(dependencies)
}

/// Parse Conan manifest file (conanfile.py)
///
/// Uses regex to extract dependencies from Python code.
/// Supports:
/// - requires = ["dep1", "dep2"]
/// - self.requires("dep")
pub fn parse_conanfile_py(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();

    // Pattern 1: requires = ["dep1", "dep2"]
    // Use word boundary and multiline mode to avoid matching inside build_requires
    let list_pattern = Regex::new(r#"(?ms)^\s*requires\b\s*=\s*\[(.*?)\]"#)?;
    if let Some(captures) = list_pattern.captures(&content) {
        let list_content = &captures[1];
        for dep_str in extract_quoted_strings(list_content) {
            match parse_conan_dependency_spec(&dep_str, path, false) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan requirement '{}' in {}: {}",
                    dep_str,
                    path.display(),
                    e
                ),
            }
        }
    }

    // Pattern 2: self.requires("dep") or self.requires('dep')
    // Support both single and double quotes
    let method_pattern = Regex::new(r#"self\.requires\s*\(\s*["']([^"']+)["']"#)?;
    for captures in method_pattern.captures_iter(&content) {
        let dep_str = &captures[1];
        match parse_conan_dependency_spec(dep_str, path, false) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan requirement '{}' in {}: {}",
                dep_str,
                path.display(),
                e
            ),
        }
    }

    // Pattern 3: build_requires = ["dep1"]
    let build_list_pattern = Regex::new(r#"build_requires\s*=\s*\[(.*?)\]"#)?;
    if let Some(captures) = build_list_pattern.captures(&content) {
        let list_content = &captures[1];
        for dep_str in extract_quoted_strings(list_content) {
            match parse_conan_dependency_spec(&dep_str, path, true) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan build requirement '{}' in {}: {}",
                    dep_str,
                    path.display(),
                    e
                ),
            }
        }
    }

    // Pattern 4: self.build_requires("dep") or self.build_requires('dep')
    let build_method_pattern = Regex::new(r#"self\.build_requires\s*\(\s*["']([^"']+)["']"#)?;
    for captures in build_method_pattern.captures_iter(&content) {
        let dep_str = &captures[1];
        match parse_conan_dependency_spec(dep_str, path, true) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan build requirement '{}' in {}: {}",
                dep_str,
                path.display(),
                e
            ),
        }
    }

    // Pattern 5: tool_requires = ["dep1"]
    let tool_list_pattern = Regex::new(r#"tool_requires\s*=\s*\[(.*?)\]"#)?;
    if let Some(captures) = tool_list_pattern.captures(&content) {
        let list_content = &captures[1];
        for dep_str in extract_quoted_strings(list_content) {
            match parse_conan_dependency_spec(&dep_str, path, true) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan tool requirement '{}' in {}: {}",
                    dep_str,
                    path.display(),
                    e
                ),
            }
        }
    }

    // Pattern 6: self.tool_requires("dep") or self.tool_requires('dep')
    let tool_method_pattern = Regex::new(r#"self\.tool_requires\s*\(\s*["']([^"']+)["']"#)?;
    for captures in tool_method_pattern.captures_iter(&content) {
        let dep_str = &captures[1];
        match parse_conan_dependency_spec(dep_str, path, true) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan tool requirement '{}' in {}: {}",
                dep_str,
                path.display(),
                e
            ),
        }
    }

    // Pattern 7: test_requires
    let test_list_pattern = Regex::new(r#"test_requires\s*=\s*\[(.*?)\]"#)?;
    if let Some(captures) = test_list_pattern.captures(&content) {
        let list_content = &captures[1];
        for dep_str in extract_quoted_strings(list_content) {
            match parse_conan_dependency_spec(&dep_str, path, true) {
                Ok(dep) => dependencies.push(dep),
                Err(e) => eprintln!(
                    "Warning: Failed to parse Conan test requirement '{}' in {}: {}",
                    dep_str,
                    path.display(),
                    e
                ),
            }
        }
    }

    // Pattern 8: self.test_requires("dep") or self.test_requires('dep')
    let test_method_pattern = Regex::new(r#"self\.test_requires\s*\(\s*["']([^"']+)["']"#)?;
    for captures in test_method_pattern.captures_iter(&content) {
        let dep_str = &captures[1];
        match parse_conan_dependency_spec(dep_str, path, true) {
            Ok(dep) => dependencies.push(dep),
            Err(e) => eprintln!(
                "Warning: Failed to parse Conan test requirement '{}' in {}: {}",
                dep_str,
                path.display(),
                e
            ),
        }
    }

    Ok(dependencies)
}

/// Extract quoted strings from a string
///
/// Example: '"dep1", "dep2"' -> ["dep1", "dep2"]
fn extract_quoted_strings(content: &str) -> Vec<String> {
    let pattern = Regex::new(r#"["']([^"']+)["']"#).unwrap();
    pattern
        .captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

/// Parse Conan dependency specification
///
/// Format: `name/version` or `name/[>=version]` or `name/version@user/channel`
/// Examples:
/// - `zlib/1.2.13`
/// - `openssl/[>=3.0]`
/// - `boost/1.82.0@user/channel`
fn parse_conan_dependency_spec(
    spec: &str,
    source_path: &Path,
    is_dev: bool,
) -> Result<Dependency, Box<dyn std::error::Error>> {
    let parts: Vec<&str> = spec.split('/').collect();
    if parts.is_empty() {
        return Err(format!("Invalid Conan dependency spec: {}", spec).into());
    }

    let name = parts[0].to_string();

    let version = if parts.len() >= 2 {
        let version_part = parts[1];
        // Remove @user/channel if present
        let version = version_part.split('@').next().unwrap_or(version_part);
        // Handle version ranges: [>=3.0] -> ">=3.0"
        let version = if version.starts_with('[') && version.ends_with(']') {
            version[1..version.len() - 1].to_string()
        } else {
            version.to_string()
        };
        version
    } else {
        "*".to_string()
    };

    Ok(Dependency {
        name,
        version,
        ecosystem: "conan".to_string(),
        source: DependencySource::Manifest,
        is_dev,
        is_direct: true,
        source_file: Some(format_source_info(
            "conan/manifest",
            source_path,
            None,
            false,
        )),
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        write!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn test_parse_conanfile_txt() {
        let content = r#"
[requires]
zlib/1.2.13
openssl/[>=3.0]
boost/1.82.0

[build_requires]
cmake/3.27.0

[tool_requires]
ninja/1.11.0

[test_requires]
gtest/1.14.0

[options]
openssl:shared=True
"#;

        let temp_file = create_temp_file(content);
        let deps = parse_conanfile_txt(temp_file.path()).unwrap();

        assert!(deps
            .iter()
            .any(|d| d.name == "zlib" && d.version == "1.2.13"));
        assert!(deps
            .iter()
            .any(|d| d.name == "openssl" && d.version == ">=3.0"));
        assert!(deps
            .iter()
            .any(|d| d.name == "boost" && d.version == "1.82.0"));
        assert!(deps.iter().any(|d| d.name == "cmake" && d.is_dev));
        assert!(deps.iter().any(|d| d.name == "ninja" && d.is_dev));
        assert!(deps.iter().any(|d| d.name == "gtest" && d.is_dev));
    }

    #[test]
    fn test_parse_conanfile_py_list_format() {
        let content = r#"
from conan import ConanFile

class MyProjectConan(ConanFile):
    name = "myproject"
    version = "1.0.0"
    requires = ["zlib/1.2.13", "openssl/3.1.2"]
    build_requires = ["cmake/3.27.0"]
"#;

        let temp_file = create_temp_file(content);
        let deps = parse_conanfile_py(temp_file.path()).unwrap();

        assert!(deps.iter().any(|d| d.name == "zlib" && !d.is_dev));
        assert!(deps.iter().any(|d| d.name == "openssl" && !d.is_dev));
        assert!(deps.iter().any(|d| d.name == "cmake" && d.is_dev));
    }

    #[test]
    fn test_parse_conanfile_py_method_calls() {
        let content = r#"
from conan import ConanFile

class MyProjectConan(ConanFile):
    def requirements(self):
        self.requires("zlib/1.2.13")
        self.requires("boost/1.82.0")

    def build_requirements(self):
        self.build_requires("cmake/3.27.0")

    def test_requirements(self):
        self.test_requires("gtest/1.14.0")
"#;

        let temp_file = create_temp_file(content);
        let deps = parse_conanfile_py(temp_file.path()).unwrap();

        assert!(deps.iter().any(|d| d.name == "zlib" && !d.is_dev));
        assert!(deps.iter().any(|d| d.name == "boost" && !d.is_dev));
        assert!(deps.iter().any(|d| d.name == "cmake" && d.is_dev));
        assert!(deps.iter().any(|d| d.name == "gtest" && d.is_dev));
    }

    #[test]
    fn test_parse_conan_dependency_spec_simple() {
        let dep =
            parse_conan_dependency_spec("zlib/1.2.13", Path::new("conanfile.txt"), false).unwrap();
        assert_eq!(dep.name, "zlib");
        assert_eq!(dep.version, "1.2.13");
        assert!(!dep.is_dev);
    }

    #[test]
    fn test_parse_conan_dependency_spec_version_range() {
        let dep = parse_conan_dependency_spec("openssl/[>=3.0]", Path::new("conanfile.txt"), false)
            .unwrap();
        assert_eq!(dep.name, "openssl");
        assert_eq!(dep.version, ">=3.0");
    }

    #[test]
    fn test_parse_conan_dependency_spec_with_channel() {
        let dep = parse_conan_dependency_spec(
            "boost/1.82.0@user/channel",
            Path::new("conanfile.txt"),
            false,
        )
        .unwrap();
        assert_eq!(dep.name, "boost");
        assert_eq!(dep.version, "1.82.0");
    }

    #[test]
    fn test_parse_conan_dependency_spec_complex_range() {
        let dep =
            parse_conan_dependency_spec("zlib/[>1.0 <2.0]", Path::new("conanfile.txt"), false)
                .unwrap();
        assert_eq!(dep.name, "zlib");
        assert_eq!(dep.version, ">1.0 <2.0");
    }

    #[test]
    fn test_extract_quoted_strings() {
        let content = r#""dep1", "dep2", "dep3""#;
        let strings = extract_quoted_strings(content);
        assert_eq!(strings, vec!["dep1", "dep2", "dep3"]);
    }
}
