# Command-Line Interface Reference

Complete reference for all command-line options.

## Synopsis

```bash
radeis_sc2sbom [OPTIONS] --path <PATH>
```

## Essential Options

| Option | Default | Description |
|--------|---------|-------------|
| `--path <PATH>` | *required* | Directory to scan |
| `--format <FORMAT>` | `all` | Output format (see [Output Formats](#output-formats)) |
| `--sbom-mode <MODE>` | `complete` | SBOM output mode (see [SBOM Modes](#sbom-modes)) |
| `--vendor <MODE>` | `include` | Vendor directory handling (see [Vendor Modes](#vendor-modes)) |
| `--production` | `false` | Production mode: runtime + optional dependencies only (v1.0.6) |
| `--scope-filter <SCOPE>` | *(none)* | Filter by dependency scope (can use multiple, v1.0.6) |
| `--fallback-import-scan` | `true` | Scan source code for imports when manifests incomplete |
| `--ros-distro <DISTRO>` | `jazzy` | ROS distribution for version resolution (see [ROS Options](#ros-options)) |

## Output Formats

| Format | Description |
|--------|-------------|
| `console` | Human-readable markdown report to `./out/` |
| `spdx-json` | SPDX 2.3 JSON format |
| `spdx-tag-value` | SPDX 2.3 Tag-Value format |
| `cyclonedx-json` | CycloneDX 1.5 JSON format |
| `all` | Generate all formats (default) |

## SBOM Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| `complete` | Include all packages (default) | Compliance, inventory, supply chain tracking |

**Examples:**
```bash
# Complete SBOM with all 689 packages
radeis_sc2sbom --path . --sbom-mode complete

```

See [SBOM Modes Guide](sbom_modes_guide.md) for detailed examples and CI/CD integration patterns.

## Vendor Modes

| Mode | Description |
|------|-------------|
| `include` | Scan vendor directories (default) |
| `skip` | Ignore vendor directories |
| `only` | Scan only vendor directories |

## Visualization Options

| Option | Default | Description |
|--------|---------|-------------|
| `--tree-style <STYLE>` | `tree` | Dependency visualization style |

### Tree Styles

| Style | Description | Use Case |
|-------|-------------|----------|
| `tree` | Classic tree with box-drawing characters | Best readability |
| `flat` | Traditional list view | Compact output |
| `compact` | Minimal tree with arrows | Balance between tree and flat |


| Mode | Description | Use Case |
|------|-------------|----------|
| `tree` | Severity-first hierarchical view | Quick security assessment |
| `summary` | Counts only | Fast overview for dashboards |


| Option | Default | Description |
|--------|---------|-------------|

### Severity Levels

- `medium` - Medium, high, and critical
- `high` - High and critical only
- `critical` - Critical only

## ROS Options

| Option | Default | Description |
|--------|---------|-------------|
| `--ros-distro <DISTRO>` | `jazzy` | ROS distribution for automatic version resolution |

### Supported ROS Distributions

**ROS 2:**
- `jazzy` - ROS 2 Jazzy Jalisco (latest stable, default)
- `humble` - ROS 2 Humble Hawksbill (LTS)
- `iron` - ROS 2 Iron Irwini
- `rolling` - ROS 2 Rolling (development)

**ROS 1:**
- `noetic` - ROS Noetic Ninjemys (LTS)
- `melodic` - ROS Melodic Morenia (legacy)

### Distribution Detection Priority

1. **CLI flag** (`--ros-distro`): Explicit override
2. **Environment variable** (`$ROS_DISTRO`): Auto-detect from environment
3. **Default**: `jazzy` (latest stable ROS 2)

**Examples:**
```bash
# Explicit distribution
radeis_sc2sbom --path ./ros2cli --ros-distro humble

# Auto-detect from environment
export ROS_DISTRO=jazzy
radeis_sc2sbom --path ./ros2cli

# Use default (jazzy)
radeis_sc2sbom --path ./ros2cli
```

### ROS Version Resolution

v0.9.1 adds automatic ROS package version resolution via rosdistro GitHub API:

**Before v0.9.1:**
```
rclpy @ unspecified
launch @ unspecified
```

**After v0.9.1:**
```
rclpy @ 7.1.9 (from rosdistro)
launch @ 3.6.1 (from rosdistro)
```

**Features:**
- Resolves 60+ package versions automatically
- Populates SPDX `downloadLocation` with GitHub URLs (47 packages)
- 5-strategy package name variant matching
- Parallel processing with global caching
- Zero performance overhead

## Dependency Scope Filtering Options (v1.0.6)

| Option | Default | Description |
|--------|---------|-------------|
| `--production` | `false` | Production mode: include only runtime + optional dependencies |
| `--scope-filter <SCOPE>` | *(none)* | Filter by dependency scope (can specify multiple) |

### Dependency Scopes

| Scope | Description | Included in Production |
|-------|-------------|----------------------|
| `runtime` | Runtime dependencies (ships in production binary) | ✅ Yes |
| `build` | Build-time only (compilers, build tools) | ❌ No |
| `test` | Test frameworks and test dependencies | ❌ No |
| `development` | Development tools (linters, formatters) | ❌ No |
| `optional` | Optional runtime dependencies | ✅ Yes |
| `provided` | Provided by platform (git submodules, etc.) | ❌ No |

**Examples:**
```bash
# Production SBOM (runtime + optional only)
radeis_sc2sbom --path . --production

# Custom filter: runtime + build dependencies
radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter build
```

Scope statistics appear automatically in the console report. No extra flag is required.

See [Scope Classification Guide](SCOPE_CLASSIFICATION.md) for comprehensive documentation.

## Advanced Options

| Option | Description |
|--------|-------------|
| `--exclude <PATTERN>` | Exclude directories (can be used multiple times) |
| `--output <DIR>` | Output directory (default: `./out`) |
| `--target-arch <ARCH>` | Resolve architecture-conditional versions (e.g. `aarch64`, `x86_64`) |
| `--compact-spdx` | ~30% smaller SPDX output (omit optional empty fields) |
| `--help` | Show complete help |
| `--version` | Show version information |

## C/C++ AST-Based SAST (Internal Build)


```bash
# Internal build
cargo build --release --features internal

# Scan with SAST enabled
```

**Output files (internal build only):**

| File | Description |
|------|-------------|
| `<project>_static_analysis.sarif` | SARIF 2.1 report for GitHub Code Scanning, VS Code, CI pipelines (v1.0.17) |


**Additional SAST options (v1.0.17–v1.0.18):**

| Option | Default | Description |
|--------|---------|-------------|
| `--sarif-output <PATH>` | `<output>/<project>_static_analysis.sarif` | Write SARIF 2.1 report to this path |
| `--sarif-baseline <PATH>` | *(none)* | Diff against a prior SARIF run; only new findings are reported |

**SARIF baseline example:**
```bash
# Save a baseline

# PR gate: only new findings
  --sarif-baseline baseline.sarif --sarif-output new-findings.sarif
```


## Examples

### Basic Scan
```bash
# Full scan with default settings
radeis_sc2sbom --path .
```

### Output Formats
```bash
# Generate specific SBOM format
radeis_sc2sbom --path . --format spdx-json > sbom.json

# Generate all formats
radeis_sc2sbom --path . --format all
```

### Visualization Styles
```bash
# Classic tree view (default)
radeis_sc2sbom --path . --tree-style tree

# Flat list view
radeis_sc2sbom --path . --tree-style flat

# Compact tree view
radeis_sc2sbom --path . --tree-style compact
```

### SBOM Modes
```bash
# Complete mode (all packages) - default
radeis_sc2sbom --path . --sbom-mode complete

radeis_sc2sbom --path . \

radeis_sc2sbom --path . \
  --format spdx-json
```

```bash
# Quick security assessment

radeis_sc2sbom --path . --max-vulns-per-severity 0


```

### Scope Filtering (v1.0.6)
```bash
# Production SBOM (runtime + optional only)
radeis_sc2sbom --path . --production

# Example: embedded-project (67 → 11 packages)
radeis_sc2sbom --path ./embedded-project \
  --production \
  --format spdx-json

# Custom scope filter
radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter build

radeis_sc2sbom --path . \
  --production \
```

Scope statistics appear automatically in the console report. No extra flag is required.

### ROS/ROS2 Projects
```bash
# Scan ROS 2 project with automatic version resolution
radeis_sc2sbom --path ./ros2cli --ros-distro jazzy

# ROS 2 Humble project
radeis_sc2sbom --path ./ros_workspace --ros-distro humble

# ROS 1 Noetic project
radeis_sc2sbom --path ./catkin_ws --ros-distro noetic

radeis_sc2sbom --path ./ros2cli \
  --ros-distro jazzy \
  --format all \
```

### Vendor Directories
```bash
# Include vendor directories (default)
radeis_sc2sbom --path . --vendor include

# Skip vendor directories
radeis_sc2sbom --path . --vendor skip

# Scan only vendor directories
radeis_sc2sbom --path . --vendor only
```

### Advanced Usage
```bash
# Exclude specific directories
radeis_sc2sbom --path . \
  --exclude tests \
  --exclude examples \
  --exclude docs

# Custom output directory
radeis_sc2sbom --path . --output ./reports

radeis_sc2sbom --path . --clear-cache

# Minimal scan (fastest)
radeis_sc2sbom --path . \
  --vendor skip \
  --fallback-import-scan=false \
```

## Scan Options (v1.0.5+)

These options control how radeis_sc2sbom scans for dependencies across different ecosystems.

| Option | Default | Description |
|--------|---------|-------------|
| `--scan-c-build-systems <BOOL>` | `true` | C/C++ build systems: CMake, pkg-config, Autotools, Makefiles, .mk files (merged in v1.0.9) |
| `--scan-meson <BOOL>` | `true` | Meson build system scanning, always parses subprojects when enabled (v1.0.9) |
| `--scan-bazel <BOOL>` | `true` | Bazel WORKSPACE/MODULE file scanning (v1.0.4) |
| `--scan-ai-models <BOOL>` | `true` | GGUF AI model file scanning (v1.0.9) |
| `--max-hash-size-gb <GB>` | `0` | Max file size (GB) for SHA-256 hashing of AI models; 0=unlimited (v1.0.9) |
| `--scan-so-files <BOOL>` | `false` | Extract versions from compiled .so binaries |

### `--scan-so-files`

**Default:** `false`

Extract version information from compiled `.so` library binaries.

**How it works:**
1. Searches for `.so` files in standard library directories:
   - `lib/`
   - `lib64/` (64-bit libraries)
   - `build/`
   - `build/lib/` (CMake out-of-tree builds)
   - `toolchains/install/lib/`
   - `usr/lib/`
   - `usr/lib64/` (64-bit system libraries)
   - `usr/local/lib/` (local installations)
   - `.libs/` (autotools)
2. Deduplicates symlinks (e.g., `libcurl.so` → `libcurl.so.4` → `libcurl.so.4.8.0`)
3. Extracts version from:
   - Filename (e.g., `libcurl.so.4.8.0` → "4.8.0")
   - ELF soname (via readelf, if available)
   - Version strings in binary content
4. Updates dependency versions

**Example:**
```bash
# Enable .so file scanning (requires built libraries)
radeis_sc2sbom --path ./embedded-project --scan-so-files=true

# Use both .mk and .so scanning
radeis_sc2sbom --path ./embedded-project \
  --scan-mk-files=true \
  --scan-so-files=true
```

**Use case:** Post-build SBOM generation with access to compiled libraries

**Note:** Requires libraries to be already built. Not suitable for source-only repositories or CI/CD without build artifacts.

### AI Model Scanning

**Default:** `true` (via `--scan-ai-models`)

Detects GGUF AI model files and includes them in the SBOM as components. This is
useful for physical AI and edge AI projects that bundle model weights alongside
application code.

**How it works:**
1. Searches for `*.gguf` files in the scan path
2. Extracts model metadata (name, quantization, parameter count) from GGUF headers
3. Adds each model as a dependency with ecosystem "gguf"

**Example:**
```bash
# Disable AI model scanning
radeis_sc2sbom --path ./edge-device --scan-ai-models=false
```

### Version Extraction Priority

When multiple version sources are available:

1. **Lock files** (highest priority) - Most reliable
2. **Package manifests** - Direct from package manager
3. **.mk build files** - Build configuration (v1.0.5)
4. **.so binaries** - Compiled artifacts (v1.0.5)
5. **"unspecified"** (fallback) - No version info available

### Real-World Example

**embedded-project project before v1.0.5:**
```json
{
  "name": "curl",
  "version": "unspecified",
  "ecosystem": "system"
}
```

**After v1.0.5 with `--scan-mk-files=true`:**
```json
{
  "name": "curl",
  "version": "8.15.0",
  "ecosystem": "system",
  "source_file": "Identified by Makefile extractor from Makefile [version from .mk file: 8.15.0]"
}
```

**Impact:** 32 system libraries upgraded from "unspecified" to precise versions in embedded-project scan.

## Output Files

When using `--format all` (default):

```
./out/
├── <project-name>_report.md          # Human-readable report
├── <project-name>_spdx.json          # SPDX 2.3 JSON
├── <project-name>_spdx.spdx          # SPDX 2.3 Tag-Value
└── <project-name>_cyclonedx.json     # CycloneDX 1.5 JSON
```

## Cache Location


## Exit Codes

| Code | Description |
|------|-------------|
| `0` | Success |
| `1` | Error during scanning or SBOM generation |

## See Also

- [Usage Guide](USAGE.md) - Detailed examples and use cases
- [Scope Classification Guide](SCOPE_CLASSIFICATION.md) - Production SBOM filtering (v1.0.6)
- [Architecture](ARCHITECTURE.md) - Technical implementation
- [Formats](FORMATS.md) - SBOM format specifications
