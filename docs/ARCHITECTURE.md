# Architecture & Design

Technical design and implementation details for `radeis_sc2sbom`.

## Overview

`radeis_sc2sbom` is a single-binary Rust CLI tool that scans directories for dependency manifest files and generates SBOMs in multiple standard formats.

**Core Design Principles:**
- **Zero Configuration**: Smart defaults, works out of the box
- **Fast Scanning**: Efficient directory traversal and parsing
- **Multi-Format**: SPDX and CycloneDX output
- **Extensible**: Easy to add new ecosystems

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        CLI Interface (clap)                      │
│                   Args parsing & validation                      │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                   Directory Scanner (walkdir)                    │
│         Recursive traversal │ Vendor filtering │ Excludes        │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      File Type Detection                         │
│          Pattern matching on filenames & extensions              │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Parser Dispatcher                            │
│                                                                   │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ npm Parser  │  │ Cargo Parser │  │  pip Parser  │  ...      │
│  │ (JSON)      │  │ (TOML)       │  │ (Text)       │           │
│  └─────────────┘  └──────────────┘  └──────────────┘           │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Dependency Aggregation                        │
│              Deduplication │ Grouping │ Metadata                 │
└────────────────────────────────┬────────────────────────────────┘
                                 │
                   ┌─────────────┴─────────────┐
                   │                           │
                   ▼                           ▼ (internal build only)
┌──────────────────────────┐   ┌──────────────────────────────────┐
│   true required)         │   │   → Vec<SastFinding>             │
└──────────────────────────┘   └──────────────────────────────────┘
                   │                           │
                   └─────────────┬─────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Format Converters                           │
│                                                                   │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐           │
│  │ Console     │  │ SPDX JSON    │  │ CycloneDX    │           │
│  │ + SAST rpt  │  │ + Tag-Value  │  │ JSON + SAST  │           │
│  └─────────────┘  └──────────────┘  └──────────────┘           │
└─────────────────────────────────────────────────────────────────┘
```

## Data Structures

### Core Types

```rust
// Dependency representation
struct Dependency {
    name: String,              // Package name
    version: String,           // Version string
    ecosystem: String,         // npm, cargo, pip, etc.
    is_dev: bool,              // Development dependency
    is_direct: bool,           // Direct vs transitive
    source: DependencySource,  // Where it was found
}

enum DependencySource {
    Manifest,    // From package.json, Cargo.toml, etc.
    LockFile,    // From lock files
    ImportScan,  // From source code scanning
}

// Main SBOM structure
struct Sbom {
    project_path: PathBuf,              // Scanned directory
    generated_at: String,                // ISO 8601 timestamp
    dependencies: Vec<Dependency>,       // Flat list
    ros_packages: Vec<RosPackageWithDeps>, // ROS multi-package
}

// ROS workspace support
struct RosPackageWithDeps {
    metadata: RosPackageMetadata,
    dependencies: Vec<Dependency>,
}

struct RosPackageMetadata {
    name: String,
    version: String,
    description: String,
    maintainers: Vec<String>,
    licenses: Vec<String>,
    build_type: String,
}
```

## Parser Implementation

### Parser Interface

All parsers follow a consistent interface:

```rust
fn parse_*_file(path: &Path) -> Result<Vec<Dependency>>
```

### Supported Parsers

| Ecosystem | Function | Format | Complexity |
|-----------|----------|--------|------------|
| npm | `parse_package_json()` | JSON | Medium |
| Cargo | `parse_cargo_toml()` | TOML | Medium |
| pip | `parse_requirements_txt()` | Text | Simple |
| Go | `parse_go_mod()` | Text | Medium |
| Ruby | `parse_gemfile()` | Text | Simple |
| PHP | `parse_composer_json()` | JSON | Simple |
| Maven | `parse_pom_xml()` | XML | Basic |
| Gradle | (Detection only) | Various | N/A |
| ROS | `parse_package_xml()` | XML | Complex |

### npm Parser Details

**Files**: `package.json`, `package-lock.json`

**Logic**:
1. Parse JSON structure
2. Extract `dependencies` section → direct deps
3. Extract `devDependencies` section → dev deps
4. Lock file provides exact versions
5. Preserve semver operators (^, ~, etc.)

### Cargo Parser Details

**Files**: `Cargo.toml`, `Cargo.lock`

**Logic**:
1. Parse TOML structure
2. Handle simple string versions: `serde = "1.0"`
3. Handle table versions: `clap = { version = "4.5", features = [...] }`
4. Extract dev-dependencies section
5. Lock file provides transitive deps

### pip Parser Details

**Files**: `requirements.txt`, `setup.py`

**Logic**:
1. Line-by-line parsing
2. Support `package==version` format
3. Support `package>=version`, `package~=version`
4. Ignore comments (`#`) and blank lines
5. Extract from `install_requires` in setup.py

### Go Parser Details

**Files**: `go.mod`, `go.sum`

**Logic**:
1. Parse `require` statements (inline and block)
2. Extract module path and version
3. Handle pseudo-versions (commit-based)
4. Indirect dependencies marked in go.mod

### ROS Parser Details

**Files**: `package.xml`, `setup.py`

**Logic**:
1. Parse XML for package metadata
2. Extract `<depend>`, `<build_depend>`, `<exec_depend>` tags
3. Cross-reference with setup.py for Python deps
4. Build hierarchy of packages in workspace

## Vendor Directory Handling

**Skip Mode** (default):
- `node_modules/`
- `vendor/`
- `target/`
- `.git/`
- `__pycache__/`
- `venv/`, `env/`

**Include Mode**:
- Scans all directories

## Import Scanning (Fallback)

When `--fallback-import-scan` is enabled:

### Python Import Scanner
```python
import requests       # Detected: requests
from django import *  # Detected: django
```

### JavaScript/TypeScript Import Scanner
```javascript
import express from 'express';  // Detected: express
const axios = require('axios'); // Detected: axios
```

### Go Import Scanner
```go
import "github.com/gin-gonic/gin"  // Detected: gin
```

**Note**: Import scanning provides package names only, no versions.

## Output Format Conversion

### Console Output
- Groups by ecosystem
- Shows dependency counts
- Emoji indicators for dev deps
- Hierarchical tree for ROS packages

### SPDX Conversion
1. Create document metadata
2. Convert each dependency to SPDXPackage
3. Generate unique SPDXIDs
4. Add Package URLs (purl)
5. Create relationships for ROS packages
6. Serialize to JSON or Tag-Value

### CycloneDX Conversion
1. Generate UUID for serial number
2. Create metadata with tool info
3. Convert dependencies to components
4. Add properties (dev, source, scope)
5. Build dependency graph
6. Serialize to JSON

## Error Handling

**Strategy**: Fail gracefully, continue on errors

| Error Type | Handling |
|------------|----------|
| Invalid path | Exit with error message |
| Not a directory | Exit with error message |
| File read error | Log warning, continue scanning |
| Parse error | Log warning, continue scanning |
| Permission denied | Skip file, continue scanning |

## Performance Characteristics

**Scanning Speed**:
- ~1000 files/second on SSD
- Minimal memory footprint (<50MB for large repos)
- Single-threaded traversal (simplicity over parallelism)

**Bottlenecks**:
- I/O bound (disk speed)
- Large manifest files (e.g., package-lock.json with 1000+ deps)

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.5 | CLI argument parsing |
| `walkdir` | 2.5 | Directory traversal |
| `toml` | 0.8 | TOML parsing (Cargo.toml) |
| `serde` | 1.0 | Serialization framework |
| `serde_json` | 1.0 | JSON parsing & serialization |
| `anyhow` | 1.0 | Error handling |
| `chrono` | 0.4 | Timestamp generation |
| `regex` | 1.10 | Pattern matching (import scanning) |
| `quick-xml` | 0.30 | XML parsing (pom.xml, package.xml) |
| `uuid` | 1.6 | UUID generation (CycloneDX) |

**Dev Dependencies**:
| Crate | Version | Purpose |
|-------|---------|---------|
| `tempfile` | 3.15 | Test fixtures |

## Security Considerations

### Input Validation
- Path traversal protection (validates input paths)
- File size limits (avoids reading huge files)
- Symlink following (disabled by default in critical operations)

### Data Privacy
- No telemetry or external network calls
- All processing happens locally
- SBOM may contain sensitive dependency information

### Known Risks
1. **Symlink Loops**: Mitigated by walkdir's loop detection
2. **Large Files**: Risk of memory exhaustion on very large manifests
3. **Malicious Manifests**: Parsers could fail on crafted inputs

## Limitations & Future Work

### Current Limitations

1. **Maven/Gradle**: Detection only, no version extraction
2. **Transitive Dependencies**: Only direct deps detected
3. **License Detection**: Not implemented
4. **Checksums**: Not calculated
5. **No Parallel Scanning**: Single-threaded for simplicity
6. **Lock File Priority**: Not always prioritized over manifests

### Planned Enhancements

**Short Term:**
- License extraction from package metadata
- Checksum calculation (SHA-256)
- Better Maven/Gradle parsing

**Medium Term:**
- Transitive dependency resolution
- Parallel file processing
- Incremental scanning
- Configuration file support

**Long Term:**
- cppcheck subprocess integration (~48 CWEs with dataflow analysis) — v1.0.17
- SBOM comparison/diff
- SBOM signing
- Plugin architecture for custom parsers



### Component Directory Resolution

Before scanning, `src/scanner/mod.rs:resolve_component_dir()` maps each manifest-declared C/C++ dependency to a vendored source directory using three strategies in order:

1. **Exact name match** — subdir with same name as dep
2. **`lib`-prefix match** — subdir named `lib{dep_name}`
3. **Case-insensitive subdir scan** — first subdir containing dep name

Dependencies with no matching vendored subdir return `None` and are skipped. This prevents inflated findings from external/system deps that appear in CMake files but are not vendored.

**Fallback mode:** When `component_dirs` is empty after manifest parsing but C/C++ files exist under the scan root (checked with `has_c_cpp_files(root, depth=3)`), a synthetic `(project_name, "C/C++") → scan_root` entry is inserted so manifest-free repos are still scanned.

### Scan Pipeline

```
component_dirs: HashMap<(name, ecosystem), PathBuf>
        │
        ▼ for each (name, "C/C++") entry
scan_file(path) → Vec<SastFinding>
        │
        ├─ read file as UTF-8 (skip on error with warning)
        ├─ tokenize: split on whitespace + punctuation
        ├─ for each token: match against CweRule table

SastFinding { component, cwe_id, cwe_name, file, line, function }
```

### Key Data Types

```rust
pub struct SastFinding {
    pub component: String,
    pub cwe_name: String,
    pub file: String,
    pub line: usize,
    pub function: String,
}

struct CweRule {
    cwe_id: &'static str,
    cwe_name: &'static str,
    functions: &'static [&'static str],
}
```

### Output Paths

| Output | Format | Condition |
|--------|--------|-----------|
| `<project>_static_analysis.md` | Markdown table | Always (internal build) |

## Code Organization

**Single File Architecture**: `src/main.rs` contains:
- Line 1-100: CLI interface & main()
- Line 100-500: Parsers (npm, Cargo, pip, Go, etc.)
- Line 500-1000: Directory scanning & file detection
- Line 1000-1500: SBOM data structures
- Line 1500-2000: Output formatters (Console, SPDX, CycloneDX)
- Line 2000-3000: Utility functions (purl generation, etc.)
- Line 3000+: Unit tests

**Rationale**: Single file simplifies deployment and reduces compile times for small projects.

## Testing Strategy

See [TESTING.md](TESTING.md) for comprehensive testing documentation.

**Test Coverage**:
- 51 unit tests
- All parsers tested
- Error handling tested
- Integration tests for multi-ecosystem projects

## Contributing

### Adding New Ecosystems

1. **Add Parser Function**:
   ```rust
   fn parse_new_ecosystem(path: &Path) -> Result<Vec<Dependency>> {
       // Implementation
   }
   ```

2. **Update File Detection**:
   ```rust
   if filename == "new-manifest-file" {
       deps.extend(parse_new_ecosystem(&entry.path())?);
   }
   ```

3. **Add purl Support**:
   ```rust
   fn create_package_url(dep: &Dependency) -> String {
       match dep.ecosystem.as_str() {
           "new-ecosystem" => format!("pkg:new-type/{}@{}", dep.name, dep.version),
           // ...
       }
   }
   ```

4. **Add Tests**:
   ```rust
   #[test]
   fn test_parse_new_ecosystem() {
       // Test implementation
   }
   ```

5. **Update Documentation**: Add to README.md and this file

### Code Style
- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Add tests for new functionality
- Update documentation

## References

- [Rust Documentation](https://doc.rust-lang.org/)
- [SPDX Specification](https://spdx.github.io/spdx-spec/)
- [CycloneDX Specification](https://cyclonedx.org/specification/overview/)
- [Package URL Specification](https://github.com/package-url/purl-spec)
