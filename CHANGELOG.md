# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.18] - 2026-05-13

### Added (Internal)
- **`--experiment-scan` flag**: Gates 17 high-FP experimental CWEs behind an opt-in flag; default scan runs 22 high-confidence CWEs at 22.0% FP; experimental scan runs 127,800 findings at 82.2% FP (Phase 25, `#[cfg(feature = "internal")]`)

### Changed

### Removed
- **cppcheck integration and `--cppcheck-path` flag**: Replaced entirely by the AST scanner (Phase 19)

## [1.0.17] - 2026-05-11

### Added (Internal)
- **`--cppcheck-path <PATH>`**: Override PATH lookup for cppcheck binary
- **SARIF 2.1 output**: `<project>_static_analysis.sarif` written alongside `_static_analysis.md`; compatible with GitHub Code Scanning, VS Code Problem Matcher, and CI/CD pipelines
- **`--sarif-output <PATH>`**: Write SARIF report to a custom path instead of the default alongside the markdown report
- **SARIF fingerprinting**: SHA-256 fingerprints per `SarifResult` for stable deduplication across runs
- **`--sarif-baseline <PATH>`**: Diff current findings against a prior SARIF run; only new findings (not present in baseline) are reported — enables CI gates that surface only regressions
- **AUTOSAR arxml parsing**: `.arxml` files are now fully parsed for dependencies — `SW-COMPONENT-PROTOTYPE`, `BSW-MODULE-DESCRIPTION`, `APPLICATION-SW-COMPONENT-TYPE`, `ECU-ABSTRACTION-SW-COMPONENT-TYPE`, `SERVICE-SW-COMPONENT-TYPE`, `COMPOSITION-SW-COMPONENT-TYPE`, and `COMPLEX-DEVICE-DRIVER-SW-COMPONENT-TYPE` elements are extracted as `autosar`-ecosystem dependencies
- **AUTOSAR version extraction**: `.epd` files (`ECUC-MODULE-DEF/REVISION-LABEL`) and Doxygen C/H headers (`SW Version : X.Y.Z`) are scanned to populate real version strings on AUTOSAR deps instead of `unspecified`
- **AUTOSAR ecosystem promotion**: `system`-ecosystem deps (Makefile `-lFoo` flags) are upgraded to `autosar` ecosystem when a matching `.epd` or Doxygen version is found — eliminates duplicate entries and misclassified linker deps

### Fixed
- AUTOSAR projects with C/C++ files at depth > 3 were not scanned for SAST findings — `has_c_cpp_files` max depth raised from 3 to 6
- AUTOSAR deps with versioned and `unspecified` entries for the same `(name, ecosystem)` now deduplicate — versioned entry wins
- `system`-ecosystem entries are suppressed when an `autosar`-ecosystem entry exists for the same name

## [1.0.16] - 2026-05-10

### Added
- **Fallback Mode**: When no manifest-derived component directories are found but C/C++ source files exist under the scan root, a synthetic `(project_name, "C/C++") → scan_root` entry is inserted — enables scanning of manifest-free repos (e.g. NIST Juliet test suite)
- **`has_c_cpp_files` helper**: Shallow `WalkDir` (max depth 3) check reusing `is_c_cpp_source` predicate; used by fallback mode to avoid false positives on non-C repos
- **`resolve_component_dir` helper**: Maps manifest-declared C/C++ dependencies to vendored source subdirectories via three strategies (exact name, `lib`-prefix, case-insensitive scan); returns `None` for deps with no matching vendored subdir, preventing inflated findings from external/system dependencies

### Internal
- `build-all.sh` extended with `--internal` flag: builds public + internal variants and appends `-internal` suffix to internal binary names

## [1.0.15] - 2026-05-09

### Added
- **AUTOSAR Detection**: Pre-pass `detect_autosar()` runs before scan; detects projects via `.arxml` files (DET-01), BSW/MCAL/RTE/AUTOSAR/SWC directory names (DET-02), and `AUTOSAR_VERSION`/`AR_VERSION` tokens in build files (DET-03)
- **AUTOSAR Classification**: `classify_autosar_components()` matches dependency names against a bundled BSW module config; upgrades matching components to `ecosystem="autosar"` with `AutosarMetadata` (module_name, layer, platform)
- **AUTOSAR Output — CycloneDX**: AUTOSAR components emit `autosar:layer` and `autosar:platform` properties (e.g. `"BSW-Memory"`, `"Classic"`)
- **AUTOSAR Output — SPDX**: AUTOSAR components emit `autosar:layer` and `autosar:platform` as `ExternalRef OTHER` entries
- **Supplier Config**: `--supplier-config <path>` accepts a YAML file mapping AUTOSAR component names to supplier strings; mapped components emit `autosar:supplier` in CycloneDX properties and SPDX ExternalRef; unmapped components emit `NOASSERTION`
- **BSW Config Override**: `--bsw-config <path>` overrides the bundled AUTOSAR BSW module config with a custom YAML file

### Changed
- `--output` now works for all single formats (`spdx-json`, `spdx-tag-value`, `cyclonedx-json`, `console`) in addition to `--format all`; when `--output` is omitted for single formats, output goes to stdout (previous default behaviour preserved)

### Internal

## [1.0.14] - 2026-04-24

### Fixed
- Scanner no longer aborts on broken symlinks — emits `Warning: skipping` and continues (all 5 WalkDir sites: scanner, fallback import scan, C parsers)
- Makefile variable references like `$(OPENSSL_VERSION)` no longer leak into SPDX `versionInfo` — filtered at parser + every SPDX/CycloneDX version-output site, emits `NOASSERTION` instead
- C/C++ library licenses now resolve from `.pc` `License:` fields and a known-library lookup table (24 common system libs) instead of all being `NOASSERTION`

### Changed
- Linux release binary now statically linked via musl (`x86_64-unknown-linux-musl`) — eliminates glibc version dependency; runs on Ubuntu 22.04+, 24.04, Alpine, and any x86_64 Linux
- Added musl cross-linker toolchain guard to `build-all.sh`

### Internal
- Extracted `warn_on_walkdir_err` helper to `src/util/mod.rs`, deduplicating 5 identical filter_map closures
- Cross-platform tests now `#[cfg(unix)]`-gated where they use Unix-only APIs

## [1.0.13] - 2026-04-14

### Added
- **AI Models**: Multimodal sub-model decomposition — composite models (Gemma-4, LLaVA, Qwen-VL) are broken down into text, vision, and audio sub-model components
- **AI Models**: New `SubModelInfo` struct capturing per-sub-model architecture: model_type, layers, hidden_size, heads, dtype, vocab_size, context window, and modality-specific fields (patch_size, conv_kernel_size, etc.)
- **AI Models**: Guard condition — sub-models only emitted for genuinely multimodal models (text_config + vision/audio_config present)
- **CycloneDX**: Nested `components` array inside parent AI model component for each sub-model with `radeis:ai:sub_model:*` properties
- **SPDX**: Child packages with `CONTAINS` relationships from parent model to each sub-model
- **Console**: Sub-model summary table showing modality, model type, layers, hidden size, heads, dtype, and modality-specific extras
- **Tests**: 5 new tests (4 safetensors + 1 GGUF) covering multimodal extraction, text-only guard, vision-text-only, and GGUF enrichment

## [1.0.12] - 2026-04-14

### Added
- **AI Models**: Rich metadata extraction from HuggingFace companion files for both Safetensors and GGUF repos
- **AI Models**: Parse `generation_config.json` — temperature, top_k, top_p inference defaults
- **AI Models**: Parse `tokenizer_config.json` — processor_class, model_max_length (with astronomical value cap)
- **AI Models**: Parse `preprocessor_config.json` — image, audio, and video processor types and parameters
- **AI Models**: Parse `README.md` YAML frontmatter — base_model (string or list), license, pipeline_tag, quantized_by, prompt_template, tags, languages, datasets
- **AI Models**: Extended `config.json` extraction — model_type, text_config (hidden_layers, hidden_size, attention_heads, max_position_embeddings), multimodal detection (vision_config, audio_config)
- **AI Models**: Dtype fallback chain — `torch_dtype` > `dtype` > `text_config.dtype`
- **AI Models**: LoRA/QLoRA adapter detection via `adapter_config.json`
- **AI Models**: GGUF companion file enrichment — binary metadata always wins, companion files fill gaps
- **AI Models**: Deduplicated union merge for tags, languages, and datasets from binary + README sources
- **AI Models**: 1 MB safety cap on all companion file reads
- **AI Models**: Case-insensitive README.md filename matching and CRLF line ending support
- **CycloneDX**: ~25 new `radeis:ai:*` properties for rich AI model metadata
- **SPDX**: Extended sourceInfo with model_type, context window, and modality summary
- **Console**: Extended AI Model Details table with architecture, multimodal, generation, and provenance sections
- **Tests**: 8 new tests in `safetensors_tests.rs`, 6 new tests in `gguf_tests.rs`

## [1.0.11] - 2026-04-13

### Added
- **AI Models**: Safetensors AI model SBOM parsing — supports `.safetensors`, `model.safetensors.index.json`, and `config.json`
- **AI Models**: Directory-level scanning — one Dependency entry per model regardless of shard count
- **AI Models**: `pkg:huggingface` PURL for HuggingFace Safetensors models
- **AI Models**: CycloneDX `machine-learning-model` component type with `modelCard` for Safetensors models
- **AI Models**: Shard deduplication — multi-shard models (e.g., `model-00001-of-00002.safetensors`) are consolidated into a single SBOM entry
- **AI Models**: New `AIModelMetadata` fields: `safetensors_format`, `total_size_bytes`, `shard_count`, `torch_dtype`, `transformers_version`, `vocab_size`
- **Tests**: 12 new tests in `tests/parser_tests/safetensors_tests.rs`

## [1.0.10] - 2026-04-13

### Added
- **Java/Gradle**: Full dependency parsing for `build.gradle` (Groovy DSL) and `build.gradle.kts` (Kotlin DSL) — previously detection-only
- **Java/Gradle**: String notation (`'group:artifact:version'`), map notation (`group: 'g', name: 'a', version: 'v'`), and platform/BOM support
- **Java/Gradle**: Scope classification — `testImplementation` → Test, `compileOnly` → Provided, `annotationProcessor`/`kapt`/`ksp`/`classpath` → Build
- **Java/Gradle**: Android project support (`androidTestImplementation`, `androidTestCompile`)

### Changed
- **Java**: Gradle status upgraded from "Detection-only" to "Production-ready" in ecosystem table

## [1.0.9] - 2026-04-10

### Added
- **AI Models**: GGUF binary parser with metadata extraction (architecture, quantization type, tensor info, context length)
- **AI Models**: CycloneDX `machine-learning-model` component type with `modelCard` (training parameters, datasets)
- **AI Models**: SPDX `pkg:huggingface` PURL for AI model dependencies
- **AI Models**: Integrity verification — tensor parameter cross-validation and SHA-256 hashing for model file authenticity
- **AI Models**: License normalization to SPDX format for common AI model licenses
- **CLI**: `--scan-ai-models` flag to enable GGUF model scanning (default: true)
- **CLI**: `--max-hash-size-gb` flag to control SHA-256 hashing limit for large models (default: 0 = unlimited)

### Changed
- **CLI**: Merged 5 C/C++ build system flags (`--scan-cmake`, `--scan-pkgconfig`, `--scan-autotools`, `--scan-makefiles`, `--scan-mk-files`) into single `--scan-c-build-systems` flag
- **CLI**: Merged `--meson-parse-subprojects` into `--scan-meson` (always on when Meson scanning is enabled)
- **Core**: `scan_directory()` simplified from 20 arguments to 13 (merged `scan_ai_models` + `max_hash_size_gb` into `Option<u64>`)

### Removed
- **CLI**: `--resolve-system-deps` flag (dead code — was never wired to any implementation)
- **CLI**: `--meson-parse-subprojects` flag (subproject parsing now always enabled when `--scan-meson` is active)

## [1.0.8] - 2026-04-02

### Fixed
- **SPDX**: `downloadLocation` now returns `NOASSERTION` when package version is `"detected"` (import-scan) — previously emitted fake URLs like `https://pypi.org/project/utime/detected/`
- **SPDX/CycloneDX**: PURLs no longer include `@detected`, `@unspecified`, or `@unknown` version components — these sentinel values are now omitted per PURL spec
- **CycloneDX**: Component `version` field is now omitted (rather than set to sentinel string) when version is `"detected"`, `"unspecified"`, or `"unknown"`
- **SPDX**: `DependencyScope::Provided` now maps to `primaryPackagePurpose: LIBRARY` instead of `SOURCE` — corrects classification of runtime link libraries (`-ldl`, `-lm`, `-lpthread`)
- **CycloneDX**: `metadata.tools` updated to non-deprecated CycloneDX 1.5 format (`{components: [{type, name, version}]}` instead of flat array)
- **SPDX**: Removed `CONTAINS NOASSERTION` relationships — these caused validation warnings in `pyspdxtools` without conveying useful information
- **SPDX**: Root package `versionInfo` now set to `NOASSERTION` instead of `"0"` per SPDX 2.3 §3.5

- **CycloneDX**: Duplicate components in ROS scans — a shared library was emitted once per ROS package that depended on it; now deduplicated per CycloneDX 1.5 spec
- **CycloneDX**: Root `metadata.component` bom-ref was missing from the `dependencies` array — the graph was disconnected; root is now included with edges to all top-level components
- **CycloneDX**: Shared ROS dep `is_dev` flag now updated when any consumer marks it non-dev — component no longer incorrectly marked dev-only when one consumer treats it as non-dev
- **SPDX**: Illegal characters in SPDXID — ecosystem values like `"npm (dev)"` were embedded raw, producing invalid IDs; illegal characters now replaced with `-`
- **SPDX**: Duplicate packages in ROS scans — mirrors the CycloneDX fix; shared deps now appear once with `DEPENDS_ON` relationships from each ROS package
- **SPDX**: Double-prefixed originator — `"Person: Person: John Doe"` no longer emitted when author already contains a `Person:`/`Organization:` prefix
- **SPDX**: `PackageVersion: NOASSERTION` in tag-value format causes parse errors (`pyspdxtools`); field is now omitted when version is unknown
- **SPDX/CycloneDX**: Tool name corrected from `sourcecode_to_sbom` to `radeis_sc2sbom` in `metadata.tools` (CycloneDX) and `CreatedBy` (SPDX)
- **Parser**: `find_repo_root` path comparison now canonicalizes both paths before comparing — previously a relative `scan_root` could fail the boundary check and allow escaping the scan directory

### Added
- **Parser**: MicroPython import detection — `.py` files containing MicroPython marker imports (`lvgl`, `utime`, `ustruct`, `SDL`, etc.) now emit `ecosystem: "micropython"` instead of `"pip"`, preventing false PyPI SBOM entries
- **Parser**: `library.json` parser for vendored C/C++ libraries — PlatformIO/LVGL format vendor manifests (e.g., `lv_drivers/library.json`) are now parsed and emitted as `ecosystem: "vendored"` dependencies
- **Scanner**: System library deduplication — when both a Makefile `-lFoo` (system) entry and a `foo.pc` (pkg-config) entry exist for the same library, the system entry is dropped in favor of the versioned pkg-config entry

## [1.0.7] - 2026-03-12



## [1.0.6] - 2026-03-04

### Added - Production SBOM Filtering with Automated Dependency Scope Classification

**Phase 1-3 Complete: Core Classification System**
- **Automated scope classification** for all dependencies (v1.0.6 Phases 1-3)
  - 6 scope types: Runtime, Build, Test, Development, Optional, Provided
  - Multi-strategy classification: ecosystem, name patterns, directory analysis
  - Confidence scores (0.0-1.0) with detailed reasoning
  - Support for 10+ ecosystems (npm, PIP, cargo, SYSTEM, BUILD-CONFIG, etc.)

**Phase 4 Complete: Comprehensive Testing & Validation**
- **42 new integration tests** validating scope filtering and classification accuracy
  - 14 scope filtering integration tests
  - 11 end-to-end production mode tests
  - 17 real-world classification validation tests
- **609 total tests passing** (203 lib + 203 bin + 200 integration + 3 doc)
- **Validated with real-world dependencies**:
  - Common C/C++ runtime libraries (zlib, curl, openssl, protobuf)
  - Build tools (cmake, gcc, ninja, meson)
  - Test frameworks (pytest, jest, gtest, unity)
  - Development tools (pylint, black, eslint, prettier)
  - Web frameworks (django, flask, express, react)

**Phase 5 Complete: Documentation**
- **Comprehensive documentation** for v1.0.6 features
  - Updated [README.md](README.md) with production mode examples
  - New [SCOPE_CLASSIFICATION.md](docs/SCOPE_CLASSIFICATION.md) - Complete scope filtering guide (300+ lines)
  - Updated [CLI.md](docs/CLI.md) with scope filtering options
  - CHANGELOG.md with Phase 5 completion notes
- **Documentation coverage**:
  - Quick start examples
  - Dependency scope explanations (6 types)
  - Classification methods (4 strategies)
  - Filtering options and examples
  - Confidence score interpretation
  - Troubleshooting guide
  - Best practices for production, security, compliance
  - Real-world validation metrics

**Scope Filtering CLI:**
- **`--scope-filter <SCOPE>`** - Filter dependencies by scope
  - Multiple values supported: runtime, build, test, development, optional, provided
  - Example: `--scope-filter runtime --scope-filter optional`
- **`--production`** - Production mode (runtime + optional only)
  - Significantly reduces SBOM size (e.g., 67→11 packages for typical projects)
  - Equivalent to `--scope-filter runtime --scope-filter optional`

**Enhanced Output & Reporting:**
- **Scope statistics** in console and markdown reports
  - Per-scope counts with percentages
  - Average classification confidence
  - Total dependency count
- **Builder pattern** for Dependency creation
  - `Dependency::new().with_scope(scope, confidence, reason)`
  - Cleaner test code and better API ergonomics

**Classification Features:**
- **Ecosystem-aware classification**:
  - SYSTEM libraries → Runtime (high confidence 0.8+)
  - BUILD-CONFIG → Build (pending link analysis refinement)
  - GIT-SUBMODULE → Provided
  - MESON-WRAP/SUBPROJECT → Build
  - PIP/npm/cargo → Context-dependent
- **Name-based heuristics**:
  - Exact matches for well-known tools (confidence 1.0)
  - Case-insensitive matching
  - Pattern-based classification
- **Comprehensive reasoning**: All classifications include detailed explanation

**Test Coverage by Category:**
- **Scope filtering** (14 tests):
  - Default behavior (no filtering)
  - Production mode filtering
  - Custom single/multiple scope filters
  - Edge cases (invalid filters, empty results)
- **End-to-end workflows** (11 tests):
  - Full classification pipeline
  - Production SBOM generation
  - CycloneDX output integration
  - SBOM size reduction validation
  - Ecosystem diversity verification
- **Real-world classification** (17 tests):
  - Common library detection accuracy
  - Build tool identification
  - Test framework recognition
  - Development tool classification
  - Confidence score distribution
  - Classification reasoning validation

### Changed
- **Dependency struct** extended with scope fields
  - `scope: DependencyScope` - Classification result
  - `scope_confidence: f32` - Confidence score (0.0-1.0)
  - `scope_reason: String` - Human-readable explanation
- **Main pipeline** now includes automatic classification (step 3.5/5)
- **SBOM struct** includes `scope_statistics: Option<ScopeStatistics>`
- **Console output** shows scope breakdown in summary

### Phase 4 Validation Results
- **Test Success Rate**: 100% (609/609 tests passing)
- **Classification Accuracy**:
  - Build tools: 100% accuracy (cmake, gcc, ninja, meson)
  - Test frameworks: 100% accuracy (pytest, jest, junit, mocha)
  - Dev tools: 100% accuracy (pylint, black, eslint, prettier)
  - Runtime libraries: High accuracy for SYSTEM ecosystem
  - BUILD-CONFIG: Defaults to Build (requires link analysis for refinement)
- **Confidence Score Distribution**:
  - Exact name matches: 0.95-1.0 (pytest, cmake, etc.)
  - Ecosystem-based: 0.7-0.9 (SYSTEM, GIT-SUBMODULE)
  - Heuristic-based: 0.5-0.8 (fallbacks)
- **Ecosystem Coverage**: 10+ ecosystems validated
- **Production SBOM Size Reduction**: 50-80% typical (e.g., 67→11 packages)

## [1.0.5] - 2026-03-02

### Added - GitHub Actions & CI/CD Infrastructure

**Multi-Platform Build System:**
- **GitHub Actions workflow** for automated cross-platform builds and releases
  - macOS (ARM64 and Intel x86_64) via osxcross cross-compilation
  - Linux (x86_64 glibc)
  - Windows (x86_64) via MinGW cross-compilation
  - Parallel builds with caching for faster CI/CD pipelines
  - Automated release creation with binary assets and checksums

**Release Automation:**
- **Automatic release notes** extraction from CHANGELOG.md
- **Version consistency validation** between git tags and Cargo.toml
- **SHA256 checksums** generation for all release binaries
- **PDF generation** for binary distribution guide using xnexus-md2pdf-tool
- VicOne-styled README.pdf with cover page and professional formatting

**Build Improvements:**
- Self-hosted runner support with enterprise GitHub integration
- GIT_TOKEN authentication for private submodule access
- Artifact retention (90 days) for build traceability
- Platform-specific build filtering (build only needed platforms)
- Configurable release/pre-release flags

### Changed - Code Quality & Dead Code Removal

**API Simplification (~155 lines removed):**
- **Removed `_with_mode` suffix** from all format functions:
  - `print_spdx_json_with_mode` → `print_spdx_json`
  - `print_spdx_tag_value_with_mode` → `print_spdx_tag_value`
  - `save_spdx_json_with_mode` → `save_spdx_json`
  - `save_spdx_tag_value_with_mode` → `save_spdx_tag_value`
  - `convert_to_spdx_with_mode` → `convert_to_spdx`
  - `convert_to_cyclonedx_with_mode` → `convert_to_cyclonedx`
  - `print_cyclonedx_json_with_mode` → `print_cyclonedx_json`
  - `save_cyclonedx_json_with_mode` → `save_cyclonedx_json`

**Compiler Warnings Eliminated:**
- Removed unused wrapper functions from format modules
- Cleaned up unused imports across all parser modules
- Removed dead code from parser functions
- Fixed test references to use new simplified API

**Documentation:**
- Updated BINARY_README.md with complete MIT License text (VicOne Inc. copyright)
- Removed internal distribution sections for customer-facing documentation
- Updated copyright holder from "William Chang" to "VicOne Inc." in LICENSE

### Migration Notes

**For library consumers only** (binary users unaffected):

The format API has been simplified with cleaner function names:

```rust
// OLD API (before v1.0.5)
save_spdx_json_with_mode(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx_with_mode(sbom, &SbomMode::Complete)

// NEW API (v1.0.5+)
save_spdx_json(sbom, path, &SbomMode::Complete, false)
convert_to_cyclonedx(sbom, &SbomMode::Complete)
```

### Infrastructure

**Release Assets:**
- Pre-built binaries for macOS (ARM64/Intel), Linux (x86_64 glibc), Windows (x86_64)
- README.md (markdown binary distribution guide)
- README.pdf (VicOne-styled PDF guide) - NEW!
- checksums.txt (SHA256 verification)

All binaries built via GitHub Actions with automated testing and verification.

## [1.0.4] - 2026-02-25

### Added - Meson & Bazel Build Systems

**Modern C/C++ Build System Support:**
- **Meson build system parser** - Modern meta-build system support
  - Parses `meson.build` files with dependency() and subproject() declarations
  - Supports wrap files (`*.wrap`) for external dependency management
  - Handles version constraints and build options
  - Integration with Conan locks (detected meson 1.2.2 in conanfile.lock)
  - ~2.5% additional C/C++ project coverage

- **Bazel build system parser** - Google's build system support
  - Parses `BUILD`, `BUILD.bazel`, and `WORKSPACE` files
  - Supports http_archive, git_repository, and maven_jar rules
  - Extracts versions from URLs and commit SHAs
  - Handles external repository references (@repo//:target)
  - ~2.5% additional C/C++ project coverage

**CLI Enhancements:**
- `--scan-meson` flag to enable/disable Meson scanning (default: true)
- `--scan-bazel` flag to enable/disable Bazel scanning (default: true)
- Comprehensive help text for all C/C++ ecosystem flags

### Added - Comprehensive Comparison Reports

**Competitive Analysis:**
- **6 comprehensive comparison reports** totaling 2,897 lines
  - OpenStudio, UD Trucks Production, VDL Bus Production
  - ROS 2 Humble Desktop, scikit-learn, Python test fixtures
- **2,561 total dependencies** tracked across all projects
- **2.1%-58.8% more packages** detected than competitors
- **4 unique capabilities** not found in commercial tools:
  - Autotools/pkg-config support (legacy C projects)
  - ROS 2 package.xml parsing
  - Git submodule recursive scanning
  - CMake FetchContent/ExternalProject

**Cost Savings Analysis:**
- **$220K-$1.65M savings** vs BlackDuck licensing
- Per-scan cost: $0 (radeis_sc2sbom) vs $50-$200 (SaaS competitors)
- Total comparison documentation: 7 reports + index

### Changed

**Coverage Impact:**
- Before v1.0.4: ~90% of C/C++ projects
- After v1.0.4: **~95% of C/C++ projects**

**Bug Fixes:**
- Fixed Bazel parser parenthesis matching in BUILD file expressions
- Corrected system package purl type to `pkg:generic/` format
- Updated scanner tests for new signature changes

**Documentation:**
- Updated README.md with Meson and Bazel ecosystem support
- Enhanced BENCHMARKS.md with detailed comparison methodology
- Added v1.0.4 release notes to WHATS_NEW.md

### Infrastructure

**Testing:**
- 105 unit tests passing (8 new Meson/Bazel tests)
- Integration tests with real-world BUILD and meson.build files
- Backward compatibility: 100% compatible with v1.0.0-1.0.3

## [1.0.3] - 2026-02-24

### Added - C Legacy Build System Support

**Traditional C/C++ Build System Parsers:**
- **pkg-config parser** - System library dependency detection
  - Parses `.pc` files (pkg-config metadata files)
  - Extracts `PKG_CHECK_MODULES` declarations from configure.ac
  - Version and dependency chain resolution
  - Handles Requires/Requires.private fields
  - ~80% coverage for system library dependencies

- **Autotools parser** - GNU build system support
  - Parses `configure.ac` files
    - `AC_CHECK_LIB(library, function)` declarations
    - `AC_SEARCH_LIBS(function, [libs...])` declarations
    - `PKG_CHECK_MODULES(PREFIX, packages)` macros
  - Parses `Makefile.am` files
    - `LDADD` and `LIBADD` linker flag extraction
    - `-l` flag parsing for library dependencies
  - ~60% coverage for pure C projects

- **Makefile heuristic parser** - Plain Makefile support
  - Pattern-based extraction of `-l` flags from LDFLAGS/LIBS
  - Detects `pkg-config --libs` calls with library names
  - Best-effort parsing without full Make evaluation
  - ~40% coverage for legacy C++ projects

**CLI Enhancements:**
- `--scan-pkgconfig` flag to enable/disable pkg-config scanning (default: true)
- `--scan-autotools` flag to enable/disable Autotools scanning (default: true)
- `--scan-makefiles` flag to enable/disable Makefile scanning (default: true)
- `--resolve-system-deps` flag to attempt system library version resolution (default: true)

### Changed

**Coverage Impact:**
- Combined with v1.0.0-1.0.2: **~90% of all C/C++ projects**
- Successfully scans legacy projects (curl, nginx, openssl patterns)
- Fills gap for projects without modern build systems

**SPDX purl Support:**
- Added `pkg:generic/{name}@{version}?type=pkg-config` format
- Added `pkg:generic/{name}@{version}?type=autotools` format
- Added `pkg:generic/{name}@{version}?type=makefile` format

**Documentation:**
- Updated README.md with C legacy build system support
- Added v1.0.3 detailed release notes to WHATS_NEW.md
- Documented heuristic parsing limitations and best practices

### Infrastructure

**Testing:**
- Unit tests for all 5 C parsers with tempfile fixtures
- Integration tests with real-world configure.ac, Makefile.am, Makefile samples
- Test fixtures include openssl.pc, curl-style configure.ac patterns

## [1.0.2] - 2026-02-24

### Added - Conan Package Manager Support

**Conan C++ Package Manager:**
- **Conan lock file parser** (`conan.lock`)
  - Parses Conan v1 lock file format (JSON)
  - Extracts package references with full version and revision
  - Supports both direct dependencies and transitive graph
  - Handles remote repository metadata
- **Conan manifest parsers** (`conanfile.txt`, `conanfile.py`)
  - Fallback when lock file unavailable
  - Version constraint parsing (>=, ~, ==, etc.)
  - Option and generator detection

**SPDX purl Support:**
- Added `pkg:conan/{name}@{version}` format
- Includes revision and remote metadata when available

**CLI Enhancements:**
- Automatic detection of `conan.lock`, `conanfile.txt`, `conanfile.py`
- Integrated into existing C/C++ scanning workflow

### Changed

**Scanner Improvements:**
- Optimized submodule scanning with depth validation
- Improved CMake parsing robustness (*.cmake module files)
- Fixed redundant depth checks in recursive scanning
- Better error handling for malformed Conan files

**Documentation:**
- Restructured README.md with improved organization
- Created comprehensive documentation in `docs/` directory
- Updated WHATS_NEW.md with v1.0.2 release notes
- Added Conan to supported ecosystems table

### Infrastructure

**Testing:**
- Conan parser unit tests with real-world lock file samples
- Integration tests for conanfile.txt and conanfile.py
- All tests passing (124 total)

**CLI:**
- Added `--output` flag for custom SBOM output directory
- Improved help text and examples

## [1.0.1] - 2026-02-23

### Added - CMake Support & Recursive Submodule Scanning

**CMake Dependency Parser:**
- **FetchContent parser** - Modern CMake external dependencies
  - Parses `FetchContent_Declare()` blocks from CMakeLists.txt
  - Supports GIT_REPOSITORY, GIT_TAG, URL, URL_HASH
  - Extracts SHA256 checksums from URL_HASH for supply chain security
  - Uses Git URL parser for proper github/gitlab/bitbucket purls
  - Version extraction from GIT_TAG or URL path
- **ExternalProject parser** - Legacy CMake external project support
  - Parses `ExternalProject_Add()` blocks
  - Handles same Git and URL patterns as FetchContent
  - Static parsing (no CMake execution required)

**Recursive Submodule Scanning:**
- Scans dependencies inside Git submodules recursively
  - package.json, Cargo.toml, CMakeLists.txt, and all supported manifest files
  - Depth limiting to prevent infinite recursion
  - Configurable max depth (default: 3)
- New `scan_submodule_recursively()` function in scanner module

**CLI Enhancements:**
- `--scan-cmake` flag to enable/disable CMake dependency scanning (default: true)

### Changed

**Scanner Signature:**
- Updated `scan_directory()` to include `scan_cmake` parameter
- All call sites updated to pass new parameter
- Tests updated with new scanner signature

**SPDX purl Support:**
- Added `pkg:cmake/{name}@{version}` format
- Fallback to `pkg:generic/{name}@{version}` for non-Git sources

**Documentation:**
- Updated README.md with CMake support and new CLI flag
- Added v1.0.1 detailed release notes to WHATS_NEW.md
- Documented CMake variable handling limitations (${VAR} skipped)

### Infrastructure

**Testing:**
- Created `tests/parser_tests/cmake_tests.rs` with 8 comprehensive tests
  - FetchContent parsing (Git and URL sources)
  - ExternalProject parsing
  - CMake variable handling (warns and skips unresolvable)
  - Checksum extraction validation
- Created test fixtures in `tests/fixtures/cmake/`
  - CMakeLists_fetchcontent.txt
  - CMakeLists_externalproject.txt
  - CMakeLists_with_variables.txt
- All 124 tests passing (116 existing + 8 new CMake tests)

## [1.0.0] - 2026-02-20

### Added - C++ Ecosystem Support

**First C++ Ecosystem Support:**
This major release (1.0.0) adds comprehensive C/C++ project SBOM generation capabilities.

**vcpkg Package Manager:**
- **vcpkg manifest parser** (`vcpkg.json`)
  - All version constraint formats: `version>=`, `version>`, `version=`, `version-semver`, `version-date`
  - Overrides section for version pinning
  - Features metadata stored in source_file field
  - Generates `pkg:vcpkg/{name}@{version}` purl format
- Integrated into scanner with automatic vcpkg.json detection

**Git Submodule Detection:**
- **Git submodule parser** (`.gitmodules`)
  - Parses INI format for submodule definitions
  - Resolves commit SHAs via `git ls-tree HEAD`
  - Supports HTTPS and SSH URL formats
  - Multi-host support: GitHub, GitLab, Bitbucket, self-hosted Git servers
  - Generates appropriate purl format based on host type:
    - `pkg:github/owner/repo@sha` for GitHub
    - `pkg:gitlab/owner/repo@sha` for GitLab
    - `pkg:bitbucket/owner/repo@sha` for Bitbucket
    - `pkg:generic/repo@sha` for self-hosted

**CLI Enhancements:**
- `--scan-submodules` flag to enable/disable submodule scanning (default: true)
- `--submodule-depth` flag to set max recursion depth for submodules (default: 3)

### New Modules

**Parser Modules:**
- `src/parsers/cpp/mod.rs` - C++ parser module entry point
- `src/parsers/cpp/vcpkg.rs` - vcpkg manifest parser (458 lines)
- `src/parsers/git/mod.rs` - Git parser module entry point
- `src/parsers/git/submodules.rs` - .gitmodules parser (292 lines)
- `src/parsers/git/commit_resolver.rs` - Git commit SHA resolver (140 lines)
- `src/parsers/git/url_parser.rs` - Git URL parser with multi-host support (299 lines)

**Total New Code:** 1,511 lines (19 files changed)

### Changed

**Scanner Integration:**
- Added vcpkg.json and .gitmodules detection to `scan_directory()`
- Integrated commit SHA resolution for accurate submodule versions
- Warning messages for unresolvable Git references

**SPDX Format:**
- Added purl generation for vcpkg packages
- Added purl generation for Git submodules with host-specific formats

**Documentation:**
- Updated README.md with C++ ecosystem support table
- Added v1.0.0 detailed release notes to WHATS_NEW.md
- Documented vcpkg version constraint formats
- Documented Git URL parsing and multi-host support

### Infrastructure

**Testing:**
- vcpkg parser tests with version constraint validation
- Git submodule parser tests with URL parsing
- Integration tests for C++ projects
- All tests passing

**Bug Fixes (Pre-1.0.0 Polish):**
- Fixed Python version operator stripping (>=, ==, ~=)
- Added transitive dependency resolution for Python
- Fixed `__future__` false positive in Python parser
- Improved error handling in Git operations

## [0.9.3] - 2026-02-10

### Added - Pipfile/Pipfile.lock & pyproject.toml Parser Support

**Pipfile/Pipfile.lock Support (Phase 1):**
- **Pipfile.lock parser** - Comprehensive lock file support for Pipenv projects
  - Parses JSON format with serde_json deserialization
  - Extracts all packages from `default` (production) and `develop` (development) sections
  - **SHA256 checksum extraction** from hashes array for supply chain security
  - Direct dependency detection via `index` field presence
  - Batch parallel metadata fetching from PyPI using rayon
  - **487% improvement**: 8 packages (v0.9.1) → 47 packages (v0.9.3)
  - **100% parity** with Black Duck for Pipenv-based projects

- **Pipfile manifest parser** - Fallback when lock file unavailable
  - Parses TOML format with existing toml crate
  - Handles version specifications: `*`, `==`, `>=`, `~=`, complex constraints
  - Distinguishes `[packages]` (production) from `[dev-packages]` (development)

**pyproject.toml Support (Phase 2):**
- **Multi-format pyproject.toml parser** - Modern Python packaging standard (PEP 517/518)
  - **PEP 621 format** - `[project]` section with `dependencies` and `optional-dependencies`
  - **Poetry format** - `[tool.poetry]` section with `dependencies` and `dev-dependencies`
  - **Poetry 1.2+ groups** - `[tool.poetry.group.*.dependencies]` format
  - **PDM format** - `[tool.pdm]` section with `dependencies` and `dev-dependencies`
  - Parses complex dependency specifications with regex
  - Dev dependency detection from optional-dependencies groups (dev, test, tests, testing)
  - Handles Poetry version constraints: caret (`^`), tilde (`~`), comparison operators
  - Filters Python version constraints automatically

**poetry.lock Checksum Extraction (Phase 3):**
- **SHA256 checksum extraction** from `[[package.files]]` section
  - Extracts first file hash from files array
  - Format: `sha256:abc123...` → `abc123...`
  - Enhances supply chain security for Poetry projects
  - Zero additional network overhead

### Changed
- Python package detection improved from 8 to 47 packages for assessment-service repository
- Eliminated all `@detected` version placeholders for Pipenv projects
- Package names use canonical PyPI format (e.g., `repoze.lru` not `repoze-lru`)
- SHA256 checksums extracted from poetry.lock and Pipfile.lock (stored in Dependency struct for future SPDX output integration)
- Spinner animation integration: "parsing Pipfile.lock..." message during scan
- Progress indicators automatically show Python package counts

### Tests
- Verified 47 Python packages detected from assessment-service Pipfile.lock
- 100% version accuracy (no "@detected" placeholders remaining)
- Perfect package name match with Black Duck (47/47 packages)
- Checksum extraction validated for both Pipfile.lock and poetry.lock

### Performance
- Batch parallel PyPI metadata fetching using rayon (existing pattern)
- Single-pass JSON/TOML parsing with serde deserialization
- No performance degradation vs v0.9.1
- Seamless integration with v0.9.2 progress indicators

### Technical Details
- **Modified files:**
  - `src/parsers/python.rs` - Added 3 new parser functions (+219 lines)
    - `parse_pipfile_lock_with_relationships()` - Pipfile.lock with checksums
    - `parse_pipfile()` - Pipfile manifest
    - `parse_pyproject_toml()` - pyproject.toml multi-format
  - `src/parsers/mod.rs` - Exported new parser functions
  - `src/scanner/mod.rs` - Registered parsers with spinner integration (+8 lines)
  - `Cargo.toml` - Version bumped to 0.9.3

- **Dependencies:**
  - No new dependencies (uses existing serde_json, toml, regex, rayon)

- **Parser priority order:**
  1. Pipfile.lock (highest - exact versions with checksums)
  2. poetry.lock (high - exact versions with checksums)
  3. requirements.txt (medium - version specs)
  4. setup.py (medium - version specs)
  5. Pipfile (medium - version specs)
  6. pyproject.toml (medium - version specs)
  7. Import scanning (lowest - no versions)

### Competitive Position
- **Matches Black Duck** for Python package detection (47/47 packages)
- **Superior package naming** - Uses canonical PyPI names
- **Comprehensive Python support** - Pipfile, Poetry, pip, setuptools, PDM, PEP 621
- **Supply chain security** - SHA256 checksums from lock files
- **Modern standards** - Full pyproject.toml support

### Migration Notes
- **Breaking change for library users**: `ScanContext.poetry_relationships` renamed to `python_lockfile_relationships` (CLI usage unaffected)
- Existing Pipenv projects automatically benefit from improved detection
- Poetry projects now include checksums in SBOM output
- pyproject.toml projects now generate accurate SBOMs

## [0.9.1.1] - 2026-01-28 (Hotfix)

### Fixed - Markdown Report Display Bug

**Problem:**
- ROS multi-package markdown reports showed empty or incomplete code blocks
- Example: "PIP (1 packages)" header with empty code block, "ROS (8 packages)" showing only 2 packages
- Caused reader confusion as package counts didn't match displayed packages

**Root Cause:**
- Console report generation used `render_dependency_list()` which only shows direct **production** dependencies
- Headers counted ALL packages (including development dependencies)
- This filtered out dev dependencies from display while still counting them in headers

**Solution:**
- Modified ROS multi-package section to include all direct dependencies (production + development)
- Uses `render_tree_classic()` with expanded filter to show all direct deps
- Maintains tree structure with proper branch characters (`├──`, `└──`)
- Now all counted packages appear in code blocks with tree visualization

**Impact:**
- Fixes empty code blocks in ROS multi-package reports
- Shows all direct dependencies (production and development) with proper tree structure
- Maintains count consistency between headers and displayed packages
- No change to regular (non-ROS) project reports

**Test Result:**
- ros2run PIP section now shows: `└── pytest @ unspecified [direct, dev]`
- ros2run ROS section now shows all 8 packages with tree structure (├──, └──)
- Previously: empty PIP block, incomplete ROS list (only 2 of 8 packages)

**Modified Files:**
- `src/formats/console.rs` (lines 1292-1324)

## [0.9.1] - 2026-01-28

### Added - ROS/rosdistro Version Resolution & Repository URL Enrichment

**ROS Package Version Resolution:**
- **Automatic ROS distribution detection** - Integrates with rosdistro GitHub API to resolve ROS package versions
  - Fetches distribution.yaml from ros/rosdistro repository
  - Supports ROS 2 distributions: jazzy, iron, humble, galactic, foxy
  - Supports ROS 1 distributions: noetic, melodic
- **CLI flag for manual override** - `--ros-distro <distro>` to explicitly specify ROS distribution
  - Priority order: CLI flag > ROS_DISTRO env var > default ("jazzy")
  - Example: `--ros-distro humble`, `--ros-distro iron`
- **Package name variant resolution** - Handles multiple naming conventions
  - Base name: `rclpy`
  - Python prefix: `python3-rclpy`
  - Distro prefix: `ros-jazzy-rclpy`
  - Underscore variants: `ament-index-python`, `python3-ament-index-python`
- **Global caching** - Single rosdistro fetch per distribution per scan session
  - 10-second timeout with graceful fallback to "unspecified"
  - Parallel resolution using rayon (matching existing metadata enrichment pattern)

**Repository URL Enrichment:**
- **GitHub URL extraction** - Populates SPDX `downloadLocation` field for ROS packages
  - Extracts `source.url` from rosdistro distribution.yaml
  - 47 packages with GitHub repository URLs (ros2cli project benchmark)
  - Full source traceability for security auditing
  - Zero performance overhead (uses existing rosdistro fetch)

### Changed
- ROS dependencies now show resolved versions instead of "unspecified"
  - Before: `rclpy @ unspecified, downloadLocation: NOASSERTION`
  - After: `rclpy @ 7.1.9, downloadLocation: https://github.com/ros2/rclpy.git` (jazzy)
- SPDX `downloadLocation` field populated for ROS packages (47 packages with URLs)
- Updated `scan_directory()` signature to accept optional `ros_distro` parameter
- Enhanced `detect_ros_distribution()` with three-tier priority system
- Extended `RosPackageInfo` struct with `repository_url` field
- Renamed `lookup_package_version()` to `lookup_package_info()` (returns version + URL)

### Tests
- Added unit test for repository URL enrichment (`test_resolve_ros_dependency_versions_with_repository_url`)
- Added 5 unit tests for rosdistro functions (version resolution, package variants, non-ROS packages)
- Added 2 integration tests for ros2cli scanning with different distributions
- All 97 tests passing

### Performance
- Single network fetch per ROS distribution (cached for session)
- 10-second timeout with graceful degradation
- No performance impact on non-ROS projects
- Parallel resolution using rayon
- Repository URL extraction has zero additional network overhead

### Technical Details
- Dependencies added: `serde_yaml` v0.9, `lazy_static` v1.4
- Modified files: `src/cli.rs`, `src/parsers/ros.rs`, `src/scanner/mod.rs`, `src/main.rs`, `src/formats/spdx.rs`
- New functions: `fetch_rosdistro_database()`, `detect_ros_distribution()`, `lookup_package_info()`, `resolve_ros_dependency_versions()`
- Extended `RosPackageInfo` struct with `repository_url: Option<String>` field
- Updated `create_download_location()` in SPDX formatter to use `dependency.repository_url`

### Competitive Position
- **ROS support**: First SBOM tool with automated ROS package version resolution via rosdistro
- **Repository URLs**: First SBOM tool to populate downloadLocation for ROS packages
- **vs. BlackDuck**: radeis detected 94 unique dependencies (23.5x more than BlackDuck's 4)
  - 15 individual ROS packages vs 3 repositories (5x granular)
  - 47 packages with GitHub URLs vs 0 in BlackDuck
  - 62 packages with resolved versions vs 3 in BlackDuck (21x more)

### Benchmark Results (ros2cli project)
- **94 unique dependencies** detected
- **15 individual ROS packages** within ros2cli repository
- **47 packages with GitHub repository URLs**
- **62 packages with resolved versions** (66% coverage)
- **223 SPDX hierarchical entries** (includes relationships)

## [0.9.0] - 2026-01-28

### Added - Checksums, Automation & Multi-Ecosystem Metadata

**Package Checksums:**
- **SHA-1 checksums** for all packages - Enables integrity verification and reproducible builds
  - Format: 40-character lowercase hexadecimal SHA-1 hashes
  - Added to SPDX `filesAnalyzed` field for each package
  - Supports supply chain security and SBOM verification workflows


**Multi-Ecosystem Metadata Extraction (Network Mode):**
- **Hybrid metadata extraction** for all ecosystems - Local files first, registry API fallback
  - npm: package.json + npm registry API (registry.npmjs.org)
  - Python: PyPI API (pypi.org/pypi) for poetry.lock packages
  - Cargo: crates.io API for Cargo.lock packages
  - PHP: Packagist API (repo.packagist.org/p2) for composer.json packages
  - Ruby: RubyGems API (rubygems.org/api/v2) for Gemfile packages
- **Parallel batch fetching** using rayon for 10-27x performance improvement
  - npm: 689 packages from 10+ min → 22.6 sec (27x speedup)
  - Python: 100-500 packages from 10-20 min → 30 sec (20-40x speedup)
  - Cargo: 100-500 packages from 7-15 min → 25 sec (17-36x speedup)
  - PHP: 10-200 packages from 2-7 min → 20 sec (6-21x speedup)
  - Ruby: 5-50 gems from 1-2 min → 10 sec (6-12x speedup)
- **Reduced API timeouts** from 5 seconds to 3 seconds for faster failure handling

### Changed
- File size optimized from 955KB (v0.8.0) to 899KB (-6% reduction)
- Efficiency improved from 1.384 KB/pkg to 1.303 KB/pkg (-6% improvement)
- Maintained all 690 packages and 689 CPE identifiers from v0.8.0
- All parser functions now use 3-pass pattern: collect → parallel fetch → create dependencies

### Performance
- **Package count**: 690 packages (maintained from v0.8.0)
- **File size**: 899 KB (6% smaller than v0.8.0's 955 KB)
- **CPE identifiers**: 689 (maintained from v0.8.0)
- **File efficiency**: 1.303 KB/pkg (improved from v0.8.0's 1.384 KB/pkg)

### Competitive Position
- **Improvement over v0.8.0**: 6% file size reduction while maintaining all features

## [0.8.0] - 2026-01-27

### Added - Rich Metadata & Security Features

**Metadata Extraction (95%+ coverage across all ecosystems):**
- **License information extraction** - Extracts and normalizes license identifiers (SPDX-compliant) for npm, Cargo, Python, ROS, PHP, Ruby ecosystems
  - Achieves 95%+ license coverage (650+/690 packages vs 0% in v0.7.0)
  - Replaces SPDX "NOASSERTION" with actual license identifiers
- **Supplier and originator tracking** - Author, maintainer, and organization metadata for supply chain transparency (90%+ coverage, 620+/690 packages)
  - Maps to SPDX supplier/originator fields with "Person:" and "Organization:" prefixes
- **Download location URLs** - Ecosystem-specific package registry URLs for verification and reproducible builds
  - npm: `https://registry.npmjs.org/{package}/-/{file}.tgz`
  - PyPI: `https://pypi.org/project/{package}/{version}/`
  - Cargo: `https://crates.io/api/v1/crates/{package}/{version}/download`
  - Full support for Composer, RubyGems, and Go ecosystems

**Enhanced SPDX Output:**
- **UUID-based SPDX IDs** - Better uniqueness than sequential IDs, no namespace collisions
  - Format: `SPDXRef-Package-{sanitized-name}-{uuid}`
  - Synthetic "main" root package for consistent document structure
- **Source file tracking** - Full audit trail showing which extractor and manifest file detected each package (98%+ coverage, 685+/690 packages)
  - Pattern: "Identified by the {extractor_type} extractor from {absolute_path}"
  - Format: `cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*`
  - Ecosystem-specific vendor extraction (npm scoped packages, Composer, Go modules)

**Test Infrastructure:**
- **Modular test structure** - Migrated all 64 tests from monolithic main.rs to organized test modules
  - 84 total tests across 7 categories (parser, format, scanner, model, error, utility, integration)
  - Reduced main.rs from 2,233 to 268 lines (88% reduction, 1,965 lines removed)
  - 18 separate test files for better organization and maintainability
- **Source tracking tests** - 11 new tests covering all parsers (npm, Cargo, Python, ROS, PHP, Ruby, Go)
- **Multi-ecosystem integration tests** - 2 comprehensive tests verifying source tracking across different parsers
- **UUID and CPE tests** - 7 new tests for SPDX ID uniqueness and CPE identifier generation

### Changed
- Enhanced `Dependency` struct with optional metadata fields (license, author, maintainers, repository_url, homepage_url, source_file)
- Updated SPDX package creation to populate license, supplier, originator, and sourceInfo fields
- Changed SPDX ID generation from sequential (`SPDXRef-Package-npm-1`) to UUID-based (`SPDXRef-Package-axios-{uuid}`)
- Relationship structure changed from flat (699 DESCRIBES in v0.7.0) to hierarchical (1 DESCRIBES + 689 CONTAINS in v0.8.0)
- All parsers now track source file path with absolute paths
- CycloneDX format now includes license and supplier information
- Created `src/lib.rs` to enable integration testing (exposes modules as library)
- Made SPDX structures public for testing (SPDXDocument, SPDXPackage, SPDXRelationship, SPDXExternalRef fields)

### Improved
- Enhanced compliance reporting capabilities with rich metadata
- Richer metadata for supply chain security and transparency
- Improved test coverage and organization for long-term maintainability
- Enhanced deduplication logic to properly handle ImportScan vs Manifest priority

### Fixed
- **Deduplication bug fix** - ImportScan entries now correctly filtered when LockFile/Manifest versions exist
  - v0.7.0 incorrectly kept ImportScan duplicates with placeholder "detected" versions
  - Removed 10 duplicate packages (9 unique packages counted twice: axios, uuid, 6 AWS SDK clients, serverless-sentry-lib, strftime)
  - Package count corrected from 699 to 690 (689 real packages + 1 synthetic "main")
- Deduplication logic now correctly prioritizes LockFile > Manifest > ImportScan sources
- Test structure now properly organized for Rust integration testing
- SPDX external refs now include both PURL and CPE identifiers in correct format

### Performance
- **Package count**: 690 packages (BUG FIX: removed 10 duplicate ImportScan entries from v0.7.0's 699)
- **File size**: 955 KB (REGRESSION: +110% from v0.7.0's 454 KB)
- **CPE identifiers**: 689 (NEW FEATURE in v0.8.0)
- **File efficiency**: 1.384 KB/pkg (REGRESSION: +113% worse than v0.7.0's 0.649 KB/pkg)
- Test execution time under 90 seconds for all 84 tests

### Technical Details
- Modified [src/models/dependency.rs](src/models/dependency.rs) - Added optional metadata fields
- Modified [src/formats/spdx.rs](src/formats/spdx.rs) - UUID-based IDs, CPE generation, hierarchical relationships, metadata population
- Modified [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) - License and supplier support
- Modified [src/parsers/mod.rs](src/parsers/mod.rs) - Enhanced deduplication with ImportScan priority fix
- Created [src/lib.rs](src/lib.rs) - Library entry point for integration testing
- Created [tests/all_tests.rs](tests/all_tests.rs) - Integration test module entry point
- Created 18 test module files under tests/ directory with organized structure

### Competitive Position
- **Metadata richness**: Now comparable to enterprise tools (95%+ license, 90%+ supplier vs BlackDuck 99.9%)
- **Multi-ecosystem leader**: Full metadata support across npm, Cargo, Python, ROS, PHP, Ruby ecosystems
- **Note**: v0.8.0 fixed ImportScan deduplication bug (removed 10 duplicates) and added CPE metadata (increased file size), both issues addressed in v0.9.0

## [0.7.0] - 2026-01-27

**⚠️ KNOWN ISSUE**: This version contains a deduplication bug that incorrectly keeps 10 ImportScan duplicate packages with version "detected". See [PACKAGE_COUNT_ANALYSIS.md](scan_reports/PACKAGE_COUNT_ANALYSIS.md) for details. Fixed in v0.8.0.

### Added
- **Package detection** - Now detects **699 packages** (was 563), but includes 10 duplicate ImportScan entries (corrected to 690 in v0.8.0)
- **Dual SBOM modes** for different use cases:
  - `--sbom-mode complete` - All packages (699 pkgs including 10 duplicates, 454KB) for compliance and inventory
- Smart manifest filtering - Automatically removes redundant package.json version ranges when exact lockfile versions exist
- Comprehensive documentation in `docs/` folder:
  - [docs/sbom_modes_guide.md](docs/sbom_modes_guide.md) - Complete guide for dual SBOM modes with CI/CD examples
  - [docs/WHATS_NEW.md](docs/WHATS_NEW.md) - Detailed v0.7.0 changes with migration guide
  - [docs/plan/improvement_plan.md](docs/plan/improvement_plan.md) - Technical design document
  - [docs/plan/implementation_summary.md](docs/plan/implementation_summary.md) - Implementation results with metrics

### Changed
- **Package detection improved by 24.2%** (563 → 699 packages, but includes 10 ImportScan duplicates)
- Deduplication algorithm now version-aware using `(name, version, ecosystem)` tuple instead of `(name, ecosystem)`
- NPM parser uses HashSet to prevent duplicate processing of same `package@version` combinations
- README.md completely revised for clarity and conciseness with focus on key improvements
- Updated all SPDX and CycloneDX generators to support mode-based filtering

### Fixed
- Multiple versions of same package now correctly preserved (e.g., `@aws-sdk/client-sso@3.632.0` and `@aws-sdk/client-sso@3.848.0` both kept)
- Manifest versions (version ranges like `^3.215.0`) automatically filtered when lockfile versions exist
- ~126 missing AWS SDK sub-packages now correctly detected in nested node_modules

### Performance
- ⚠️ **699 packages includes 10 ImportScan duplicates** (corrected to 690 in v0.8.0)

### Technical Details
- Modified [src/parsers/mod.rs](src/parsers/mod.rs) - Version-aware deduplication with manifest filtering
- Modified [src/parsers/npm.rs](src/parsers/npm.rs) - HashSet-based duplicate prevention
- Modified [src/cli.rs](src/cli.rs) - Added SbomMode enum
- Modified [src/formats/spdx.rs](src/formats/spdx.rs) - Mode-based filtering for SPDX output
- Modified [src/formats/cyclonedx.rs](src/formats/cyclonedx.rs) - Mode-based filtering for CycloneDX output
- Modified [src/main.rs](src/main.rs) - Pass mode parameter to all format generators

### Competitive Position
- Dual SBOM modes provide flexibility for both compliance and security workflows
- **Known issue**: Deduplication bug allows ImportScan entries with version "detected" to survive when LockFile versions exist (fixed in v0.8.0)

## [0.6.0] - 2026-01-26

### Added
- True hierarchical dependency trees from lock files for npm, Cargo, and Poetry
  - Phase 1: npm (package-lock.json) - Full parent-child relationships
  - Phase 2: Cargo (Cargo.lock) - Parses dependencies arrays for Rust projects
  - Phase 3: Poetry (poetry.lock) - Parses [package.dependencies] tables for Python projects
- Accurate [direct] vs transitive markers using graph-based analysis
- Reorganized report structure with distinct packages list and appendix sections
- Circular dependency detection and handling
- Modular architecture refactoring - Split main.rs (6,561 lines) into 7 modules across 24 files

### Changed
- Report structure now shows direct production deps first, then distinct list, with dev/transitive in appendix
- Main.rs reduced from 6,561 to 2,130 lines (67.6% reduction)

### Fixed
- Corrected is_direct flags based on actual parent-child relationships instead of file paths
- Console summary counts now use corrected flags from dependency graph
- VendorMode::Only bug that prevented scanning files inside vendor directories
- Duplicate create_package_url functions consolidated
- Direct dependency counting standardized across report sections

### Tests
- All 59 comprehensive unit tests passing
- Added tests for Cargo and Poetry relationship parsing
- Zero compilation warnings

## [0.5.0] - 2026-01-23

### Breaking Changes
- Tree-style visualization now enabled by default (use --tree-style flat for old format)

### Added
- Tree-style dependency visualization with 3 modes (flat, tree, compact)
- Summary statistics section with at-a-glance overview
- Emoji severity indicators (🔴 Critical, 🟠 High, 🟡 Medium, 🟢 Low)
- Risk assessment in summary section

### Changed
- Enhanced console output with Unicode box-drawing characters
- Better visual hierarchy with consistent separators

## [0.4.0] - 2026-01-22

### Breaking Changes
- Vendor directory scanning enabled by default
- Import fallback scanning enabled by default

### Added
- Enhanced markdown reports with dependency source tracking (direct vs transitive)

### Changed
- Improved transitive dependency detection for npm packages

## [0.3.1] - 2026-01-21

### Added
- Multi-platform build system (Windows + Linux)
- Cross-compilation automation
- Enhanced build documentation

## [0.3.0] - 2026-01-20

### Added
- ROS/ROS2 multi-package support
- Hierarchical tree output
- SPDX relationships (DESCRIBES, DEPENDS_ON)
- Import scanning fallback for Python, JS/TS, Go
- 51 comprehensive unit tests

## [0.2.0] - 2026-01-19

### Added
- SPDX 2.3 support (JSON + Tag-Value)
- Package URL (purl) implementation
- Multi-format output

## [0.1.0] - 2026-01-16

### Added
- Initial release
- 8 ecosystem support (npm, Cargo, pip, Go, RubyGems, Composer, Maven, ROS)
- Console output
- 18 unit tests

[0.9.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/VicOne-RD/radeis_sc2sbom/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/VicOne-RD/radeis_sc2sbom/releases/tag/v0.1.0
