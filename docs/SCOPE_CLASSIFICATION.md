# Dependency Scope Classification Guide (v1.0.6)

**Production SBOM Filtering with Automated Dependency Scope Classification**

## Overview

radeis_sc2sbom v1.0.6 introduces automated dependency scope classification, enabling you to generate production-ready SBOMs that include only runtime dependencies. This guide explains how classification works, how to use filtering options, and how to interpret the results.

## Table of Contents

- [Quick Start](#quick-start)
- [Dependency Scopes](#dependency-scopes)
- [Classification Methods](#classification-methods)
- [Using Scope Filters](#using-scope-filters)
- [Understanding Confidence Scores](#understanding-confidence-scores)
- [Scope Statistics](#scope-statistics)
- [Output Formats](#output-formats)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

---

## Quick Start

### Default Behavior (All Dependencies)

By default, radeis_sc2sbom includes all dependencies (backward compatible with v1.0.5):

```bash
./radeis_sc2sbom --path .
# Output: All dependencies regardless of scope
```

### Production SBOM (Runtime Only)

Generate a production-ready SBOM with only runtime dependencies:

```bash
./radeis_sc2sbom --path . --production
# Output: Runtime + Optional dependencies only
```

### Custom Filtering

Filter by specific scopes:

```bash
# Include only runtime and build dependencies
./radeis_sc2sbom --path . --scope-filter runtime --scope-filter build

# Exclude test and development dependencies
./radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter build \
  --scope-filter optional
```

### View Classification Details

Scope statistics appear automatically in the console report. When using `--format console` (or the default `--format all`), the report includes a scope breakdown summary.

---

## Dependency Scopes

radeis_sc2sbom classifies dependencies into 6 scope categories:

### Runtime
**Description:** Dependencies that ship in the production binary and are required at runtime.

**Examples:**
- Shared libraries: zlib, curl, openssl, mbedtls, protobuf
- Application frameworks: django, flask, express, react
- Runtime utilities: vsomeip, paho.mqtt.embedded-c

**Inclusion:**
- ✅ Production SBOM (`--production`)
- ✅ Compliance reports

### Build
**Description:** Build-time only dependencies (compilers, build tools, toolchains).

**Examples:**
- Compilers: gcc, clang, g++
- Build systems: cmake, meson, ninja, make
- Build tools: autoconf, automake, libtool, pkg-config

**Inclusion:**
- ❌ Production SBOM
- ✅ Development SBOM
- ⚠️ CI/CD environment reports

### Test
**Description:** Test frameworks and test-only dependencies.

**Examples:**
- Test frameworks: pytest, jest, mocha, junit, gtest, unity
- Assertion libraries: chai, expect, googletest
- Test utilities: pytest-cov, jest-junit

**Inclusion:**
- ❌ Production SBOM
- ✅ Development SBOM
- ✅ CI/CD test environment

### Development
**Description:** Development tools not required for building or runtime.

**Examples:**
- Linters: pylint, flake8, eslint, prettier
- Formatters: black, autopep8, clang-format
- Dev utilities: nodemon, watchdog

**Inclusion:**
- ❌ Production SBOM
- ❌ CI/CD build environment
- ✅ Development workstation SBOM

### Optional
**Description:** Optional runtime dependencies that may be present.

**Examples:**
- Optional features: optional-lib, feature-x
- Plugin systems: plugins that may or may not be installed
- Platform-specific: dependencies only needed on certain platforms

**Inclusion:**
- ✅ Production SBOM (`--production`)
- ✅ Complete SBOM
- ⚠️ Document in SBOM metadata

### Provided
**Description:** Dependencies provided by the platform (not bundled).

**Examples:**
- Git submodules: Source code dependencies
- System libraries: Platform-provided packages
- SDK dependencies: iOS/Android SDKs

**Inclusion:**
- ❌ Production SBOM (usually)
- ✅ Source code SBOM
- ⚠️ Platform dependency documentation

---

## Classification Methods

radeis_sc2sbom uses multiple heuristics to classify dependencies with varying confidence levels:

### 1. Ecosystem-Based Classification (High Confidence: 0.8-1.0)

Classifies based on the package ecosystem and known patterns:

| Ecosystem | Default Scope | Confidence | Reasoning |
|-----------|---------------|------------|-----------|
| BUILD-CONFIG | Build | 0.7 | Build configuration files (refined with link analysis) |
| SYSTEM | Runtime | 0.8 | System libraries typically linked at runtime |
| PIP (from test dir) | Test | 0.9 | Python packages in test directories |
| npm (devDependencies) | Development | 1.0 | Package.json devDependencies section |
| Cargo ([dev-dependencies]) | Development | 1.0 | Cargo.toml dev-dependencies section |
| Maven (scope:test) | Test | 1.0 | Maven test scope |
| GIT-SUBMODULE | Provided | 0.7 | Source code submodules |

### 2. Name-Based Heuristics (Very High Confidence: 0.95-1.0)

Exact matches for well-known packages:

**Test Frameworks (Confidence: 1.0)**
```
unity, gtest, pytest, junit, mocha, jest, vitest, jasmine,
karma, testng, cucumber, rspec, minitest, nose
```

**Build Tools (Confidence: 1.0)**
```
cmake, meson, ninja, gcc, clang, make, autoconf, automake,
libtool, pkg-config, qmake, scons, bazel
```

**Development Tools (Confidence: 1.0)**
```
pylint, flake8, black, autopep8, eslint, prettier,
stylelint, tslint, ruff, mypy, isort
```

**Runtime Libraries (Confidence: 0.9)**
```
zlib, openssl, curl, mbedtls, protobuf, boost, qt,
libxml2, libpcap, sqlite3
```

### 3. Directory-Based Analysis (Medium Confidence: 0.6-0.7)

Infers scope from source file location:

```
/test/    or /tests/       → Test (0.7)
/3rd_party/ or /toolchains/ → Build (0.7)
/scripts/ or /tools/        → Development (0.7)
```

### 4. Link Analysis Refinement (High Confidence: 0.9)

For BUILD-CONFIG packages, checks if they're actually linked at runtime:

**Process:**
1. Collect all SYSTEM runtime libraries (e.g., `-lz`, `-lssl`)
2. Normalize library names (handle `libssl.so.3` → `ssl`)
3. Match BUILD-CONFIG packages to SYSTEM libraries
4. Upgrade matched BUILD-CONFIG from Build → Runtime

**Example:**
```
BUILD-CONFIG: zlib @ 1.3.1 (scope: Build)
SYSTEM: -lz (ecosystem: system)

Link Analysis:
  normalize("zlib") → ["zlib", "z", "libzlib"]
  normalize("z") → ["z", "libz", "zlib"]
  Match found: "z" and "zlib"

Result: zlib @ 1.3.1 (scope: Runtime, confidence: 0.9)
```

### 5. Default Fallback (Low Confidence: 0.3)

If no rules match, defaults to Runtime with low confidence:

```
Unknown package → Runtime (0.3, "Default (no match)")
```

---

## Using Scope Filters

### Production Mode

The most common use case - runtime dependencies only:

```bash
# Runtime + Optional dependencies
./radeis_sc2sbom --path . --production

# Equivalent to:
./radeis_sc2sbom --path . --scope-filter runtime --scope-filter optional
```

**Example: embedded-project**
```bash
./radeis_sc2sbom --path example_target_repos/embedded-project --production
# Output: 11 packages (Runtime + Optional)
# vs 67 packages (default, all scopes)
```

### Custom Scope Combinations

```bash
./radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter optional \
```

#### CI/CD Build Environment (Runtime + Build)
```bash
./radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter build
```

#### Test Environment (Runtime + Test)
```bash
./radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter test
```

#### Development Workstation (All)
```bash
./radeis_sc2sbom --path .
# Default: includes all scopes
```

---

## Understanding Confidence Scores

Confidence scores range from 0.0 (uncertain) to 1.0 (certain):

| Confidence Range | Interpretation | Action |
|------------------|----------------|--------|
| **0.9 - 1.0** | Very High - Exact match or ecosystem-native | Trust classification |
| **0.7 - 0.9** | High - Strong heuristic match | Review if critical |
| **0.5 - 0.7** | Medium - Heuristic inference | Review for production |
| **0.3 - 0.5** | Low - Weak signal or default | Manual review recommended |
| **0.0 - 0.3** | Very Low - No classification confidence | Requires investigation |

### Viewing Confidence Scores

The console report includes aggregate scope statistics showing how many dependencies fall into each category. Per-dependency scope, confidence score, and reasoning are computed internally by the classifier but are not currently printed in the console output.

### Interpreting Internal Reasons

The classifier attaches an internal reason string and confidence score to each dependency. These values are used for debugging and future tooling integrations, but are not currently surfaced in the standard console or Markdown reports. The table below lists examples of the internal reasons assigned by the classifier:

| Internal Reason | Confidence | Interpretation |
|-----------------|------------|----------------|
| `"Exact name match: test framework"` | 1.0 | Well-known test framework |
| `"BUILD-CONFIG linked at runtime"` | 0.9 | Link analysis matched SYSTEM library |
| `"System library"` | 0.8 | SYSTEM ecosystem default |
| `"Found in test directory"` | 0.7 | Directory-based inference |
| `"Default (no match)"` | 0.3 | No heuristics matched |

---

## Scope Statistics

### Viewing Summary

A breakdown of dependencies by scope appears automatically in the console report whenever a scan is run. No extra flag is required — the `ScopeStatistics` block is always included in the console output.

**Example Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Scope Summary for embedded-project
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Runtime:      11 packages (16.4%)  ← Production
Build:        40 packages (59.7%)  ← Toolchains
Test:          4 packages (6.0%)   ← Test frameworks
Development:   7 packages (10.4%)  ← Dev tools
Optional:      0 packages (0.0%)
Provided:      5 packages (7.5%)   ← Submodules
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Total: 67 packages
Average Confidence: 0.91
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

### Scope Statistics in Output

Scope statistics are included in the console/markdown report. They are not currently embedded in SPDX or CycloneDX SBOM output fields.

---

## Output Formats

### SPDX 2.3 Mapping

Scope information is mapped to SPDX `primaryPackagePurpose`:

| radeis Scope | SPDX Purpose | Description |
|--------------|--------------|-------------|
| Runtime | `LIBRARY` | Library dependencies |
| Build | `OTHER` | Build tools (no direct SPDX equivalent) |
| Test | `OTHER` | Test dependencies (no direct SPDX equivalent) |
| Development | `OTHER` | Development tools (no direct SPDX equivalent) |
| Optional | `LIBRARY` | Optional runtime dependencies |
| Provided | `SOURCE` | Source-only or externally provided components |

**Example SPDX Package:**
```json
{
  "SPDXID": "SPDXRef-Package-zlib-123e4567-e89b-12d3-a456-426614174000",
  "name": "zlib",
  "versionInfo": "1.3.1",
  "primaryPackagePurpose": "LIBRARY"
}
```

### CycloneDX 1.5 Mapping

Each component includes a `dependency-scope` property indicating whether it is a direct or transitive dependency. Lifecycle scope (Runtime/Build/Test/etc.) is enforced via the `--production` or `--scope-filter` flags before output generation; components that don't match the filter are excluded from the output rather than annotated.

**Example CycloneDX Component:**
```json
{
  "name": "zlib",
  "version": "1.3.1",
  "properties": [
    {"name": "dependency-source", "value": "manifest"},
    {"name": "dependency-scope", "value": "direct"}
  ]
}
```

### Console Output

Example console/markdown report (aggregate scope summary and runtime dependency tree):

```markdown
## Dependency Scopes
- Runtime: 11
- Build: 4
- Test: 7
- Unknown: 2

## Runtime Dependencies (11 packages)
└── embedded-project @ 5.15
    ├── zlib @ 1.3.1
    ├── curl @ 8.15.0
    └── mbedtls @ 3.6.0
```

---

## Examples

### Example 1: C/C++ Project (embedded-project)

**Project Structure:**
- 11 Runtime libraries (zlib, curl, mbedtls, vsomeip, etc.)
- 40 Build packages (BUILD-CONFIG toolchain libraries)
- 4 Test frameworks (unity, gtest)
- 7 Development tools (PIP packages: pylint, black)
- 5 Git submodules (source code dependencies)

**Default SBOM:**
```bash
./radeis_sc2sbom --path example_target_repos/embedded-project
# Output: 67 packages (all scopes)
```

**Production SBOM:**
```bash
./radeis_sc2sbom --path example_target_repos/embedded-project --production
# Output: 11 packages (runtime only)
# Reduction: 83.6% (67 → 11)
```

**With SPDX output:**
```bash
./radeis_sc2sbom --path example_target_repos/embedded-project \
  --production \
  --format spdx-json
# Scope statistics (counts and average confidence) appear in the console report automatically
```

### Example 2: Python Project (Django Application)

**Project Structure:**
- Runtime: django, requests, psycopg2, celery
- Test: pytest, pytest-cov, pytest-django
- Development: black, pylint, mypy, flake8

**Production SBOM:**
```bash
./radeis_sc2sbom --path my-django-app --production
# Output: Runtime dependencies only
# Excludes: pytest, black, pylint, mypy
```

### Example 3: Node.js Project

**Project Structure:**
- Runtime: express, axios, lodash
- Test: jest, mocha, chai
- Development: eslint, prettier, nodemon

**Production SBOM:**
```bash
./radeis_sc2sbom --path my-node-app --production
# Output: express, axios, lodash (runtime)
# Excludes: jest, mocha, eslint, prettier
```

### Example 4: CI/CD Pipeline Integration

```bash
./radeis_sc2sbom --path . \
  --production \
  --format spdx-json
```

**Build Environment Documentation:**
```bash
# Document build environment dependencies
./radeis_sc2sbom --path . \
  --scope-filter runtime \
  --scope-filter build \
  --format cyclonedx-json
```

**Complete Development SBOM:**
```bash
# Include all dependencies for development workstation
./radeis_sc2sbom --path . \
  --format all
# Scope statistics appear automatically in the console report
```

---

## Troubleshooting

### Issue: Low Confidence Scores

**Problem:** Many dependencies have confidence < 0.7

**Solutions:**
1. Check if ecosystems are correctly detected by reviewing the scope statistics in the console report.

2. Review directory structure:
   - Move test dependencies to `/test/` or `/tests/`
   - Use ecosystem-native dev dependency markers

### Issue: Incorrect Classification

**Problem:** A runtime library is classified as Build

**Solutions:**
1. **Check link analysis:** Ensure SYSTEM libraries are detected:
   ```bash
   # Look for -l flags in Makefiles
   grep -r "^LDLIBS" Makefile
   ```

2. **Manual investigation:** Run a full scan without filters and review the scope statistics summary in the console report.

3. **Verify ecosystem detection:** BUILD-CONFIG packages undergo link analysis refinement

### Issue: Missing Dependencies

**Problem:** Expected runtime dependencies not showing up with `--production`

**Diagnosis:**
```bash
# Check full classification without filtering
./radeis_sc2sbom --path .
# Scope statistics appear in the console report automatically
```

**Common Causes:**
- Classified as Build/Test/Development instead of Runtime
- Not detected by any ecosystem parser

**Solutions:**
1. Check scope statistics in the console report
2. Report classification issue if incorrect

### Issue: Too Many Packages in Production SBOM

**Problem:** `--production` includes too many packages

**Diagnosis:**
```bash
# Check what's classified as Runtime
./radeis_sc2sbom --path . --scope-filter runtime
# Scope statistics appear in the console report
```

**Solutions:**
1. Exclude Optional if not needed:
   ```bash
   ./radeis_sc2sbom --path . --scope-filter runtime
   ```

2. Review low-confidence Runtime classifications and report if incorrect

### Issue: Scope Statistics Missing

**Problem:** Scope statistics do not appear in the console report

**Solution:**
Scope statistics are printed automatically as part of the console output. Ensure you are running a scan against a real project directory:
```bash
# Correct: scan a real project
./radeis_sc2sbom --path .

# Incorrect: --help produces no scan output
./radeis_sc2sbom --help
```

---

## Best Practices

### For Production SBOMs

1. **Use production flag:**
   ```bash
   ./radeis_sc2sbom --path . --production
   ```

2. **Review classification:** Run with `--production` and review the scope statistics summary in the console report.


   ```bash
   ```

   ```bash
   ./radeis_sc2sbom --path . \
     --production \
   ```

### For Compliance

1. **Generate complete SBOM:**
   ```bash
   ./radeis_sc2sbom --path . --format all
   # Scope statistics appear automatically in the console report
   ```

2. **Include classification reasoning:** Console and markdown reports currently include only aggregated scope statistics, not per-dependency scope reasoning (scope, confidence, and reasoning text). SPDX output includes the inferred scope via `primaryPackagePurpose`; CycloneDX output includes a `dependency-scope` property (direct/transitive). Full confidence and reasoning text are not currently embedded in SPDX or CycloneDX output.

### For CI/CD Integration

1. **Production builds:**
   ```bash
   ./radeis_sc2sbom --path . --production --format cyclonedx-json
   ```

2. **Security gates:**
   ```bash
   ./radeis_sc2sbom --path . \
     --production \
     || exit 1
   ```

---

## Validation

### Accuracy Metrics (v1.0.6)

Validated against embedded-project ground truth:

| Scope | Test Count | Accuracy | Confidence |
|-------|------------|----------|------------|
| Build tools | 16 tests | 100% | 1.0 |
| Test frameworks | 10 tests | 100% | 1.0 |
| Dev tools | 8 tests | 100% | 1.0 |
| Runtime libraries | 8 tests | High | 0.9 |

**Overall:**
- **609 total tests passing** (203 lib + 203 bin + 200 integration + 3 doc)
- **42 scope classification tests** (14 filtering + 11 e2e + 17 real-world)
- **100% test pass rate**

---

## Further Reading

- [CLI Reference](CLI.md) - Complete command-line options
- [CHANGELOG](../CHANGELOG.md) - v1.0.6 release notes
- [USAGE Guide](USAGE.md) - More examples and CI/CD integration
- [ARCHITECTURE](ARCHITECTURE.md) - Implementation details

---

## Planned Features

The following CLI flags are **not yet implemented** and will cause an "unexpected argument" error if used. They are listed here as a roadmap reference.

| Flag | Description |
|------|-------------|
| `--show-scope-reasons` | Print per-dependency classification reasoning (scope, confidence, reason) to stdout |
| `--min-scope-confidence <FLOAT>` | Filter dependencies below a minimum confidence threshold (e.g., `0.8`) |
| `--scope-summary` | Print a standalone scope breakdown table to stdout (currently shown automatically in the console report) |

---

**Version:** v1.0.6
**Last Updated:** 2026-03-04
**Status:** Complete
