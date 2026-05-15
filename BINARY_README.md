# radeis_sc2sbom - Binary Distribution

Pre-built binaries for generating Software Bill of Materials (SBOM) from source code.

## Available Binaries

Each release ships two variants per platform: a **public** binary and an **internal** binary. The internal binary includes the C/C++ AST-based SAST scanner (v1.0.18+); the public binary is identical in all other respects.

### macOS (Apple Silicon - M1/M2/M3)
- **File:** `radeis_sc2sbom-macos-arm64` / `radeis_sc2sbom-macos-arm64-internal`
- **Architecture:** ARM64 (aarch64)
- **Platform:** macOS 11.0 or later

### macOS (Intel)
- **File:** `radeis_sc2sbom-macos-x86_64` / `radeis_sc2sbom-macos-x86_64-internal`
- **Architecture:** x86_64
- **Platform:** macOS 10.12 or later

### Linux (x86_64 static)
- **File:** `radeis_sc2sbom-linux` / `radeis_sc2sbom-linux-internal`
- **Architecture:** x86_64
- **Platform:** Any Linux x86_64 (static binary, no glibc required — runs on Ubuntu 22.04+, 24.04+, Alpine, etc.)

### Windows (x86_64)
- **File:** `radeis_sc2sbom-windows.exe` / `radeis_sc2sbom-windows-internal.exe`
- **Architecture:** x86_64
- **Platform:** Windows 10 or later

## Installation

### macOS / Linux

1. Download the appropriate binary for your platform
2. Make it executable:
   ```bash
   chmod +x radeis_sc2sbom-macos-arm64
   ```
3. (Optional) Move to a directory in your PATH:
   ```bash
   sudo mv radeis_sc2sbom-macos-arm64 /usr/local/bin/radeis_sc2sbom
   ```

### Windows

1. Download `radeis_sc2sbom-windows.exe`
2. Run from command prompt or PowerShell
3. (Optional) Add to PATH for easier access

## Quick Start

**Note:** Replace `<binary-name>` with your platform-specific binary:
- macOS ARM: `radeis_sc2sbom-macos-arm64`
- macOS Intel: `radeis_sc2sbom-macos-x86_64`
- Linux: `radeis_sc2sbom-linux`
- Windows: `radeis_sc2sbom-windows.exe`

```bash
# Basic usage - scan current directory
./<binary-name> --path .

# Scan specific project
./<binary-name> --path /path/to/project

# Generate SPDX format
./<binary-name> --path . --format spdx
```

## Common Options

| Option | Description | Example |
|--------|-------------|---------|
| `--path <PATH>` | Path to scan (default: current directory) | `--path ./my-project` |
| `--format <FORMAT>` | Output format: console, spdx, cyclonedx, all | `--format spdx` |
| `--output <DIR>` | Output directory (default: ./out) | `--output ./sbom-reports` |
| `--tree-style <STYLE>` | Dependency tree style: classic, compact, flat | `--tree-style compact` |
| `--vendor` | Include vendor directories | `--vendor` |
| `--exclude <PATTERN>` | Exclude patterns (can be used multiple times) | `--exclude "test/*"` |
| `--bsw-config <PATH>` | Custom AUTOSAR BSW module config (YAML) | `--bsw-config ./bsw.yaml` |
| `--supplier-config <PATH>` | AUTOSAR component-to-supplier mapping (YAML) | `--supplier-config ./suppliers.yaml` |

## Supported Ecosystems

### Lock Files (with hierarchical dependency trees)
- **npm** - package-lock.json
- **Cargo** (Rust) - Cargo.lock
- **Poetry** (Python) - poetry.lock

### Manifest Files
- **npm** - package.json
- **Cargo** (Rust) - Cargo.toml
- **Python** - requirements.txt, setup.py, pyproject.toml
- **Go** - go.mod
- **Maven** (Java) - pom.xml
- **ROS** - package.xml
- **JavaScript/TypeScript** - Source code imports

### C / C++ (Build Systems & Package Managers)
- **Makefile** - GNU Make build files
- **Makefile.am** - Automake build files
- **configure.ac** - Autotools configuration (pkg-config detection)
- **.mk files** - Architecture-aware Make fragment files
- **pkg-config (.pc files)** - Library dependency descriptors
- **Shared libraries (.so scanner)** - Dynamic library dependency detection
- **Vendored 3rd-party** - `3rdparty/`, `3rd_party/`, `third_party/` directory detection
- **Conan** - conanfile.txt / conanfile.py
- **vcpkg** - vcpkg.json
- **CMake** - CMakeLists.txt via FetchContent and ExternalProject_Add

### AUTOSAR
- **`.arxml` files** - Auto-detected; BSW components classified by layer, platform, and supplier
- **BSW module config** - Bundled defaults; override with `--bsw-config`
- **Supplier mapping** - Optional YAML via `--supplier-config`

## Examples

**Note:** Examples use `<binary-name>` as a placeholder. Replace with your platform-specific binary name.

### Generate complete SBOM for a Node.js project
```bash
./<binary-name> \
  --path ./my-node-app \
  --format all \
  --tree-style classic
```

### Scan Rust project
```bash
./<binary-name> \
  --path ./my-rust-app \
  --format cyclonedx
```

### Scan Python project excluding test directories
```bash
./<binary-name> \
  --path ./my-python-app \
  --exclude "tests/*" \
  --exclude "venv/*" \
  --format spdx
```

### Generate console report only with compact tree
```bash
./<binary-name> \
  --path . \
  --format console \
  --tree-style compact \
  --output ./reports
```

## Output Formats

### Console Format
Human-readable markdown report with:
- Project summary
- Dependency statistics by ecosystem
- Hierarchical dependency trees

### SPDX Format (spdx.json / spdx.spdx)
Industry-standard SBOM format compatible with:
- SPDX tools and validators
- License compliance tools
- Supply chain security platforms

### CycloneDX Format (cdx.json)
Lightweight SBOM format with:
- Component inventory
- Dependency relationships

## Help

For full command-line reference:
```bash
./<binary-name> --help
```

## Source Code & Issues

- **Repository:** https://github.com/VicOne-RD/radeis_sc2sbom
- **Issues:** https://github.com/VicOne-RD/radeis_sc2sbom/issues
- **Documentation:** See main repository README for detailed documentation

## Version Information

Check binary version:
```bash
./<binary-name> --version
```


## License

MIT License

Copyright (c) 2026 VicOne Inc.

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

