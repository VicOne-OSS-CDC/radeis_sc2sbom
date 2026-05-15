# What's New

## v1.0.18 — Tree-sitter AST Scanner (2026-05-13)

### Overview


### Tree-sitter AST Scanner


- **Lexical fallback** — Files that fail to parse (syntax errors, preprocessor constructs) fall back to the lexical scanner automatically so no file is silently skipped.


**22 default high-confidence CWEs (default scan):**

|-----|----------|


### `--experiment-scan`

The `--experiment-scan` flag activates 17 high-FP experimental CWEs on top of the 22 default rules. This flag is available in internal builds only (`#[cfg(feature = "internal")]`).

| Mode | Findings | FP Rate |
|------|----------|---------|
| Default scan (22 CWEs) | 22,939 | **22.0%** |
| Experimental scan (39 CWEs) | 127,800 | 82.2% |

The experimental CWEs cover patterns that are real weaknesses but generate many false positives on typical automotive/embedded codebases: buffer operations that are bounded by context the scanner cannot see, constant-condition checks used for intentional unreachable-code guards, and crypto function families where non-cryptographic uses are common.

```bash
# Default scan (22 high-confidence CWEs)

# Experimental scan (all 39 CWEs, higher FP)
```

### Phase 24 Tuning

17 targeted rule-level changes were applied before release to reduce false positives. Notable changes:


Net result: −89,479 false positives vs the pre-v1.0.18 baseline (217,279 → 127,800 total findings).

### Migration Notes

`--cppcheck-path` is no longer accepted; remove it from any build scripts. All other internal-build flags are unchanged. Public builds are unaffected.

---

## v1.0.17 — Advanced C/C++ SAST & AUTOSAR Version Extraction (2026-05-11)

### Overview

v1.0.17 extends the internal SAST scanner with argument-value inspection and cppcheck subprocess integration, adds SARIF 2.1 output for CI/IDE pipelines, and closes three AUTOSAR accuracy gaps: full arxml dependency parsing, component version extraction from `.epd` files and Doxygen headers, and correct ecosystem classification for linker-flag-discovered libraries.

### Argument-Value Matching

The lexical scanner now inspects argument values at call sites — not just function names — to detect misuse that cannot be inferred from the callee name alone:

|-----|---------|---------|

### cppcheck Integration

When cppcheck is installed, the scanner invokes it as a subprocess and merges its dataflow-backed findings with the lexical results:

- cppcheck output is parsed from its XML report format
- Graceful degradation: if cppcheck is not found on `PATH` or `--cppcheck-path`, lexical-only results are used with a note in the report
- Use `--cppcheck-path <PATH>` to specify a non-default binary location

### SARIF 2.1 Output

Every internal-build scan now writes a `<project>_static_analysis.sarif` file alongside the markdown report:

```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [{
    "tool": { "driver": { "name": "radeis_sc2sbom", ... } },
    "results": [
      {
        "fingerprints": { "sha256/v1": "..." },
        "locations": [{ "physicalLocation": { "artifactLocation": { "uri": "lib/url.c" }, "region": { "startLine": 1234 } } }]
      }
    ]
  }]
}
```

The SARIF file is compatible with GitHub Code Scanning (`upload-sarif` action), VS Code SARIF Viewer, and any CI pipeline that consumes SARIF 2.1.

Use `--sarif-output <PATH>` to override the default output path.

### SARIF Baseline Diffing

The `--sarif-baseline <PATH>` flag accepts a prior SARIF run and suppresses findings that already appeared in it. Only **new** findings are reported — making it suitable for PR-level CI gates that should not fire on pre-existing issues:

```bash
# Save baseline

# Later: report only new findings
  --sarif-baseline baseline.sarif --sarif-output new-findings.sarif
```

Matching uses SHA-256 fingerprints (`fingerprints["sha256/v1"]`) computed from rule ID + file URI + start line. A finding is suppressed if its fingerprint appears in the baseline.

### AUTOSAR arxml Dependency Parsing

`.arxml` files are now fully parsed for inter-component dependencies. Three AUTOSAR element types are extracted:

| Element | Dependency type |
|---------|----------------|
| `SW-COMPONENT-PROTOTYPE` | SWC-to-SWC composition |
| `BSW-MODULE-DESCRIPTION` | BSW module references |
| SWC type definition elements (`APPLICATION-SW-COMPONENT-TYPE`, `SENSOR-ACTUATOR-SW-COMPONENT-TYPE`, `SERVICE-SW-COMPONENT-TYPE`, etc.) | Component type declarations |

Previously, arxml files were only used for AUTOSAR project detection — they are now a primary dependency source.

### AUTOSAR Version Extraction

Three sources are now combined to replace `unspecified` version strings for AUTOSAR components:

**`.epd` files (BSW modules)**

Standard ECUC module definition files contain `ECUC-MODULE-DEF` elements with a `REVISION-LABEL` field under `ADMIN-DATA/DOC-REVISIONS`. The scanner collects these via `collect_epd_versions(root)`, building a `SHORT-NAME → version` map. First found across variant files wins.

**Doxygen-style C/H headers (SWC directories)**

NXP/Freescale MCAL source files embed version strings in file-header comments:

```c
* SW Version          : 1.0.1
```

The scanner reads the first 50 lines of every `.c`/`.h` file, matches the pattern `^\s*\*\s+SW Version\s*:\s*(\S+)`, and keys results by the immediate parent directory name via `collect_doxygen_versions(root)`.

**Version resolution priority:** `.epd` → Doxygen header → `"unspecified"`

**Ecosystem upgrade pass:** After the main directory walk, linker-flag-discovered `system` ecosystem entries (e.g. `-lAdc`, `-lMcu`) are matched against the epd/Doxygen maps. Matching entries are upgraded to the `autosar` ecosystem with the real version string, eliminating the duplicate `Mcu @ 1.0.1 (autosar)` + `Mcu @ unspecified (system)` split.



### Migration Notes

No breaking changes. All new CLI flags are additive. Public builds (`cargo build --release` without `--features internal`) are unchanged. Internal builds gain `_static_analysis.sarif` alongside the existing `_static_analysis.md`.

---


### Overview





|-----|----------|-------------------|


### Scan Scope

The scanner is scoped to **component-mapped C/C++ directories** only — not the full source tree. A component directory is one that appears in the `component_dirs` map built from CMake, Autotools, pkg-config, or Makefile manifests.

**Fallback mode for standalone C projects:** When no manifest-derived component directories are found but C/C++ source files exist under the scan root, a synthetic `(project_name, "C/C++") → scan_root` entry is inserted. This allows repos like the NIST Juliet test suite (54,484 C files, no package manifests) to be scanned without any configuration.

### Output — Static Analysis Report

A new `<project>_static_analysis.md` file is written for every internal-build scan:

```markdown
# Static Analysis Report


## Summary

|-----------|-----|------|-------|
| ...

## Findings


| File | Line | Function |
|------|------|----------|
| lib/url.c | 1234 | strcpy |
| ...
```

### Output — CycloneDX SAST Findings


```json
{
  "description": "Buffer Copy Without Checking Size of Input ('Classic Buffer Overflow')",
  "affects": [{ "ref": "curl" }],
  "properties": [
    { "name": "sc2sbom:finding:file", "value": "lib/url.c" },
    { "name": "sc2sbom:finding:line", "value": "1234" }
  ]
}
```


### Internal Build

The scanner is compiled only when the `internal` Cargo feature is enabled:

```bash
# Public build (no scanner)
cargo build --release

# Internal build (scanner included)
cargo build --release --features internal
```

The `--features internal` gate means the public binary has zero code from the scanner — no dead code, no stripped functions, no runtime flag needed.

### Migration Notes

No breaking changes. Public builds are byte-for-byte identical to v1.0.15. Internal builds gain the `_static_analysis.md` output file and SAST entries in `_cyclonedx.json`.

---


### Overview


### AUTOSAR Project Detection

The scanner runs a pre-pass `detect_autosar()` before the main directory walk. Three signals are checked in order (short-circuit on first match):

| Signal | Trigger |
|--------|---------|
| DET-01 | Any `.arxml` file at any depth |
| DET-02 | A directory named `BSW`, `MCAL`, `RTE`, `AUTOSAR`, or `SWC` |
| DET-03 | Token `AUTOSAR_VERSION` or `AR_VERSION` in a CMake or Makefile at root or one level deep |

When any signal fires, `ScanContext.is_autosar` is set to `true` and the AUTOSAR classification pipeline activates automatically.

### AUTOSAR Component Classification

`classify_autosar_components()` matches discovered dependency names against a bundled BSW module config (overridable with `--bsw-config <path>`). Matching components are upgraded to the `autosar` ecosystem and annotated with:

- **`module_name`** — e.g. `NvM`, `Det`, `CanIf`
- **`layer`** — e.g. `BSW-Memory`, `BSW-SystemServices`, `BSW-Communication`
- **`platform`** — `Classic` or `Adaptive`

### AUTOSAR Output — CycloneDX

AUTOSAR components appear as standard CycloneDX components with additional properties:

```json
{
  "name": "NvM",
  "properties": [
    { "name": "autosar:layer",    "value": "BSW-Memory" },
    { "name": "autosar:platform", "value": "Classic" },
    { "name": "autosar:supplier", "value": "Vector Informatik" }
  ]
}
```

### AUTOSAR Output — SPDX

AUTOSAR components carry matching `ExternalRef OTHER` entries:

```
ExternalRef OTHER autosar:layer BSW-Memory
ExternalRef OTHER autosar:platform Classic
ExternalRef OTHER autosar:supplier Vector-Informatik
```

### Supplier Mapping (`--supplier-config`)

A YAML file mapping component names to supplier strings can be passed via `--supplier-config`:

```yaml
NvM: "Vector Informatik"
CanIf: "ETAS"
Det: "In-house"
```

- Mapped components emit the supplier string as `autosar:supplier`
- Unmapped components emit `NOASSERTION`
- Non-AUTOSAR components are unaffected
- Missing or malformed YAML causes a hard error with a clear message



**SPDX — `SECURITY` ExternalRef:**
```
```

**CycloneDX — `cwes` array:**
```json
{
  "cwes": [1321, 94, 77]
}
```


**Implementation details:**
- NVD responses cached at `~/.cache/sc2sbom/nvd/` (TTL controlled by `--cache-ttl`, default 24 h)
- 6-second rate-limit per HTTP request (NVD public API limit); cache hits are instant
- HTTP failures are logged to stderr and skipped; scanning continues

### `--output` for Single Formats

Previously `--output <dir>` only took effect with `--format all`. In v1.0.15 it works for all formats:

```bash
# Write SPDX JSON to a file instead of stdout
radeis_sc2sbom --path ./project --format spdx-json --output ./out

# Write CycloneDX JSON to a file
radeis_sc2sbom --path ./project --format cyclonedx-json --output ./out
```

When `--output` is omitted for single formats, output continues to go to stdout (previous behaviour preserved).

### Migration Notes


---

## v1.0.14 - Reliability & SBOM Quality (2026-04-24)

### Overview

v1.0.14 is a reliability and SBOM-quality milestone driven by user-reported bugs from real-world C/C++ project scans. Four phases close long-standing gaps: the scanner no longer aborts on broken symlinks, Makefile variable references no longer leak into `versionInfo`, common C/C++ libraries now resolve to real SPDX license identifiers, and the Linux release binary is statically linked so it runs on every supported distro without glibc drift. The scan that previously failed now completes cleanly end-to-end.

### Broken Symlink Tolerance

Previously a single broken symlink anywhere under the scan path would abort the whole traversal. The scanner now warns and continues:

```
Warning: skipping /path/to/broken-link: No such file or directory (os error 2)
```

All 5 WalkDir traversal sites are covered — the main scanner, the fallback import scan in `main.rs`, and the C/C++ parsers (`makefile.rs`, `mk_file.rs`, etc.). A shared `warn_on_walkdir_err` helper in `src/util/mod.rs` replaces 5 duplicated `filter_map` closures.

### Makefile `$(VAR)` Filtering

Makefile fragments often contain unresolved variable references like `DOPENSSL_VERSION := $(OPENSSL_VERSION)`. Before this release those references leaked into SBOM output verbatim:

**Before (v1.0.13):**
```json
{
  "name": "openssl",
  "versionInfo": "$(OPENSSL_VERSION)",
  "downloadLocation": "https://example.invalid/openssl-$(OPENSSL_VERSION).tar.gz"
}
```

**After (v1.0.14):**
```json
{
  "name": "openssl",
  "versionInfo": "NOASSERTION",
  "downloadLocation": "NOASSERTION"
}
```

The filter is applied at the `mk_file.rs` parser layer and also guarded defense-in-depth at every version-output site: `version_info_or_noassertion`, `generate_cpe_identifier`, `create_download_location`, `create_package_url`, and the CycloneDX component version field. Any value matching `$(...)` is replaced with `NOASSERTION` instead of being emitted.

### C/C++ License Resolution

Native C/C++ dependencies previously defaulted to `NOASSERTION` because neither `Makefile` nor `pkg-config` files expose a standard license field consistently. v1.0.14 adds two resolution paths:

1. **`.pc` `License:` parsing** — when pkg-config files carry a `License:` line, it is parsed and promoted to `licenseConcluded`.
2. **`known_licenses.rs` table** — a curated lookup for 24 common system libraries (openssl → Apache-2.0, zlib → Zlib, libcurl → curl, libssh2 → BSD-3-Clause, ncurses → X11, etc.). Used as a fallback in `makefile.rs`, `mk_file.rs`, `pkgconfig.rs`, and `pkgconfig_detector.rs`.

Sample output for `openssl@3.0.7` now looks like:

```json
{
  "name": "openssl",
  "versionInfo": "3.0.7",
  "licenseConcluded": "Apache-2.0",
  "licenseDeclared": "Apache-2.0"
}
```

instead of `NOASSERTION` on both license fields.

### Static Linux Binary via musl

The Linux release binary now targets `x86_64-unknown-linux-musl` instead of `x86_64-unknown-linux-gnu`. Previously the binary carried a GLIBC 2.39 dependency from the build host, which broke downstream on Ubuntu 22.04 (`version GLIBC_2.39 not found`). The musl-linked static binary has no glibc dependency and runs on any x86_64 Linux — Ubuntu 22.04+, 24.04, Alpine, Debian, RHEL, etc. A musl cross-linker toolchain guard was added to `build-all.sh` and the CI Ubuntu runner is configured to provide `musl-tools`.

### Migration Notes

No CLI or output-schema changes. Existing SBOMs regenerated on v1.0.14 may show additional license concluded fields on C/C++ components and fewer `$(VAR)`-style version strings — both are strict improvements. Linux users previously stuck on the glibc-linked binary can drop the v1.0.14 musl binary into place without any other changes.

### Test Target

Validated against the real-world C/C++ project that originally surfaced all four bugs. The scan now runs to completion without warnings beyond the expected broken-symlink notices, and the emitted SPDX/CycloneDX pass schema validation.

See [v1.0.14 plan](plan/v1.0.14_user_reported_bugfixes.md) for full design details.

---

## v1.0.13 - Multimodal Sub-Model Components (2026-04-14)

### Overview

v1.0.13 decomposes multimodal AI models into their constituent sub-model components. A model like `google/gemma-4-E2B-it` is now represented as a parent model containing separate text, vision, and audio sub-models — each with its own architecture metadata in the SBOM.

### Sub-Model Decomposition

Multimodal models with `text_config` alongside `vision_config` and/or `audio_config` in `config.json` are automatically decomposed:

```
gemma4 (parent — Gemma4ForConditionalGeneration)
├── gemma4_text  (35 layers, 1536 hidden, 8 heads, 262K vocab, 131K context)
├── gemma4_vision (16 layers, 768 hidden, 12 heads, patch_size=16)
└── gemma4_audio  (12 layers, 1024 hidden, 8 heads, conv_kernel=5)
```

### Output Format Support

- **CycloneDX** — Nested `components` array inside parent component, each sub-model as `machine-learning-model` with `radeis:ai:sub_model:*` properties
- **SPDX** — Child packages with `CONTAINS` relationships from parent model
- **Console** — Sub-model summary table with modality, model type, layers, hidden size, heads, dtype, and modality-specific extras

### Guard Condition

Sub-models are only emitted for genuinely multimodal models — a text-only model with `text_config` (but no `vision_config` or `audio_config`) will NOT generate spurious sub-model entries.

### Test Coverage

- 4 new Safetensors tests: multimodal extraction, text-only guard, no-text-config guard, vision+text-only (LLaVA-style)
- 1 new GGUF test: companion config.json enrichment with sub-models

See [v1.0.13 plan](plan/v1.0.13_multimodal_sub_model_components.md) for full design details.

---

## v1.0.12 - Safetensors Rich Metadata (2026-04-14)

### Overview

v1.0.12 closes the metadata gap between GGUF and Safetensors SBOM quality by extracting rich metadata from all HuggingFace companion config files. Both Safetensors and GGUF model directories now produce deeply detailed SBOMs covering architecture depth, inference defaults, multimodal capabilities, provenance, and adapter detection.

### Rich Metadata from Companion Files

- **`config.json` extended** — model_type, text_config (hidden_layers, hidden_size, attention_heads, context window), multimodal detection (vision_config, audio_config), dtype fallback chain
- **`generation_config.json`** — temperature, top_k, top_p inference parameters
- **`tokenizer_config.json`** — processor_class, model_max_length (astronomical values safely capped)
- **`preprocessor_config.json`** — image, audio, and video processor types with sequence lengths and sampling rates
- **`README.md` frontmatter** — base_model (handles both string and list), license, model_creator, pipeline_tag, quantized_by, prompt_template, tags, languages, datasets
- **`adapter_config.json`** — LoRA/QLoRA adapter detection with base model reference

### GGUF Companion File Enrichment

GGUF repos now benefit from the same companion file parsing. Binary KV metadata always wins; companion files fill gaps only. Tags, languages, and datasets use deduplicated union merging from both sources.

### Output Enhancements

- **CycloneDX** — ~25 new `radeis:ai:*` properties covering architecture, multimodal, generation, processor, provenance, and adapter metadata
- **SPDX** — sourceInfo extended with model_type, context window, and modality summary (kept concise)
- **Console** — AI Model Details table expanded with architecture, multimodal, generation params, and provenance rows

### Safety & Edge Cases

- 1 MB cap on all companion file reads to prevent memory issues
- Case-insensitive README.md filename matching (README.md, readme.md, Readme.md)
- CRLF line ending normalization for Windows-generated files
- Astronomical model_max_length values (e.g., 1e30 in Gemma-4) safely discarded
- prompt_template capped at 512 chars (storage) and 80 chars (console display)

### Test Coverage

- 8 new tests in `safetensors_tests.rs` — companion file parsing, multimodal detection, dtype fallback chain, model_max_length cap
- 6 new tests in `gguf_tests.rs` — config.json extraction, README frontmatter (string and list base_model, CRLF, lowercase filename), adapter detection, tags union merge

See [v1.0.12 plan](plan/v1.0.12_safetensors_rich_metadata.md) for full design details.

---

## v1.0.11 - Safetensors AI Model SBOM (2026-04-13)

### Overview

v1.0.11 adds Safetensors AI model support, extending `radeis_sc2sbom` to cover the dominant format for modern transformer models (LLaMA, Mistral, Falcon, etc.). Models are scanned at the directory level — regardless of how many shards the model is split into, a single Dependency entry is emitted with accurate total size, dtype, and architecture metadata.

### Safetensors AI Model Support

- **File detection** — scans `.safetensors`, `model.safetensors.index.json`, and `config.json` for metadata
- **Directory-level deduplication** — multi-shard models (e.g., `model-00001-of-00002.safetensors`) are consolidated into one SBOM entry per model directory
- **CycloneDX output** uses the `machine-learning-model` component type with a `modelCard` containing architecture, dtype, and size metadata
- **SPDX output** emits `pkg:huggingface` PURLs for model identification in the HuggingFace ecosystem
- **New `AIModelMetadata` fields**: `safetensors_format`, `total_size_bytes`, `shard_count`, `torch_dtype`, `transformers_version`, `vocab_size`

### Test Coverage

- 12 new tests in `tests/parser_tests/safetensors_tests.rs` covering single-shard, multi-shard, index-based, and config.json-driven scenarios

---

## v1.0.10 - Java Complete (2026-04-13)

### Overview

v1.0.10 transforms Gradle from detection-only to full dependency parsing, supporting both Groovy DSL (`build.gradle`) and Kotlin DSL (`build.gradle.kts`). This is critical for Physical AI projects using Android-based robotics controllers and edge AI inference on Android devices.

### Gradle Dependency Parsing

- **Groovy DSL** (`build.gradle`) — parses string notation (`'group:artifact:version'`), map notation (`group: 'g', name: 'a', version: 'v'`), and platform/BOM declarations
- **Kotlin DSL** (`build.gradle.kts`) — parses function notation (`implementation("group:artifact:version")`) and platform BOM
- **Scope classification** — `testImplementation` → Test, `compileOnly` → Provided, `annotationProcessor`/`kapt`/`ksp` → Build, with confidence 1.0
- **Android support** — `androidTestImplementation` and `androidTestCompile` correctly classified as test dependencies
- **PURL format** — `pkg:maven/{group}/{artifact}@{version}` (same as Maven)

See [v1.0.10 plan](plan/v1.0.10_gradle_support.md) for full design details.

---

## v1.0.9 - Physical AI Ready (2026-04-10)

### Overview

v1.0.9 adds GGUF AI model support, making `radeis_sc2sbom` the first SBOM generator with native AI model binary parsing and integrity verification. This release also simplifies the CLI by consolidating C/C++ build system flags.

### GGUF AI Model Support

- **Binary parser** extracts metadata directly from `.gguf` files — architecture, quantization type, tensor layout, context length, and embedded licensing info
- **CycloneDX output** uses the `machine-learning-model` component type with a `modelCard` containing training parameters and dataset references
- **SPDX output** emits `pkg:huggingface` PURLs for model identification in the Hugging Face ecosystem
- Enable with `--scan-ai-models true`

### AI Model Integrity Verification

- **Tensor parameter cross-validation** — declared tensor counts and dimensions are verified against the actual binary layout to detect truncated or corrupted model files
- **SHA-256 hashing** — each model file gets a content hash for supply-chain authenticity checks

### CLI Simplification

- 5 individual C/C++ flags (`--scan-cmake`, `--scan-pkgconfig`, `--scan-autotools`, `--scan-makefiles`, `--scan-mk-files`) merged into `--scan-c-build-systems`
- `--meson-parse-subprojects` removed (always on when `--scan-meson` is enabled)
- `--resolve-system-deps` removed (dead code)
- `scan_directory()` reduced from 20 to 13 arguments

See [v1.0.9 plan](plan/v1.0.9_physical_ai_ready.md) for full design details.

---

## v1.0.8 - SBOM Quality & Spec Compliance

### Overview

v1.0.8 fixes spec violations in CycloneDX 1.5 and SPDX 2.3 output and improves detection accuracy. Spec fixes were identified during customer testing with the MCUTest project; detection improvements were discovered by analyzing the `op_kraken_04a_uart_360x360` NXP GUI Guider project. Generated SBOMs now pass official validators (`pyspdxtools`, `cyclonedx-cli validate`) and work correctly in downstream tools such as Dependency-Track.

### Spec Compliance Fixes

- **PURL sentinel versions removed** — PURLs no longer contain `@detected` or `@unspecified` as version tokens (both are invalid per the PURL spec). The version component is omitted when the version is unknown.
  ```
  Before: pkg:pypi/utime@detected
  After:  pkg:pypi/utime
  ```

- **Fake `downloadLocation` URLs fixed** — PyPI packages discovered via import scanning (version `"detected"`) no longer produce fake URLs like `https://pypi.org/project/utime/detected/`. Returns `NOASSERTION` instead, which is the correct SPDX value.

- **CycloneDX `version` field omitted for unknown versions** — Components with `version: "detected"` or `version: "unspecified"` no longer include a version field. CycloneDX 1.5 specifies `version` is optional and should be omitted when unknown.

- **`primaryPackagePurpose` corrected for system runtime libs** — System libraries detected from Makefile `-l` flags (`-ldl`, `-lm`, `-lpthread`) are now correctly classified as `LIBRARY` instead of `SOURCE`.

- **CycloneDX `metadata.tools` updated to non-deprecated format** — Updated from the legacy flat array `[{vendor, name, version}]` to the CycloneDX 1.5 recommended format `{"components": [{type, name, version}]}`.

- **`CONTAINS NOASSERTION` relationships removed** — These relationships caused validator warnings in most SPDX tools and conveyed no meaningful information. Removed from default output.

- **Root package `versionInfo` fixed** — Changed from `"0"` to `"NOASSERTION"`, which is the correct SPDX 2.3 value when the project version is unknown.

- **CycloneDX duplicate components in ROS scans fixed** — A library shared by multiple ROS packages was previously emitted once per package, producing inflated component counts (e.g. 107 entries for 42 unique libraries). Each unique dependency now appears exactly once per CycloneDX 1.5 spec.

- **CycloneDX root not in dependency graph fixed** — `metadata.component` (the root project node) was never added to the `dependencies` array, leaving the graph disconnected. Root now appears with edges to all top-level components so Dependency-Track and other tools can traverse the full graph.


- **SPDX illegal characters in SPDXID fixed** — Ecosystem values containing spaces or parentheses (e.g. `"npm (dev)"`) were embedded raw into SPDXID strings, producing IDs like `SPDXRef-Dep-1-npm (dev)-1` that violate SPDX 2.3 §2.2 (`[a-zA-Z0-9.-]` only). Illegal characters are now replaced with `-`.

- **SPDX duplicate packages in ROS scans fixed** — Mirrors the CycloneDX fix: a library shared by N ROS packages previously appeared N times as separate `PackageVersion` elements. Now deduplicated to a single element with `DEPENDS_ON` relationships from each ROS package.

- **SPDX double-prefixed originator fixed** — If `dep.author` already contained a `"Person: "` or `"Organization: "` prefix (e.g. from upstream metadata), `create_originator_field()` would emit `"Person: Person: John Doe"`. The prefix is now added only when not already present.

- **SPDX `PackageVersion: NOASSERTION` omitted in tag-value** — `PackageVersion` is an optional field in SPDX 2.3 Tag-Value format and `NOASSERTION` is not a valid value for it (unlike `PackageDownloadLocation`). The field is now omitted entirely when the version is unknown, fixing parse errors from `pyspdxtools`.

- **Tool name corrected to `radeis_sc2sbom`** — CycloneDX `metadata.tools.components[0].name` and SPDX `CreatedBy` were emitting the old internal name `sourcecode_to_sbom`. Both now emit `radeis_sc2sbom` to match the actual binary name.

### Detection Improvements

- **MicroPython files no longer misclassified as PyPI** — Python files using MicroPython-specific imports (`lvgl`, `utime`, `ustruct`, `lodepng`, `SDL`, etc.) are now detected as `micropython` ecosystem instead of `pip`. This prevents false PyPI entries for modules that do not exist on PyPI.
  ```
  Before: pkg:pypi/utime  (incorrect — utime is a MicroPython built-in)
  After:  pkg:generic/utime?type=micropython
  ```

- **`library.json` files now parsed** — Vendored C/C++ libraries that include a PlatformIO/Arduino-style `library.json` file are now detected and included in the SBOM with their correct name, version, and repository URL.
  ```
  New:  lv_drivers @ 7.11.0  (vendored, from library.json)
  ```

- **System library / pkg-config deduplication** — When both a Makefile `-lFoo` flag (version unknown) and a `foo.pc` pkg-config file (version known) refer to the same library, the versioned pkg-config entry now wins and the unversioned system entry is dropped.
  ```
  Before: SDL2 @ unspecified (system) + sdl2 @ 2.0.12 (pkg-config)  ← duplicate
  After:  sdl2 @ 2.0.12 (pkg-config)
  ```

### Impact

No changes to CLI flags. All fixes are in the serialization and parser layers. Existing projects will produce cleaner, spec-compliant SBOMs with more accurate dependency detection after upgrading.

---


### Overview


### Changes

- **Output is clean by default**
  - Risk assessment section omitted from markdown report

### Migration

```bash
./radeis_sc2sbom --path .


./radeis_sc2sbom --path . \
```

---

## v1.0.6 - Production SBOM Filtering with Automated Dependency Scope Classification (2026-03-04)

### Overview

Introduces automated dependency scope classification, enabling production-ready SBOMs that contain only the packages actually needed at runtime. The new `--production` flag and `--scope-filter` option let you dramatically reduce SBOM size without losing accuracy.

### Key Features

#### Automated Scope Classification
- **6 scope types**: Runtime, Build, Test, Development, Optional, Provided
- **Multi-heuristic engine**: ecosystem-based, name-based, and directory-based rules combined
- **Confidence scores**: 0.0–1.0 range with a human-readable reason per classification
- **10+ ecosystems supported**: npm, pip, cargo, SYSTEM, BUILD-CONFIG, GIT-SUBMODULE, MESON-WRAP, and more

#### Production Filtering
- **`--production`** flag: includes Runtime + Optional dependencies only
  - Example: embedded-project reduced from 106 packages → 33 (68.9% reduction)
- **`--scope-filter <SCOPE>`**: select any combination of scope types
  - Multiple values supported: `--scope-filter runtime --scope-filter optional`
- **Scope statistics** shown in console and markdown reports (count + percentage per scope)

#### Enhanced SBOM Output
- **SPDX 2.3**: `primaryPackagePurpose` field populated from scope classification
- **CycloneDX 1.5**: `scope` field populated from scope classification

#### Validated Classification Accuracy
| Category | Accuracy | Examples |
|---|---|---|
| Build tools | 100% | cmake, gcc, ninja, meson |
| Test frameworks | 100% | pytest, jest, gtest, unity |
| Dev tools | 100% | pylint, black, eslint, prettier |
| Runtime libraries | High | zlib, curl, openssl, protobuf |

### Test Coverage
- **609 total tests passing** (203 lib + 203 bin + 200 integration + 3 doc)
- **42 new integration tests** covering scope filtering, production mode, and real-world classification

### Backward Compatibility
- Default behavior is unchanged from v1.0.5 — scope classification runs automatically but does not filter output unless `--production` or `--scope-filter` is specified.

---

## v1.0.5 - Enhanced Version Extraction with Dual-Mode .mk Scanning (2026-02-26)

### Overview

Solves the "unspecified version" problem for system/runtime libraries detected from Makefiles while also enabling independent .mk file manifest parsing for build system repositories. Features **intelligent two-mode architecture** that automatically adapts to different repository types without configuration. This release adds comprehensive .mk file parsing with dual operating modes plus optional .so binary scanning for post-build version extraction.

### Problem Statement

**Before v1.0.5:**
```json
{
  "name": "z",
  "version": "unspecified",  // ❌ No version information
  "ecosystem": "system"
}
```

**After v1.0.5 (with .mk file scanning):**
```json
{
  "name": "z",
  "version": "1.3.1",  // ✅ Extracted from zlib.mk
  "ecosystem": "system",
  "source_file": "... [version from .mk file: 1.3.1]"
}
```

### Key Features

#### Two-Mode Architecture

The .mk file scanner automatically selects the appropriate mode based on repository structure:

**Mode 1: Version Resolution** (Application Projects)
- **Trigger:** Makefile exists with `-l` flags detecting system libraries
- **Process:** Resolves versions for Makefile-detected libraries from .mk files
- **Ecosystem:** "system" (preserves Makefile detection context)
- **Example:** embedded-project project - 16 system libraries upgraded from "unspecified" to precise versions
- **Use Case:** Application projects that link against system libraries

**Mode 2: Independent Manifest Parsing** (Build System Repositories)
- **Trigger:** .mk files exist without Makefile (or no libraries detected in Makefile)
- **Process:** Creates dependencies directly from ALL `*_VERSION` variables in .mk files
- **Ecosystem:** "BUILD-CONFIG" (indicates build system source code)
- **Example:** embedded-toolchains project - 35 BUILD-CONFIG packages detected with 100% version coverage
- **Use Case:** Build system repositories that define build-time dependencies

**Automatic Deduplication:**
- When both modes detect the same library, Mode 1 ("system") takes precedence over Mode 2 ("BUILD-CONFIG")
- Prevents duplicate entries while maintaining accurate dependency classification
- Ecosystem-aware logic in `deduplicate_dependencies()` function

#### .mk File Parsing

Extracts version information from build configuration files common in embedded systems.

**Discovery Strategy:** Uses glob pattern `**/*.mk` to find .mk files **anywhere** in the repository, without assumptions about directory structure.

**Example .mk file:**
```makefile
# toolchains/3rd_party/curl/curl.mk (or any location)
CURL_VERSION ?= 8.15.0
CURL_NAME := curl-$(CURL_VERSION)
LIBCURL_SO := $(LIBCURL).4.8.0
```

**Mapping Strategy:**
1. Detect library from Makefile: `-lcurl` → library name: "curl"
2. Search for .mk files using glob: `**/*.mk` (finds curl.mk anywhere in repo)
3. Parse all .mk files and extract `CURL_VERSION ?= 8.15.0`
4. Map version variable to library: `CURL_VERSION` → "curl" → "8.15.0"
5. Update dependency version: "curl" @ "8.15.0"

**Supported Patterns:**
- `VAR_VERSION ?= value` (conditional assignment)
- `VAR_VERSION := value` (simple assignment)
- `VAR_VERSION = value` (recursive assignment)

**Library Name Normalization:** (Mode 1 only - for resolving Makefile `-l` flags)
- `z` → `zlib`
- `ssl` → `openssl`
- `ssh2` → `libssh2`
- `pcap` → `libpcap`
- `xml2` → `libxml2`
- `pthread` → `pthreads`
- `m` → `libm`
- `dl` → `libdl`
- `rt` → `librt`
- `jpeg` → `libjpeg`
- `png` → `libpng`
- Generic: `foo` → `libfoo` (tries both variants)

**Build Tool Filtering:** (Mode 2 only - prevents false positives)
- Filters out build tools: make, cmake, gcc, clang, python, perl, ruby, autoconf, automake, libtool, ninja, meson, bash, sh, awk, sed
- Prevents false positive dependencies like "make@4.3" or "cmake@3.25.0" appearing in SBOM

#### .so Binary Scanning (Priority 2)

Extracts version from built library binaries (post-build approach).

**Techniques:**
1. **Parse .so filename:** `libcurl.so.4.8.0` → version "4.8.0"
2. **Read ELF soname:** `readelf -d libcurl.so | grep SONAME` (if readelf available)
3. **Extract version strings:** Search binary content for version patterns

**Search Directories:**
- `lib/`
- `lib64/` (64-bit libraries)
- `build/`
- `build/lib/` (CMake out-of-tree builds)
- `toolchains/install/lib/`
- `usr/lib/`
- `usr/lib64/` (64-bit system libraries)
- `usr/local/lib/` (local installations)
- `.libs/` (autotools)

**Symlink Deduplication:**
- Automatically resolves symlink chains (e.g., `libcurl.so` → `libcurl.so.4` → `libcurl.so.4.8.0`)
- Uses canonical paths to prevent scanning the same library multiple times
- Ensures consistent version reporting across symlinked libraries

**Limitation:** Requires libraries to be already built (not suitable for source-only repos).

### CLI Flags

```bash
# .mk file version extraction (enabled by default)
--scan-mk-files=true/false     # Default: true

# .so binary version extraction (disabled by default, requires built libraries)
--scan-so-files=true/false     # Default: false
```

**Rationale for defaults:**
- `--scan-mk-files=true`: Safe for source-only repos, low overhead, works with any .mk file location
- `--scan-so-files=false`: Requires built libraries, may not exist in CI/CD

### Real-World Impact

**embedded-project Project:**
- **Before v1.0.5:** 32 system libraries with "unspecified" versions
- **After v1.0.5:** 32 system libraries with precise versions extracted from .mk files
  - curl @ 8.15.0
  - elfutils @ 0.191
  - zlib @ 1.3.1
  - openssl @ 3.2.5
  - libssh2 @ 1.11.0
  - And 27 more...

### Backward Compatibility

✅ **100% backward compatible**
- Existing projects without .mk files: No change in behavior
- Default `--scan-mk-files=true` only affects Makefile-based projects
- Projects without .mk files continue to show "unspecified" versions
- Version resolution is additive: no existing versions are replaced

### Testing

- **154 unit tests passing** - All existing tests + new .mk and .so parser tests
- **3 new integration tests** - Mode 1/Mode 2 deduplication, build tool filtering, multi-file scenarios
- **Comprehensive code review** - All issues addressed (deduplication logic, symlink handling, directory traversal optimization)
- **Integration tested** with embedded-project and embedded-toolchains project structures
- **Zero regressions** in existing parsers

### Files Modified

**New Files:**
- `src/parsers/c/mk_file.rs` - .mk file parser with version extraction
- `src/parsers/c/so_scanner.rs` - .so binary scanner with version extraction

**Modified Files:**
- `src/parsers/c/makefile.rs` - Added version resolution logic for Mode 1
- `src/parsers/mod.rs` - Added ecosystem-aware deduplication for Mode 1/Mode 2
- `src/cli.rs` - Added `--scan-mk-files` and `--scan-so-files` flags
- `src/scanner/mod.rs` - Pass new flags to Makefile parser + Mode 2 trigger
- `src/main.rs` - Wire CLI flags through to scanner
- `Cargo.toml` - Added `glob` dependency for .mk file discovery
- `tests/parser_tests/c_tests.rs` - Added integration tests for deduplication and filtering

### Future Enhancements (v1.0.6+)

1. **Smart .mk pattern detection** - Learn .mk file patterns from multiple projects
2. **pkg-config .pc generation** - Generate .pc files from .mk files for other tools
3. **Build system plugin architecture** - Support custom build systems via plugins
4. **Version constraint resolution** - Resolve ">=3.0" constraints to actual versions
5. **License extraction from .mk files** - Extract license info from build configuration

---

## v1.0.4 - Meson & Bazel Build Systems (2026-02-25)

### Overview

Adds support for modern C/C++ build systems (Meson and Bazel), completing radeis_sc2sbom's comprehensive C/C++ ecosystem coverage. Combined with v1.0.0-1.0.3 (vcpkg, Conan, CMake, Git submodules, Autotools, pkg-config, Makefiles), radeis now supports **~95% of C/C++ projects**.

### Key Features

#### Meson Build System Support

Parses `meson.build` files for `dependency()` declarations:

```python
# meson.build example
project('myapp', 'cpp')

# dependency() with version constraint
zlib_dep = dependency('zlib', version: '>=1.2.11')

# dependency() without version
openssl_dep = dependency('openssl')

# System library via find_library()
cc = meson.get_compiler('c')
math_dep = cc.find_library('m')

# Subproject reference
libfoo_proj = subproject('libfoo')

executable('myapp', 'src/main.cpp',
  dependencies: [zlib_dep, openssl_dep, math_dep])
```

**Supported Features:**
- Extract library names from `dependency()` declarations
- Extract version constraints (>=, ==, >, <, !=) from `version:` arguments (when present)
- Extract system libraries from `cc.find_library()` calls
- Detect subproject references from `subproject()` calls (actual resolution via `.wrap` files)
- PURL format: `pkg:generic/{name}@{version}?type=meson` (version included when available)

**Note:** The parser currently extracts dependency names and version constraints. Advanced features like `modules:` arrays and `required:` flags are recognized syntactically but not currently captured in structured output.

**Real-World Validation:**
- OpenStudio project includes meson 1.2.2 as dev dependency in conan.lock
- Successfully detected and validated in production scan
- Demonstrates immediate relevance for C++ projects transitioning to Meson

#### Bazel Build System Support

Parses `WORKSPACE`/`WORKSPACE.bazel` and `MODULE.bazel` files for external dependencies:

```python
# WORKSPACE example
http_archive(
    name = "com_google_googletest",
    urls = ["https://github.com/google/googletest/archive/release-1.12.1.tar.gz"],
    strip_prefix = "googletest-release-1.12.1",
)

git_repository(
    name = "com_google_absl",
    remote = "https://github.com/abseil/abseil-cpp.git",
    tag = "20230802.1",
)

# MODULE.bazel example (Bazel 6.0+ bzlmod)
module(name = "myproject")

bazel_dep(name = "abseil-cpp", version = "20230802.1")
bazel_dep(name = "googletest", version = "1.14.0")
```

**Supported Features:**
- Parse `WORKSPACE`/`WORKSPACE.bazel` for `http_archive`, `git_repository`, and local repositories
- Parse `MODULE.bazel` (Bazel 6.0+ bzlmod) for `bazel_dep()` declarations
- Extract URLs and versions from external dependency declarations
- Support for multi-line dependency declarations
- PURL format: `pkg:generic/{name}@{version}?type=bazel` or `pkg:github/{owner}/{repo}@{version}?type=bazel` for git-based deps

### CLI Flags

```bash
# Enable/disable Meson and Bazel support (both enabled by default)
--scan-meson=true/false        # meson.build and .wrap files
--scan-bazel=true/false        # WORKSPACE, WORKSPACE.bazel, MODULE.bazel files
```

### Coverage Impact

**Before v1.0.4:**
- Modern C++ (vcpkg, Conan, CMake): ~70-80%
- Legacy C (Autotools, pkg-config, Makefiles): ~80-90%
- Combined Coverage: **~90% of C/C++ projects**

**After v1.0.4:**
- Modern C++ (vcpkg, Conan, CMake, Meson, Bazel): ~75-85%
- Legacy C (Autotools, pkg-config, Makefiles): ~80-90%
- Combined Coverage: **~95% of C/C++ projects**

### Real-World Project Support

Successfully tested with:
- **OpenStudio** (Conan) - Detected meson 1.2.2 as dev dependency
- **Unit tests** - 105 tests passing (Meson and Bazel parsers)

### Production Validation

**OpenStudio Scan Results (v1.0.4):**
- 49 total packages (48 Conan + 1 Python)
- Meson 1.2.2 detected in conan.lock as dev dependency
- Complete conan.lock parsing with dev dependency classification

This validates that v1.0.4's Meson support is immediately relevant for real-world C++ projects using modern build systems.

### Backward Compatibility

✅ **100% Maintained** - No regressions in existing ecosystem parsers:
- All 105 unit tests passing
- curl results identical to v1.0.3 (verified)
- npm, Python, ROS, Conan, Git submodules, CMake, Autotools all stable
- Meson and Bazel parsers are additive only

### Comprehensive Comparison Reports

As part of v1.0.4 validation, we've completed comprehensive comparison reports (375-596 lines each) for 6 diverse repositories:

1. **curl** (C library) - 446 lines
2. **nodejs-service** (Node.js) - 444 lines
3. **nodejs-project** (Multi-cloud) - 375 lines
4. **OpenStudio** (C++ Conan) - 398 lines
5. **mrpt** (Robotics C++) - 596 lines
6. **ros2cli** (ROS 2) - 590 lines

**Total:** 2,897 lines of comprehensive analysis

**Key Findings:**
- **2,561 total dependencies** tracked across all projects
- **4 unique capabilities** no other tool provides:
  - C/C++ Autotools (curl: 29 libs)
  - ROS 2 (ros2cli: 223 components)
  - Git submodules (mrpt: 8 submodules with SHAs)
  - CMake ExternalProject (mrpt: 3 deps)
- **$220K-$1.65M savings** vs BlackDuck over 3 years

See [scan_reports/COMPARISON_REPORTS_INDEX.md](../scan_reports/COMPARISON_REPORTS_INDEX.md) for all reports.

---

## v1.0.3 - C Legacy Support (pkg-config + Autotools + Makefile) (2026-02-24)

### Overview

Adds comprehensive support for traditional C project build systems, enabling SBOM generation for legacy C/C++ projects using GNU Autotools, pkg-config, and plain Makefiles. This fills the critical gap left by modern package manager support (vcpkg, Conan, CMake), achieving ~90% coverage of C/C++ projects.

### Key Features

#### pkg-config (.pc file) Support

Parses `.pc` (pkg-config) files to extract system library dependencies:

```
Name: OpenSSL
Version: 3.0.2
Description: Secure Sockets Layer and cryptography libraries
Requires: libcrypto libssl
```

**Supported Features:**
- Extract package name, version, and description from .pc files
- Detect PKG_CHECK_MODULES() calls in configure.ac
- Detect pkg-config shell invocations in Makefiles
- PURL format: `pkg:generic/{name}@{version}?type=pkg-config`

#### Autotools (configure.ac/Makefile.am) Support

Parses GNU Autotools configuration files for library dependencies:

**configure.ac:**
```bash
AC_CHECK_LIB([pthread], [pthread_create])
AC_SEARCH_LIBS([sqrt], [m])
PKG_CHECK_MODULES([GLIB], [glib-2.0 >= 2.50])
```

**Makefile.am:**
```makefile
myapp_LDADD = -lssl -lcrypto -lpthread
libfoo_la_LIBADD = -lz
```

**Supported Features:**
- Extract dependencies from AC_CHECK_LIB, AC_SEARCH_LIBS, PKG_CHECK_MODULES
- Extract -l flags from LDADD/LIBADD variables in Makefile.am
- Version constraints preserved from PKG_CHECK_MODULES
- PURL format: `pkg:generic/{name}@{version}?type=autotools`

#### Plain Makefile Heuristic Parser

Best-effort parsing of handwritten Makefiles:

```makefile
LDFLAGS = -lssl -lcrypto -lpthread -lz
OPENSSL_CFLAGS = $(shell pkg-config --cflags openssl)
```

**Supported Features:**
- Extract -l flags (system libraries) using regex
- Detect pkg-config invocations
- Deduplication by library name
- PURL format: `pkg:generic/{name}@{version}?type=makefile`

**Limitations:**
- No variable expansion (`$(FOO)`)
- No conditional blocks (`ifeq`)
- Skip Makefile parsing in Autotools projects

### CLI Flags

```bash
# Enable/disable C legacy support (all enabled by default)
--scan-pkgconfig=true/false       # .pc files and PKG_CHECK_MODULES
--scan-autotools=true/false       # configure.ac and Makefile.am
--scan-makefiles=true/false       # Plain Makefiles (heuristic)
--resolve-system-deps=false       # System pkg-config resolution (disabled by default)
```

### Coverage Impact

**Before v1.0.3:**
- Modern C++ (vcpkg, Conan, CMake): ~70-80%
- Legacy C (Autotools): <1%
- System libraries: <1%

**After v1.0.3:**
- Modern C++ (vcpkg, Conan, CMake): ~70-80%
- Legacy C++ (Makefiles): ~40%
- Pure C (Autotools): ~60%
- System library deps: ~80%

**Combined Coverage: ~90% of C/C++ projects**

### Real-World Project Support

Successfully tested with:
- **curl** (Autotools) - Detected openssl, zlib, nghttp2
- **nginx** (Makefile) - Detected openssl, pcre, zlib
- Standard C libraries (pthread, m, ssl, crypto, z)

### Example Output

```
PKG-CONFIG (3 packages)
├── openssl @ >=3.0 [direct]
├── OpenSSL @ 3.0.2 [direct]
└── glib-2.0 @ >=2.50 [direct]

AUTOTOOLS (5 packages)
├── ssl @ unspecified [direct]
├── crypto @ unspecified [direct]
├── z @ unspecified [direct]
├── m @ unspecified [direct]
└── pthread @ unspecified [direct]
```

### PURL Examples

- **pkg-config**: `pkg:generic/openssl@3.0.2?type=pkg-config`
- **Autotools**: `pkg:generic/pthread@unspecified?type=autotools`
- **Makefile**: `pkg:generic/ssl@unspecified?type=makefile`

---

## v1.0.2 - Conan C++ Package Manager Support (2026-02-24)

### Overview

Adds complete support for the Conan C/C++ package manager, enabling SBOM generation for projects using Conan 2.x. Parses lock files, INI-format manifests, and Python-format manifests with support for runtime, build, tool, and test dependencies.

### Key Features

#### Conan Lock File Parsing (conan.lock)

Parses Conan 2.x lock files with exact versions and recipe revisions:

```json
{
  "version": "0.5",
  "requires": [
    "zlib/1.2.13#416618fa04d433c6bd94279ed2e93638%1680515805"
  ],
  "build_requires": ["cmake/3.27.0"],
  "tool_requires": ["ninja/1.11.1"],
  "test_requires": ["gtest/1.14.0"]
}
```

**Supported Features:**
- Extract package name and version from references
- Recipe revision hash stored as checksum
- Distinguish between runtime, build, tool, and test dependencies
- Lock file takes precedence over manifests in the same directory

#### Conan Manifest Parsing (conanfile.txt)

Parses INI-format Conan manifests:

```ini
[requires]
zlib/1.2.13
openssl/[>=3.0]
boost/1.82.0

[build_requires]
cmake/3.27.0

[tool_requires]
ninja/1.11.1

[test_requires]
gtest/1.14.0
```

**Supported Features:**
- Version constraints: `[>=3.0]`, `[>1.0 <2.0]`, `[~=1.82]`, `[^1.0]`
- User/channel notation: `package/version@user/channel`
- Build, tool, and test dependencies marked as dev dependencies

#### Conan Python Manifest Parsing (conanfile.py)

Parses Python-format Conan manifests using regex extraction:

```python
from conan import ConanFile

class MyProjectConan(ConanFile):
    requires = ["zlib/1.2.13", "openssl/3.1.2"]
    build_requires = ["cmake/3.27.0"]

    def requirements(self):
        self.requires("boost/1.82.0")

    def build_requirements(self):
        self.build_requires("doxygen/1.9.8")
```

**Supported Patterns:**
- List format: `requires = ["dep1", "dep2"]`
- Method calls: `self.requires("dep")`
- Build requirements: `build_requires`, `self.build_requires()`
- Tool requirements: `tool_requires`, `self.tool_requires()`
- Test requirements: `test_requires`, `self.test_requires()`

#### SBOM Output

```json
{
  "name": "zlib",
  "versionInfo": "1.2.13",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:conan/zlib@1.2.13"
  }],
  "checksums": [{
    "algorithm": "SHA256",
    "checksumValue": "416618fa04d433c6bd94279ed2e93638"
  }],
  "sourceInfo": "conan/lock extractor from conan.lock"
}
```

### Technical Implementation

**Files Added:**
- [src/parsers/cpp/conan.rs](../src/parsers/cpp/conan.rs) - conan.lock parser
- [src/parsers/cpp/conan_manifest.rs](../src/parsers/cpp/conan_manifest.rs) - conanfile.txt/py parsers
- [tests/parser_tests/conan_tests.rs](../tests/parser_tests/conan_tests.rs) - 10 test cases

**Integration:**
- Scanner detects `conan.lock`, `conanfile.txt`, `conanfile.py`
- Lock file precedence: skips manifests if `conan.lock` exists
- SPDX purl format: `pkg:conan/{name}@{version}`
- CycloneDX component support

### Test Coverage

```bash
cargo test conan_tests
```

**10 test cases covering:**
- Lock file parsing with recipe revisions
- INI manifest parsing with version constraints
- Python manifest parsing (list and method formats)
- Version range handling
- User/channel notation
- Malformed input handling
- Empty manifest handling
- purl format generation

### Examples

```bash
# Scan project with Conan dependencies
./target/release/radeis_sc2sbom --path /path/to/conan-project --format spdx-json

# Example output
{
  "packages": [
    {
      "name": "zlib",
      "versionInfo": "1.2.13",
      "externalRefs": [{"referenceLocator": "pkg:conan/zlib@1.2.13"}]
    },
    {
      "name": "cmake",
      "versionInfo": "3.27.0",
      "externalRefs": [{"referenceLocator": "pkg:conan/cmake@3.27.0"}],
      "properties": [{"name": "dev-dependency", "value": "true"}]
    }
  ]
}
```

### Design Decisions

1. **Lock File Precedence**: `conan.lock` provides exact versions and is trusted over manifests
2. **Recipe Revisions**: Stored in `checksum_sha256` field for traceability
3. **Dev Dependencies**: Build, tool, and test requirements marked with `is_dev: true`
4. **Version Constraints**: Preserved as-is from manifests (e.g., `>=3.0`)
5. **Ecosystem Identifier**: `"conan"` for all Conan dependencies

---

## v1.0.1 - CMake Support & Recursive Submodule Scanning (2026-02-23)

### Overview

Adds static CMake dependency detection (FetchContent/ExternalProject) and recursive dependency scanning inside Git submodules. Enables complete dependency discovery without requiring CMake build execution.

### Key Features

#### CMake Dependency Parsing

Static parsing of CMakeLists.txt files to extract FetchContent_Declare and ExternalProject_Add dependencies:

```cmake
# FetchContent_Declare (Modern CMake 3.11+)
FetchContent_Declare(
  json
  GIT_REPOSITORY https://github.com/nlohmann/json.git
  GIT_TAG        v3.11.2
)

# ExternalProject_Add (Legacy pattern)
ExternalProject_Add(
  zlib
  URL https://zlib.net/zlib-1.2.13.tar.gz
  URL_HASH SHA256=abc123...
)
```

**Supported Features:**
- GIT_REPOSITORY + GIT_TAG extraction
- URL-based dependencies with version extraction from URLs
- URL_HASH (SHA256) checksum extraction
- Skips entries with CMake variables `${VAR}` (cannot be resolved statically)
- Multi-host Git URL parsing (GitHub, GitLab, Bitbucket, generic)

**SBOM Output:**
```json
{
  "name": "nlohmann/json",
  "versionInfo": "v3.11.2",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:github/nlohmann/json@v3.11.2"
  }],
  "sourceInfo": "cmake/fetchcontent extractor from CMakeLists.txt"
}
```

#### Recursive Submodule Scanning

Automatically scans dependencies inside Git submodules (package.json, Cargo.toml, CMakeLists.txt, etc.) with depth limiting:

**Example:** Project with submodule containing npm dependencies
```
.
├── .gitmodules
├── libs/
│   └── json/          # Git submodule
│       ├── package.json   # Scanned recursively
│       └── CMakeLists.txt # Also scanned
```

**Features:**
- Detects all manifest types inside submodules (npm, Cargo, vcpkg, CMake, etc.)
- Nested submodule support (submodules within submodules)
- Configurable depth limit (default: 3 levels) via `--submodule-depth`
- Source attribution shows submodule origin

**SBOM Output for nested dependencies:**
```json
{
  "name": "typescript",
  "versionInfo": "5.0.0",
  "ecosystem": "npm",
  "sourceInfo": "javascript/packagejson extractor from libs/json/package.json (submodule: libs/json)"
}
```

#### CLI Enhancements

**New flag:**
- `--scan-cmake=<true|false>` - Enable/disable CMake scanning (default: true)

**Existing flags (now fully functional):**
- `--submodule-depth=<N>` - Maximum recursion depth for nested submodules (default: 3)

**Example:**
```bash
# Scan with CMake and recursive submodules (default)
./target/release/radeis_sc2sbom /path/to/project

# Disable CMake scanning
./target/release/radeis_sc2sbom /path/to/project --scan-cmake=false

# Limit submodule recursion to 1 level
./target/release/radeis_sc2sbom /path/to/project --submodule-depth=1
```

### Technical Details

#### CMake Parser Implementation
- **Files:** [src/parsers/cmake/mod.rs](../src/parsers/cmake/mod.rs), [src/parsers/cmake/fetchcontent.rs](../src/parsers/cmake/fetchcontent.rs), [src/parsers/cmake/external_project.rs](../src/parsers/cmake/external_project.rs)
- **Pattern:** Regex-based parsing with `(?is)` flags (case-insensitive + multiline)
- **Limitation:** Cannot resolve CMake variables - requires static values only

#### Recursive Scanning Implementation
- **Function:** `scan_submodule_recursively()` in [src/scanner/mod.rs](../src/scanner/mod.rs)
- **Safety:** Depth limit prevents infinite loops from circular references
- **Coverage:** All existing parsers (npm, Cargo, Python, Go, vcpkg, CMake, etc.)

### Competitive Advantages

|---------|----------------|-------------|-----------|
| **CMake FetchContent** | ✅ Static parsing | ❌ | ✅ Build capture only |
| **CMake ExternalProject** | ✅ Static parsing | ❌ | ✅ Build capture only |
| **Nested deps in submodules** | ✅ Recursive scanning | ❌ | ❌ |
| **No build required** | ✅ All static | ✅ | ❌ CMake needs build |

### Migration Notes

**API Changes:**
- `scan_directory()` signature updated with `scan_cmake` parameter (breaks custom integrations)
- Tests updated to include `scan_cmake` argument

**Breaking Changes:** None for CLI users (backward compatible)

---

## v1.0.0 - C++ Support (2026-02-20)

### Overview

First C++ ecosystem support with vcpkg manifest parsing and Git submodule detection. Enables SBOM generation for C/C++ projects using modern package managers.

### Key Features

#### vcpkg Manifest Parser

Full vcpkg.json support with all version constraint formats:

```json
{
  "name": "my-project",
  "dependencies": [
    "zlib",
    { "name": "openssl", "version>=": "3.0" },
    { "name": "boost", "version-semver": "1.82.0", "features": ["filesystem"] },
    { "name": "fmt", "version-date": "2023-01-15" }
  ],
  "overrides": [
    { "name": "zlib", "version": "1.2.13" }
  ]
}
```

**Supported version formats:**
- Simple string dependencies: `"zlib"`
- `version>=`: Minimum version constraint
- `version>`: Greater than version
- `version=`: Exact version
- `version-semver`: Semantic version
- `version-date`: Date-based version
- `port-version`: Port revision number
- Overrides section for version pinning
- Features stored in source_file metadata

**SBOM Output:**
```json
{
  "name": "boost",
  "versionInfo": "1.82.0",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:vcpkg/boost@1.82.0"
  }],
  "sourceInfo": "cpp/vcpkg extractor from vcpkg.json [filesystem]"
}
```

#### Git Submodule Detection

Automatic parsing of `.gitmodules` files with commit SHA resolution:

```ini
[submodule "libs/json"]
    path = libs/json
    url = https://github.com/nlohmann/json.git
    branch = master
```

**Features:**
- Parse submodule name, path, URL, and branch
- Resolve commit SHAs via `git ls-tree HEAD`
- Support for HTTPS and SSH URLs
- Multi-host support: GitHub, GitLab, Bitbucket, self-hosted

**SBOM Output:**
```json
{
  "name": "nlohmann/json",
  "versionInfo": "bc889af",
  "externalRefs": [{
    "referenceType": "purl",
    "referenceLocator": "pkg:github/nlohmann/json@bc889af"
  }],
  "sourceInfo": "git-submodule extractor from .gitmodules"
}
```



```
   Package version 'detected' is unknown (detected via import scanning).
   Actual version may not be affected.
```

This helps distinguish between:

#### Version Format Fix

- **Before:** `"==2.6.0"` (invalid purl format)
- **After:** `"2.6.0"` (correct purl format)


#### Python Detection Methodology

radeis scans **declared dependencies** from manifest files (requirements.txt, pyproject.toml), producing more accurate SBOMs than tools that scan installed environments:

| Approach | radeis | Environment scanners |
|----------|--------|---------------------|
| Detection Method | Declared dependencies | Installed packages |
| What it captures | Project requirements | Environment-specific packages |
| Accuracy | More accurate | May include false positives |

**Key insight:** Environment scanners may include packages like `importlib-metadata` and `zipp` (Python < 3.10 backports) that aren't actual project requirements.

### New CLI Options

```bash
--scan-submodules <BOOL>   # Enable Git submodule scanning (default: true)
--submodule-depth <N>      # Maximum recursion depth (default: 3)
```

**Examples:**
```bash
# Scan C++ project with vcpkg
radeis_sc2sbom --path ./cpp_project --format spdx-json

# Disable submodule scanning
radeis_sc2sbom --path . --scan-submodules false

# Limit submodule recursion depth
radeis_sc2sbom --path . --submodule-depth 1
```

### purl Format Support

| Ecosystem | purl Format | Example |
|-----------|-------------|---------|
| vcpkg | `pkg:vcpkg/{name}@{version}` | `pkg:vcpkg/zlib@1.2.13` |
| GitHub | `pkg:github/{owner}/{repo}@{commit}` | `pkg:github/nlohmann/json@bc889af` |
| GitLab | `pkg:gitlab/{owner}/{repo}@{commit}` | `pkg:gitlab/owner/repo@abc123` |
| Bitbucket | `pkg:bitbucket/{owner}/{repo}@{commit}` | `pkg:bitbucket/owner/repo@def456` |
| Generic | `pkg:generic/{name}@{version}` | `pkg:generic/custom-lib@1.0.0` |

### Technical Changes

**New Files:**
- `src/parsers/cpp/mod.rs` - C++ parser module entry point
- `src/parsers/cpp/vcpkg.rs` - vcpkg.json parser
- `src/parsers/git/mod.rs` - Git parser module entry point
- `src/parsers/git/submodules.rs` - .gitmodules parser
- `src/parsers/git/commit_resolver.rs` - Git commit SHA resolver
- `src/parsers/git/url_parser.rs` - Git URL parser

**Modified Files:**
- `src/cli.rs` - New CLI options
- `src/scanner/mod.rs` - vcpkg.json and .gitmodules detection
- `src/formats/spdx.rs` - vcpkg/git-submodule purl support
- `src/parsers/python.rs` - Version format fix (strip operators)
- `src/parsers/mod.rs` - Added `__future__` to Python stdlib
- `src/main.rs` - Pass new args to scanner

### Migration from v0.9.3

**No breaking changes.** All existing features work unchanged.

New features activate automatically when:
- `vcpkg.json` is found → vcpkg parser runs
- `.gitmodules` is found → submodule detection runs (if git available)

### Future Roadmap (v1.0.x)

- v1.0.1: CMake FetchContent/ExternalProject parsing
- v1.0.2: Conan package manager support
- Recursive scanning inside submodules

---

## v0.9.3 - Python Excellence (2026-02-10)

### Overview

Complete Pipenv and pyproject.toml support with **487% improvement** in Python package detection.

### Key Improvements

#### Pipfile/Pipfile.lock Parser
Full Pipenv integration with lock file parsing.

**Before:**
```json
{
  "name": "azure-identity",
  "versionInfo": "detected",
  "sourceInfo": "Import scanner"
}
```

**After:**
```json
{
  "name": "azure-identity",
  "versionInfo": "1.12.0",
  "downloadLocation": "https://pypi.org/project/azure-identity/1.12.0/",
  "sourceInfo": "python/pipfilelock extractor"
}
```

**Results (nodejs-service):**
- 47 packages from Pipfile.lock (vs 8 before)
- 100% version accuracy (zero "@detected")
- Perfect parity with Black Duck (47/47)
- SHA256 checksums extracted

#### pyproject.toml Multi-Format Parser
Supports three formats automatically:

**PEP 621 (Standard):**
```toml
[project]
dependencies = ["requests>=2.28.0", "click>=8.0.0"]

[project.optional-dependencies]
dev = ["pytest>=7.0"]
```

**Poetry:**
```toml
[tool.poetry.dependencies]
python = "^3.8"
requests = "^2.28.0"

[tool.poetry.dev-dependencies]
pytest = "^7.0"
```

**PDM:**
```toml
[tool.pdm]
dependencies = ["requests>=2.28.0"]
```

#### SHA256 Checksum Extraction
Extracts checksums from lock files for supply chain security:
- Pipfile.lock: First SHA256 from hashes array
- poetry.lock: Hash from files[0].hash field
- Stored internally (not yet in SPDX/CycloneDX output)

#### Duplicate Prevention
Lock files now skip corresponding manifest files:
- Pipfile.lock exists → Skip Pipfile
- poetry.lock exists → Skip pyproject.toml

### Performance Metrics

| Metric | v0.9.3 | v0.9.2 | Improvement |
|--------|--------|--------|-------------|
| Python packages | 47 | 8 | +487% |
| Real versions | 47 | 0 | +47 |
| "@detected" versions | 0 | 8 | -100% |
| Black Duck parity | 100% | 17% | +83% |

### Competitive Position

| Tool | Packages | Version Accuracy | Checksums | Formats |
|------|----------|------------------|-----------|---------|
| **radeis v0.9.3** | 47 🏆 | 100% 🏆 | ✅ SHA256 🏆 | Pipfile, poetry, pyproject 🏆 |
| Black Duck | 47 | 100% | Unknown | Pipfile, poetry |

### New File Support

**Pipfile.lock (Lock File)**
- Exact versions (`==1.12.0`)
- SHA256 hashes
- Direct dependency detection (`index` field)
- Dev dependencies (`develop` section)

**Pipfile (Manifest)**
- Version specs (`*`, `>=1.0`, `==1.2.3`)
- Dev dependencies
- Multiple sources

**pyproject.toml (Multi-Format)**
- PEP 621, Poetry, PDM formats
- Auto-detection
- Version specs and extras

### Technical Changes

**Modified Files:**
- `src/parsers/python.rs` (+219 lines) - New parsers
- `src/scanner/mod.rs` (+30 lines) - Parser registration
- `Cargo.toml` - Version bump to 0.9.3

**Key Algorithms:**
```rust
// Pipfile.lock structure
#[derive(Deserialize)]
struct PipfileLock {
    default: HashMap<String, PipfilePackage>,
    develop: Option<HashMap<String, PipfilePackage>>,
}

// Checksum extraction
fn extract_first_sha256(hashes: &[String]) -> Option<String> {
    hashes.iter()
        .find(|h| h.starts_with("sha256:"))
        .map(|h| h.trim_start_matches("sha256:").to_string())
}

// Multi-format pyproject.toml
pub fn parse_pyproject_toml(path: &Path) -> Result<Vec<Dependency>> {
    // Try PEP 621, then Poetry, then PDM
    if let Some(project) = pyproject.get("project") { ... }
    if let Some(poetry) = pyproject.get("tool").and_then(|t| t.get("poetry")) { ... }
    if let Some(pdm) = pyproject.get("tool").and_then(|t| t.get("pdm")) { ... }
}
```

### Use Cases

**Pipenv Project:**
```bash
radeis_sc2sbom --path ./python_project --format spdx-json

# Output: 47 packages with 100% version accuracy
```

**Modern Python Project:**
```bash
radeis_sc2sbom --path ./pyproject_project --format all

# Auto-detects PEP 621, Poetry, or PDM format
```

**Supply Chain Audit:**
```bash
radeis_sc2sbom --path ./workspace --format spdx-json

# SHA256 checksums extracted from lock files
```

### Migration from v0.9.2

**No breaking changes.** All features work automatically:

```bash
# v0.9.2: 8 packages, all "@detected"
radeis_sc2sbom --path ./nodejs-service --format spdx-json

# v0.9.3: 47 packages, all with real versions
# (same command, automatic improvement)
```

**What changed:**
- ✅ Pipfile/Pipfile.lock parsed automatically
- ✅ pyproject.toml parsed (all formats)
- ✅ SHA256 checksums extracted
- ✅ 100% version accuracy
- ✅ Zero performance impact

### Bug Fixes

1. Python version detection - Eliminated "@detected" via lock files
2. Checksum extraction - Added SHA256 support
3. Parser priority - Lock files override manifests
4. Duplicate prevention - Skip manifests when lock files exist

---

## v0.9.2 - User Experience (2026-02-08)

### Progress Indicators

Real-time feedback during scanning:

```
[1/5] Walking directory tree... 47 entries scanned
[2/5] Parsing complete... 54 dependencies discovered
[3/5] Deduplicating dependencies... 54 → 47 unique
[5/5] Scan complete
```

### Features

- Progress bar with percentage completion
- Spinner animations for long operations
- 5-stage pipeline visibility
- ETA calculations

---

## v0.9.1 - ROS Integration (2026-01-28)

### Automatic Version Resolution

ROS packages lack version info in `package.xml`. v0.9.1 adds automatic resolution via rosdistro API.

**Before:**
```
rclpy @ unspecified
downloadLocation: NOASSERTION
```

**After:**
```
rclpy @ 7.1.9 (from rosdistro)
downloadLocation: https://github.com/ros2/rclpy.git
```

### ros2cli Benchmark

- 94 unique dependencies (vs BlackDuck's 4)
- 15 ROS packages detected
- 47 packages with GitHub URLs
- 62 packages with versions (66%)

### Features

- Parallel API fetching (10-27x speedup)
- SHA-1 checksums
- Compact SPDX mode (30% smaller)

**Supported distributions:** jazzy, iron, humble, rolling, noetic, melodic

---

## v0.9.1.1 - Hotfix (2026-01-28)

### Fixed: Markdown Report Display

**Problem:** Empty code blocks in ROS multi-package reports

**Fix:** Display all direct dependencies in ROS report sections

---

## v0.8.0 - Metadata & Security (2026-01-22)

### Rich Metadata

- 95%+ license coverage across ecosystems
- 90%+ supplier/originator tracking
- UUID-based SPDX IDs
- CPE identifiers for security tools

### Improvements

- Parallel metadata fetching
- Enhanced license detection
- Improved supplier tracking
- Better CPE generation

---

## Future Roadmap

### v0.9.4+
- Display checksums in SPDX/CycloneDX output
- Enhanced PyPI metadata caching
- pdm.lock support
- conda environment.yml support

---

**Latest Release:** v1.0.7 (March 12, 2026)
**Status:** Production-Ready
**Python Support:** Industry-Leading 🏆
