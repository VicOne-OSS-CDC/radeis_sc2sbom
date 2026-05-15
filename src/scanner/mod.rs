use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::cli::VendorMode;
use crate::util::warn_on_walkdir_err;
use crate::models::{
    Dependency, DependencySource, RosPackageMetadata, RosPackageWithDeps, ScanContext,
};
use crate::parsers::{
    format_source_info,
    is_git_available,
    parse_all_wraps,
    parse_cargo_lock_with_relationships,
    parse_cargo_toml,
    // v1.0.1: CMake parser
    parse_cmake_file,
    parse_composer_json,
    // v1.0.2: Conan C++ package manager
    parse_conan_lock,
    parse_conanfile_py,
    parse_conanfile_txt,
    parse_configure_ac,
    parse_gemfile,
    parse_git_url,
    // v1.0.9: GGUF AI model parser
    parse_gguf_file,
    // v1.0.11: Safetensors AI model parser
    collect_safetensors_shard_paths,
    parse_safetensors_dir,
    parse_gitmodules,
    parse_go_mod,
    // v1.0.10: Gradle dependency parsing
    parse_gradle_build,
    collect_doxygen_versions,
    collect_epd_versions,
    parse_arxml,
    parse_gradle_kts_build,
    // v1.0.8: library.json parser for vendored C/C++ libraries
    parse_library_json,
    parse_makefile,
    parse_makefile_am,
    // v1.0.4: Meson and Bazel build systems
    parse_meson_build,
    // v1.0.5: .mk file parser
    parse_mk_files_as_dependencies,
    parse_module_bazel,
    parse_package_json,
    parse_package_lock_json_with_relationships,
    // v1.0.3: C legacy build systems
    parse_pc_file,
    parse_pipfile,
    parse_pipfile_lock_with_relationships,
    parse_poetry_lock_with_relationships,
    parse_pom_xml,
    parse_pyproject_toml,
    parse_requirements_txt,
    parse_ros_package,
    // v1.0.0: C++ and Git submodule parsers
    parse_vcpkg_json,
    parse_workspace,
    parse_yarn_lock,
    // v1.0.0: Transitive dependency resolution
    resolve_requirements_txt_transitive,
    resolve_submodule_commits,
    // v1.0.6: vendored 3rd_party scanner
    scan_vendored_3rdparty,
};

pub fn is_vendor_directory(path: &Path) -> bool {
    let vendor_dirs = [
        "node_modules",
        "site-packages",
        "venv",
        "env",
        ".venv",
        "__pycache__",
        "vendor",
        "target",
        ".git",
        ".svn",
        ".hg",
        ".bzr",
        "dist",
        "build",
    ];

    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        vendor_dirs.contains(&dir_name)
    } else {
        false
    }
}

/// Check if a path is within a vendor directory (including the vendor directory itself)
fn is_within_vendor_directory(path: &Path) -> bool {
    // Check if the path itself is a vendor directory
    if is_vendor_directory(path) {
        return true;
    }

    // Check if any ancestor is a vendor directory
    for ancestor in path.ancestors().skip(1) {
        if is_vendor_directory(ancestor) {
            return true;
        }
    }

    false
}

pub fn should_process_entry(
    entry: &DirEntry,
    vendor_mode: &VendorMode,
    excludes: &[String],
) -> bool {
    let path = entry.path();
    let is_vendor = is_vendor_directory(path);

    // Check custom exclusions
    if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
        if excludes.iter().any(|pattern| dir_name == pattern) {
            return false;
        }
    }

    // Apply vendor mode logic
    match vendor_mode {
        VendorMode::Skip => !is_vendor,
        VendorMode::Include => true,
        VendorMode::Only => {
            // Include root directory and anything within vendor directories
            entry.depth() == 0 || is_within_vendor_directory(path)
        }
    }
}

/// AUTOSAR project detection pre-pass (Phase 6, v1.0.15).
///
/// Returns `true` if any of the following signals is present:
/// - DET-01: A `.arxml` file at any depth in the tree.
/// - DET-02: A directory named exactly `BSW`, `MCAL`, `RTE`, `AUTOSAR`, or `SWC` (case-sensitive).
/// - DET-03: A `CMakeLists.txt`, `Makefile`, or `GNUmakefile` at the root or one level deep
///           containing the standalone token `AUTOSAR_VERSION` or `AR_VERSION`.
///
/// Short-circuits on first match. Respects `vendor_mode` and `excludes` for
/// filesystem traversal (signals 1 and 2). WalkDir errors are downgraded to
/// warnings via `warn_on_walkdir_err`.
pub fn detect_autosar(path: &Path, vendor_mode: &VendorMode, excludes: &[String]) -> bool {
    // DET-01: .arxml files anywhere in the tree
    let has_arxml = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
        .filter_map(warn_on_walkdir_err)
        .any(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "arxml")
                .unwrap_or(false)
        });
    if has_arxml {
        return true;
    }

    // DET-02: canonical AUTOSAR directory names
    let autosar_dirs = ["BSW", "MCAL", "RTE", "AUTOSAR", "SWC"];
    let has_autosar_dir = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
        .filter_map(warn_on_walkdir_err)
        .any(|e| {
            e.file_type().is_dir()
                && e.file_name()
                    .to_str()
                    .map(|name| autosar_dirs.contains(&name))
                    .unwrap_or(false)
        });
    if has_autosar_dir {
        return true;
    }

    // DET-03: AUTOSAR_VERSION or AR_VERSION in CMake/Makefile at root + one level deep
    use std::sync::OnceLock;
    static AUTOSAR_RE: OnceLock<Regex> = OnceLock::new();
    let re = AUTOSAR_RE.get_or_init(|| {
        Regex::new(r"\b(AUTOSAR_VERSION|AR_VERSION)\b").expect("valid AUTOSAR regex")
    });
    let build_file_names = ["CMakeLists.txt", "Makefile", "GNUmakefile"];
    let mut search_dirs: Vec<PathBuf> = vec![path.to_path_buf()];
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let ep = entry.path();
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false)
                && !is_vendor_directory(&ep)
                && !excludes.iter().any(|ex| {
                    ep.file_name().and_then(|n| n.to_str()) == Some(ex.as_str())
                })
            {
                search_dirs.push(ep);
            }
        }
    }
    for dir in &search_dirs {
        for name in &build_file_names {
            let candidate = dir.join(name);
            if candidate.is_file() {
                if let Ok(content) = std::fs::read_to_string(&candidate) {
                    if re.is_match(&content) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/// Recursively discover nested Git submodules under a submodule directory.
///
/// This function walks the given `submodule_path`, looks for nested `.gitmodules`
/// and corresponding submodule directories, and converts those nested submodules
/// into `Dependency` entries.
///
/// It does *not* parse package manifests (such as `package.json`, `Cargo.toml`,
/// `CMakeLists.txt`, etc.) inside the submodule, and it does not perform
/// manifest-level attribution of packages to a particular submodule. Its sole
/// responsibility is to discover nested submodules and model them as dependencies.
///
/// # Arguments
/// * `submodule_path` - Path to the root of the (already-resolved) Git submodule.
/// * `submodule_name` - Name/identifier of the parent submodule used for source attribution.
/// * `vendor_mode` - Vendor directory handling mode, forwarded to directory traversal logic.
/// * `excludes` - Custom exclusion patterns applied while walking the filesystem.
/// * `current_depth` - Current recursion depth when following nested submodules.
/// * `max_depth` - Maximum recursion depth to prevent infinite loops through nested submodules.
/// * `scan_c_build_systems` - Whether higher-level scanning logic should consider C/C++ build system files; forwarded but not used for manifest parsing here.
/// * `resolve_transitive` - Whether transitive dependencies should be resolved by callers; this helper itself only models submodules.
/// * `visited` - Set of visited paths to detect and avoid circular references between submodules.
///
/// # Returns
/// Vector of `Dependency` entries representing nested submodules discovered under `submodule_path`.
fn scan_submodule_recursively(
    submodule_path: &Path,
    submodule_name: &str,
    vendor_mode: &VendorMode,
    excludes: &[String],
    current_depth: usize,
    max_depth: usize,
    scan_c_build_systems: bool,
    resolve_transitive: bool,
    visited: &mut std::collections::HashSet<std::path::PathBuf>,
) -> Result<Vec<Dependency>> {
    // Detect circular references
    let canonical_path = match submodule_path.canonicalize() {
        Ok(p) => p,
        Err(_) => submodule_path.to_path_buf(), // Use non-canonical if canonicalize fails
    };

    if visited.contains(&canonical_path) {
        eprintln!(
            "Warning: Circular submodule reference detected at {}, skipping",
            submodule_path.display()
        );
        return Ok(Vec::new());
    }

    visited.insert(canonical_path);

    // Enforce depth limit after circular reference check
    // This ensures we track visited paths even at max depth to prevent circular refs
    if current_depth >= max_depth {
        return Ok(Vec::new());
    }

    let mut submodule_dependencies = Vec::new();

    // NOTE:
    // We intentionally do not recursively scan dependencies inside submodules here.
    // The main scan_directory() WalkDir traversal will already descend into
    // submodule directories and parse their manifests. Performing an additional
    // scan via scan_submodule_recursively() would re-parse the same manifests,
    // producing duplicate Dependency entries that later deduplication has to
    // resolve, and potentially dropping the more specific submodule-attributed
    // entries. To avoid this duplication and unstable attribution, we rely
    // solely on the primary directory walk for manifest parsing.
    //
    // This function ONLY handles recursive .gitmodules scanning to discover
    // nested submodules and their commit SHAs.

    // `.gitmodules` lives at the submodule repo root, so avoid a full
    // recursive directory walk here. Just check the root-level `.gitmodules`
    // file and let nested submodules be discovered recursively from there.
    let path = submodule_path.join(".gitmodules");
    if !path.is_file() {
        return Ok(Vec::new());
    }

    // Parse nested .gitmodules for recursive submodule scanning
    let deps_result: Result<Vec<Dependency>> = if !is_git_available() {
        Ok(Vec::new())
    } else {
        match parse_gitmodules(&path) {
            Ok(mut nested_submodules) => {
                // Resolve commit SHAs for nested submodules
                if let Some(repo_root) = path.parent() {
                    let _ = resolve_submodule_commits(repo_root, &mut nested_submodules);
                }

                // Convert nested submodules to dependencies and recursively scan them
                let mut nested_deps = Vec::new();
                for nested_submodule in &nested_submodules {
                    let version = nested_submodule
                        .commit_sha
                        .clone()
                        .unwrap_or_else(|| "uninitialized".to_string());

                    let (name, repo_url) = if let Some(info) = parse_git_url(&nested_submodule.url)
                    {
                        (
                            format!("{}/{}", info.owner, info.repo),
                            Some(nested_submodule.url.clone()),
                        )
                    } else {
                        (
                            nested_submodule.name.clone(),
                            Some(nested_submodule.url.clone()),
                        )
                    };

                    nested_deps.push(Dependency {
                        name,
                        version,
                        ecosystem: "git-submodule".to_string(),
                        source: DependencySource::Manifest,
                        is_dev: false,
                        is_direct: true,
                        repository_url: repo_url,
                        source_file: Some(format!(
                            "git-submodule extractor from {}",
                            path.display()
                        )),
                        ..Default::default()
                    });

                    // Recursively scan the nested submodule
                    if let Some(repo_root) = path.parent() {
                        let nested_path = repo_root.join(&nested_submodule.path);
                        if nested_path.exists() && nested_path.is_dir() {
                            match scan_submodule_recursively(
                                &nested_path,
                                &nested_submodule.name,
                                vendor_mode,
                                excludes,
                                current_depth + 1,
                                max_depth,
                                scan_c_build_systems,
                                resolve_transitive,
                                visited,
                            ) {
                                Ok(deps) => nested_deps.extend(deps),
                                Err(e) => {
                                    eprintln!(
                                        "Warning: Failed to scan nested submodule {}: {}",
                                        nested_submodule.name, e
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(nested_deps)
            }
            Err(e) => {
                eprintln!("Warning: Failed to parse nested .gitmodules: {}", e);
                Ok(Vec::new())
            }
        }
    };

    // Mark dependencies as coming from submodule
    if let Ok(mut deps) = deps_result {
        for dep in &mut deps {
            // Update source_file to indicate submodule origin
            if let Some(ref source) = dep.source_file {
                dep.source_file = Some(format!("{} (submodule: {})", source, submodule_name));
            } else {
                dep.source_file = Some(format!("from submodule: {}", submodule_name));
            }
        }
        submodule_dependencies.extend(deps);
    }

    Ok(submodule_dependencies)
}

/// Resolve the directory to scan for a C/C++ component.
///
/// Tries, in order:
///   1. `parent/name`          (exact match)
///   2. `parent/lib{name}`     (lib-prefix variant)
///   3. A case-insensitive match among immediate subdirs of `parent`
///
/// Returns `None` if no matching subdir is found, meaning the dep is an
/// external/system library not vendored in this repo and should not be scanned.
pub fn resolve_component_dir(parent: &Path, name: &str) -> Option<PathBuf> {
    // 1. Exact
    let exact = parent.join(name);
    if exact.is_dir() {
        return Some(exact);
    }
    // 2. lib-prefix
    let lib_prefixed = parent.join(format!("lib{}", name));
    if lib_prefixed.is_dir() {
        return Some(lib_prefixed);
    }
    // 3. Case-insensitive scan of immediate subdirs
    let name_lower = name.to_lowercase();
    let lib_lower = format!("lib{}", name_lower);
    if let Ok(entries) = std::fs::read_dir(parent) {
        for entry in entries.flatten() {
            if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                let entry_name = entry.file_name();
                let entry_str = entry_name.to_string_lossy().to_lowercase();
                if entry_str == name_lower || entry_str == lib_lower {
                    return Some(entry.path());
                }
            }
        }
    }
    None
}

pub fn scan_directory(
    path: &Path,
    vendor_mode: &VendorMode,
    excludes: &[String],
    _ros_distro: Option<&str>,
    scan_submodules: bool,
    submodule_depth: usize,
    scan_c_build_systems: bool,
    resolve_transitive: bool,
    scan_meson: bool,
    scan_bazel: bool,
    scan_so_files: bool,
    target_arch: Option<&str>,
    scan_ai_models: Option<u64>,     // v1.0.9: None=disabled, Some(0)=unlimited hash, Some(N)=skip hash >N GB
    is_autosar: bool,                // v1.0.15: pre-computed by detect_autosar()
) -> Result<ScanContext> {
    let mut all_dependencies = Vec::new();
    let mut npm_relationships = Vec::new();
    let mut cargo_relationships = Vec::new();
    let mut python_lockfile_relationships = Vec::new();
    let git_submodule_relationships = Vec::new();
    let mut ros_metadata: Option<RosPackageMetadata> = None;
    let mut ros_packages: Vec<RosPackageWithDeps> = Vec::new();
    // Phase 11 (D-01): (name, ecosystem) -> dependency-manifest parent directory.
    // Populated by the six C/C++ parser arms below; consumed by run_lexical_scanner.
    // Components from so-scanner discoveries have no entry (D-03) — scanner skips them.
    let mut component_dirs: HashMap<(String, String), PathBuf> = HashMap::new();

    // Progress counters
    let mut manifest_count = 0;
    let mut lock_count = 0;
    let mut file_counter = 0;

    // Track parsed subprojects directories to avoid re-scanning
    let mut parsed_subprojects_dirs = std::collections::HashSet::new();

    // Create spinner with custom style
    let spinner = ProgressBar::new_spinner();
    let spinner_style = ProgressStyle::default_spinner()
        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        .template("[1/5] Walking directory tree... {msg} {spinner}")
        .map_err(|e| anyhow::anyhow!("Failed to create spinner template: {}", e))?;
    spinner.set_style(spinner_style);
    spinner.enable_steady_tick(std::time::Duration::from_millis(100)); // 100 ms tick (10 Hz) spinner animation

    // Capture scan root before the WalkDir loop shadows `path` with the current entry path
    let scan_root = path;

    // v1.0.17 (BUG-03): Collect AUTOSAR version maps once per scan.
    // Built unconditionally (cheap no-ops when no .epd or C/H files exist).
    let epd_versions = collect_epd_versions(scan_root);
    let doxygen_versions = collect_doxygen_versions(scan_root);

    // v1.0.11: Pre-scan pass for safetensors model directories.
    // Safetensors models are directory-level (one model = multiple shard files).
    // We detect model dirs first and produce one Dependency per model, then
    // record all shard .safetensors paths to skip in the main file-level loop.
    let mut handled_safetensors_paths: HashSet<PathBuf> = HashSet::new();
    if let Some(max_hash_gb) = scan_ai_models {
        // Discover model dirs using a WalkDir that applies the same vendor_mode/excludes
        // filtering as the main file loop, so excluded directories (e.g. target, node_modules,
        // custom --exclude paths) are not scanned during the pre-scan pass either.
        let model_dirs: Vec<PathBuf> = WalkDir::new(path)
            .follow_links(true)
            .max_depth(50)
            .into_iter()
            .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
            .filter_map(warn_on_walkdir_err)
            .filter(|e| e.file_type().is_dir())
            .filter(|e| crate::parsers::safetensors::is_safetensors_model_dir(e.path()))
            .map(|e| e.into_path())
            .collect();

        for model_dir in &model_dirs {
            spinner.set_message(format!(
                "scanning safetensors model {}...",
                model_dir.display()
            ));
            match parse_safetensors_dir(model_dir, max_hash_gb) {
                Ok(deps) => {
                    if !deps.is_empty() {
                        manifest_count += 1;
                    }
                    all_dependencies.extend(deps);
                    // Only mark shards as handled after a successful parse so the main
                    // loop can still attempt them if the parse failed.
                    for shard_path in collect_safetensors_shard_paths(model_dir) {
                        handled_safetensors_paths.insert(shard_path);
                    }
                }
                Err(_) => {
                    // Parse failed — do not mark shards as handled; the main walk
                    // will encounter them and can attempt a second parse.
                }
            }
        }
    }

    for entry in WalkDir::new(path)
        .follow_links(true)
        .max_depth(50)
        .into_iter()
        .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
        .filter_map(warn_on_walkdir_err)
    {
        let path = entry.path();
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Update counter and spinner message
        file_counter += 1;
        if file_counter % 10 == 0 {
            spinner.set_message(format!("{} entries scanned", file_counter));
        }

        match file_name {
            "package.json" => {
                if let Ok(deps) = parse_package_json(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "Cargo.toml" => {
                if let Ok(deps) = parse_cargo_toml(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "requirements.txt" => {
                // v1.0.0: Optionally resolve transitive dependencies using pip
                if resolve_transitive {
                    if let Ok(data) = resolve_requirements_txt_transitive(path, true) {
                        all_dependencies.extend(data.dependencies);
                        python_lockfile_relationships.extend(data.relationships);
                        manifest_count += 1;
                    }
                } else if let Ok(deps) = parse_requirements_txt(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "pyproject.toml" => {
                // Skip if poetry.lock exists in same directory (lock file takes precedence)
                if let Some(parent) = path.parent() {
                    let poetry_lock_path = parent.join("poetry.lock");
                    if poetry_lock_path.exists() {
                        // Skip: poetry.lock will provide accurate pinned versions
                        continue;
                    }
                }
                // TODO: Wire enable_network to CLI flag (default: true for backward compatibility)
                if let Ok(deps) = parse_pyproject_toml(path, true) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "go.mod" => {
                if let Ok(deps) = parse_go_mod(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "pom.xml" => {
                if let Ok(deps) = parse_pom_xml(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "package.xml" => {
                if let Ok((pkg_metadata, deps)) = parse_ros_package(path) {
                    all_dependencies.extend(deps.clone());
                    manifest_count += 1;

                    // Store first package metadata for backward compatibility
                    if ros_metadata.is_none() {
                        ros_metadata = pkg_metadata.clone();
                    }

                    // Store all ROS packages with their dependencies
                    if let Some(metadata) = pkg_metadata {
                        ros_packages.push(RosPackageWithDeps {
                            metadata,
                            dependencies: deps,
                        });
                    }
                }
            }
            "build.gradle" => {
                if let Ok(deps) = parse_gradle_build(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "build.gradle.kts" => {
                if let Ok(deps) = parse_gradle_kts_build(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "Gemfile" => {
                if let Ok(deps) = parse_gemfile(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            "composer.json" => {
                if let Ok(deps) = parse_composer_json(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.0: C++ vcpkg manifest
            "vcpkg.json" => {
                if let Ok(deps) = parse_vcpkg_json(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.2: Conan lock file (takes precedence over manifests)
            "conan.lock" => {
                spinner.set_message("parsing conan.lock...");
                if let Ok(deps) = parse_conan_lock(path) {
                    all_dependencies.extend(deps);
                    lock_count += 1;
                }
            }
            // v1.0.2: Conan manifest (INI format)
            "conanfile.txt" => {
                // Skip if conan.lock exists in same directory (lock file takes precedence)
                if let Some(parent) = path.parent() {
                    let lock_path = parent.join("conan.lock");
                    if lock_path.exists() {
                        continue; // Skip: conan.lock provides accurate pinned versions
                    }
                }
                spinner.set_message("parsing conanfile.txt...");
                if let Ok(deps) = parse_conanfile_txt(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.2: Conan manifest (Python format)
            "conanfile.py" => {
                // Skip if conan.lock exists in same directory
                if let Some(parent) = path.parent() {
                    let lock_path = parent.join("conan.lock");
                    if lock_path.exists() {
                        continue;
                    }
                }
                spinner.set_message("parsing conanfile.py...");
                if let Ok(deps) = parse_conanfile_py(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.1: CMake FetchContent/ExternalProject
            "CMakeLists.txt" => {
                if !scan_c_build_systems {
                    continue;
                }
                spinner.set_message("parsing CMakeLists.txt...");
                if let Ok(deps) = parse_cmake_file(path) {
                    if let Some(parent) = path.parent() {
                        for dep in &deps {
                            if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                                component_dirs
                                    .entry((dep.name.clone(), dep.ecosystem.clone()))
                                    .or_insert(dir);
                            }
                        }
                    }
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.1: CMake *.cmake files (module scripts)
            name if name.ends_with(".cmake") && scan_c_build_systems => {
                spinner.set_message(format!("parsing {}...", name));
                if let Ok(deps) = parse_cmake_file(path) {
                    if let Some(parent) = path.parent() {
                        for dep in &deps {
                            if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                                component_dirs
                                    .entry((dep.name.clone(), dep.ecosystem.clone()))
                                    .or_insert(dir);
                            }
                        }
                    }
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.17 (BUG-01): AUTOSAR .arxml composition/BSW module files
            name if name.ends_with(".arxml") && is_autosar => {
                spinner.set_message(format!("parsing {}...", name));
                if let Ok(deps) = parse_arxml(path, &epd_versions, &doxygen_versions) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.3: pkg-config .pc files
            name if name.ends_with(".pc") && scan_c_build_systems => {
                spinner.set_message(format!("parsing {}...", name));
                if let Ok(dep) = parse_pc_file(path) {
                    if let Some(parent) = path.parent() {
                        if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                            component_dirs
                                .entry((dep.name.clone(), dep.ecosystem.clone()))
                                .or_insert(dir);
                        }
                    }
                    all_dependencies.push(dep);
                    manifest_count += 1;
                }
            }
            // v1.0.9: GGUF AI model files
            name if name.ends_with(".gguf") && scan_ai_models.is_some() => {
                spinner.set_message(format!("parsing {}...", name));
                if let Ok(deps) = parse_gguf_file(path, scan_ai_models.unwrap_or(0)) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.11: Safetensors shard files already handled by pre-scan pass — skip
            name if name.ends_with(".safetensors") && scan_ai_models.is_some() => {
                // Model was already produced by the pre-scan pass; skip individual shard file
                let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
                if !handled_safetensors_paths.contains(&canonical)
                    && !handled_safetensors_paths.contains(path)
                {
                    // Standalone .safetensors not inside a recognized model dir — parse it
                    if let Some(parent) = path.parent() {
                        if let Ok(deps) = parse_safetensors_dir(parent, scan_ai_models.unwrap_or(0)) {
                            all_dependencies.extend(deps);
                            manifest_count += 1;
                        }
                        // Mark all shards in parent as handled
                        for shard in collect_safetensors_shard_paths(parent) {
                            handled_safetensors_paths.insert(shard);
                        }
                    }
                }
            }
            // v1.0.3: Autotools configure.ac
            "configure.ac" | "configure.in" => {
                if !scan_c_build_systems {
                    continue;
                }
                spinner.set_message("parsing configure.ac...");
                if let Ok(deps) = parse_configure_ac(path) {
                    if let Some(parent) = path.parent() {
                        for dep in &deps {
                            if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                                component_dirs
                                    .entry((dep.name.clone(), dep.ecosystem.clone()))
                                    .or_insert(dir);
                            }
                        }
                    }
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.3: Autotools Makefile.am
            "Makefile.am" => {
                if !scan_c_build_systems {
                    continue;
                }
                spinner.set_message("parsing Makefile.am...");
                if let Ok(deps) = parse_makefile_am(path) {
                    if let Some(parent) = path.parent() {
                        for dep in &deps {
                            if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                                component_dirs
                                    .entry((dep.name.clone(), dep.ecosystem.clone()))
                                    .or_insert(dir);
                            }
                        }
                    }
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.3: Plain Makefiles (only if NOT an Autotools project)
            "Makefile" | "makefile" => {
                if !scan_c_build_systems {
                    continue;
                }
                // Check if this is an Autotools project by looking for configure.ac/in or Makefile.am
                // Check current directory and up to 2 parent directories to handle multi-directory projects
                let is_autotools_project = if let Some(mut parent) = path.parent() {
                    let mut is_autotools = false;
                    for _ in 0..3 {
                        if parent.join("configure.ac").exists()
                            || parent.join("configure.in").exists()
                            || parent.join("Makefile.am").exists()
                        {
                            is_autotools = true;
                            break;
                        }
                        if let Some(grandparent) = parent.parent() {
                            parent = grandparent;
                        } else {
                            break;
                        }
                    }
                    is_autotools
                } else {
                    false
                };

                if is_autotools_project {
                    continue; // Skip Makefile parsing in Autotools projects
                }

                spinner.set_message("parsing Makefile...");
                if let Ok(deps) =
                    parse_makefile(path, scan_c_build_systems, scan_so_files, target_arch, scan_root)
                {
                    if let Some(parent) = path.parent() {
                        for dep in &deps {
                            if let Some(dir) = resolve_component_dir(parent, &dep.name) {
                                component_dirs
                                    .entry((dep.name.clone(), dep.ecosystem.clone()))
                                    .or_insert(dir);
                            }
                        }
                    }
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.4: Meson build system
            "meson.build" => {
                if !scan_meson {
                    continue;
                }
                spinner.set_message("parsing meson.build...");
                if let Ok(deps) = parse_meson_build(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }

                // Check for subprojects/*.wrap files if enabled
                if scan_meson {
                    if let Some(parent) = path.parent() {
                        let subprojects_dir = parent.join("subprojects");
                        if subprojects_dir.exists() {
                            // Canonicalize path to avoid re-scanning the same directory
                            let canonical_dir = subprojects_dir
                                .canonicalize()
                                .unwrap_or(subprojects_dir.clone());
                            if parsed_subprojects_dirs.insert(canonical_dir) {
                                spinner.set_message("parsing Meson subprojects...");
                                if let Ok(wrap_deps) = parse_all_wraps(&subprojects_dir) {
                                    let wrap_count = wrap_deps.len();
                                    all_dependencies.extend(wrap_deps);
                                    manifest_count += wrap_count;
                                }
                            }
                        }
                    }
                }
            }
            // v1.0.4: Bazel WORKSPACE
            "WORKSPACE" | "WORKSPACE.bazel" => {
                if !scan_bazel {
                    continue;
                }
                spinner.set_message("parsing WORKSPACE...");
                if let Ok(deps) = parse_workspace(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.4: Bazel MODULE.bazel (Bazel 6.0+ bzlmod)
            "MODULE.bazel" => {
                if !scan_bazel {
                    continue;
                }
                spinner.set_message("parsing MODULE.bazel...");
                if let Ok(deps) = parse_module_bazel(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.0: Git submodules
            ".gitmodules" => {
                if !scan_submodules {
                    continue;
                }

                // Check if git is available
                if !is_git_available() {
                    eprintln!("Warning: git command not found, skipping submodule scanning");
                    continue;
                }

                spinner.set_message("parsing .gitmodules...");

                if let Ok(mut submodules) = parse_gitmodules(path) {
                    // Resolve commit SHAs for each submodule
                    if let Some(repo_root) = path.parent() {
                        if let Err(e) = resolve_submodule_commits(repo_root, &mut submodules) {
                            eprintln!("Warning: Failed to resolve submodule commits: {}", e);
                        }
                    }

                    // Convert each submodule to a dependency
                    for submodule in &submodules {
                        let version = submodule
                            .commit_sha
                            .clone()
                            .unwrap_or_else(|| "uninitialized".to_string());

                        // Extract repo info for better naming
                        let (name, repo_url) = if let Some(info) = parse_git_url(&submodule.url) {
                            (
                                format!("{}/{}", info.owner, info.repo),
                                Some(submodule.url.clone()),
                            )
                        } else {
                            (submodule.name.clone(), Some(submodule.url.clone()))
                        };

                        all_dependencies.push(Dependency {
                            name,
                            version,
                            ecosystem: "git-submodule".to_string(),
                            source: DependencySource::Manifest,
                            is_dev: false,
                            is_direct: true,
                            repository_url: repo_url,
                            source_file: Some(format_source_info(
                                "git-submodule",
                                path,
                                None,
                                false,
                            )),
                            ..Default::default()
                        });
                    }

                    // v1.0.1: Recursively scan dependencies inside submodules
                    if let Some(repo_root) = path.parent() {
                        let mut visited = std::collections::HashSet::new();
                        for submodule in &submodules {
                            let submodule_path = repo_root.join(&submodule.path);
                            if submodule_path.exists() && submodule_path.is_dir() {
                                match scan_submodule_recursively(
                                    &submodule_path,
                                    &submodule.name,
                                    vendor_mode,
                                    excludes,
                                    1, // Start at depth 1
                                    submodule_depth,
                                    scan_c_build_systems,
                                    resolve_transitive,
                                    &mut visited,
                                ) {
                                    Ok(deps) => {
                                        all_dependencies.extend(deps);
                                    }
                                    Err(e) => {
                                        eprintln!(
                                            "Warning: Failed to scan submodule {}: {}",
                                            submodule.name, e
                                        );
                                    }
                                }
                            }
                        }
                    }

                    manifest_count += 1;
                }
            }
            // Lock files
            "package-lock.json" => {
                spinner.set_message("parsing package-lock.json...");
                if let Ok(data) = parse_package_lock_json_with_relationships(path) {
                    all_dependencies.extend(data.dependencies);
                    npm_relationships.extend(data.relationships);
                    lock_count += 1;
                }
            }
            "yarn.lock" => {
                spinner.set_message("parsing yarn.lock...");
                if let Ok(deps) = parse_yarn_lock(path) {
                    all_dependencies.extend(deps);
                    lock_count += 1;
                }
            }
            "Cargo.lock" => {
                spinner.set_message("parsing Cargo.lock...");
                if let Ok(data) = parse_cargo_lock_with_relationships(path) {
                    all_dependencies.extend(data.dependencies);
                    cargo_relationships.extend(data.relationships);
                    lock_count += 1;
                }
            }
            "poetry.lock" => {
                spinner.set_message("parsing poetry.lock...");
                // TODO: Wire enable_network to CLI flag (default: true for backward compatibility)
                if let Ok(data) = parse_poetry_lock_with_relationships(path, true) {
                    all_dependencies.extend(data.dependencies);
                    python_lockfile_relationships.extend(data.relationships);
                    lock_count += 1;
                }
            }
            "Pipfile.lock" => {
                spinner.set_message("parsing Pipfile.lock...");
                // TODO: Wire enable_network to CLI flag (default: true for backward compatibility)
                if let Ok(data) = parse_pipfile_lock_with_relationships(path, true) {
                    all_dependencies.extend(data.dependencies);
                    python_lockfile_relationships.extend(data.relationships); // Store Python lock file relationships
                    lock_count += 1;
                }
            }
            "Pipfile" => {
                // Skip if Pipfile.lock exists in same directory (lock file takes precedence)
                if let Some(parent) = path.parent() {
                    let pipfile_lock_path = parent.join("Pipfile.lock");
                    if pipfile_lock_path.exists() {
                        // Skip: Pipfile.lock will provide accurate pinned versions
                        continue;
                    }
                }
                // TODO: Wire enable_network to CLI flag (default: true for backward compatibility)
                if let Ok(deps) = parse_pipfile(path, true) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            // v1.0.8: PlatformIO/LVGL library.json for vendored C/C++ libraries
            "library.json" => {
                spinner.set_message("parsing library.json...");
                if let Ok(deps) = parse_library_json(path) {
                    all_dependencies.extend(deps);
                    manifest_count += 1;
                }
            }
            _ => {}
        }
    }

    // v1.0.5: Independent .mk file scanning (runs even without Makefile)
    // This enables version detection for repositories like xcar-toolchains that have
    // .mk files with VERSION variables but no actual Makefile
    if scan_c_build_systems {
        spinner.set_message("scanning .mk files for versions...");
        match parse_mk_files_as_dependencies(path, target_arch) {
            Ok(mk_deps) => {
                if !mk_deps.is_empty() {
                    all_dependencies.extend(mk_deps);
                    manifest_count += 1; // Count as 1 manifest file, not per-dependency
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to scan .mk files: {}", e);
            }
        }
    }

    // Vendored 3rd-party libraries (v1.0.6)
    spinner.set_message("scanning vendored 3rd_party libraries...");
    match scan_vendored_3rdparty(path, excludes) {
        Ok(vendored_deps) => {
            if !vendored_deps.is_empty() {
                // Phase 11 (D-01): record per-dep directory for lexical scanner scoping.
                // DependencySource has no path variant; use source_file string when available,
                // otherwise fall back to the vendored scan root.
                for dep in &vendored_deps {
                    let dir: PathBuf = dep
                        .source_file
                        .as_deref()
                        .and_then(|sf| std::path::Path::new(sf).parent())
                        .map(|p| p.to_path_buf())
                        .unwrap_or_else(|| path.to_path_buf());
                    component_dirs
                        .entry((dep.name.clone(), dep.ecosystem.clone()))
                        .or_insert(dir);
                }
                all_dependencies.extend(vendored_deps);
                manifest_count += 1;
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to scan vendored 3rd_party: {}", e);
        }
    }

    // Finish spinner and print summary
    spinner.finish_with_message(format!(
        "parsed {} manifest files, {} lock files",
        manifest_count, lock_count
    ));

    // v1.0.17 (BUG-03): For AUTOSAR projects, upgrade system deps that match epd_versions.
    // BSW modules appear in the Makefile as -lAdc/-lGpt/etc (system ecosystem) but their
    // versions live in the corresponding plugin *.epd files. Convert those to autosar ecosystem
    // with the real version so they display alongside arxml-discovered components.
    if is_autosar {
        for dep in all_dependencies.iter_mut() {
            if dep.ecosystem == "system" {
                if let Some(ver) = epd_versions.get(&dep.name) {
                    dep.version = ver.clone();
                    dep.ecosystem = "autosar".to_string();
                } else if let Some(ver) = doxygen_versions.get(&dep.name) {
                    dep.version = ver.clone();
                    dep.ecosystem = "autosar".to_string();
                }
            }
        }
    }

    // Progress for parsing phase
    let total_deps = all_dependencies.len();
    if total_deps > 0 {
        eprintln!(
            "[2/5] Parsing complete... {} dependencies discovered",
            total_deps
        );
    } else {
        eprintln!("[2/5] Parsing complete... no dependencies found");
    }

    // Note: Deduplication moved to main.rs to happen after optional import scanning
    // This ensures proper priority: LockFile > Manifest > ImportScan

    Ok(ScanContext {
        dependencies: all_dependencies,
        npm_relationships,
        cargo_relationships,
        python_lockfile_relationships,
        ros_metadata,
        ros_packages,
        git_submodule_relationships,
        is_autosar,
        component_dirs,
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use tempfile::TempDir;

    #[test]
    fn test_scan_tolerates_broken_symlink() {
        let dir = TempDir::new().unwrap();
        let broken_link = dir.path().join("broken_link");
        symlink("/nonexistent/target/path", &broken_link).unwrap();

        // scan_directory must return Ok (not Err) even with a broken symlink present
        let result = scan_directory(
            dir.path(),
            &crate::cli::VendorMode::Include,
            &[],
            None,  // ros_distro
            false, // scan_submodules
            1,     // submodule_depth
            false, // scan_c_build_systems
            false, // resolve_transitive
            false, // scan_meson
            false, // scan_bazel
            false, // scan_so_files
            None,  // target_arch
            None,  // scan_ai_models
            false, // is_autosar
        );
        assert!(result.is_ok(), "scan_directory should not abort on broken symlink");
    }
}
