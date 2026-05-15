pub mod bazel;
pub mod c;
pub mod gguf;
pub mod safetensors;
pub mod cargo;
pub mod cmake;
pub mod cpp;
pub mod git;
pub mod go;
pub mod java;
pub mod meson;
pub mod npm;
pub mod php;
pub mod python;
pub mod ros;
pub mod ruby;
pub mod source_scanner;

use crate::models::{Dependency, DependencySource};
use std::collections::{HashMap, HashSet};
use std::path::Path;

// Re-export parser functions
pub use cargo::{parse_cargo_lock_with_relationships, parse_cargo_toml};
pub use go::parse_go_mod;
pub use java::{parse_gradle_build, parse_gradle_kts_build, parse_pom_xml};
pub use npm::{parse_package_json, parse_package_lock_json_with_relationships, parse_yarn_lock};
pub use php::parse_composer_json;
pub use python::{
    parse_pipfile, parse_pipfile_lock_with_relationships, parse_poetry_lock_with_relationships,
    parse_pyproject_toml, parse_requirements_txt, resolve_requirements_txt_transitive,
};
pub use ros::parse_ros_package;
pub use ruby::parse_gemfile;
pub use source_scanner::{scan_go_imports, scan_js_ts_imports, scan_python_imports};

// C++ ecosystem parsers
pub use cpp::{parse_conan_lock, parse_conanfile_py, parse_conanfile_txt, parse_vcpkg_json};

// Git ecosystem parsers
pub use git::{
    is_git_available, is_git_repo_url, parse_git_url, parse_gitmodules, resolve_submodule_commits,
    GitHostType,
};

// CMake ecosystem parsers
pub use cmake::parse_cmake_file;

// C ecosystem parsers
pub use c::{
    collect_doxygen_versions, collect_epd_versions, parse_arxml, parse_configure_ac,
    parse_library_json, parse_makefile, parse_makefile_am, parse_mk_files_as_dependencies,
    parse_pc_file, scan_vendored_3rdparty,
};

// Meson ecosystem parsers
pub use meson::{parse_all_wraps, parse_meson_build};

// Bazel ecosystem parsers
pub use bazel::{parse_module_bazel, parse_workspace};

// GGUF AI model parser
pub use gguf::parse_gguf_file;

// Safetensors AI model parser (v1.0.11)
pub use safetensors::{
    collect_safetensors_shard_paths, parse_safetensors_dir,
};

// Utility functions

pub fn deduplicate_dependencies(deps: Vec<Dependency>) -> Vec<Dependency> {
    let mut map: HashMap<(String, String, String), Dependency> = HashMap::new();

    for dep in deps {
        let key = (dep.name.clone(), dep.version.clone(), dep.ecosystem.clone());

        if let Some(existing) = map.get(&key) {
            // Priority: LockFile > Manifest > ImportScan
            let should_replace = match (&dep.source, &existing.source) {
                // LockFile replaces Manifest
                (DependencySource::LockFile, DependencySource::Manifest) => true,
                // ImportScan never replaces (lowest priority)
                (DependencySource::ImportScan, _) => false,
                // Manifest/LockFile replace ImportScan
                (DependencySource::Manifest, DependencySource::ImportScan) => true,
                (DependencySource::LockFile, DependencySource::ImportScan) => true,
                // Same source type, keep existing
                _ => false,
            };

            if should_replace {
                map.insert(key, dep);
            }
        } else {
            map.insert(key, dep);
        }
    }

    let mut deduplicated: Vec<Dependency> = map.into_values().collect();

    // Post-processing: Remove lower-priority entries when higher-priority versions exist
    // Build sets of (name, ecosystem) pairs for each source type
    let lockfile_packages: HashSet<(String, String)> = deduplicated
        .iter()
        .filter(|dep| dep.source == DependencySource::LockFile)
        .map(|dep| (dep.name.clone(), dep.ecosystem.clone()))
        .collect();

    let manifest_packages: HashSet<(String, String)> = deduplicated
        .iter()
        .filter(|dep| dep.source == DependencySource::Manifest)
        .map(|dep| (dep.name.clone(), dep.ecosystem.clone()))
        .collect();

    // Filter based on priority: LockFile > Manifest > ImportScan
    deduplicated.retain(|dep| {
        let package_key = (dep.name.clone(), dep.ecosystem.clone());
        match dep.source {
            DependencySource::LockFile => {
                // Always keep lockfile entries (highest priority)
                true
            }
            DependencySource::Manifest => {
                // Keep manifest only if no lockfile version exists
                !lockfile_packages.contains(&package_key)
            }
            DependencySource::ImportScan => {
                // Keep import scan only if no lockfile or manifest version exists
                !lockfile_packages.contains(&package_key)
                    && !manifest_packages.contains(&package_key)
            }
        }
    });

    // Drop "unspecified" entries when a versioned entry exists for the same (name, ecosystem).
    // This handles arxml deps collected before version maps are resolved vs after (BUG-03).
    let versioned_packages: HashSet<(String, String)> = deduplicated
        .iter()
        .filter(|dep| dep.version != "unspecified")
        .map(|dep| (dep.name.clone(), dep.ecosystem.clone()))
        .collect();

    if !versioned_packages.is_empty() {
        deduplicated.retain(|dep| {
            if dep.version == "unspecified" {
                !versioned_packages.contains(&(dep.name.clone(), dep.ecosystem.clone()))
            } else {
                true
            }
        });
    }

    // Special case: Prefer "system" ecosystem over "BUILD-CONFIG" for the same package name.
    // This handles Mode 1 (Makefile version resolution) vs Mode 2 (independent .mk parsing),
    // even when versions differ or are left as "unspecified" in the system deps.
    let system_packages: HashSet<String> = deduplicated
        .iter()
        .filter(|dep| dep.ecosystem == "system")
        .map(|dep| dep.name.clone())
        .collect();

    deduplicated.retain(|dep| {
        if dep.ecosystem == "BUILD-CONFIG" {
            !system_packages.contains(&dep.name)
        } else {
            true
        }
    });

    // Special case (v1.0.6): Prefer BUILD-CONFIG over VENDORED for the same package name
    // BUILD-CONFIG versions come from .mk files which are more authoritative than vendored detection
    let build_config_packages: HashSet<String> = deduplicated
        .iter()
        .filter(|dep| dep.ecosystem == "BUILD-CONFIG")
        .map(|dep| dep.name.clone())
        .collect();

    deduplicated.retain(|dep| {
        if dep.ecosystem == "VENDORED" {
            !build_config_packages.contains(&dep.name)
        } else {
            true
        }
    });

    // v1.0.8: When both system (-lFoo) and pkg-config (foo.pc) entries exist for the
    // same library (case-insensitive name match), prefer the pkg-config entry since
    // it carries an actual version number.
    let pkgconfig_names: HashSet<String> = deduplicated
        .iter()
        .filter(|d| d.ecosystem == "pkg-config")
        .map(|d| d.name.to_lowercase())
        .collect();

    if !pkgconfig_names.is_empty() {
        deduplicated.retain(|d| {
            if d.ecosystem == "system" {
                !pkgconfig_names.contains(&d.name.to_lowercase())
            } else {
                true
            }
        });
    }

    // v1.0.17: When an AUTOSAR ecosystem entry exists, drop system (-lFoo) entries
    // for the same name — arxml-extracted deps are more authoritative than linker flags.
    let autosar_names: HashSet<String> = deduplicated
        .iter()
        .filter(|d| d.ecosystem == "autosar")
        .map(|d| d.name.clone())
        .collect();

    if !autosar_names.is_empty() {
        deduplicated.retain(|d| {
            if d.ecosystem == "system" {
                !autosar_names.contains(&d.name)
            } else {
                true
            }
        });
    }

    deduplicated
}

/// Check if a module is part of Python's standard library
pub fn is_python_stdlib(module: &str) -> bool {
    const PYTHON_STDLIB: &[&str] = &[
        "__future__",
        "_thread",
        "abc",
        "aifc",
        "argparse",
        "array",
        "ast",
        "asynchat",
        "asyncio",
        "asyncore",
        "atexit",
        "audioop",
        "base64",
        "bdb",
        "binascii",
        "binhex",
        "bisect",
        "builtins",
        "bz2",
        "calendar",
        "cgi",
        "cgitb",
        "chunk",
        "cmath",
        "cmd",
        "code",
        "codecs",
        "codeop",
        "collections",
        "colorsys",
        "compileall",
        "concurrent",
        "configparser",
        "contextlib",
        "contextvars",
        "copy",
        "copyreg",
        "cprofile",
        "csv",
        "ctypes",
        "curses",
        "dataclasses",
        "datetime",
        "dbm",
        "decimal",
        "difflib",
        "dis",
        "distutils",
        "doctest",
        "email",
        "encodings",
        "ensurepip",
        "enum",
        "errno",
        "faulthandler",
        "fcntl",
        "filecmp",
        "fileinput",
        "fnmatch",
        "formatter",
        "fractions",
        "ftplib",
        "functools",
        "gc",
        "getopt",
        "getpass",
        "gettext",
        "glob",
        "grp",
        "gzip",
        "hashlib",
        "heapq",
        "hmac",
        "html",
        "http",
        "imaplib",
        "imghdr",
        "imp",
        "importlib",
        "inspect",
        "io",
        "ipaddress",
        "itertools",
        "json",
        "keyword",
        "lib2to3",
        "linecache",
        "locale",
        "logging",
        "lzma",
        "mailbox",
        "mailcap",
        "marshal",
        "math",
        "mimetypes",
        "mmap",
        "modulefinder",
        "msilib",
        "msvcrt",
        "multiprocessing",
        "netrc",
        "nis",
        "nntplib",
        "numbers",
        "operator",
        "optparse",
        "os",
        "ossaudiodev",
        "parser",
        "pathlib",
        "pdb",
        "pickle",
        "pickletools",
        "pipes",
        "pkgutil",
        "platform",
        "plistlib",
        "poplib",
        "posix",
        "posixpath",
        "pprint",
        "profile",
        "pstats",
        "pty",
        "pwd",
        "py_compile",
        "pyclbr",
        "pydoc",
        "queue",
        "quopri",
        "random",
        "re",
        "readline",
        "reprlib",
        "resource",
        "rlcompleter",
        "runpy",
        "sched",
        "secrets",
        "select",
        "selectors",
        "shelve",
        "shlex",
        "shutil",
        "signal",
        "site",
        "smtpd",
        "smtplib",
        "sndhdr",
        "socket",
        "socketserver",
        "spwd",
        "sqlite3",
        "ssl",
        "stat",
        "statistics",
        "string",
        "stringprep",
        "struct",
        "subprocess",
        "sunau",
        "symbol",
        "symtable",
        "sys",
        "sysconfig",
        "syslog",
        "tabnanny",
        "tarfile",
        "telnetlib",
        "tempfile",
        "termios",
        "test",
        "textwrap",
        "threading",
        "time",
        "timeit",
        "tkinter",
        "token",
        "tokenize",
        "trace",
        "traceback",
        "tracemalloc",
        "tty",
        "turtle",
        "turtledemo",
        "types",
        "typing",
        "unicodedata",
        "unittest",
        "urllib",
        "uu",
        "uuid",
        "venv",
        "warnings",
        "wave",
        "weakref",
        "webbrowser",
        "winreg",
        "winsound",
        "wsgiref",
        "xdrlib",
        "xml",
        "xmlrpc",
        "zipapp",
        "zipfile",
        "zipimport",
        "zlib",
    ];
    PYTHON_STDLIB.contains(&module)
}

/// Check if a module is a Node.js built-in module
pub fn is_nodejs_builtin(module: &str) -> bool {
    const NODEJS_BUILTINS: &[&str] = &[
        "assert",
        "async_hooks",
        "buffer",
        "child_process",
        "cluster",
        "console",
        "constants",
        "crypto",
        "dgram",
        "diagnostics_channel",
        "dns",
        "domain",
        "events",
        "fs",
        "http",
        "http2",
        "https",
        "inspector",
        "module",
        "net",
        "os",
        "path",
        "perf_hooks",
        "process",
        "punycode",
        "querystring",
        "readline",
        "repl",
        "stream",
        "string_decoder",
        "sys",
        "timers",
        "tls",
        "trace_events",
        "tty",
        "url",
        "util",
        "v8",
        "vm",
        "wasi",
        "worker_threads",
        "zlib",
    ];
    NODEJS_BUILTINS.contains(&module)
}

/// Check if an import path is from Go standard library
pub fn is_go_stdlib(import_path: &str) -> bool {
    // Go stdlib has no domain in path (no '.')
    // Exception: golang.org/x/ packages are also considered stdlib
    if import_path.starts_with("golang.org/x/") {
        return true;
    }
    !import_path.contains('.')
}

/// Extract root package name from Python import path
/// Example: "django.core.utils" -> "django"
pub fn extract_python_package(import_path: &str) -> String {
    import_path
        .split('.')
        .next()
        .unwrap_or(import_path)
        .to_string()
}

/// Extract package name from JavaScript/TypeScript import path
/// Handles scoped packages: "@babel/core/lib" -> "@babel/core"
/// Handles regular packages: "express/router" -> "express"
pub fn extract_js_package(import_path: &str) -> String {
    if import_path.starts_with('@') {
        // Scoped package: @scope/package or @scope/package/subpath
        let parts: Vec<&str> = import_path.split('/').collect();
        if parts.len() >= 2 {
            format!("{}/{}", parts[0], parts[1])
        } else {
            import_path.to_string()
        }
    } else {
        // Regular package: extract first component
        import_path
            .split('/')
            .next()
            .unwrap_or(import_path)
            .to_string()
    }
}

/// Format source file info (v0.9.0: supports compact format for file size optimization)
///
/// Default format: "Identified by the javascript/packagejson extractor from /full/path/to/file.json"
/// Compact format: "javascript/packagejson:/relative/path/file.json"
pub fn format_source_info(
    extractor_type: &str,
    file_path: &Path,
    project_root: Option<&Path>,
    compact: bool,
) -> String {
    if compact {
        // Compact format: "extractor:relative/path"
        let relative_path = if let Some(root) = project_root {
            file_path
                .strip_prefix(root)
                .unwrap_or(file_path)
                .display()
                .to_string()
        } else {
            file_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string()
        };
        format!("{}:{}", extractor_type, relative_path)
    } else {
        // Original verbose format
        let absolute_path = file_path
            .canonicalize()
            .unwrap_or_else(|_| file_path.to_path_buf());
        format!(
            "Identified by the {} extractor from {}",
            extractor_type,
            absolute_path.display()
        )
    }
}
