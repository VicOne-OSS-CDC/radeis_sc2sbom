mod classifier;
mod cli;
mod formats;
mod models;
mod parsers;
mod scanner;
mod supplier;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::{Args, OutputFormat, VendorMode};
use formats::{
    print_cyclonedx_json, print_sbom, print_spdx_json, print_spdx_tag_value, save_console_report,
    save_cyclonedx_json, save_spdx_json, save_spdx_tag_value,
};
use models::{Dependency, DependencyRelationship, Sbom, ScopeStatistics};
use parsers::{deduplicate_dependencies, scan_go_imports, scan_js_ts_imports, scan_python_imports};
use scanner::{detect_autosar, scan_directory, should_process_entry};
use std::path::Path;
use util::warn_on_walkdir_err;
use walkdir::WalkDir;

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Scanning folder: {:?}", args.path);

    if !args.path.exists() {
        anyhow::bail!("Path does not exist: {:?}", args.path);
    }

    if !args.path.is_dir() {
        anyhow::bail!("Path is not a directory: {:?}", args.path);
    }

    // Phase 6 (v1.0.15): AUTOSAR project detection pre-pass
    let is_autosar = detect_autosar(&args.path, &args.vendor, &args.exclude);

    // Phase 8 (v1.0.15): Optional supplier config for AUTOSAR components.
    // Hard error on missing/unparseable file (per D-02). Plan 02 threads
    // `supplier_resolver.as_ref()` into SPDX/CycloneDX formatters.
    let supplier_resolver: Option<crate::supplier::SupplierResolver> =
        match args.supplier_config.as_deref() {
            Some(path) => Some(crate::supplier::SupplierResolver::load(path)?),
            None => None,
        };
    // Phase 1: Scan manifests and lock files
    let scan_context = scan_directory(
        &args.path,
        &args.vendor,
        &args.exclude,
        args.ros_distro.as_deref(),
        args.scan_submodules,
        args.submodule_depth,
        args.scan_c_build_systems,
        args.resolve_transitive,
        args.scan_meson,              // v1.0.4
        args.scan_bazel,              // v1.0.4
        args.scan_so_files,           // v1.0.5
        args.target_arch.as_deref(),  // v1.0.6
        if args.scan_ai_models { Some(args.max_hash_size_gb) } else { None }, // v1.0.9
        is_autosar,                   // v1.0.15
    )?;
    let mut dependencies = scan_context.dependencies;
    let ros_package = scan_context.ros_metadata;
    let mut ros_packages = scan_context.ros_packages;
    // Phase 2: Optionally scan source files for imports
    if args.fallback_import_scan {
        let import_deps = scan_source_files(&args.path, &args.vendor, &args.exclude)?;

        // Show warning if import scanning found dependencies
        if !import_deps.is_empty() {
            eprintln!(
                "⚠️  Warning: Import scanning detected {} dependencies from source code.",
                import_deps.len()
            );
            eprintln!("   These dependencies show version='detected' and may include:");
            eprintln!("   - Unused conditional imports");
            eprintln!("   - Dev dependencies marked as production");
            eprintln!("   For accurate SBOMs, ensure manifest files are complete.");
            eprintln!();
        }

        dependencies.extend(import_deps);
    }

    // Deduplicate with priority: LockFile > Manifest > ImportScan
    let initial_count = dependencies.len();
    let mut dependencies = deduplicate_dependencies(dependencies);
    let final_count = dependencies.len();

    // Print deduplication progress
    if initial_count != final_count {
        eprintln!(
            "[3/5] Deduplicating dependencies... {} → {} unique dependencies",
            initial_count, final_count
        );
    } else {
        eprintln!(
            "[3/5] Deduplicating dependencies... {} unique dependencies",
            final_count
        );
    }

    // v1.0.6: Classify dependencies by scope
    eprintln!("[3.5/5] Classifying dependencies by scope...");
    dependencies = classifier::classify_dependencies(dependencies);
    // v1.0.6: Refine BUILD-CONFIG packages using link analysis (Step 4)
    classifier::refine_build_config_classification(&mut dependencies);
    eprintln!("[3.5/5] Classifying dependencies by scope... done");

    // v1.0.15 (Phase 7): AUTOSAR BSW classification — gated on is_autosar
    // (Pitfall 3 from 07-RESEARCH.md). Loads bundled BSW config or
    // user-supplied --bsw-config override.
    if is_autosar {
        use classifier::autosar::{classify_autosar_components, BswConfig};
        let bsw_config = match args.bsw_config.as_ref() {
            Some(path) => BswConfig::load_from_file(path)?,
            None => BswConfig::load_bundled(),
        };
        classify_autosar_components(&mut dependencies, &bsw_config);
        eprintln!("[3.6/5] Classified AUTOSAR BSW components");
    }

    // v1.0.6: Apply scope filtering if requested
    if let Some(scope_filters) = args.parse_scope_filters().map_err(|e| anyhow::anyhow!(e))? {
        let before_filter = dependencies.len();
        dependencies.retain(|dep| scope_filters.contains(&dep.scope));
        let after_filter = dependencies.len();

        let scope_names: Vec<String> = scope_filters.iter().map(|s| format!("{:?}", s)).collect();

        eprintln!(
            "[3.7/5] Filtered by scope [{}]... {} → {} dependencies",
            scope_names.join(", "),
            before_filter,
            after_filter
        );
    }

    // v0.9.1: Resolve ROS dependency versions using rosdistro database
    // This happens AFTER deduplication to ensure all final ROS packages are resolved
    let has_ros_deps = dependencies
        .iter()
        .any(|d| d.ecosystem.eq_ignore_ascii_case("ros"));
    if has_ros_deps || !ros_packages.is_empty() {
        eprintln!("[4/5] Resolving ROS package versions...");
        crate::parsers::ros::resolve_ros_dependency_versions(
            &mut dependencies,
            args.ros_distro.as_deref(),
        );

        // v0.9.1: Also resolve versions in ros_packages structure (used by SPDX serialization)
        for ros_pkg in &mut ros_packages {
            crate::parsers::ros::resolve_ros_dependency_versions(
                &mut ros_pkg.dependencies,
                args.ros_distro.as_deref(),
            );
        }
        eprintln!("[4/5] Resolving ROS package versions... done");
    } else {
        eprintln!("[4/5] Skipping ROS version resolution (no ROS packages detected)");
    }

    eprintln!("[5/5] Scan complete");

    // v1.0.6: Compute scope statistics
    let scope_statistics = Some(ScopeStatistics::from_dependencies(&dependencies));

    let sbom = Sbom {
        project_path: args.path.clone(),
        generated_at: chrono::Utc::now().to_rfc3339(),
        dependencies,
        ros_package,
        ros_packages,
        scope_statistics,
    };

    // Combine all relationships from different ecosystems
    let all_relationships: Vec<DependencyRelationship> = [
        scan_context.npm_relationships,
        scan_context.cargo_relationships,
        scan_context.python_lockfile_relationships,
        scan_context.git_submodule_relationships,
    ]
    .concat();

    match args.format {
        OutputFormat::Console => {
            if let Some(ref out) = args.output {
                let project_name = sbom.project_path.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("sbom");
                let out_dir = Path::new(out);
                std::fs::create_dir_all(out_dir)?;
                let out_path = out_dir.join(format!("{}_report.md", project_name));
                let out_path_str = out_path.to_string_lossy();
                save_console_report(
                    &sbom,
                    &out_path_str,
                    &args.tree_style,
                    &all_relationships,
                    args.summary_only,
                )?;
                eprintln!("✓ Console report saved to: {}", out_path.display());
                #[cfg(feature = "internal")]
                save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;

            } else {
                print_sbom(
                    &sbom,
                    &args.tree_style,
                    args.compact,
                    &all_relationships,
                );
            }
        }
        OutputFormat::SpdxJson => {
            if let Some(ref out) = args.output {
                let project_name = sbom.project_path.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("sbom");
                let out_dir = Path::new(out);
                std::fs::create_dir_all(out_dir)?;
                let out_path = out_dir.join(format!("{}_spdx.json", project_name));
                let out_path_str = out_path.to_string_lossy();
                save_spdx_json(&sbom, &out_path_str, args.compact_spdx, supplier_resolver.as_ref())?;
                eprintln!("✓ SPDX JSON saved to: {}", out_path.display());
            } else {
                print_spdx_json(&sbom, args.compact_spdx, supplier_resolver.as_ref())?;
            }
        }
        OutputFormat::SpdxTagValue => {
            if let Some(ref out) = args.output {
                let project_name = sbom.project_path.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("sbom");
                let out_dir = Path::new(out);
                std::fs::create_dir_all(out_dir)?;
                let out_path = out_dir.join(format!("{}_spdx.spdx", project_name));
                let out_path_str = out_path.to_string_lossy();
                save_spdx_tag_value(&sbom, &out_path_str, args.compact_spdx, supplier_resolver.as_ref())?;
                eprintln!("✓ SPDX Tag-Value saved to: {}", out_path.display());
            } else {
                print_spdx_tag_value(&sbom, args.compact_spdx, supplier_resolver.as_ref());
            }
        }
        OutputFormat::CyclonedxJson => {
            if let Some(ref out) = args.output {
                let project_name = sbom.project_path.file_name()
                    .and_then(|n| n.to_str()).unwrap_or("sbom");
                let out_dir = Path::new(out);
                std::fs::create_dir_all(out_dir)?;
                let out_path = out_dir.join(format!("{}_cyclonedx.json", project_name));
                let out_path_str = out_path.to_string_lossy();
                save_cyclonedx_json(&sbom, &out_path_str, supplier_resolver.as_ref())?;
                eprintln!("✓ CycloneDX JSON saved to: {}", out_path.display());
            } else {
                print_cyclonedx_json(&sbom, supplier_resolver.as_ref())?;
            }
        }
        OutputFormat::All => {
            // Generate all formats and save to files in ./out directory
            let project_name = sbom
                .project_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("sbom");

            // Create output directory
            let out_dir_str = args.output.as_deref().unwrap_or("out");
            let out_dir = Path::new(out_dir_str);
            std::fs::create_dir_all(out_dir)?;

            // 1. Console report (markdown) to file
            let console_path = out_dir.join(format!("{}_report.md", project_name));
            let console_path_str = console_path.to_string_lossy();
            save_console_report(
                &sbom,
                &console_path_str,
                &args.tree_style,
                &all_relationships,
                args.summary_only,
            )?;
            eprintln!("✓ Console report saved to: {}", console_path.display());
            #[cfg(feature = "internal")]
            save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;


            // 2. SPDX JSON
            let spdx_json_path = out_dir.join(format!("{}_spdx.json", project_name));
            let spdx_json_path_str = spdx_json_path.to_string_lossy();
            save_spdx_json(
                &sbom,
                &spdx_json_path_str,
                args.compact_spdx,
                supplier_resolver.as_ref(),
            )?;
            eprintln!("✓ SPDX JSON saved to: {}", spdx_json_path.display());

            // 3. SPDX Tag-Value
            let spdx_tag_path = out_dir.join(format!("{}_spdx.spdx", project_name));
            let spdx_tag_path_str = spdx_tag_path.to_string_lossy();
            save_spdx_tag_value(
                &sbom,
                &spdx_tag_path_str,
                args.compact_spdx,
                supplier_resolver.as_ref(),
            )?;
            eprintln!("✓ SPDX Tag-Value saved to: {}", spdx_tag_path.display());

            // 4. CycloneDX JSON
            let cdx_json_path = out_dir.join(format!("{}_cyclonedx.json", project_name));
            let cdx_json_path_str = cdx_json_path.to_string_lossy();
            save_cyclonedx_json(&sbom, &cdx_json_path_str, supplier_resolver.as_ref())?;
            eprintln!("✓ CycloneDX JSON saved to: {}", cdx_json_path.display());

            eprintln!(
                "\n✓ All SBOM formats generated successfully in {} directory!",
                out_dir_str
            );
        }
    }

    Ok(())
}

/// Parse ROS/ROS2 package.xml and optionally adjacent setup.py
/// Returns (package_metadata, dependencies)
/// Normalize Python package name according to PEP-503
/// Converts to lowercase and replaces runs of [-_.] with a single underscore
/// Parse Python setup.py for ROS package dependencies
/// Returns (package_version, dependencies)
/// Helper to parse Python list of dependencies like ['pkg1', 'pkg2>=1.0']
// Lock File Parsers

/// Parses Cargo.lock with relationship data for hierarchical dependency trees.
/// Extracts parent-child relationships from the dependencies array in each package entry.
///
/// # Returns
/// LockFileData containing:
/// - dependencies: All packages from Cargo.lock
/// - relationships: Parent→child mappings from dependencies arrays
/// Parses poetry.lock with relationship data for hierarchical dependency trees.
/// Extracts parent-child relationships from the [package.dependencies] table in each package entry.
///
/// # Returns
/// LockFileData containing:
/// - dependencies: All packages from poetry.lock
/// - relationships: Parent→child mappings from dependencies tables
// ========== Import Scanning Helper Functions ==========

/// Check if a module is part of Python's standard library
/// Check if a module is a Node.js built-in module
/// Check if an import path is from Go standard library
/// Extract root package name from Python import path
/// Example: "django.core.utils" -> "django"
/// Extract package name from JavaScript/TypeScript import path
/// Handles scoped packages: "@babel/core/lib" -> "@babel/core"
/// Handles regular packages: "express/router" -> "express"
// ========== Import Scanner Functions ==========

/// Scan a Python file for import statements and extract dependencies
/// Scan a JavaScript/TypeScript file for import/require statements and extract dependencies
/// Scan a Go file for import statements and extract dependencies
/// Orchestrator function to scan all source files for imports
fn scan_source_files(
    path: &Path,
    vendor_mode: &VendorMode,
    excludes: &[String],
) -> Result<Vec<Dependency>> {
    let mut all_dependencies = Vec::new();

    for entry in WalkDir::new(path)
        .follow_links(true)
        .max_depth(50)
        .into_iter()
        .filter_entry(|e| should_process_entry(e, vendor_mode, excludes))
        .filter_map(warn_on_walkdir_err)
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let extension = path.extension().and_then(|s| s.to_str()).unwrap_or("");

        let file_deps = match extension {
            // Python files
            "py" => scan_python_imports(path)?,

            // JavaScript/TypeScript files
            "js" | "jsx" | "ts" | "tsx" | "mjs" | "cjs" => scan_js_ts_imports(path)?,

            // Go files
            "go" => scan_go_imports(path)?,

            // Skip other file types
            _ => continue,
        };

        all_dependencies.extend(file_deps);
    }

    // Deduplicate across all files
    let mut map: std::collections::HashMap<(String, String), Dependency> =
        std::collections::HashMap::new();

    for dep in all_dependencies {
        let key = (dep.name.clone(), dep.ecosystem.clone());
        map.entry(key).or_insert(dep);
    }

    Ok(map.into_values().collect())
}
