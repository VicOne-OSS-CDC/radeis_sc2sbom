//! .gitmodules parser
//!
//! Parses Git submodule configuration from .gitmodules files.
//!
//! ## Format
//! The .gitmodules file uses an INI-like format:
//! ```ini
//! [submodule "libs/json"]
//!     path = libs/json
//!     url = https://github.com/nlohmann/json.git
//!     branch = master
//! ```
//!
//! ## Extracted data
//! - Submodule name (from section header)
//! - Path (local directory)
//! - URL (remote repository)
//! - Branch (optional, defaults to default branch)

use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// Represents a Git submodule with its configuration
#[derive(Debug, Clone)]
pub struct GitSubmodule {
    /// Submodule name (from [submodule "name"] section)
    pub name: String,
    /// Local path relative to repository root
    pub path: PathBuf,
    /// Remote repository URL
    pub url: String,
    /// Resolved commit SHA (populated by commit_resolver)
    pub commit_sha: Option<String>,
    /// Branch name (if specified in .gitmodules)
    #[allow(dead_code)]
    pub branch: Option<String>,
}

/// Builder for constructing GitSubmodule from parsed .gitmodules entries
struct GitSubmoduleBuilder {
    name: String,
    path: Option<PathBuf>,
    url: Option<String>,
    branch: Option<String>,
}

impl GitSubmoduleBuilder {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            path: None,
            url: None,
            branch: None,
        }
    }

    fn build(self) -> Result<GitSubmodule, &'static str> {
        Ok(GitSubmodule {
            name: self.name,
            path: self.path.ok_or("missing path in .gitmodules entry")?,
            url: self.url.ok_or("missing url in .gitmodules entry")?,
            commit_sha: None,
            branch: self.branch,
        })
    }
}

/// Parse a .gitmodules file and extract submodule configurations
///
/// # Arguments
/// * `path` - Path to the .gitmodules file
///
/// # Returns
/// A vector of GitSubmodule structs representing each submodule
///
/// # Example
/// ```ignore
/// let submodules = parse_gitmodules(Path::new(".gitmodules"))?;
/// for sm in submodules {
///     println!("{}: {} -> {}", sm.name, sm.path.display(), sm.url);
/// }
/// ```
pub fn parse_gitmodules(path: &Path) -> Result<Vec<GitSubmodule>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut submodules = Vec::new();
    let mut current_builder: Option<GitSubmoduleBuilder> = None;

    for line in content.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Detect submodule section: [submodule "name"]
        if line.starts_with("[submodule ") && line.ends_with(']') {
            // Save previous submodule if exists
            if let Some(builder) = current_builder.take() {
                match builder.build() {
                    Ok(submodule) => submodules.push(submodule),
                    Err(e) => eprintln!("Warning: Skipping malformed submodule entry: {}", e),
                }
            }

            // Extract name from: [submodule "libs/json"]
            let name = line
                .trim_start_matches("[submodule ")
                .trim_end_matches(']')
                .trim_matches('"')
                .trim_matches('\'');
            current_builder = Some(GitSubmoduleBuilder::new(name));
        }
        // Parse key-value pairs
        else if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();

            if let Some(ref mut builder) = current_builder {
                match key {
                    "path" => builder.path = Some(PathBuf::from(value)),
                    "url" => builder.url = Some(value.to_string()),
                    "branch" => builder.branch = Some(value.to_string()),
                    _ => {} // Ignore unknown keys (e.g., ignore, shallow, update)
                }
            }
        }
    }

    // Don't forget the last submodule
    if let Some(builder) = current_builder {
        match builder.build() {
            Ok(submodule) => submodules.push(submodule),
            Err(e) => eprintln!("Warning: Skipping malformed submodule entry: {}", e),
        }
    }

    Ok(submodules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_parse_basic_gitmodules() {
        let content = r#"
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    branch = master

[submodule "libs/fmt"]
    path = libs/fmt
    url = https://github.com/fmtlib/fmt.git
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert_eq!(submodules.len(), 2);

        assert_eq!(submodules[0].name, "libs/json");
        assert_eq!(submodules[0].path, PathBuf::from("libs/json"));
        assert_eq!(submodules[0].url, "https://github.com/nlohmann/json.git");
        assert_eq!(submodules[0].branch, Some("master".to_string()));

        assert_eq!(submodules[1].name, "libs/fmt");
        assert_eq!(submodules[1].path, PathBuf::from("libs/fmt"));
        assert_eq!(submodules[1].url, "https://github.com/fmtlib/fmt.git");
        assert_eq!(submodules[1].branch, None);
    }

    #[test]
    fn test_parse_ssh_urls() {
        let content = r#"
[submodule "libs/json"]
    path = libs/json
    url = git@github.com:nlohmann/json.git
[submodule "libs/spdlog"]
    path = libs/spdlog
    url = git@gitlab.com:gabime/spdlog.git
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert_eq!(submodules.len(), 2);
        assert!(submodules[0].url.starts_with("git@"));
        assert!(submodules[1].url.starts_with("git@"));
    }

    #[test]
    fn test_parse_malformed_entries() {
        // Missing URL in first entry, missing path in second
        let content = r#"
[submodule "libs/valid"]
    path = libs/valid
    url = https://github.com/owner/repo.git

[submodule "libs/missing-url"]
    path = libs/missing-url

[submodule "libs/missing-path"]
    url = https://github.com/owner/repo2.git
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        // Should only contain the valid entry
        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].name, "libs/valid");
    }

    #[test]
    fn test_parse_empty_file() {
        let content = "";

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert!(submodules.is_empty());
    }

    #[test]
    fn test_parse_with_comments() {
        let content = r#"
# This is a comment
; This is also a comment
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].name, "libs/json");
    }

    #[test]
    fn test_parse_single_quoted_names() {
        let content = r#"
[submodule 'libs/json']
    path = libs/json
    url = https://github.com/nlohmann/json.git
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert_eq!(submodules.len(), 1);
        assert_eq!(submodules[0].name, "libs/json");
    }

    #[test]
    fn test_parse_with_extra_keys() {
        let content = r#"
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    ignore = dirty
    shallow = true
    update = checkout
"#;

        let file = create_temp_file(content);
        let submodules = parse_gitmodules(file.path()).unwrap();

        assert_eq!(submodules.len(), 1);
        // Extra keys should be ignored, not cause errors
        assert_eq!(submodules[0].name, "libs/json");
    }
}
