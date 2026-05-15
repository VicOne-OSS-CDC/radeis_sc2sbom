use crate::models::DependencyScope;
use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable console output
    Console,
    /// SPDX 2.3 JSON format
    SpdxJson,
    /// SPDX 2.3 Tag-Value format
    SpdxTagValue,
    /// CycloneDX 1.5 JSON format
    CyclonedxJson,
    /// Generate all formats (Console markdown + SPDX JSON + SPDX Tag-Value + CycloneDX JSON)
    All,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum VendorMode {
    /// Skip vendor/node_modules directories
    Skip,
    /// Include vendor/node_modules directories (default)
    Include,
    /// Scan only vendor/node_modules directories
    Only,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TreeStyle {
    /// Current flat list style
    Flat,
    /// Classic tree with box-drawing characters
    Tree,
    /// Compact tree with arrows
    Compact,
}

#[derive(Parser, Debug)]
#[command(name = "SBOM Scanner")]
#[command(about = "Scans a folder for open source dependencies and generates SBOM", long_about = None)]
#[cfg_attr(
    feature = "cn-release",
    command(after_help = "\
══════════════════════════════════════════════════════════════════════\n\
📊 SBOM vulnerability assessment service\n\
══════════════════════════════════════════════════════════════════════\n\
📄 Example vulnerability assessment report:\n\
   https://robot-scan.s3.cn-northwest-1.amazonaws.com.cn/Vulnerability_Assessment_Report.pdf\n\
 \n\
📧 Want a FREE vulnerability report for your SBOM?\n\
   Contact: allofviconecdcrd@vicone.com\n\
══════════════════════════════════════════════════════════════════════")
)]
pub struct Args {
    /// Path to the folder to scan
    #[arg(short, long)]
    pub path: PathBuf,

    /// Output format
    #[arg(short = 'f', long, value_enum, default_value = "all")]
    pub format: OutputFormat,

    /// How to handle vendor directories (node_modules, vendor, site-packages, etc.)
    #[arg(long, value_enum, default_value = "include")]
    pub vendor: VendorMode,

    /// Exclude specific directory patterns (can be used multiple times)
    #[arg(long)]
    pub exclude: Vec<String>,

    /// Enable import scanning fallback for detecting dependencies from source code
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub fallback_import_scan: bool,

    /// Enable experimental CWE rules (higher false-positive rate). Adds 17 additional
    /// rules to the default 30. (v1.0.18)
    #[cfg(feature = "internal")]
    #[arg(long, action = ArgAction::SetTrue)]
    pub experiment_scan: bool,

    /// Dependency tree visualization style (flat, tree, compact)
    #[arg(long, value_enum, default_value = "tree")]
    pub tree_style: TreeStyle,

    /// Use compact output format with reduced spacing
    #[arg(long, action = ArgAction::SetTrue)]
    pub compact: bool,

    /// Generate summary-only report (no dependency trees, significantly smaller output)
    #[arg(long, action = ArgAction::SetTrue)]
    pub summary_only: bool,

    /// Compact SPDX format: omit transitive dependency markers (CONTAINS NOASSERTION)
    /// This reduces file size significantly (~30%) with minimal information loss (v0.9.0)
    #[arg(long, action = ArgAction::SetTrue)]
    pub compact_spdx: bool,

    /// ROS distribution to use for version resolution (e.g., humble, iron, jazzy)
    /// Falls back to ROS_DISTRO environment variable, then defaults to "jazzy"
    #[arg(long)]
    pub ros_distro: Option<String>,

    /// Enable recursive Git submodule scanning (v1.0.0)
    /// When enabled, scans .gitmodules files and recursively scans dependencies
    /// within submodules
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub scan_submodules: bool,

    /// Maximum depth for nested submodule detection (v1.0.0)
    /// Reserved for future recursive scanning implementation
    #[arg(long, default_value = "3")]
    pub submodule_depth: usize,

    /// Enable C/C++ build system scanning (v1.0.9)
    /// Includes: CMake, pkg-config, Autotools, Makefiles, and .mk version files
    /// When disabled, skips all C/C++ build system detection
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub scan_c_build_systems: bool,

    /// Enable Meson build system parsing (v1.0.4)
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub scan_meson: bool,

    /// Enable Bazel build system parsing (v1.0.4)
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub scan_bazel: bool,


    /// Enable transitive dependency resolution for requirements.txt (v1.0.0)
    /// Uses pip to resolve the full dependency tree including transitive deps.
    /// Requires pip to be installed and network access.
    #[arg(long, action = ArgAction::SetTrue)]
    pub resolve_transitive: bool,

    /// Output directory for generated SBOM files. When specified, single formats
    /// write to a file instead of stdout. Defaults to "out" for --format all.
    #[arg(long)]
    pub output: Option<String>,

    /// Target architecture for resolving conditional .mk file expressions (v1.0.6)
    /// Required when .mk files have arch-conditional version assignments,
    /// e.g., VSOMEIP_VERSION set to different values for qnx_7_0_0_x86_64 vs qnx_8_0_0_aarch64le.
    /// Example: --target-arch qnx_8_0_0_aarch64le
    #[arg(long)]
    pub target_arch: Option<String>,

    /// Scan compiled .so files for version information (v1.0.5)
    /// Extracts version from .so filenames and ELF metadata
    /// Requires libraries to be already built
    #[arg(long, action = ArgAction::Set, default_value_t = false)]
    pub scan_so_files: bool,

    /// Enable AI model file scanning (GGUF) (v1.0.9)
    /// Parses .gguf files to extract model metadata (architecture, quantization, provenance)
    #[arg(long, action = ArgAction::Set, default_value_t = true)]
    pub scan_ai_models: bool,

    /// Path to custom BSW module config (YAML). Overrides bundled default. (v1.0.15)
    #[arg(long)]
    pub bsw_config: Option<PathBuf>,

    /// Maximum file size in GB for SHA-256 hashing of AI model files (v1.0.9)
    /// Files larger than this limit will skip hashing (metadata is still parsed).
    /// Set to 0 for unlimited (hash all files regardless of size).
    /// Default: 0 (unlimited) — ensures integrity verification for all models including 120B+
    #[arg(long, default_value = "0")]
    pub max_hash_size_gb: u64,

    /// Filter dependencies by scope (v1.0.6)
    /// Multiple scopes can be specified. Valid values: runtime, build, test, development, optional, provided
    /// Example: --scope-filter runtime --scope-filter optional
    /// Cannot be combined with --production (use one or the other)
    #[arg(
        long = "scope-filter",
        value_name = "SCOPE",
        conflicts_with = "production"
    )]
    pub scope_filter: Vec<String>,

    /// Production mode: Include only Runtime and Optional dependencies (v1.0.6)
    /// Equivalent to --scope-filter runtime --scope-filter optional
    /// This is a convenience flag for generating production SBOMs
    /// Cannot be combined with --scope-filter (use one or the other)
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "scope_filter")]
    pub production: bool,

    /// YAML file mapping AUTOSAR component names to supplier strings (v1.0.15, Phase 8).
    /// Components not in the file emit NOASSERTION for autosar:supplier.
    /// File must exist and parse as a flat map of String → String; otherwise the binary exits with an error.
    #[arg(long)]
    pub supplier_config: Option<PathBuf>,

    /// SARIF output file path for static analysis findings (v1.0.17)
    /// Defaults to {out_dir}/{project_name}_static_analysis.sarif
    #[cfg(feature = "internal")]
    #[arg(long)]
    pub sarif_output: Option<String>,

    /// SARIF baseline file for new-findings-only CI gate (v1.0.17, Phase 16)
    /// When provided, compares current scan fingerprints against the baseline.
    /// Exits 1 if new findings are found, 0 if none. A missing or invalid baseline
    /// triggers a warning and the scan continues (does NOT abort).
    #[cfg(feature = "internal")]
    #[arg(long)]
    pub sarif_baseline: Option<String>,
}

impl Args {
    /// Parse scope filter strings into DependencyScope enum values.
    /// Returns `Ok(None)` if no filtering is requested.
    /// Returns `Err` if any provided scope value is unrecognised (rather than silently
    /// falling back to no-filter, which would produce a full SBOM without warning).
    pub fn parse_scope_filters(&self) -> Result<Option<Vec<DependencyScope>>, String> {
        if self.production {
            // Production mode: Runtime + Optional
            return Ok(Some(vec![
                DependencyScope::Runtime,
                DependencyScope::Optional,
            ]));
        }

        if self.scope_filter.is_empty() {
            return Ok(None);
        }

        let mut scopes = Vec::new();
        for filter in &self.scope_filter {
            match filter.to_lowercase().as_str() {
                "runtime" => scopes.push(DependencyScope::Runtime),
                "build" => scopes.push(DependencyScope::Build),
                "test" => scopes.push(DependencyScope::Test),
                "development" | "dev" => scopes.push(DependencyScope::Development),
                "optional" => scopes.push(DependencyScope::Optional),
                "provided" => scopes.push(DependencyScope::Provided),
                _ => {
                    return Err(format!(
                        "Unknown scope filter '{}'. Valid values: runtime, build, test, development, optional, provided",
                        filter
                    ));
                }
            }
        }

        Ok(Some(scopes))
    }
}
