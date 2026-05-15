use super::mk_file::{normalize_library_name, scan_mk_files_for_versions};
use super::pkgconfig_detector::extract_pkgconfig_from_makefile;
use super::so_scanner::scan_so_files_for_versions;
use crate::models::dependency::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

lazy_static::lazy_static! {
    /// Regex for extracting -l flags from Makefile
    /// Pattern: Match start-of-line, whitespace, or special chars (excluding '-'), then -l followed by library name
    /// Filters out:
    /// - Variable references like $(my-ldflags-y) - rejected by filtering "flags" in name
    /// - Configure-style options like --without-libfoo, since -l must not be preceded by another '-'
    /// Note: Rust regex doesn't support look-behind, so we approximate context with this prefix pattern
    /// The (?:^|...) allows matching -l at the beginning of a line or after common separators
    /// Including comma to match linker forms like -Wl,-lssl, and quotes for LDFLAGS="-lssl"
    static ref LIB_FLAG_PATTERN: Regex = Regex::new(r#"(?:^|[\s=,"])-l([a-zA-Z][a-zA-Z0-9_+.-]*)"#).unwrap();
}

/// Parse plain Makefile for dependencies using heuristics
///
/// This is a best-effort parser for handwritten Makefiles.
/// Extracts:
/// 1. -l flags (system libraries)
/// 2. pkg-config invocations
/// 3. Version resolution from .mk files (v1.0.5)
/// 4. Version resolution from .so binaries (v1.0.5)
///
/// Limitations:
/// - No variable expansion
/// - No conditional blocks
/// - No recursive make
///
/// # Arguments
/// * `path` - Path to the Makefile
/// * `scan_mk_files` - Enable .mk file version extraction (default: true)
/// * `scan_so_files` - Enable .so binary version extraction (default: false)
/// * `target_arch` - Optional target architecture for resolving arch-conditional .mk expressions
pub fn parse_makefile(
    path: &Path,
    scan_mk_files: bool,
    scan_so_files: bool,
    target_arch: Option<&str>,
    scan_root: &Path,
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

    // Pattern 1: Extract -l flags (system libraries)
    let lib_flags = extract_lib_flags(&content);
    for lib in lib_flags {
        if seen.insert(lib.clone()) {
            let mut dep = Dependency {
                name: lib,
                version: "unspecified".to_string(),
                ecosystem: "system".to_string(),
                source: DependencySource::Manifest,
                source_file: Some(format_source_info("makefile", path, None, false)),
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

    // Pattern 2: Extract pkg-config invocations
    if let Ok(pkg_deps) = extract_pkgconfig_from_makefile(path) {
        for mut dep in pkg_deps {
            if seen.insert(dep.name.clone()) {
                if dep.license.is_none() {
                    dep.license = crate::parsers::c::known_licenses::lookup(&dep.name);
                }
                dependencies.push(dep);
            }
        }
    }

    // Pattern 3: Resolve versions from .mk files and .so binaries (v1.0.5)
    if scan_mk_files || scan_so_files {
        if let Ok(repo_root) = find_repo_root(path, scan_root) {
            if let Err(e) = resolve_versions(
                &mut dependencies,
                &repo_root,
                scan_mk_files,
                scan_so_files,
                target_arch,
            ) {
                eprintln!("Warning: Version resolution failed: {}", e);
            }
        }
    }

    Ok(dependencies)
}

/// Extract library names from -l flags
/// Pattern: -l{name}
///
/// Examples:
/// - LDFLAGS = -lssl -lcrypto
/// - LIBS = -lpthread -lm
/// - $(CC) -o app app.o -lz
fn extract_lib_flags(content: &str) -> Vec<String> {
    let mut libs = Vec::new();

    for cap in LIB_FLAG_PATTERN.captures_iter(content) {
        let lib_name = &cap[1];

        // Skip common false positives
        if lib_name.starts_with('$') || lib_name.starts_with('@') {
            continue;
        }

        // Skip variable/option names that look like flags
        // e.g., "dflags", "flags", "ink-path", "ocal", "dap"
        let lowercase = lib_name.to_lowercase();
        if lowercase.contains("flags") || lowercase.contains("dflags") {
            continue;
        }

        // Note: We previously tried to filter out false positives from configure options
        // like --without-libidn2 -> "idn2", but that logic was fragile and could hide
        // real dependencies (e.g., libibverbs, libidn2, librtmp are all legitimate libraries).
        // The regex pattern already provides sufficient filtering by requiring library names
        // to start with [a-zA-Z] and filtering "flags"/"config" patterns above.
        // If false positives occur in practice, they can be addressed via allowlist/blocklist.

        libs.push(lib_name.to_string());
    }

    libs
}

/// Find the repository root directory by walking up from the Makefile
///
/// Searches for common repository markers:
/// - .git directory
/// - Cargo.toml (for Rust projects)
/// - package.json (for Node projects)
///
/// # Arguments
/// * `path` - Path to the Makefile
///
/// # Returns
/// * `Ok(PathBuf)` - Repository root directory
/// * `Err` - If repository root cannot be found
fn find_repo_root(path: &Path, scan_root: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Canonicalize both paths so that byte-equality comparison works correctly
    // regardless of whether the caller passed a relative or absolute scan_root.
    // Fall back to the original path if canonicalization fails (e.g. path doesn't exist yet).
    let canonical_scan_root = scan_root
        .canonicalize()
        .unwrap_or_else(|_| scan_root.to_path_buf());
    let canonical_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());

    let parent = canonical_path.parent().ok_or("No parent directory")?;
    let mut current_buf = parent.to_path_buf();
    let mut fallback: Option<PathBuf> = None;

    // Walk up the directory tree looking for repository markers.
    // Never walk above scan_root — this prevents escaping the scanned project
    // into the tool's own repository when the project has no .git of its own.
    loop {
        let current: &Path = &current_buf;
        // Check for .git directory (highest priority)
        // Note: Must be a directory, not a file (git submodules have .git as a file)
        let git_path = current.join(".git");
        if git_path.is_dir() {
            return Ok(current.to_path_buf());
        } else if git_path.is_file() {
            // Git submodule - keep searching for the actual repo root
        }

        // Check for other project markers (save as fallback but keep searching for .git)
        if fallback.is_none()
            && (current.join("Cargo.toml").exists()
                || current.join("package.json").exists()
                || current.join("CMakeLists.txt").exists())
        {
            fallback = Some(current.to_path_buf());
        }

        // Stop at scan_root — do not escape the scanned project directory
        if current == canonical_scan_root {
            return Ok(fallback.unwrap_or_else(|| canonical_scan_root.clone()));
        }

        // Move to parent directory
        current_buf = match current.parent() {
            Some(p) => p.to_path_buf(),
            None => {
                // Reached filesystem root, use fallback or original parent
                return Ok(fallback.unwrap_or_else(|| {
                    canonical_path
                        .parent()
                        .unwrap_or(&canonical_path)
                        .to_path_buf()
                }));
            }
        };
    }
}

/// Resolve versions for system libraries detected from Makefile
///
/// Uses multiple strategies to extract precise versions:
/// 1. Parse .mk build configuration files (Priority 1)
/// 2. Scan compiled .so binaries (Priority 2)
///
/// # Arguments
/// * `dependencies` - Mutable reference to dependencies to update
/// * `repo_root` - Repository root directory
/// * `scan_mk_files` - Enable .mk file scanning
/// * `scan_so_files` - Enable .so binary scanning
///
/// # Returns
/// * `Ok(())` - Version resolution completed
/// * `Err` - If version resolution fails
fn resolve_versions(
    dependencies: &mut [Dependency],
    repo_root: &Path,
    scan_mk_files: bool,
    scan_so_files: bool,
    target_arch: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Strategy 1: Parse .mk files for version information
    let mk_versions = if scan_mk_files {
        scan_mk_files_for_versions(repo_root, target_arch)?
    } else {
        HashMap::new()
    };

    // Strategy 2: Scan .so files for version information
    let so_versions = if scan_so_files {
        scan_so_files_for_versions(repo_root)?
    } else {
        HashMap::new()
    };

    // Update dependencies with resolved versions
    for dep in dependencies.iter_mut() {
        // Only update if version is currently "unspecified"
        if dep.version == "unspecified" {
            // Try all normalized library names
            let candidates = normalize_library_name(&dep.name);

            for candidate in &candidates {
                // Priority 1: .mk file versions
                if let Some(version) = mk_versions.get(candidate) {
                    dep.version = version.clone();
                    if let Some(ref mut src) = dep.source_file {
                        src.push_str(&format!(" [version from .mk file: {}]", version));
                    }
                    break;
                }

                // Priority 2: .so file versions
                if let Some(version) = so_versions.get(candidate) {
                    dep.version = version.clone();
                    if let Some(ref mut src) = dep.source_file {
                        src.push_str(&format!(" [version from .so file: {}]", version));
                    }
                    break;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_lib_flags_simple() {
        let content = "LDFLAGS = -lssl -lcrypto -lpthread";
        let libs = extract_lib_flags(content);
        assert_eq!(libs.len(), 3);
        assert!(libs.contains(&"ssl".to_string()));
        assert!(libs.contains(&"crypto".to_string()));
        assert!(libs.contains(&"pthread".to_string()));
    }

    #[test]
    fn test_extract_lib_flags_in_command() {
        let content = "$(CC) -o app app.o -lz -lm";
        let libs = extract_lib_flags(content);
        assert!(libs.contains(&"z".to_string()));
        assert!(libs.contains(&"m".to_string()));
    }

    #[test]
    fn test_parse_makefile() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "LDFLAGS = -lssl -lcrypto").unwrap();
        writeln!(file, "LIBS = -lpthread").unwrap();

        let scan_root = file.path().parent().unwrap();
        let deps = parse_makefile(file.path(), false, false, None, scan_root).unwrap();
        assert_eq!(deps.len(), 3);
        assert!(deps
            .iter()
            .any(|d| d.name == "ssl" && d.ecosystem == "system"));
        assert!(deps.iter().any(|d| d.name == "pthread"));
    }

    #[test]
    fn test_parse_makefile_with_pkgconfig() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "CFLAGS = $(shell pkg-config --cflags openssl)").unwrap();
        writeln!(file, "LDFLAGS = -lz").unwrap();

        let scan_root = file.path().parent().unwrap();
        let deps = parse_makefile(file.path(), false, false, None, scan_root).unwrap();
        assert!(deps
            .iter()
            .any(|d| d.name == "openssl" && d.ecosystem == "pkg-config"));
        assert!(deps
            .iter()
            .any(|d| d.name == "z" && d.ecosystem == "system"));
    }

    #[test]
    fn test_parse_makefile_deduplication() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "LDFLAGS = -lssl -lssl").unwrap();

        let scan_root = file.path().parent().unwrap();
        let deps = parse_makefile(file.path(), false, false, None, scan_root).unwrap();
        assert_eq!(deps.len(), 1);
    }

    #[test]
    fn test_extract_lib_flags_special_names() {
        let content = "LIBS = -lc++ -lstdc++ -lglib-2.0";
        let libs = extract_lib_flags(content);
        assert!(libs.contains(&"c++".to_string()));
        assert!(libs.contains(&"stdc++".to_string()));
        assert!(libs.contains(&"glib-2.0".to_string()));
    }

    #[test]
    fn test_extract_lib_flags_wl_linker_syntax() {
        // Test -Wl, linker forms (Issue #11: comma separator in linker arguments)
        let content = "LDFLAGS = -Wl,-lssl,-lcrypto -Wl,--start-group,-lpthread,-lm,--end-group";
        let libs = extract_lib_flags(content);
        assert!(libs.contains(&"ssl".to_string()), "Should match -Wl,-lssl");
        assert!(
            libs.contains(&"crypto".to_string()),
            "Should match -Wl,-lcrypto"
        );
        assert!(
            libs.contains(&"pthread".to_string()),
            "Should match -Wl,...-lpthread"
        );
        assert!(libs.contains(&"m".to_string()), "Should match -Wl,...-lm");
    }

    #[test]
    fn test_parse_makefile_ssl_license_fallback() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "LDFLAGS = -lssl").unwrap();

        let scan_root = file.path().parent().unwrap();
        let deps = parse_makefile(file.path(), false, false, None, scan_root).unwrap();
        let ssl_dep = deps.iter().find(|d| d.name == "ssl").expect("ssl dep not found");
        assert_eq!(ssl_dep.license, Some("Apache-2.0".to_string()));
    }

    #[test]
    fn test_extract_lib_flags_configure_options() {
        // Test that configure-style options like --without-libfoo don't create spurious dependencies
        let content = r#"
./configure --prefix=/usr \
    --without-libidn2 \
    --without-librtmp \
    --enable-ipv6 \
    --with-ssl=/usr/local \
    LDFLAGS="-lssl -lcrypto"
        "#;
        let libs = extract_lib_flags(content);

        // Should match real -l flags
        assert!(libs.contains(&"ssl".to_string()), "Should match -lssl");
        assert!(
            libs.contains(&"crypto".to_string()),
            "Should match -lcrypto"
        );

        // Should NOT match --without-libfoo patterns (would capture "ibfoo", "ibidn2", "ibrtmp")
        assert!(!libs.iter().any(|l| l.starts_with("ib") && l.len() > 2),
            "Should not capture 'ibfoo', 'ibidn2', 'ibrtmp' from --without-lib* patterns. Found: {:?}", libs);
    }
}
