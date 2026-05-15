//! .mk File Parser for C Build Systems (v1.0.6)
//!
//! This module provides two modes for extracting version information from .mk build configuration files:
//!
//! ## Two-Mode Architecture
//!
//! ### Mode 1: Version Resolution (Makefile Integration)
//! - **Trigger:** Called from `parse_makefile()` when a Makefile exists
//! - **Functions:** `scan_mk_files_for_versions()` + `normalize_library_name()`
//! - **Purpose:** Resolve versions for libraries detected in Makefile `-l` flags
//! - **Ecosystem:** "system"
//! - **Use Case:** Application projects that use system libraries (e.g., xcar-qnx)
//! - **Example:** Makefile has `-lcurl`, .mk file has `CURL_VERSION = 8.15.0` → `curl@8.15.0`
//!
//! ### Mode 2: Independent Manifest Parsing
//! - **Trigger:** Called from `scan_directory()` when .mk files exist (no Makefile required)
//! - **Function:** `parse_mk_files_as_dependencies()`
//! - **Purpose:** Parse .mk files as a dependency manifest
//! - **Ecosystem:** "BUILD-CONFIG"
//! - **Use Case:** Build system repositories that define build-time dependencies (e.g., xcar-toolchains)
//! - **Example:** Any .mk file with `CURL_VERSION = 8.15.0` → `curl@8.15.0` dependency
//!
//! ## v1.0.6: Conditional Evaluation
//!
//! Supports architecture-specific conditionals:
//! ```makefile
//! ifeq ($(filter $(ARCH), qnx_8_0_0_x86_64 qnx_8_0_0_aarch64le), $(ARCH))
//! VSOMEIP_VERSION ?= 3.5.5    # For QNX 8.0
//! else
//! VSOMEIP_VERSION ?= 3.1.20.3 # For QNX 7.0
//! endif
//! ```
//!
//! When architecture is known, evaluates conditionals and extracts correct version.
//! When architecture is unknown, any arch-conditional variable is omitted entirely (with a warning
//! suggesting `--target-arch`); unconditional variables are always returned.
//!
//! ## Deduplication
//!
//! When both modes detect the same library:
//! - Mode 1 ("system") takes precedence over Mode 2 ("BUILD-CONFIG")
//! - Deduplication logic in `parsers/mod.rs::deduplicate_dependencies()` handles this
//!
//! ## Supported .mk File Formats
//!
//! Extracts variables matching `*_VERSION ?= value` or `*_VERSION := value`:
//! - `CURL_VERSION ?= 8.15.0` → library "curl", version "8.15.0"
//! - `OPENSSL_VERSION := 3.0.0` → library "openssl", version "3.0.0"
//! - `LIBXML2_VERSION = 2.9.14` → library "libxml2", version "2.9.14"
//!
//! ## Library Name Normalization
//!
//! Maps short library names to full names for version resolution:
//! - `z` → `zlib`
//! - `ssl` → `openssl`
//! - `pthread` → `pthreads`
//! - `m` → `libm`
//! - And many more (see `normalize_library_name()`)

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

lazy_static::lazy_static! {
    /// Regex for extracting version variable definitions from .mk files
    /// Pattern: VAR_NAME_(VERSION|VER|LIB_VER) ?= value or := value or = value
    /// Examples:
    /// - CURL_VERSION ?= 8.15.0
    /// - ELFUTILS_VERSION ?= 0.191
    /// - BUSYBOX_VERSION := 1.37.0
    /// - TDTS_LIB_VER := 2.0.0
    /// - NBUTIL_VER = 1.0.0
    static ref VERSION_VAR_PATTERN: Regex = Regex::new(r"(\w+(?:_VERSION|_LIB_VER|_VER))\s*(?:\?=|:=|=)\s*([^\s#]+)").unwrap();

    /// Regex for ifeq directive: ifeq ($(filter $(ARCH), qnx_8_0_0_x86_64), $(ARCH))
    static ref IFEQ_PATTERN: Regex = Regex::new(r"^\s*ifeq\s*\((.*)\)").unwrap();

    /// Regex for else directive
    static ref ELSE_PATTERN: Regex = Regex::new(r"^\s*else\s*$").unwrap();

    /// Regex for endif directive
    static ref ENDIF_PATTERN: Regex = Regex::new(r"^\s*endif\s*$").unwrap();

    /// Regex for filter function: $(filter $(ARCH), qnx_8_0_0_x86_64 qnx_8_0_0_aarch64le)
    static ref FILTER_PATTERN: Regex = Regex::new(r"\$\(filter\s+\$\((\w+)\),\s*([^)]+)\)").unwrap();
}

/// Known build tool and toolchain variables that should NOT be treated as runtime dependencies
/// These are filtered out in Mode 2 (parse_mk_files_as_dependencies) to prevent
/// false positives like "make@4.3" or "cmake@3.25.0" or "raspbian_toolchains@10.2" appearing in the SBOM
const BUILD_TOOL_VARIABLES: &[&str] = &[
    // Build tools
    "make",
    "cmake",
    "gcc",
    "clang",
    "python",
    "perl",
    "ruby",
    "autoconf",
    "automake",
    "libtool",
    "pkg_config",
    "pkgconfig",
    "ninja",
    "meson",
    "bash",
    "sh",
    "awk",
    "sed",
    // Toolchains and kernels (suffixes)
    "toolchain",
    "toolchains",
    "kl", // kernel (kl), toolchain
    // Platform identifiers (v1.0.6)
    "ndk",
    "raspbian",
    "ub_x86_64",
    "nvidia_jetson_orin",
];

/// Parse context for .mk files with architecture support (v1.0.6)
#[derive(Debug, Clone)]
pub struct MkParseContext {
    /// Target architecture (e.g., "qnx_7_0_0_x86_64")
    pub arch: Option<String>,
    /// Variable assignments collected during parsing
    pub variables: HashMap<String, String>,
}

impl MkParseContext {
    /// Create new parse context with optional architecture
    pub fn new(arch: Option<String>) -> Self {
        MkParseContext {
            arch,
            variables: HashMap::new(),
        }
    }

    /// Evaluate a conditional expression
    pub fn evaluate_condition(&self, condition: &str) -> bool {
        // Check for $(filter) function
        if let Some(cap) = FILTER_PATTERN.captures(condition) {
            let var_name = &cap[1];
            let arch_list = &cap[2];

            // Get variable value (usually ARCH)
            let var_value = if var_name == "ARCH" {
                self.arch.as_deref().unwrap_or("")
            } else {
                self.variables
                    .get(var_name)
                    .map(|s| s.as_str())
                    .unwrap_or("")
            };

            // Check if var_value is in arch_list
            let architectures: Vec<&str> = arch_list.split_whitespace().collect();
            return architectures.contains(&var_value);
        }

        // Simple equality check: $(ARCH), qnx_8_0_0_x86_64
        let parts: Vec<&str> = condition.split(',').collect();
        if parts.len() == 2 {
            let left = parts[0].trim();
            let right = parts[1].trim();

            let left_value = if left == "$(ARCH)" {
                self.arch.as_deref().unwrap_or("")
            } else {
                left
            };

            let right_value = if right.starts_with("$(") && right.ends_with(")") {
                let var_name = &right[2..right.len() - 1];
                if var_name == "ARCH" {
                    self.arch.as_deref().unwrap_or("")
                } else {
                    self.variables
                        .get(var_name)
                        .map(|s| s.as_str())
                        .unwrap_or("")
                }
            } else {
                right
            };

            return left_value == right_value;
        }

        // Unknown condition format, assume false
        false
    }
}

/// Data structure holding version information extracted from .mk files
#[derive(Debug, Clone)]
pub struct MkVersions {
    /// Map of VERSION variable names to their values
    /// Example: "CURL_VERSION" -> "8.15.0"
    pub versions: HashMap<String, String>,
}

/// Parse a .mk build configuration file to extract version information
///
/// Searches for *_VERSION variable definitions in the format:
/// - VAR_VERSION ?= value
/// - VAR_VERSION := value
/// - VAR_VERSION = value
///
/// Common in embedded Linux projects using custom build systems (xcar-qnx, etc.)
///
/// # Arguments
/// * `path` - Path to the .mk file
///
/// # Returns
/// * `Ok(MkVersions)` - Extracted version variables
/// * `Err` - If file cannot be read or parsed
#[allow(dead_code)]
pub fn parse_mk_file(path: &Path) -> Result<MkVersions, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    parse_mk_content(&content)
}

/// Parse .mk file content to extract version variables
///
/// Separated from parse_mk_file for easier testing
#[allow(dead_code)]
pub fn parse_mk_content(content: &str) -> Result<MkVersions, Box<dyn std::error::Error>> {
    parse_mk_content_with_arch(content, None)
}

/// Parse .mk file content with architecture-aware conditional evaluation (v1.0.6)
///
/// # Arguments
/// * `content` - Content of the .mk file
/// * `arch` - Optional target architecture for conditional evaluation
///
/// # Returns
/// * `Ok(MkVersions)` - Extracted version variables. When architecture is unknown,
///   any variable with arch-conditional assignments is omitted (with a warning to
///   stderr suggesting `--target-arch`); unconditional variables are always returned.
/// * `Err` - If file cannot be read or parsed (I/O or syntax error only)
pub fn parse_mk_content_with_arch(
    content: &str,
    arch: Option<&str>,
) -> Result<MkVersions, Box<dyn std::error::Error>> {
    let mut context = MkParseContext::new(arch.map(|s| s.to_string()));
    // Track unconditional assignments (last-wins) and conditional assignments separately.
    // last_was_conditional tracks whether the most recent assignment to each variable was
    // inside a conditional block. This is key: if an unconditional assignment follows all
    // conditional ones, it is the definitive "last-wins" value and should be kept.
    let mut unconditional: HashMap<String, String> = HashMap::new();
    let mut conditional: HashMap<String, Vec<String>> = HashMap::new();
    let mut last_was_conditional: HashMap<String, bool> = HashMap::new();

    // Track conditional state
    let mut condition_stack: Vec<bool> = Vec::new();
    let mut in_active_branch = true;

    for line in content.lines() {
        let line = line.trim();

        // Skip comments
        if line.starts_with('#') {
            continue;
        }

        // Check for ifeq
        if let Some(cap) = IFEQ_PATTERN.captures(line) {
            let condition = &cap[1];
            let result = if arch.is_some() {
                context.evaluate_condition(condition)
            } else {
                // No architecture: record both branches
                true
            };
            condition_stack.push(in_active_branch);
            in_active_branch = in_active_branch && result;
            continue;
        }

        // Check for else
        if ELSE_PATTERN.is_match(line) {
            if let Some(&parent_active) = condition_stack.last() {
                in_active_branch = parent_active && !in_active_branch;
            }
            continue;
        }

        // Check for endif
        if ENDIF_PATTERN.is_match(line) {
            if let Some(parent_active) = condition_stack.pop() {
                in_active_branch = parent_active;
            }
            continue;
        }

        // Extract version variables (only from active branches when arch is known)
        if arch.is_some() {
            if in_active_branch {
                if let Some(cap) = VERSION_VAR_PATTERN.captures(line) {
                    let var_name = cap[1].to_string();
                    let version = cap[2].trim().to_string();
                    if version.contains("$(") {
                        continue;
                    }
                    context.variables.insert(var_name.clone(), version);
                }
            }
        } else if condition_stack.is_empty() {
            // Unconditional assignment: last-wins
            if let Some(cap) = VERSION_VAR_PATTERN.captures(line) {
                let var_name = cap[1].to_string();
                let version = cap[2].trim().to_string();
                if version.contains("$(") {
                    // Still mark as unconditional so a prior conditional entry does not
                    // trigger a spurious "arch-ambiguous" warning for this variable.
                    last_was_conditional.insert(var_name, false);
                    continue;
                }
                unconditional.insert(var_name.clone(), version);
                last_was_conditional.insert(var_name, false);
            }
        } else {
            // Inside a conditional block: collect all branches for ambiguity detection
            if let Some(cap) = VERSION_VAR_PATTERN.captures(line) {
                let var_name = cap[1].to_string();
                let version = cap[2].trim().to_string();
                if version.contains("$(") {
                    continue;
                }
                conditional
                    .entry(var_name.clone())
                    .or_default()
                    .push(version);
                last_was_conditional.insert(var_name, true);
            }
        }
    }

    // When architecture is known, return context variables
    if arch.is_some() {
        return Ok(MkVersions {
            versions: context.variables,
        });
    }

    // When architecture is unknown, omit variables whose *last* assignment was inside a
    // conditional block — those are arch-ambiguous. Variables whose last assignment was
    // unconditional are safe to return even if they also appeared in a conditional earlier
    // (the unconditional assignment is the definitive last-wins value).
    let mut final_versions = unconditional;
    for (var_name, version_list) in conditional {
        // Only omit if the last assignment to this variable was conditional
        if *last_was_conditional.get(&var_name).unwrap_or(&true) {
            let mut unique_versions: Vec<String> = version_list.into_iter().collect();
            unique_versions.sort();
            unique_versions.dedup();

            eprintln!(
                "Warning: version for {} is arch-conditional (values {:?}) and target \
                 architecture is unknown; omitting. Use --target-arch to resolve.",
                var_name, unique_versions
            );
            final_versions.remove(&var_name);
        }
        // If last_was_conditional is false, an unconditional assignment came after all
        // conditional ones — keep it in final_versions as the definitive value.
    }

    Ok(MkVersions {
        versions: final_versions,
    })
}

/// Scan all .mk files in a repository to extract version information
///
/// Uses glob pattern **/*.mk to recursively find all .mk files.
///
/// # Arguments
/// * `repo_root` - Root directory of the repository to search
/// * `arch` - Optional target architecture for conditional evaluation (v1.0.6)
///
/// # Returns
/// * Map of library names to their versions
///   Example: "curl" -> "8.15.0", "elfutils" -> "0.191"
pub fn scan_mk_files_for_versions(
    repo_root: &Path,
    arch: Option<&str>,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut library_versions = HashMap::new();

    // Use glob pattern to find all .mk files
    // Escape repo_root to handle paths with glob metacharacters (*, ?, [, ])
    let escaped_root = glob::Pattern::escape(&repo_root.to_string_lossy());
    let pattern = format!("{}/**/*.mk", escaped_root);

    if let Ok(entries) = glob::glob(&pattern) {
        // Collect and sort paths for deterministic behavior when multiple .mk files
        // define the same VERSION variable (lexicographic ordering)
        let mut sorted_entries: Vec<_> = entries.flatten().collect();
        sorted_entries.sort();

        for entry in sorted_entries {
            // Read file content and parse with architecture
            if let Ok(content) = fs::read_to_string(&entry) {
                match parse_mk_content_with_arch(&content, arch) {
                    Ok(mk_versions) => {
                        for (var_name, version) in mk_versions.versions {
                            if let Some(lib_name) = extract_library_name_from_version_var(&var_name)
                            {
                                library_versions.insert(lib_name, version);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: skipping {} - {}", entry.display(), e);
                    }
                }
            }
        }
    }

    Ok(library_versions)
}

/// Normalize library names for matching
///
/// Maps common library name variations to their full names.
/// Example: "z" -> "zlib", "ssl" -> "openssl", "ssh2" -> "libssh2"
///
/// # Arguments
/// * `lib_name` - Library name from Makefile -l flag
///
/// # Returns
/// * Vector of candidate library names to search for
pub fn normalize_library_name(lib_name: &str) -> Vec<String> {
    let mut candidates = vec![lib_name.to_string()];

    // Common library name mappings
    match lib_name {
        "z" => candidates.push("zlib".to_string()),
        "ssl" => candidates.push("openssl".to_string()),
        "crypto" => candidates.push("openssl".to_string()),
        "ssh2" => candidates.push("libssh2".to_string()),
        "pcap" => candidates.push("libpcap".to_string()),
        "xml2" => candidates.push("libxml2".to_string()),
        "nl" => candidates.push("libnl".to_string()),
        "sqlite3" => candidates.push("sqlite".to_string()),
        // Issue #5: Add missing common library mappings
        "pthread" => candidates.push("pthreads".to_string()),
        "m" => candidates.push("libm".to_string()),
        "dl" => candidates.push("libdl".to_string()),
        "rt" => candidates.push("librt".to_string()),
        "jpeg" => candidates.push("libjpeg".to_string()),
        "png" => candidates.push("libpng".to_string()),
        _ => {
            // Try with "lib" prefix if it doesn't already have it
            if !lib_name.starts_with("lib") {
                candidates.push(format!("lib{}", lib_name));
            }
            // Try without "lib" prefix if it has it
            if lib_name.starts_with("lib") {
                candidates.push(lib_name[3..].to_string());
            }
        }
    }

    candidates
}

/// Extract library name from VERSION variable name
///
/// Converts VERSION variable names to library names.
/// Supports multiple patterns: *_VERSION, *_LIB_VER, *_VER
///
/// # Examples
/// - "CURL_VERSION" -> "curl"
/// - "OPENSSL_VERSION" -> "openssl"
/// - "TDTS_LIB_VER" -> "tdts"
/// - "NBUTIL_VER" -> "nbutil"
///
/// # Arguments
/// * `var_name` - VERSION variable name (e.g., "CURL_VERSION", "TDTS_LIB_VER")
///
/// # Returns
/// * `Some(String)` - Library name in lowercase
/// * `None` - If variable name doesn't match version patterns
pub fn extract_library_name_from_version_var(var_name: &str) -> Option<String> {
    // Skip _PATCH_VERSION variables - these are metadata, not actual dependencies (v1.0.6)
    if var_name.contains("_PATCH_") || var_name.ends_with("_PATCH_VERSION") {
        return None;
    }

    // Try _VERSION suffix first (most common)
    if var_name.ends_with("_VERSION") {
        let lib_name = &var_name[..var_name.len() - 8]; // Remove "_VERSION"
                                                        // Skip *_PATCH_VERSION variables (e.g., ZLIB_PATCH_VERSION) — these are patch-level
                                                        // metadata, not standalone version declarations
        if lib_name.ends_with("_PATCH") {
            return None;
        }
        return Some(lib_name.to_lowercase());
    }

    // Try _LIB_VER suffix (e.g., TDTS_LIB_VER)
    if var_name.ends_with("_LIB_VER") {
        let lib_name = &var_name[..var_name.len() - 8]; // Remove "_LIB_VER"
        return Some(lib_name.to_lowercase());
    }

    // Try _VER suffix (e.g., NBUTIL_VER)
    if var_name.ends_with("_VER") {
        let lib_name = &var_name[..var_name.len() - 4]; // Remove "_VER"
        return Some(lib_name.to_lowercase());
    }

    None
}

/// Parse all .mk files in a repository as dependencies
///
/// This function scans all .mk files in the repository and extracts VERSION variables,
/// creating Dependency objects for each library found. This works independently of
/// Makefile detection, making it useful for repositories that have .mk files but no
/// actual Makefile (like xcar-toolchains).
///
/// # Arguments
/// * `repo_root` - Root directory of the repository to search
/// * `arch` - Optional target architecture for conditional evaluation (v1.0.6)
///
/// # Returns
/// * Vector of Dependency objects with extracted versions
/// * Empty vector if no .mk files or VERSION variables are found
///
/// # Example
/// ```ignore
/// // Scans all .mk files in xcar-toolchains/3rd_party/
/// // Finds: CURL_VERSION ?= 8.15.0 -> dependency "curl@8.15.0"
/// let deps = parse_mk_files_as_dependencies(repo_root, Some("qnx_7_0_0_x86_64"))?;
/// ```
pub fn parse_mk_files_as_dependencies(
    repo_root: &Path,
    arch: Option<&str>,
) -> Result<Vec<crate::models::Dependency>, Box<dyn std::error::Error>> {
    use crate::models::{Dependency, DependencySource};
    use crate::parsers::format_source_info;

    // Use HashMap to track (lib_name -> (version, source_file)) for deterministic last-wins conflict resolution
    // This matches Mode 1 behavior where HashMap.insert() overwrites previous entries
    let mut lib_map = std::collections::HashMap::new();

    // Use glob pattern to find all .mk files
    // Escape repo_root to handle paths with glob metacharacters (*, ?, [, ])
    let escaped_root = glob::Pattern::escape(&repo_root.to_string_lossy());
    let pattern = format!("{}/**/*.mk", escaped_root);

    if let Ok(entries) = glob::glob(&pattern) {
        // Collect and sort paths for deterministic behavior when multiple .mk files
        // define the same VERSION variable (lexicographic ordering, last-wins after sorting)
        let mut sorted_entries: Vec<_> = entries.flatten().collect();
        sorted_entries.sort();

        for entry in sorted_entries {
            // Parse the .mk file with architecture support
            if let Ok(content) = fs::read_to_string(&entry) {
                match parse_mk_content_with_arch(&content, arch) {
                    Ok(mk_versions) => {
                        for (var_name, version) in mk_versions.versions {
                            // Extract library name from VERSION variable
                            if let Some(lib_name) = extract_library_name_from_version_var(&var_name)
                            {
                                let lib_lower = lib_name.to_lowercase();

                                // Skip known build tool variables (Issue #8: prevent false positives)
                                if BUILD_TOOL_VARIABLES.contains(&lib_lower.as_str()) {
                                    continue;
                                }

                                // Skip toolchain and kernel variables by suffix pattern (v1.0.6)
                                // e.g., raspbian_toolchains, nvidia_jetson_orin_kl, nxp_s32g2_yocto_kl
                                let mut is_toolchain = false;
                                for suffix in &["_toolchain", "_toolchains", "_kl"] {
                                    if lib_lower.ends_with(suffix) {
                                        is_toolchain = true;
                                        break;
                                    }
                                }
                                if is_toolchain {
                                    continue;
                                }

                                // Insert/overwrite to use last-wins conflict resolution (consistent with Mode 1)
                                let source_file =
                                    format_source_info("mk-file", &entry, None, false);
                                lib_map.insert(lib_name, (version, source_file));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Warning: skipping {} - {}", entry.display(), e);
                    }
                }
            }
        }
    }

    // Convert HashMap to Vec<Dependency>
    let dependencies: Vec<_> = lib_map
        .into_iter()
        .map(|(lib_name, (version, source_file))| {
            let mut dep = Dependency {
                name: lib_name,
                version,
                ecosystem: "BUILD-CONFIG".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct: true,
                source_file: Some(source_file),
                ..Default::default()
            };
            if dep.license.is_none() {
                dep.license = crate::parsers::c::known_licenses::lookup(&dep.name);
            }
            dep
        })
        .collect();

    Ok(dependencies)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mk_content_curl() {
        let content = r#"
CURL_VERSION ?= 8.15.0
CURL_NAME := curl-$(CURL_VERSION)
LIBCURL_SO := $(LIBCURL).4.8.0
        "#;

        let versions = parse_mk_content(content).unwrap();
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
    }

    #[test]
    fn test_parse_mk_content_multiple_versions() {
        let content = r#"
CURL_VERSION ?= 8.15.0
ELFUTILS_VERSION ?= 0.191
BUSYBOX_VERSION := 1.37.0
ZLIB_VERSION = 1.3.1
        "#;

        let versions = parse_mk_content(content).unwrap();
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
        assert_eq!(
            versions.versions.get("ELFUTILS_VERSION"),
            Some(&"0.191".to_string())
        );
        assert_eq!(
            versions.versions.get("BUSYBOX_VERSION"),
            Some(&"1.37.0".to_string())
        );
        assert_eq!(
            versions.versions.get("ZLIB_VERSION"),
            Some(&"1.3.1".to_string())
        );
    }

    #[test]
    fn test_parse_mk_content_with_comments() {
        let content = r#"
# This is a comment
CURL_VERSION ?= 8.15.0  # inline comment
# DISABLED_VERSION ?= 1.0.0
OPENSSL_VERSION := 3.2.5
        "#;

        let versions = parse_mk_content(content).unwrap();
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
        assert_eq!(
            versions.versions.get("OPENSSL_VERSION"),
            Some(&"3.2.5".to_string())
        );
        assert_eq!(versions.versions.get("DISABLED_VERSION"), None);
    }

    #[test]
    fn test_normalize_library_name_z() {
        let candidates = normalize_library_name("z");
        assert!(candidates.contains(&"z".to_string()));
        assert!(candidates.contains(&"zlib".to_string()));
    }

    #[test]
    fn test_normalize_library_name_ssl() {
        let candidates = normalize_library_name("ssl");
        assert!(candidates.contains(&"ssl".to_string()));
        assert!(candidates.contains(&"openssl".to_string()));
    }

    #[test]
    fn test_normalize_library_name_ssh2() {
        let candidates = normalize_library_name("ssh2");
        assert!(candidates.contains(&"ssh2".to_string()));
        assert!(candidates.contains(&"libssh2".to_string()));
    }

    #[test]
    fn test_normalize_library_name_generic() {
        let candidates = normalize_library_name("foo");
        assert!(candidates.contains(&"foo".to_string()));
        assert!(candidates.contains(&"libfoo".to_string()));
    }

    #[test]
    fn test_normalize_library_name_with_lib_prefix() {
        let candidates = normalize_library_name("libfoo");
        assert!(candidates.contains(&"libfoo".to_string()));
        assert!(candidates.contains(&"foo".to_string()));
    }

    #[test]
    fn test_extract_library_name_from_version_var() {
        assert_eq!(
            extract_library_name_from_version_var("CURL_VERSION"),
            Some("curl".to_string())
        );
        assert_eq!(
            extract_library_name_from_version_var("OPENSSL_VERSION"),
            Some("openssl".to_string())
        );
        assert_eq!(
            extract_library_name_from_version_var("ELFUTILS_VERSION"),
            Some("elfutils".to_string())
        );
        assert_eq!(
            extract_library_name_from_version_var("SOME_OTHER_VAR"),
            None
        );
    }

    #[test]
    fn test_parse_mk_content_edge_cases() {
        let content = r#"
# Empty lines and whitespace

CURL_VERSION    ?=    8.15.0
OPENSSL_VERSION:=3.2.5
ZLIB_VERSION =1.3.1
        "#;

        let versions = parse_mk_content(content).unwrap();
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
        assert_eq!(
            versions.versions.get("OPENSSL_VERSION"),
            Some(&"3.2.5".to_string())
        );
        assert_eq!(
            versions.versions.get("ZLIB_VERSION"),
            Some(&"1.3.1".to_string())
        );
    }

    #[test]
    fn test_parse_mk_with_conditionals_qnx8() {
        let content = r#"
ifeq ($(filter $(ARCH), qnx_8_0_0_x86_64 qnx_8_0_0_aarch64le), $(ARCH))
VSOMEIP_VERSION ?= 3.5.5
else
VSOMEIP_VERSION ?= 3.1.20.3
endif
        "#;

        let versions = parse_mk_content_with_arch(content, Some("qnx_8_0_0_x86_64")).unwrap();
        assert_eq!(
            versions.versions.get("VSOMEIP_VERSION"),
            Some(&"3.5.5".to_string())
        );
    }

    #[test]
    fn test_parse_mk_with_conditionals_qnx7() {
        let content = r#"
ifeq ($(filter $(ARCH), qnx_8_0_0_x86_64 qnx_8_0_0_aarch64le), $(ARCH))
VSOMEIP_VERSION ?= 3.5.5
else
VSOMEIP_VERSION ?= 3.1.20.3
endif
        "#;

        let versions = parse_mk_content_with_arch(content, Some("qnx_7_0_0_x86_64")).unwrap();
        assert_eq!(
            versions.versions.get("VSOMEIP_VERSION"),
            Some(&"3.1.20.3".to_string())
        );
    }

    #[test]
    fn test_parse_mk_without_arch_single_version() {
        let content = r#"
CURL_VERSION ?= 8.15.0
OPENSSL_VERSION := 3.2.5
        "#;

        let versions = parse_mk_content_with_arch(content, None).unwrap();
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
        assert_eq!(
            versions.versions.get("OPENSSL_VERSION"),
            Some(&"3.2.5".to_string())
        );
    }

    #[test]
    fn test_parse_mk_without_arch_omits_conditional_variables() {
        // When arch is unknown, any arch-conditional variable is omitted entirely (warning emitted)
        // to avoid silently picking the wrong version. Unconditional variables are still returned.
        let content = r#"
CURL_VERSION = 8.0.0
ifeq ($(filter $(ARCH), qnx_8_0_0_x86_64), $(ARCH))
VSOMEIP_VERSION ?= 3.5.5
else
VSOMEIP_VERSION ?= 3.1.20.3
endif
        "#;

        let result = parse_mk_content_with_arch(content, None);
        assert!(result.is_ok());
        let versions = result.unwrap();
        // Arch-conditional variable is omitted (regardless of how many unique values)
        assert!(!versions.versions.contains_key("VSOMEIP_VERSION"));
        // Unconditional variable is still returned
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.0.0".to_string())
        );
    }

    #[test]
    fn test_parse_mk_without_arch_keeps_unconditional_override_after_conditional() {
        // If an unconditional assignment follows all conditional ones, it is the last-wins
        // definitive value and must NOT be dropped even though the variable also appeared
        // inside a conditional block earlier.
        let content = r#"
ifeq ($(ARCH), qnx_8_0_0_x86_64)
CURL_VERSION = 7.88.0
endif
CURL_VERSION = 8.15.0
        "#;

        let result = parse_mk_content_with_arch(content, None);
        assert!(result.is_ok());
        let versions = result.unwrap();
        // The unconditional override after the conditional is the definitive value
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );
    }

    #[test]
    fn test_parse_mk_nested_conditionals() {
        let content = r#"
ifeq ($(ARCH), qnx_8_0_0_x86_64)
VSOMEIP_VERSION ?= 3.5.5
CURL_VERSION ?= 8.15.0
else
ifeq ($(ARCH), qnx_7_0_0_x86_64)
VSOMEIP_VERSION ?= 3.1.20.3
CURL_VERSION ?= 7.88.0
else
VSOMEIP_VERSION ?= 3.0.0
CURL_VERSION ?= 7.0.0
endif
endif
        "#;

        let versions = parse_mk_content_with_arch(content, Some("qnx_8_0_0_x86_64")).unwrap();
        assert_eq!(
            versions.versions.get("VSOMEIP_VERSION"),
            Some(&"3.5.5".to_string())
        );
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"8.15.0".to_string())
        );

        let versions = parse_mk_content_with_arch(content, Some("qnx_7_0_0_x86_64")).unwrap();
        assert_eq!(
            versions.versions.get("VSOMEIP_VERSION"),
            Some(&"3.1.20.3".to_string())
        );
        assert_eq!(
            versions.versions.get("CURL_VERSION"),
            Some(&"7.88.0".to_string())
        );
    }

    #[test]
    fn test_mk_parse_context_evaluate_filter() {
        let context = MkParseContext::new(Some("qnx_8_0_0_x86_64".to_string()));

        // Test filter function
        let condition = "$(filter $(ARCH), qnx_8_0_0_x86_64 qnx_8_0_0_aarch64le), $(ARCH)";
        assert!(context.evaluate_condition(condition));

        let context2 = MkParseContext::new(Some("qnx_7_0_0_x86_64".to_string()));
        assert!(!context2.evaluate_condition(condition));
    }

    #[test]
    fn test_mk_parse_context_evaluate_simple() {
        let context = MkParseContext::new(Some("qnx_8_0_0_x86_64".to_string()));

        // Test simple equality
        let condition = "$(ARCH), qnx_8_0_0_x86_64";
        assert!(context.evaluate_condition(condition));

        let condition2 = "$(ARCH), qnx_7_0_0_x86_64";
        assert!(!context.evaluate_condition(condition2));
    }

    #[test]
    fn test_mk_variable_ref_not_stored_as_version() {
        // DOPENSSL_VERSION := $(OPENSSL_VERSION) must not produce a version entry
        // because the RHS is an unexpandable Makefile variable reference
        let content = "DOPENSSL_VERSION := $(OPENSSL_VERSION)\n";
        let versions = parse_mk_content(content).unwrap();
        // No entry with a version containing "$(" should exist
        for (_, v) in &versions.versions {
            assert!(!v.contains("$("), "version '{}' contains unexpanded variable ref", v);
        }
        // Specifically, DOPENSSL_VERSION must not have any version stored
        assert!(
            versions.versions.get("DOPENSSL_VERSION").is_none(),
            "DOPENSSL_VERSION should not be stored when its value is a variable reference"
        );
    }

    #[test]
    fn test_mk_literal_version_still_resolved() {
        // OPENSSL_VERSION := 3.0.7 must produce a version entry for openssl
        let content = "OPENSSL_VERSION := 3.0.7\n";
        let versions = parse_mk_content(content).unwrap();
        assert_eq!(
            versions.versions.get("OPENSSL_VERSION"),
            Some(&"3.0.7".to_string()),
            "Literal version assignment must still be captured correctly"
        );
    }
}
