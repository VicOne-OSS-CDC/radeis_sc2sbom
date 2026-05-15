# Usage Examples

Common use cases and command patterns for radeis_sc2sbom.

## Quick Start Examples

### Basic Scan

Scan current directory and display dependency tree in console:

```bash
./target/release/radeis_sc2sbom --path .
```

**Output:**
```
NPM                                690 packages (105 direct, 585 transitive)
───────────────────────────────────────────────────
├── express @ 4.21.2 [direct]
│   ├── body-parser @ 1.20.3
│   │   ├── bytes @ 3.1.2
│   │   └── debug @ 2.6.9
│   └── cookie @ 0.7.1

───────────────────────────────────────────────────
⚠️  axios @ 1.6.2 - CRITICAL
    Fix: Upgrade to axios@1.6.3

⚠️  lodash @ 4.17.21 - HIGH
    Fix: Upgrade to lodash@4.17.22
```

### Generate SPDX SBOM

Create industry-standard SPDX SBOM file:

```bash
# Option 1: Generate all formats to ./out/ directory
./target/release/radeis_sc2sbom \
  --path . \
  --format all
# Output: ./out/<project>_spdx.json, ./out/<project>_cyclonedx.json, etc.

# Option 2: Single format to stdout (redirect to file)
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json > my_sbom.json
```

**Note:** `--format all` writes files to `./out/` (or custom directory via `--output`). Single formats print to stdout.

---


### Daily Security Scan


```bash
./target/release/radeis_sc2sbom --path .
```

### CI/CD Security Gate


```bash
#!/bin/bash
./target/release/radeis_sc2sbom \
  --path . \

# Exit code:
```

### Security Report Only


```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format console
```

### Custom Severity Threshold


```bash
./target/release/radeis_sc2sbom \
  --path . \
```

---

## SBOM Generation

### Compliance SBOM (Fast)


```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json \
```


### Compact SBOM

30% smaller SPDX output:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json \
  --compact-spdx
```

**Use case:** Storage optimization, large projects

### All Formats

Generate all supported formats:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format all
```

**Output:**
```
./out/
├── <project>_spdx.json        # SPDX 2.3 JSON
├── <project>_spdx.spdx        # SPDX 2.3 Tag-Value
├── <project>_cyclonedx.json   # CycloneDX 1.5
```

### Custom Output Directory

Organize scan results:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format all \
  --output ./scan_results
```

---

## C/C++ Projects

### CMake Project

Scan CMake dependencies (FetchContent, ExternalProject):

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --scan-c-build-systems true
```

**Detects:**
- `FetchContent_Declare()` in CMakeLists.txt
- `ExternalProject_Add()` in CMakeLists.txt
- ExternalProject in `*.cmake` module files

### vcpkg Project

Scan vcpkg manifest:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

**Detects:** vcpkg.json dependencies with version constraints

### Git Submodules

Scan C++ project with Git submodules:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --scan-submodules true \
  --submodule-depth 3
```

**Features:**
- Detects all Git submodules
- Resolves commit SHAs
- Recursively scans nested submodules
- Depth limiting prevents infinite loops

### Mixed C++/Python Project

Full scan with all C++ features:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --scan-c-build-systems true \
  --scan-submodules true \
  --format all \
  --output ./scan_reports
```

**Example:** MRPT robotics library
- 4 CMake dependencies (assimp, eigen3, jpeg, libfyaml)
- 8 Git submodules (googletest, zlib, nanoflann, etc.)
- 39 Python packages (documentation tools)

---

## ROS/ROS2 Projects

### ROS2 Package

Automatic version resolution:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --ros-distro jazzy
```

**Features:**
- Parses package.xml
- Resolves versions from rosdistro API
- Detects GitHub repository URLs

### ROS1 Package

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --ros-distro noetic
```

### Multi-Package Workspace

Scan entire ROS workspace:

```bash
./target/release/radeis_sc2sbom \
  --path ./src \
  --ros-distro humble \
  --format all
```

**Detects:** All ROS packages in workspace with dependency relationships

---

## Python Projects

### requirements.txt

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

### Poetry Project

Detects poetry.lock with checksums:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

### Pipfile Project

Supports Pipfile and Pipfile.lock:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

### pyproject.toml

Supports PEP 621, Poetry, and PDM formats:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

---

## JavaScript Projects

### npm with Dependency Tree

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format console
```

**Output:** Full dependency tree with 105 direct + 585 transitive packages

### Yarn Project

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json
```

**Detects:** yarn.lock with full dependency resolution

---

## Rust Projects

### Cargo with Dependency Tree

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format console
```

**Features:** Full dependency tree from Cargo.lock

---

## CI/CD Integration

### GitHub Actions

```yaml
name: SBOM Security Scan

on: [push, pull_request]

jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          submodules: recursive  # For Git submodules

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Build radeis_sc2sbom
        run: |
          git clone <repository-url>
          cd radeis_sc2sbom
          cargo build --release

      - name: Security Scan
        run: |
          ./radeis_sc2sbom/target/release/radeis_sc2sbom \
            --path . \

      - name: Upload SBOM
        uses: actions/upload-artifact@v4
        with:
          name: sbom
          path: ./out/
```

### GitLab CI

```yaml
sbom-scan:
  stage: security
  script:
    - git clone <repository-url>
    - cd radeis_sc2sbom
    - cargo build --release
    - cd ..
  artifacts:
    paths:
      - ./out/
    expire_in: 30 days
  only:
    - main
    - merge_requests
```

### Jenkins Pipeline

```groovy
pipeline {
    agent any

    stages {
        stage('SBOM Security Scan') {
            steps {
                sh '''
                    git clone <repository-url>
                    cd radeis_sc2sbom
                    cargo build --release
                    cd ..
                    ./radeis_sc2sbom/target/release/radeis_sc2sbom \
                        --path . \
                        --format all \
                        --output ./sbom-results
                '''
            }
        }

        stage('Archive SBOM') {
            steps {
                archiveArtifacts artifacts: 'sbom-results/**', fingerprint: true
            }
        }
    }
}
```

---

## Advanced Use Cases

### Offline Mode


```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json \
```

### Vendor Directory Modes

Skip node_modules/vendor directories:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --vendor skip
```

Include vendor dependencies:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --vendor include
```

### Custom Excludes

Exclude specific directories:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --exclude "tests/*" \
  --exclude "docs/*" \
  --exclude "*.test.js"
```

### Submodule Depth Control

Limit recursive submodule scanning:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --scan-submodules true \
  --submodule-depth 1  # Only immediate submodules
```

---

## Output Examples

### Console Tree Output

```
NPM                                690 packages (105 direct, 585 transitive)
───────────────────────────────────────────────────
├── express @ 4.21.2 [direct]
│   ├── body-parser @ 1.20.3
│   ├── cookie @ 0.7.1
│   └── debug @ 2.6.9

GIT SUBMODULES                     8 packages
───────────────────────────────────────────────────
├── madler/zlib @ cacf7f1d4e
├── google/googletest @ 52eb8108c5
└── jlblancoc/nanoflann @ 92911c0bc3

CMAKE                              4 packages
───────────────────────────────────────────────────
├── EP_assimp @ 5.3.1
├── EP_eigen3 @ 3.3.7
└── EP_JPEG @ 1.5.90
```


```markdown

**Scan Date:** 2026-02-23


### axios @ 1.6.2
- **Severity:** CRITICAL
- **Description:** Server-side request forgery
- **Fix:** Upgrade to axios@1.6.3


### lodash @ 4.17.21
- **Severity:** HIGH
- **Fix:** Upgrade to lodash@4.17.22
```

---

## Tips & Best Practices

### 1. Regular Security Scans


```bash
# Add to cron or CI/CD
```

### 2. Version Control SBOMs

Track SBOM changes over time:

```bash
./target/release/radeis_sc2sbom \
  --path . \
  --format spdx-json \
  --output ./sbom/$(date +%Y%m%d)
```

### 3. Combine with Git Hooks

Pre-commit hook for security:

```bash
#!/bin/bash
# .git/hooks/pre-commit
./radeis_sc2sbom/target/release/radeis_sc2sbom \
  --path . \
```

### 4. Multi-Project Scanning

Scan multiple projects:

```bash
#!/bin/bash
for project in project1 project2 project3; do
  ./target/release/radeis_sc2sbom \
    --path ./$project \
    --format all \
    --output ./scans/$project
done
```

---

**See Also:**
- [CLI Reference](CLI.md) - Complete command-line options
- [Benchmarks](BENCHMARKS.md) - Performance comparisons
- [What's New](WHATS_NEW.md) - Latest features
