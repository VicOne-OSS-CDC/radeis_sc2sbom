use crate::models::{Dependency, DependencySource};
use crate::parsers::{
    extract_js_package, extract_python_package, is_go_stdlib, is_nodejs_builtin, is_python_stdlib,
};
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::path::Path;

/// MicroPython-specific module names that do not exist on PyPI.
/// Presence of any of these in a .py file indicates it targets MicroPython, not CPython.
const MICROPYTHON_MARKER_MODULES: &[&str] = &[
    "lvgl",
    "utime",
    "ustruct",
    "usys",
    "uos",
    "uio",
    "ujson",
    "urequests",
    "uasyncio",
    "micropython",
    "lodepng",
    "SDL",
];

fn is_micropython_file(content: &str) -> bool {
    MICROPYTHON_MARKER_MODULES.iter().any(|marker| {
        content.split('\n').any(|line| {
            let trimmed = line.trim_start();
            // Skip comment lines
            if trimmed.starts_with('#') {
                return false;
            }
            // Match "import <marker>" with word boundary (end, space, comma, or 'as')
            if let Some(rest) = trimmed.strip_prefix("import ") {
                if let Some(after) = rest.strip_prefix(*marker) {
                    return after.is_empty()
                        || after.starts_with(' ')
                        || after.starts_with('\t')
                        || after.starts_with(',')
                        || after.starts_with('\r');
                }
            }
            // Match "from <marker> import ..."
            if let Some(rest) = trimmed.strip_prefix("from ") {
                if let Some(after) = rest.strip_prefix(*marker) {
                    return after.starts_with(" import");
                }
            }
            false
        })
    })
}

/// Scan a Python file for import statements and extract dependencies
pub fn scan_python_imports(path: &Path) -> Result<Vec<Dependency>> {
    // Check file size (skip files larger than 1MB to avoid minified/bundled code)
    const MAX_FILE_SIZE: u64 = 1_048_576; // 1MB
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let ecosystem = if is_micropython_file(&content) {
        "micropython"
    } else {
        "pip"
    };
    let mut dependencies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Regex patterns for Python imports
    // Matches: import foo, import foo, bar, from foo import bar, from foo.bar import baz
    let import_re = Regex::new(
        r"(?m)^\s*(?:import\s+([a-zA-Z0-9_,\s]+?)(?:\s|$)|from\s+([a-zA-Z0-9_\.]+)\s+import)",
    )
    .unwrap();

    for cap in import_re.captures_iter(&content) {
        // For bare `import` statements, we may have a comma-separated list like "os, sys, json".
        // For `from x import ...`, we only care about the module `x`.
        let modules: Vec<&str> = if let Some(m) = cap.get(1) {
            m.as_str()
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect()
        } else if let Some(m) = cap.get(2) {
            vec![m.as_str()]
        } else {
            continue;
        };

        for module in modules {
            // Handle aliases: "import os as operating_system" -> extract "os"
            let module = module.split_whitespace().next().unwrap_or(module);

            // Skip relative imports (start with '.')
            if module.starts_with('.') {
                continue;
            }

            // Extract root package name
            let package_name = extract_python_package(module);

            // Skip standard library modules
            if is_python_stdlib(&package_name) {
                continue;
            }

            // Deduplicate within this file
            if seen.contains(&package_name) {
                continue;
            }
            seen.insert(package_name.clone());

            dependencies.push(Dependency {
                name: package_name,
                version: "detected".to_string(),
                ecosystem: ecosystem.to_string(),
                source: DependencySource::ImportScan,
                is_dev: false, // Will be determined by scope classification
                is_direct: true,
                source_file: Some(path.to_string_lossy().to_string()), // v1.0.6: Pass source file for directory-based classification
                ..Default::default()
            });
        }
    }

    Ok(dependencies)
}

/// Scan a JavaScript/TypeScript file for import/require statements and extract dependencies
pub fn scan_js_ts_imports(path: &Path) -> Result<Vec<Dependency>> {
    // Check file size (skip files larger than 1MB)
    const MAX_FILE_SIZE: u64 = 1_048_576; // 1MB
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let mut dependencies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Regex for both require() and ES6 import statements
    // Matches: require('pkg'), import ... from 'pkg', import 'pkg'
    let import_re = Regex::new(
        r#"(?:require\s*\(\s*['"]([^'"]+)['"]\s*\)|import\s+.*?\s*from\s*['"]([^'"]+)['"]|import\s*['"]([^'"]+)['"])"#
    ).unwrap();

    for cap in import_re.captures_iter(&content) {
        let module = if let Some(m) = cap.get(1) {
            m.as_str()
        } else if let Some(m) = cap.get(2) {
            m.as_str()
        } else if let Some(m) = cap.get(3) {
            m.as_str()
        } else {
            continue;
        };

        // Skip relative imports (start with './' or '../')
        if module.starts_with("./") || module.starts_with("../") {
            continue;
        }

        // Handle Node.js "node:" prefix for built-ins (e.g., "node:fs")
        if module.starts_with("node:") {
            continue;
        }

        // Extract package name (handles scoped packages)
        let package_name = extract_js_package(module);

        // Skip Node.js built-in modules
        if is_nodejs_builtin(&package_name) {
            continue;
        }

        // Deduplicate within this file
        if seen.contains(&package_name) {
            continue;
        }
        seen.insert(package_name.clone());

        dependencies.push(Dependency {
            name: package_name,
            version: "detected".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::ImportScan,
            is_dev: false, // Will be determined by scope classification
            is_direct: true,
            source_file: Some(path.to_string_lossy().to_string()), // v1.0.6: Pass source file for directory-based classification
            ..Default::default()
        });
    }

    Ok(dependencies)
}

/// Scan a Go file for import statements and extract dependencies
pub fn scan_go_imports(path: &Path) -> Result<Vec<Dependency>> {
    // Check file size (skip files larger than 1MB)
    const MAX_FILE_SIZE: u64 = 1_048_576; // 1MB
    let metadata = fs::metadata(path)?;
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(path)?;
    let mut dependencies = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Regex for Go import statements
    // Single line: import "pkg" or import ( "pkg" )
    // Block imports: import ( ... "pkg" ... )
    let import_single_re = Regex::new(r#"^\s*import\s+"([^"]+)""#).unwrap();
    let import_block_start_re = Regex::new(r#"^\s*import\s*\("#).unwrap();
    let import_path_re = Regex::new(r#"^\s*"([^"]+)""#).unwrap();

    // Track if we're inside an import block
    let mut in_import_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check for single-line import
        if let Some(cap) = import_single_re.captures(trimmed) {
            if let Some(import_path) = cap.get(1) {
                let import_path = import_path.as_str();

                // Skip standard library packages
                if is_go_stdlib(import_path) {
                    continue;
                }

                // Deduplicate
                if seen.contains(import_path) {
                    continue;
                }
                seen.insert(import_path.to_string());

                dependencies.push(Dependency {
                    name: import_path.to_string(),
                    version: "detected".to_string(),
                    ecosystem: "go".to_string(),
                    source: DependencySource::ImportScan,
                    is_dev: false,
                    is_direct: true,
                    source_file: Some(path.to_string_lossy().to_string()),
                    ..Default::default()
                });
            }
            continue;
        }

        // Detect import block start (including single-line blocks)
        if import_block_start_re.is_match(trimmed) {
            in_import_block = true;

            // Handle single-line block import: import ( "pkg" )
            if trimmed.contains(')') {
                if let Some(cap) = import_path_re.captures(trimmed) {
                    if let Some(import_path) = cap.get(1) {
                        let import_path = import_path.as_str();
                        if !is_go_stdlib(import_path) && !seen.contains(import_path) {
                            seen.insert(import_path.to_string());
                            dependencies.push(Dependency {
                                name: import_path.to_string(),
                                version: "detected".to_string(),
                                ecosystem: "go".to_string(),
                                source: DependencySource::ImportScan,
                                is_dev: false,
                                is_direct: true,
                                source_file: Some(path.to_string_lossy().to_string()),
                                ..Default::default()
                            });
                        }
                    }
                }
                in_import_block = false;
            }
            continue;
        }

        // Detect import block end
        if in_import_block && trimmed.contains(')') {
            in_import_block = false;
            continue;
        }

        // Inside import block - match import paths
        if in_import_block {
            if let Some(cap) = import_path_re.captures(trimmed) {
                if let Some(import_path) = cap.get(1) {
                    let import_path = import_path.as_str();

                    // Skip standard library packages
                    if is_go_stdlib(import_path) {
                        continue;
                    }

                    // Deduplicate
                    if seen.contains(import_path) {
                        continue;
                    }
                    seen.insert(import_path.to_string());

                    dependencies.push(Dependency {
                        name: import_path.to_string(),
                        version: "detected".to_string(),
                        ecosystem: "go".to_string(),
                        source: DependencySource::ImportScan,
                        is_dev: false,
                        is_direct: true,
                        source_file: Some(path.to_string_lossy().to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(dependencies)
}

// Note: scan_source_files orchestrator function is defined outside this module
// because it depends on CLI configuration (VendorMode from cli.rs) and entry
// filtering logic (should_process_entry from scanner/mod.rs)
