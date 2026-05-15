//! Heuristic Classification Rules (v1.0.6)
//!
//! Multi-strategy dependency classification based on:
//! - Ecosystem type
//! - Package name patterns
//! - Source directory location
//! - Enhanced library name normalization

use crate::models::{Dependency, DependencyScope};

/// Classify dependency by ecosystem
pub fn classify_by_ecosystem(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    let ecosystem = dep.ecosystem.to_lowercase();

    // Check if in test directory for context-aware classification
    // Normalize to lowercase and replace backslashes to handle Windows paths and mixed casing
    let in_test_dir = dep
        .source_file
        .as_ref()
        .map(|s| {
            let normalized = s.to_lowercase().replace('\\', "/");
            normalized.contains("/test/") || normalized.contains("/tests/")
        })
        .unwrap_or(false);

    match ecosystem.as_str() {
        // Python packages - let name-based classification handle known test frameworks
        "pip" if in_test_dir => Some((
            DependencyScope::Test,
            0.9,
            "PIP package from test directory".to_string(),
        )),
        "pip" => {
            // Let name-based classification handle known test frameworks/tools
            None
        }

        // Meson build system dependencies
        "meson-wrap" | "meson-subproject" => Some((
            DependencyScope::Build,
            0.8,
            "Meson build dependency".to_string(),
        )),

        // Git submodules - provided source code
        "git-submodule" => Some((
            DependencyScope::Provided,
            0.7,
            "Source code submodule".to_string(),
        )),

        // Vendored libraries - runtime dependencies (v1.0.6)
        "vendored" => Some((
            DependencyScope::Runtime,
            0.9,
            "Vendored 3rd-party library".to_string(),
        )),

        // BUILD-CONFIG - classify as Build by default
        // Known runtime libraries linked by SYSTEM are refined later by refine_build_config_classification
        "build-config" => Some((
            DependencyScope::Build,
            0.6,
            "BUILD-CONFIG package - Build dependency pending link analysis".to_string(),
        )),

        // System libraries - let name-based classification determine scope
        // (cmake → Build at 1.0, zlib/curl → Runtime at 0.9, etc.)
        "system" => None,

        _ => None,
    }
}

/// Classify dependency by name patterns
pub fn classify_by_name(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    let name_lower = dep.name.to_lowercase();

    // Test frameworks (highest confidence)
    const TEST_FRAMEWORKS: &[&str] = &[
        "unity",
        "gtest",
        "googletest",
        "catch2",
        "doctest", // C/C++
        "pytest",
        "unittest",
        "nose",
        "tox", // Python
        "junit",
        "testng",
        "mockito", // Java
        "mocha",
        "jest",
        "vitest",
        "jasmine",
        "karma",
        "chai", // JavaScript
        "rspec",
        "minitest", // Ruby
        "xunit",
        "nunit", // .NET
    ];

    for framework in TEST_FRAMEWORKS {
        if name_lower == *framework
            || name_lower.starts_with(&format!("{}-", framework))
            || name_lower.starts_with(&format!("{}_", framework))
        // v1.0.6: Support underscore separators
        {
            return Some((
                DependencyScope::Test,
                1.0,
                format!("Test framework: {}", framework),
            ));
        }
    }

    // Build tools (highest confidence)
    const BUILD_TOOLS: &[&str] = &[
        "cmake",
        "meson",
        "ninja",
        "make",
        "automake",
        "autoconf",
        "libtool",
        "gcc",
        "clang",
        "g++",
        "llvm",
        "rustc",
        "pkg-config",
        "pkgconfig",
    ];

    for tool in BUILD_TOOLS {
        if name_lower == *tool
            || name_lower.starts_with(&format!("{}-", tool))
            || name_lower.starts_with(&format!("{}_", tool))
        // v1.0.6: Support underscore separators
        {
            return Some((DependencyScope::Build, 1.0, format!("Build tool: {}", tool)));
        }
    }

    // Development tools (high confidence)
    const DEV_TOOLS: &[&str] = &[
        "eslint",
        "prettier",
        "tslint",
        "stylelint", // JS/TS linters
        "pylint",
        "flake8",
        "black",
        "isort",
        "mypy",
        "ruff", // Python tools
        "clippy",
        "rustfmt", // Rust tools
        "rubocop", // Ruby tools
        "phpcs",
        "php-cs-fixer", // PHP tools
    ];

    for tool in DEV_TOOLS {
        if name_lower == *tool
            || name_lower.starts_with(&format!("{}-", tool))
            || name_lower.starts_with(&format!("{}_", tool))
        // v1.0.6: Support underscore separators
        {
            return Some((
                DependencyScope::Development,
                1.0,
                format!("Development tool: {}", tool),
            ));
        }
    }

    // Sanitizers (high confidence - Development scope)
    const SANITIZERS: &[&str] = &[
        "asan", "ubsan", "msan", "tsan",
        "lsan", // Address, Undefined Behavior, Memory, Thread, Leak sanitizers
    ];

    for sanitizer in SANITIZERS {
        if name_lower == *sanitizer
            || name_lower.starts_with(&format!("{}-", sanitizer))
            || name_lower.starts_with(&format!("{}_", sanitizer))
        {
            return Some((
                DependencyScope::Development,
                1.0,
                format!("Sanitizer tool: {}", sanitizer),
            ));
        }
    }

    // Standard C/C++ libraries (high confidence - Provided scope)
    const STANDARD_C_LIBS: &[&str] = &[
        "c", "m", "dl", "pthread", "rt", "socket", "curses", "ncurses",
    ];

    for lib in STANDARD_C_LIBS {
        if name_lower == *lib || name_lower == format!("lib{}", lib) {
            return Some((
                DependencyScope::Provided,
                1.0,
                format!("Standard C library: {}", lib),
            ));
        }
    }

    // Runtime libraries (medium-high confidence)
    const RUNTIME_LIBS: &[&str] = &[
        // Compression & Crypto
        "zlib",
        "openssl",
        "mbedtls",
        // Network & Communication
        "curl",
        "vsomeip",
        "nanopb",
        "protobuf",
        // System & Utilities
        "elfutils",
        // Data & XML
        "libxml2",
        "libxslt",
        "sqlite",
        "postgresql",
        "mysql",
        // Frameworks
        "boost",
        "qt",
        "gtk",
        "glib",
    ];

    for lib in RUNTIME_LIBS {
        if name_lower == *lib
            || name_lower.starts_with(&format!("lib{}", lib))
            || name_lower.starts_with(&format!("{}-", lib))
        {
            return Some((
                DependencyScope::Runtime,
                0.9,
                format!("Known runtime library: {}", lib),
            ));
        }
    }

    // Name patterns
    if name_lower.ends_with("-dev") || name_lower.ends_with("-devel") {
        return Some((
            DependencyScope::Development,
            0.7,
            "Package name ends with -dev/-devel".to_string(),
        ));
    }

    if name_lower.starts_with("test-") || name_lower.ends_with("-test") {
        return Some((
            DependencyScope::Test,
            0.7,
            "Package name contains 'test'".to_string(),
        ));
    }

    None
}

/// Classify dependency by source directory
pub fn classify_by_directory(dep: &Dependency) -> Option<(DependencyScope, f32, String)> {
    if let Some(ref source_file) = dep.source_file {
        let source_lower = source_file.to_lowercase();

        // Test directories
        if source_lower.contains("/test/")
            || source_lower.contains("/tests/")
            || source_lower.contains("/unit_test/")  // v1.0.6: Add unit_test pattern
            || source_lower.contains("\\test\\")
            || source_lower.contains("\\tests\\")
            || source_lower.contains("\\unit_test\\")
        {
            return Some((
                DependencyScope::Test,
                0.7,
                "Found in test directory".to_string(),
            ));
        }

        // Build directories
        // BUT: Skip BUILD-CONFIG ecosystem - those are runtime libraries from .mk files
        let is_build_config = dep.ecosystem == "BUILD-CONFIG";

        if !is_build_config
            && (source_lower.contains("/3rd_party/")
            || source_lower.contains("/toolchains/")
            || source_lower.contains("/build/")
            || source_lower.contains("/pb/")       // v1.0.6: Protocol buffer generated code
            || source_lower.contains("/lib/atom_utils/")  // v1.0.6: Git submodule (provides build utilities)
            || source_lower.contains("/lib/quark/")       // v1.0.6: Git submodule
            || source_lower.contains("\\3rd_party\\")
            || source_lower.contains("\\toolchains\\")
            || source_lower.contains("\\pb\\")
            || source_lower.contains("\\lib\\atom_utils\\")
            || source_lower.contains("\\lib\\quark\\"))
        {
            return Some((
                DependencyScope::Build,
                0.7,
                "Found in build directory".to_string(),
            ));
        }

        // Development directories
        if source_lower.contains("/scripts/")
            || source_lower.contains("/tools/")
            || source_lower.contains("/util/")    // v1.0.6: Add util pattern
            || source_lower.contains("/utils/")   // v1.0.6: Also match utils (plural)
            || source_lower.contains("_utils/")   // v1.0.6: Match underscore variant (atom_utils)
            || source_lower.contains("/dev/")
            || source_lower.contains("\\scripts\\")
            || source_lower.contains("\\tools\\")
            || source_lower.contains("\\util\\")
            || source_lower.contains("\\utils\\")
            || source_lower.contains("_utils\\")
        {
            return Some((
                DependencyScope::Development,
                0.7,
                "Found in development directory".to_string(),
            ));
        }
    }

    None
}

/// Enhanced library name normalization
///
/// Handles:
/// - Version suffixes (.so.3, .dll, .dylib)
/// - Library prefixes (lib)
/// - Common library name variations
/// - Qt libraries (Qt5Core, Qt6Core)
pub fn normalize_lib_name_enhanced(name: &str) -> Vec<String> {
    let mut result = Vec::new();

    // Remove version suffixes: libssl.so.3 → libssl.so → libssl → ssl
    let without_so = name.split(".so").next().unwrap_or(name);
    let without_dll = without_so.split(".dll").next().unwrap_or(without_so);
    let without_dylib = without_dll.split(".dylib").next().unwrap_or(without_dll);
    let base = without_dylib.trim_start_matches("lib");

    result.push(name.to_string()); // exact
    result.push(base.to_string()); // base without prefix/suffix

    // Common library mappings
    match base {
        "z" => {
            result.push("zlib".into());
            result.push("libz".into());
        }
        "ssl" | "crypto" => {
            result.push("ssl".into());
            result.push("crypto".into());
            result.push("openssl".into());
            result.push("libssl".into());
            result.push("libcrypto".into());
        }
        "curl" => {
            result.push("curl".into());
            result.push("libcurl".into());
        }
        "xml2" => {
            result.push("xml2".into());
            result.push("libxml2".into());
        }
        "pcap" => {
            result.push("pcap".into());
            result.push("libpcap".into());
        }
        "pthread" => {
            result.push("pthread".into());
            result.push("pthreads".into());
            result.push("libpthread".into());
        }
        "m" => {
            result.push("m".into());
            result.push("libm".into());
            result.push("math".into());
        }
        "dl" => {
            result.push("dl".into());
            result.push("libdl".into());
        }
        _ => {
            // Generic lib prefix handling
            if !base.is_empty() {
                result.push(format!("lib{}", base));

                // Qt special case: Qt5Core, Qt6Core, etc.
                if base.starts_with("Qt") || base.starts_with("qt") {
                    result.push(base.to_string());
                    // Also handle Qt5Core → QtCore
                    if let Some(stripped) = base
                        .strip_prefix("Qt5")
                        .or_else(|| base.strip_prefix("Qt6"))
                    {
                        result.push(format!("Qt{}", stripped));
                    }
                }

                // Handle lib prefix removal
                result.push(base.to_string());
            }
        }
    }

    // Deduplicate
    result.sort();
    result.dedup();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::DependencySource;

    fn create_test_dep(name: &str, ecosystem: &str, source_file: Option<String>) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: "1.0.0".to_string(),
            ecosystem: ecosystem.to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            source_file,
            ..Default::default()
        }
    }

    #[test]
    fn test_classify_by_ecosystem_pip() {
        // PIP ecosystem now returns None to let name-based classification handle tools
        let dep = create_test_dep("black", "PIP", None);
        let result = classify_by_ecosystem(&dep);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_by_ecosystem_system() {
        // System ecosystem now returns None to let name-based classification handle tools
        let dep = create_test_dep("zlib", "system", None);
        let result = classify_by_ecosystem(&dep);
        assert!(result.is_none());
    }

    #[test]
    fn test_classify_by_name_test_framework() {
        let dep = create_test_dep("pytest", "PIP", None);
        let result = classify_by_name(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Test);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_classify_by_name_build_tool() {
        let dep = create_test_dep("cmake", "system", None);
        let result = classify_by_name(&dep);
        assert!(result.is_some());
        let (scope, conf, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Build);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_classify_by_directory_test() {
        let dep = create_test_dep("mylib", "system", Some("/path/to/tests/test.c".to_string()));
        let result = classify_by_directory(&dep);
        assert!(result.is_some());
        let (scope, _, _) = result.unwrap();
        assert_eq!(scope, DependencyScope::Test);
    }

    #[test]
    fn test_normalize_lib_name_enhanced_version_suffix() {
        let result = normalize_lib_name_enhanced("libssl.so.3");
        assert!(result.contains(&"ssl".to_string()));
        assert!(result.contains(&"libssl".to_string()));
        assert!(result.contains(&"openssl".to_string()));
    }

    #[test]
    fn test_normalize_lib_name_enhanced_qt() {
        let result = normalize_lib_name_enhanced("libQt5Core.so.5.15.2");
        assert!(result.contains(&"Qt5Core".to_string()));
        assert!(result.contains(&"libQt5Core".to_string()));
        assert!(result.contains(&"QtCore".to_string()));
    }

    #[test]
    fn test_normalize_lib_name_enhanced_curl() {
        let result = normalize_lib_name_enhanced("curl");
        assert!(result.contains(&"curl".to_string()));
        assert!(result.contains(&"libcurl".to_string()));
    }

    #[test]
    fn test_normalize_lib_name_enhanced_zlib() {
        let result = normalize_lib_name_enhanced("z");
        assert!(result.contains(&"z".to_string()));
        assert!(result.contains(&"zlib".to_string()));
        assert!(result.contains(&"libz".to_string()));
    }
}
