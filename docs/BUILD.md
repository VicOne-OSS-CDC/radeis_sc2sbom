# Build Guide

Complete guide for building `radeis_sc2sbom` on all platforms with cross-compilation support.

## Quick Build (All Platforms)

### Single Platform Build

**macOS/Linux:**
```bash
cargo build --release
# Binary: ./target/release/radeis_sc2sbom
```

**Windows:**
```cmd
cargo build --release
REM Binary: target\release\radeis_sc2sbom.exe
```

### Multi-Platform Build (Windows + Linux)

**From macOS/Linux:**
```bash
./build-all.sh
```

Builds both:
- `target/x86_64-pc-windows-gnu/release/radeis_sc2sbom.exe`
- `target/x86_64-unknown-linux-musl/release/radeis_sc2sbom`

## Prerequisites

### Rust Toolchain
```bash
# Install Rust (all platforms)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Cross-Compilation Toolchains

#### macOS
```bash
# MinGW-w64 for Windows cross-compilation
brew install mingw-w64

# Linux cross-compilation toolchain (musl for static binaries)
brew install FiloSottile/musl-cross/musl-cross

# Add Rust targets
rustup target add x86_64-pc-windows-gnu
rustup target add x86_64-unknown-linux-musl
```

#### Linux (Ubuntu/Debian)
```bash
# MinGW-w64 for Windows cross-compilation
sudo apt-get update
sudo apt-get install mingw-w64

# Add Rust target
rustup target add x86_64-pc-windows-gnu
```

#### Linux (RHEL/CentOS/Fedora)
```bash
# MinGW-w64 for Windows cross-compilation
sudo dnf install mingw64-gcc

# Add Rust target
rustup target add x86_64-pc-windows-gnu
```

## Manual Cross-Compilation

### Build for Windows (from macOS/Linux)

```bash
# Build
cargo build --release --target x86_64-pc-windows-gnu

# Output
target/x86_64-pc-windows-gnu/release/radeis_sc2sbom.exe
```

### Build for Linux (from macOS)

```bash
# Build (static binary via musl)
cargo build --release --target x86_64-unknown-linux-musl

# Output
target/x86_64-unknown-linux-musl/release/radeis_sc2sbom
```

### Build for macOS ARM64 (M1/M2/M3)

```bash
# Native build on Apple Silicon
cargo build --release
# Output: target/release/radeis_sc2sbom
```

### Build for macOS Intel (x86_64)

```bash
# Cross-compile on Apple Silicon
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Output: target/x86_64-apple-darwin/release/radeis_sc2sbom
```

## Build Configuration

### Cargo Configuration (`.cargo/config.toml`)

```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"

[target.x86_64-unknown-linux-musl]
linker = "musl-gcc"
```

This configuration is already included in the repository. The musl linker defaults
to `musl-gcc` (the Linux-native name provided by Ubuntu/Debian's `musl-tools`).
macOS users installing via `brew install FiloSottile/musl-cross/musl-cross` get
the binary named `x86_64-linux-musl-gcc` — `build-all.sh` auto-detects which is
available and sets `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER` accordingly.
When invoking `cargo build` directly on macOS, either export that env var yourself
or create a `musl-gcc` symlink pointing at `x86_64-linux-musl-gcc`.

## Build Script Details

The `build-all.sh` script performs:

1. **Prerequisite Checks:**
   - Verifies `rustup` installation
   - Checks for required targets
   - Validates cross-compilation toolchains

2. **Parallel Builds:**
   - Windows (x86_64-pc-windows-gnu)
   - Linux (x86_64-unknown-linux-musl, static)

3. **Error Handling:**
   - Exits on first build failure
   - Provides clear error messages
   - Shows binary locations and sizes

### Running the Build Script

```bash
# Make executable (first time only)
chmod +x build-all.sh

# Run build
./build-all.sh
```

## Installation

### System-Wide Installation

**Linux/macOS:**
```bash
# Option 1: Copy to /usr/local/bin
sudo cp ./target/release/radeis_sc2sbom /usr/local/bin/

# Option 2: Copy to ~/.local/bin (user-specific)
mkdir -p ~/.local/bin
cp ./target/release/radeis_sc2sbom ~/.local/bin/
# Add to PATH: export PATH="$HOME/.local/bin:$PATH"

# Verify
radeis_sc2sbom --version
```

**Windows:**
```cmd
REM Option 1: Copy to Windows directory
copy target\release\radeis_sc2sbom.exe C:\Windows\System32\

REM Option 2: Create custom bin folder
mkdir C:\bin
copy target\release\radeis_sc2sbom.exe C:\bin\
REM Add C:\bin to PATH via System Properties

REM Verify
radeis_sc2sbom --version
```

### Shell Alias

**Bash/Zsh:**
```bash
# Add to ~/.bashrc or ~/.zshrc
alias sbom='radeis_sc2sbom'

# Reload shell
source ~/.bashrc  # or ~/.zshrc

# Use
sbom --path .
```

**PowerShell:**
```powershell
# Add to $PROFILE
Set-Alias sbom radeis_sc2sbom

# Use
sbom --path .
```

## Distribution

### Creating Release Artifacts

```bash
# Build all platforms
./build-all.sh

# Create distribution directory
mkdir -p dist

# Copy binaries
cp target/x86_64-pc-windows-gnu/release/radeis_sc2sbom.exe dist/radeis_sc2sbom-windows-x86_64.exe
cp target/x86_64-unknown-linux-musl/release/radeis_sc2sbom dist/radeis_sc2sbom-linux-x86_64
cp target/release/radeis_sc2sbom dist/radeis_sc2sbom-macos-$(uname -m)

# Copy documentation
cp README.md dist/
cp LICENSE dist/

# Create archives
cd dist
tar -czf radeis_sc2sbom-linux-x86_64.tar.gz radeis_sc2sbom-linux-x86_64 README.md LICENSE

tar -czf radeis_sc2sbom-macos-$(uname -m).tar.gz radeis_sc2sbom-macos-$(uname -m) README.md LICENSE
zip radeis_sc2sbom-windows-x86_64.zip radeis_sc2sbom-windows-x86_64.exe README.md LICENSE
```

### GitHub Release Automation

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            artifact: radeis_sc2sbom-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: radeis_sc2sbom-macos-x86_64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: radeis_sc2sbom-windows-x86_64.exe

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: ${{ matrix.target }}

      - name: Build
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v3
        with:
          name: ${{ matrix.artifact }}
          path: target/${{ matrix.target }}/release/radeis_sc2sbom*
```

## Optimization

### Release Profile

The `Cargo.toml` includes optimizations:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization
strip = true         # Remove debug symbols
```

### Binary Size Reduction

```bash
# Additional stripping (if needed)
strip target/release/radeis_sc2sbom

# UPX compression (optional)
upx --best --lzma target/release/radeis_sc2sbom
```

## Troubleshooting

### Common Issues

**Issue: MinGW linker not found**
```
error: linker `x86_64-w64-mingw32-gcc` not found
```
**Solution:**
```bash
# macOS
brew install mingw-w64

# Linux
sudo apt-get install mingw-w64
```

**Issue: Linux cross-compiler not found**
```
error: linker `musl-gcc` not found
```
or
```
error: linker `x86_64-linux-musl-gcc` not found
```

The musl toolchain ships under two different binary names:
- Ubuntu/Debian `musl-tools` provides `musl-gcc`
- macOS `FiloSottile/musl-cross/musl-cross` provides `x86_64-linux-musl-gcc`

`.cargo/config.toml` defaults to `musl-gcc`. `build-all.sh` auto-detects which is
present and overrides via `CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER`; when
running `cargo build` directly you may need to do this yourself.

**Solution (Ubuntu/Debian):**
```bash
sudo apt-get install musl-tools
```

**Solution (macOS):**
```bash
brew install FiloSottile/musl-cross/musl-cross
# Then either export the env var for direct cargo invocations:
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
# Or create a symlink so the default config works:
ln -s "$(brew --prefix)/bin/x86_64-linux-musl-gcc" "$(brew --prefix)/bin/musl-gcc"
```

**Issue: Rust target not installed**
```
error: target `x86_64-pc-windows-gnu` not found
```
**Solution:**
```bash
rustup target add x86_64-pc-windows-gnu
```

**Issue: Permission denied on build-all.sh**
```
bash: ./build-all.sh: Permission denied
```
**Solution:**
```bash
chmod +x build-all.sh
```

### Build Performance

**Parallel Builds:**
```bash
# Use all CPU cores
cargo build --release --jobs $(nproc)

# Or specify number
cargo build --release --jobs 4
```

**Incremental Builds:**
```bash
# Enable incremental compilation (default in dev)
export CARGO_INCREMENTAL=1
cargo build
```

**Clean Build:**
```bash
# Remove all build artifacts
cargo clean

# Rebuild from scratch
cargo build --release
```

## Verification

### Test Binaries

```bash
# Test native binary
./target/release/radeis_sc2sbom --version

# Test Windows binary (with Wine on Linux/macOS)
wine target/x86_64-pc-windows-gnu/release/radeis_sc2sbom.exe --version

# Test Linux binary (on Linux)
./target/x86_64-unknown-linux-musl/release/radeis_sc2sbom --version
```

### Binary Information

```bash
# File type
file target/release/radeis_sc2sbom

# Size
ls -lh target/release/radeis_sc2sbom

# Dependencies (Linux)
ldd target/release/radeis_sc2sbom

# Dependencies (macOS)
otool -L target/release/radeis_sc2sbom

# Dependencies (Windows with objdump)
objdump -p target/x86_64-pc-windows-gnu/release/radeis_sc2sbom.exe | grep DLL
```

## Additional Platforms

### Linux ARM64

```bash
# Add target
rustup target add aarch64-unknown-linux-gnu

# Install cross-compiler
sudo apt-get install gcc-aarch64-linux-gnu

# Build
cargo build --release --target aarch64-unknown-linux-gnu
```

### Windows ARM64

```bash
# Add target
rustup target add aarch64-pc-windows-msvc

# Build (requires Windows SDK)
cargo build --release --target aarch64-pc-windows-msvc
```

## Resources

- [Rust Cross-Compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [Cargo Build Reference](https://doc.rust-lang.org/cargo/commands/cargo-build.html)
- [MinGW-w64 Documentation](https://www.mingw-w64.org/)
- [GitHub Actions for Rust](https://github.com/actions-rs)
