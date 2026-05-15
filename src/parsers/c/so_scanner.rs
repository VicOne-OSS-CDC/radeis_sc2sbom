use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

use crate::util::warn_on_walkdir_err;

lazy_static::lazy_static! {
    /// Regex for parsing .so filename versions
    /// Pattern: lib{name}.so.{version}
    /// Examples:
    /// - libcurl.so.4.8.0 -> captures "4.8.0"
    /// - libssl.so.3 -> captures "3"
    /// - libz.so.1.3.1 -> captures "1.3.1"
    static ref SO_VERSION_PATTERN: Regex = Regex::new(r"\.so\.([0-9.]+)$").unwrap();

    /// Regex for extracting library name from .so filename
    /// Pattern: lib{name}.so[.version]
    /// Examples:
    /// - libcurl.so.4.8.0 -> captures "curl"
    /// - libssl.so.3 -> captures "ssl"
    /// - libglib-2.0.so.0 -> captures "glib-2.0"
    static ref SO_NAME_PATTERN: Regex = Regex::new(r"^lib([a-zA-Z0-9_+.-]+?)\.so").unwrap();

    /// Regex for parsing SONAME from readelf output
    /// Pattern: Library soname: [libname.so.version]
    static ref SONAME_READELF_PATTERN: Regex = Regex::new(r"Library soname:\s*\[([^\]]+)\]").unwrap();

    // Regex patterns for extracting versions from binary strings
    // Used by extract_version_from_strings() to find version numbers in .so file strings
    static ref VERSION_PATTERN_1: Regex = Regex::new(r"(\d+\.\d+\.\d+)").unwrap();
    static ref VERSION_PATTERN_2: Regex = Regex::new(r"version[:\s]+(\d+\.\d+\.\d+)").unwrap();
    static ref VERSION_PATTERN_3: Regex = Regex::new(r"/(\d+\.\d+\.\d+)").unwrap();
}

/// Data structure holding version information from a .so file
#[derive(Debug, Clone)]
pub struct SoVersion {
    /// Library name (e.g., "curl", "ssl", "z")
    pub library_name: String,
    /// Version string (e.g., "8.15.0", "3.2.5", "1.3.1")
    pub version: String,
}

/// Scan a compiled .so file to extract version information
///
/// Uses multiple strategies in priority order:
/// 1. Parse filename (libcurl.so.4.8.0 -> "4.8.0")
/// 2. Read ELF soname using readelf (if available)
/// 3. Search for version strings in binary content
///
/// # Arguments
/// * `path` - Path to the .so file
///
/// # Returns
/// * `Ok(SoVersion)` - Extracted library name and version
/// * `Err` - If file cannot be read or parsed
pub fn scan_so_file(path: &Path) -> Result<SoVersion, Box<dyn std::error::Error>> {
    let library_name = extract_library_name(path).ok_or("Failed to extract library name")?;
    let mut version = None;

    // Method 1: Parse filename (primary, most reliable)
    if let Some(ver) = extract_version_from_filename(path) {
        version = Some(ver);
    }

    // Method 2: Read ELF soname (fallback, requires readelf)
    if version.is_none() {
        if let Some(ver) = extract_version_from_soname(path) {
            version = Some(ver);
        }
    }

    // Method 3: Search for version strings in binary (last resort)
    if version.is_none() {
        if let Some(ver) = extract_version_from_strings(path) {
            version = Some(ver);
        }
    }

    Ok(SoVersion {
        library_name,
        version: version.unwrap_or_else(|| "unspecified".to_string()),
    })
}

/// Extract version from .so filename
///
/// Parses version numbers from .so filenames following the pattern:
/// lib{name}.so.{version}
///
/// # Examples
/// - libcurl.so.4.8.0 -> Some("4.8.0")
/// - libssl.so.3 -> Some("3")
/// - libz.so.1.3.1 -> Some("1.3.1")
/// - libcurl.so -> None
pub fn extract_version_from_filename(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;

    if let Some(cap) = SO_VERSION_PATTERN.captures(filename) {
        return Some(cap[1].to_string());
    }

    None
}

/// Extract library name from .so filename
///
/// Parses library names from .so filenames following the pattern:
/// lib{name}.so[.version]
///
/// # Examples
/// - libcurl.so.4.8.0 -> Some("curl")
/// - libssl.so.3 -> Some("ssl")
/// - libz.so.1.3.1 -> Some("z")
pub fn extract_library_name(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_str()?;

    if let Some(cap) = SO_NAME_PATTERN.captures(filename) {
        return Some(cap[1].to_string());
    }

    None
}

/// Extract soname from ELF binary using readelf command
///
/// Uses readelf to extract the SONAME field from the ELF dynamic section.
/// This method requires readelf to be installed on the system.
///
/// # Arguments
/// * `path` - Path to the .so file
///
/// # Returns
/// * `Some(String)` - Version extracted from soname
/// * `None` - If readelf is not available or soname doesn't contain version
fn extract_version_from_soname(path: &Path) -> Option<String> {
    // Try to use readelf command (if available)
    let output = Command::new("readelf")
        .args(&["-d", path.to_str()?])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if let Some(cap) = SONAME_READELF_PATTERN.captures(line) {
            let soname = &cap[1];
            // Extract version from soname (e.g., "libcurl.so.4" -> "4")
            if let Some(ver_cap) = SO_VERSION_PATTERN.captures(soname) {
                return Some(ver_cap[1].to_string());
            }
        }
    }

    None
}

/// Search binary for version strings
///
/// Searches the binary content for printable strings that match common
/// version string patterns used by libraries.
///
/// # Arguments
/// * `path` - Path to the .so file
///
/// # Returns
/// * `Some(String)` - Version string found in binary
/// * `None` - If no version string pattern matches
fn extract_version_from_strings(path: &Path) -> Option<String> {
    // Read binary content
    let content = fs::read(path).ok()?;
    let strings = extract_printable_strings(&content, 4);

    // Common version string patterns:
    // - "curl/8.15.0"
    // - "OpenSSL 3.2.5"
    // - "zlib 1.3.1"
    // - "version: 1.0.0"
    // - "libfoo-1.2.3"

    // Use lazy_static patterns to avoid recompiling regexes on every call
    let version_patterns = [
        &*VERSION_PATTERN_1,
        &*VERSION_PATTERN_2,
        &*VERSION_PATTERN_3,
    ];

    for string in strings {
        for pattern in &version_patterns {
            if let Some(cap) = pattern.captures(&string) {
                return Some(cap[1].to_string());
            }
        }
    }

    None
}

/// Extract printable ASCII strings from binary data
///
/// Searches for sequences of printable ASCII characters in binary content.
///
/// # Arguments
/// * `data` - Binary data to search
/// * `min_length` - Minimum length of strings to extract (default: 4)
///
/// # Returns
/// * Vector of printable strings found in the binary
fn extract_printable_strings(data: &[u8], min_length: usize) -> Vec<String> {
    let mut strings = Vec::new();
    let mut current = Vec::new();

    for &byte in data {
        if byte.is_ascii_graphic() || byte == b' ' {
            current.push(byte);
        } else {
            if current.len() >= min_length {
                if let Ok(s) = String::from_utf8(current.clone()) {
                    strings.push(s);
                }
            }
            current.clear();
        }
    }

    // Handle last string if any
    if current.len() >= min_length {
        if let Ok(s) = String::from_utf8(current) {
            strings.push(s);
        }
    }

    strings
}

/// Search for .so files in common library directories
///
/// Searches standard library paths where compiled .so files are typically located.
/// Directories searched:
/// - lib/
/// - build/
/// - toolchains/install/lib/
/// - usr/lib/
/// - .libs/ (autotools)
///
/// # Arguments
/// * `repo_root` - Root directory of the repository
///
/// # Returns
/// * Vector of paths to .so files found
pub fn find_so_files(repo_root: &Path) -> Vec<PathBuf> {
    let search_dirs = vec![
        repo_root.join("lib"),
        repo_root.join("lib64"),
        repo_root.join("build"),
        repo_root.join("build/lib"),
        repo_root.join("toolchains/install/lib"),
        repo_root.join("usr/lib"),
        repo_root.join("usr/lib64"),
        repo_root.join("usr/local/lib"),
        repo_root.join(".libs"),
    ];

    let mut so_files = Vec::new();
    let mut seen_canonicals = HashSet::new();

    // Canonicalize repo_root once before iterating (performance optimization)
    let (canonical_repo_root, follow_symlinks) = match repo_root.canonicalize() {
        Ok(path) => (Some(path), true),
        // Security: If repo_root can't be canonicalized, disable symlink following
        // to prevent traversal outside the repository via malicious symlinks
        Err(_) => (None, false),
    };

    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }

        // Use WalkDir to recursively search for .so files
        // Note: follow_links disabled when repo_root canonicalization fails (security)
        for entry in WalkDir::new(&dir)
            .follow_links(follow_symlinks)
            .max_depth(5)
            .into_iter()
            .filter_map(warn_on_walkdir_err)
        {
            let path = entry.path();
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Match .so files:
            // - *.so
            // - *.so.*
            if path.is_file() && (filename.ends_with(".so") || filename.contains(".so.")) {
                // Canonicalize to avoid symlink duplicates and verify path is within repo_root
                if let Ok(canonical) = path.canonicalize() {
                    // Security check: Ensure canonical path is still within repo_root
                    // This prevents following malicious symlinks to system directories
                    if let Some(ref canonical_root) = canonical_repo_root {
                        if canonical.starts_with(canonical_root) {
                            if seen_canonicals.insert(canonical.clone()) {
                                so_files.push(canonical);
                            }
                        }
                        // else: Skip paths outside repo_root (symlink traversal attack)
                    } else {
                        // Fallback: If repo_root canonicalization failed, enforce non-canonical boundary check
                        // to prevent symlink traversal outside the repository
                        if canonical.strip_prefix(repo_root).is_ok() {
                            if seen_canonicals.insert(canonical.clone()) {
                                so_files.push(canonical);
                            }
                        }
                        // else: Skip paths that resolve outside repo_root (symlink attack)
                    }
                } else {
                    // If canonicalization fails, add the path anyway (non-symlink file)
                    so_files.push(path.to_path_buf());
                }
            }
        }
    }

    so_files
}

/// Scan all .so files in a repository to extract version information
///
/// Searches common library directories for .so files and extracts version
/// information from each one.
///
/// # Arguments
/// * `repo_root` - Root directory of the repository
///
/// # Returns
/// * Map of library names to their versions
///   Example: "curl" -> "4.8.0", "ssl" -> "3"
pub fn scan_so_files_for_versions(
    repo_root: &Path,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut library_versions = HashMap::new();
    let so_files = find_so_files(repo_root);

    for so_file in so_files {
        if let Ok(so_version) = scan_so_file(&so_file) {
            // Only store if we found a valid version (not "unspecified")
            if so_version.version != "unspecified" {
                library_versions.insert(so_version.library_name, so_version.version);
            }
        }
    }

    Ok(library_versions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version_from_filename() {
        assert_eq!(
            extract_version_from_filename(Path::new("libcurl.so.4.8.0")),
            Some("4.8.0".to_string())
        );
        assert_eq!(
            extract_version_from_filename(Path::new("libssl.so.3")),
            Some("3".to_string())
        );
        assert_eq!(
            extract_version_from_filename(Path::new("libz.so.1.3.1")),
            Some("1.3.1".to_string())
        );
        assert_eq!(extract_version_from_filename(Path::new("libcurl.so")), None);
    }

    #[test]
    fn test_extract_library_name() {
        assert_eq!(
            extract_library_name(Path::new("libcurl.so.4.8.0")),
            Some("curl".to_string())
        );
        assert_eq!(
            extract_library_name(Path::new("libssl.so.3")),
            Some("ssl".to_string())
        );
        assert_eq!(
            extract_library_name(Path::new("libz.so.1.3.1")),
            Some("z".to_string())
        );
        assert_eq!(
            extract_library_name(Path::new("libpcap.so.1.10.4")),
            Some("pcap".to_string())
        );
    }

    #[test]
    fn test_extract_printable_strings() {
        let data = b"Hello\x00World\x01Test\x00Version 1.2.3\x00";
        let strings = extract_printable_strings(data, 4);

        assert!(strings.iter().any(|s| s.contains("Hello")));
        assert!(strings.iter().any(|s| s.contains("World")));
        assert!(strings.iter().any(|s| s.contains("Test")));
        assert!(strings.iter().any(|s| s.contains("Version 1.2.3")));
    }

    #[test]
    fn test_extract_library_name_with_hyphens() {
        // Library names can include hyphens and dots (e.g., glib-2.0)
        assert_eq!(
            extract_library_name(Path::new("libglib-2.0.so.0")),
            Some("glib-2.0".to_string())
        );
        // Simple case with hyphens
        assert_eq!(
            extract_library_name(Path::new("libfoo-bar.so.1")),
            Some("foo-bar".to_string())
        );
    }

    #[test]
    fn test_extract_version_from_filename_complex() {
        assert_eq!(
            extract_version_from_filename(Path::new("/usr/lib/libcurl.so.4.8.0")),
            Some("4.8.0".to_string())
        );
        assert_eq!(
            extract_version_from_filename(Path::new("./build/lib/libssl.so.3.2.5")),
            Some("3.2.5".to_string())
        );
    }
}
