# Usage Guide

Detailed examples and use cases for `radeis_sc2sbom`.

## Table of Contents

- [Common Use Cases](#common-use-cases)
- [Visualization Features](#visualization-features)
- [CI/CD Integration](#cicd-integration)
- [Configuration Examples](#configuration-examples)
- [Supported Ecosystems](#supported-ecosystems)

## Common Use Cases

### Development Workflow

```bash
radeis_sc2sbom --path .

radeis_sc2sbom --path . \

# Quick dependency inventory without security scan

# Use classic flat list view
radeis_sc2sbom --path . --tree-style flat
```

### SBOM Generation

```bash
# Generate all SBOM formats (default)
radeis_sc2sbom --path .

# Generate specific format for compliance
radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json

# SBOM without vendor dependencies (only direct deps)
radeis_sc2sbom --path . --vendor skip --format cyclonedx-json > sbom.cdx.json
```


```bash
# Hierarchical security report (recommended)

# Summary view for dashboards

# Filter critical issues only

radeis_sc2sbom --path . \
  --max-vulns-per-severity 0

radeis_sc2sbom --path . --clear-cache
```

## Visualization Features

### Hierarchical Dependency Trees

The tool builds true parent-child dependency relationships from lock files.

**Supported Formats:**
- ✅ **npm** - package-lock.json (full hierarchy)
- ✅ **Cargo** - Cargo.lock (full hierarchy)
- ✅ **Poetry** - poetry.lock (full hierarchy)
- ❌ **Others** - Flat list (no lock file support yet)

**Key Features:**
- Shows which package depends on which (e.g., express → body-parser → bytes)
- Accurate [direct] markers - only root dependencies marked as direct
- Circular dependency detection

**Report Structure:**
1. **Main Section** - Direct production dependencies with hierarchical trees
2. **Distinct Packages List** - Flat alphabetical list grouped by type
3. **Appendix** - Development and transitive-only dependencies

### Tree Style Examples

#### Tree Style (Default)
```
NPM                                346 packages (105 direct, 241 transitive, 240 dev)
───────────────────────────────────────────────────
├── express @ 4.21.2 [direct]
│   ├── body-parser @ 1.20.3
│   │   ├── bytes @ 3.1.2
│   │   └── debug @ 2.6.9
│   │       └── ms @ 2.0.0
│   └── cookie @ 0.7.1
└── axios @ 1.6.2 [direct]
    └── follow-redirects @ 1.15.9
```

#### Flat Style
```
NPM                                 45 packages
───────────────────────────────────────────────────
  express @ 4.18.2 [direct]
  react @ 18.2.0 [direct, dev]
  axios @ 1.2.0 [direct]
```

#### Compact Style
```
NPM (45 packages)
→ express @ 4.18.2 [direct]
→ react @ 18.2.0 [direct, dev]
→ axios @ 1.2.0 [direct]
```


#### Tree Mode (Recommended)
```bash
```

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
│  Dependency chain: setuptools @ detected [direct]
│
└── (4 more high-severity, use --max-vulns-per-severity 0)
```

#### Detailed Mode
```bash
```

- Full description
- Complete reference links
- Dependency chain with full tree

#### Summary Mode
```bash
```

```
- 🔴 Critical: 0
- 🟠 High: 5
- 🟡 Medium: 5
- 🟢 Low: 0
```


### How It Works

3. **Coverage** - Checks all dependencies across all ecosystems

### Report Details

- Source (direct or transitive, with dev flag)
- Severity (Critical, High, Medium, Low)
- Reference links

### SBOM Format Support

- **Console** - Human-readable markdown (saved to `./out/`)

## CI/CD Integration

### GitHub Actions - Security Gate

```yaml
name: Security Scan
on: [push, pull_request]

jobs:
  sbom-security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

        run: |
          radeis_sc2sbom \
            --path . \
            --format cyclonedx-json \

        run: |
          if [ "$CRITICAL" -gt 0 ]; then
            exit 1
          fi

      - name: Upload SBOM
        uses: actions/upload-artifact@v3
        with:
          name: sbom-report
          path: |
            sbom.json
            out/*.md
```

### GitLab CI - Automated SBOM

```yaml
sbom-generation:
  stage: security
  script:
    - radeis_sc2sbom --path . --format spdx-json > sbom.json
  artifacts:
    paths:
      - sbom.json
      - out/
    reports:
      cyclonedx: sbom.json
```

### Jenkins Pipeline

```groovy
stage('SBOM Generation') {
    steps {
        sh 'radeis_sc2sbom --path . --format all'
        archiveArtifacts artifacts: 'out/**'
    }
}

stage('Security Check') {
    steps {
        sh '''
            radeis_sc2sbom --path . \
        '''
    }
}
```

## Configuration Examples

### Minimal Scan (Fastest)

```bash
radeis_sc2sbom \
  --path . \
  --vendor skip \
  --fallback-import-scan=false \
```

### Complete Audit (Most Thorough)

```bash
radeis_sc2sbom \
  --path . \
  --vendor include \
  --fallback-import-scan=true \
  --clear-cache \
```

### Production SBOM

```bash
radeis_sc2sbom \
  --path . \
  --exclude tests \
  --exclude examples \
  --exclude docs \
  --format spdx-json > production-sbom.json
```

### Security-Focused Scan

```bash
radeis_sc2sbom \
  --path . \
  --max-vulns-per-severity 0
```

## Supported Ecosystems

| Ecosystem | Manifest Files | Hierarchical Trees | Notes |
|-----------|----------------|-------------------|-------|
| **npm** | package.json, package-lock.json | ✅ Full | Dev dependencies marked, true hierarchical trees |
| **Cargo** | Cargo.toml, Cargo.lock | ✅ Full | True hierarchical trees with parent-child relationships |
| **pip** | requirements.txt, setup.py, poetry.lock | ✅ Poetry only | Hierarchical trees for Poetry lock files |
| **Go** | go.mod, go.sum | ❌ | Handles pseudo-versions |
| **RubyGems** | Gemfile, Gemfile.lock | ❌ | Version operators supported |
| **Composer** | composer.json, composer.lock | ❌ | PHP version filtering |
| **Maven** | pom.xml | ❌ | Detection only |
| **Gradle** | build.gradle, build.gradle.kts | ❌ | Detection only |
| **ROS/ROS2** | package.xml, setup.py | ❌ | Multi-package workspaces |

### What Gets Scanned

By default, the tool performs comprehensive analysis:

- ✅ Vendor directories (node_modules, vendor, etc.)
- ✅ Manifest files (package.json, Cargo.toml, requirements.txt, etc.)
- ✅ Lock files (package-lock.json, Cargo.lock, etc.)
- ✅ Source code imports (fallback when manifests incomplete)
- ✅ Tree visualization (hierarchical dependency display)

## Output Examples

### Console Report Structure

```
═══════════════════════════════════════════════════
SBOM SUMMARY
═══════════════════════════════════════════════════
Project Path, Generated At, Dependencies, Ecosystems,

═══════════════════════════════════════════════════
📦 DEPENDENCIES
═══════════════════════════════════════════════════

NPM (Hierarchical Tree)
Direct Production Dependencies
├── express @ 4.21.2 [direct]
│   └── ...
└── axios @ 1.6.2 [direct]

NPM Distincted Packages List
Direct Production:
  express, axios, lodash

Direct Development:
  eslint, jest, prettier

Transitive:
  (all transitive dependencies)

═══════════════════════════════════════════════════
═══════════════════════════════════════════════════

═══════════════════════════════════════════════════
📋 APPENDIX
═══════════════════════════════════════════════════
Development and transitive-only dependency trees
```

## Known Limitations

- **Maven/Gradle** - Detection only, version extraction not implemented
- **Hierarchical Trees** - Full support for npm, Cargo, Poetry only
- **Direct vs Transitive** - Some ecosystems mark all packages as direct
- **License Detection** - Not extracted from manifests
- **Checksums** - Not calculated for packages

## See Also

- [CLI Reference](CLI.md) - Complete command-line options
- [Architecture](ARCHITECTURE.md) - Technical implementation
- [Formats](FORMATS.md) - SBOM format specifications
- [Changelog](../CHANGELOG.md) - Version history
