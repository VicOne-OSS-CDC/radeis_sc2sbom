#!/bin/bash

# Build script for multiple platforms
# Default: Builds ARM-based macOS only (Apple Silicon M1/M2/M3)
# With --all flag: Builds Intel macOS, ARM macOS, Linux, and Windows
# With --internal flag: also builds internal variants (--features internal) alongside public ones
# Artifacts are copied to ./dist by default
#
# Output naming:
#   Public:   radeis_sc2sbom-{platform}          (e.g. radeis_sc2sbom-macos-arm64)
#   Internal: radeis_sc2sbom-{platform}-internal  (e.g. radeis_sc2sbom-macos-arm64-internal)

set -euo pipefail  # Exit on error, treat unset vars as error, and fail on pipeline errors

# Output directory
DIST_DIR="${DIST_DIR:-./dist}"

# Build modes
BUILD_ALL=false
BUILD_INTERNAL=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        -all|--all)
            BUILD_ALL=true
            shift
            ;;
        --internal)
            BUILD_INTERNAL=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -all, --all  Build all platforms (Intel macOS, ARM macOS, Linux, Windows)"
            echo "  --internal   Also build internal variants (--features internal) for each platform"
            echo "  --help, -h   Show this help message"
            echo ""
            echo "Environment variables:"
            echo "  DIST_DIR     Output directory (default: ./dist)"
            echo ""
            echo "Examples:"
            echo "  $0                         # Build ARM macOS public only (default)"
            echo "  $0 --internal              # Build ARM macOS public + internal"
            echo "  $0 --all                   # Build all platforms, public only"
            echo "  $0 --all --internal        # Build all platforms, public + internal"
            echo "  DIST_DIR=./out $0 --all    # Custom output directory"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo "======================================"
if [ "$BUILD_ALL" = true ]; then
    echo "Building for all platforms..."
else
    echo "Building for ARM macOS only..."
fi
if [ "$BUILD_INTERNAL" = true ]; then
    echo "Variants: public + internal (--features internal)"
else
    echo "Variants: public only"
fi
echo "======================================"
echo ""

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}Error: rustup is not installed${NC}"
    echo "Please install Rust and rustup from https://rustup.rs/"
    exit 1
fi

# Detect host OS for platform-specific warnings
HOST_OS=$(uname -s)
if [ "$BUILD_ALL" = false ] && [ "$HOST_OS" != "Darwin" ]; then
    echo -e "${YELLOW}Warning: Default target is ARM macOS (aarch64-apple-darwin)${NC}"
    echo -e "${YELLOW}Building macOS targets on non-macOS hosts requires Apple cross-compilation toolchains${NC}"
    echo -e "${YELLOW}This build may fail. You can use '--all' to also build for your native platform;${NC}"
    echo -e "${YELLOW}macOS targets may still fail without Apple cross-compilation toolchains.${NC}"
    echo ""
fi

# Define targets
MACOS_ARM_TARGET="aarch64-apple-darwin"
MACOS_INTEL_TARGET="x86_64-apple-darwin"
LINUX_TARGET="x86_64-unknown-linux-musl"
WINDOWS_TARGET="x86_64-pc-windows-gnu"

# Check and install ARM macOS target (always needed)
if ! rustup target list --installed | grep -q "$MACOS_ARM_TARGET"; then
    echo -e "${YELLOW}Info: ARM macOS target ($MACOS_ARM_TARGET) is not installed${NC}"
    echo "Installing target..."
    rustup target add "$MACOS_ARM_TARGET"
fi

# Check and install other targets if -all flag is set
if [ "$BUILD_ALL" = true ]; then
    if ! rustup target list --installed | grep -q "$MACOS_INTEL_TARGET"; then
        echo -e "${YELLOW}Info: Intel macOS target ($MACOS_INTEL_TARGET) is not installed${NC}"
        echo "Installing target..."
        rustup target add "$MACOS_INTEL_TARGET"
    fi

    if ! rustup target list --installed | grep -q "$LINUX_TARGET"; then
        echo -e "${YELLOW}Info: Linux target ($LINUX_TARGET) is not installed${NC}"
        echo "Installing target..."
        rustup target add "$LINUX_TARGET"
    fi

    if ! rustup target list --installed | grep -q "$WINDOWS_TARGET"; then
        echo -e "${YELLOW}Info: Windows target ($WINDOWS_TARGET) is not installed${NC}"
        echo "Installing target..."
        rustup target add "$WINDOWS_TARGET"
    fi

    # Check if MinGW-w64 toolchain is available for Windows
    if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
        echo -e "${YELLOW}Warning: MinGW-w64 toolchain (x86_64-w64-mingw32-gcc) not found${NC}"
        echo "Windows cross-compilation may fail. Install with:"
        echo "  - macOS: brew install mingw-w64"
        echo "  - Ubuntu/Debian: sudo apt-get install mingw-w64"
        echo ""
    fi

    # Check if musl toolchain is available for Linux.
    # The binary ships under two different names depending on platform:
    #   - Ubuntu/Debian `musl-tools` provides `musl-gcc`
    #   - macOS `FiloSottile/musl-cross/musl-cross` provides `x86_64-linux-musl-gcc`
    # `.cargo/config.toml` defaults to `musl-gcc`; if only the macOS-style binary
    # is present, override the linker via env var so Cargo finds it.
    if command -v musl-gcc &> /dev/null; then
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
    elif command -v x86_64-linux-musl-gcc &> /dev/null; then
        export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=x86_64-linux-musl-gcc
    else
        echo -e "${YELLOW}Warning: musl cross-compilation toolchain not found${NC}"
        echo "Looked for: musl-gcc (Linux) and x86_64-linux-musl-gcc (macOS)"
        echo "Linux static build may fail. Install with:"
        echo "  - macOS: brew install FiloSottile/musl-cross/musl-cross"
        echo "  - Ubuntu/Debian: sudo apt-get install musl-tools"
        echo ""
    fi
fi

echo -e "${GREEN}✓ Prerequisites check complete${NC}"
echo ""

# Determine build count (each platform = 1 public + optionally 1 internal)
PLATFORM_COUNT=1
[ "$BUILD_ALL" = true ] && PLATFORM_COUNT=4
VARIANT_COUNT=1
[ "$BUILD_INTERNAL" = true ] && VARIANT_COUNT=2
BUILD_COUNT=$((PLATFORM_COUNT * VARIANT_COUNT))

BUILD_NUM=1

# Helper: build one target, one variant
build_target() {
    local label="$1"
    local target="$2"
    local features="$3"   # "" for public, "--features internal" for internal

    if [ -n "$features" ]; then
        echo -e "${BLUE}[$BUILD_NUM/$BUILD_COUNT] Building $label (internal) ($target)...${NC}"
        cargo build --release $features --target "$target"
    else
        echo -e "${BLUE}[$BUILD_NUM/$BUILD_COUNT] Building $label (public) ($target)...${NC}"
        cargo build --release --target "$target"
    fi
    echo -e "${GREEN}✓ Done${NC}"
    echo ""
    BUILD_NUM=$((BUILD_NUM + 1))
}

# Build for ARM macOS (always built)
build_target "ARM macOS" "$MACOS_ARM_TARGET" ""
[ "$BUILD_INTERNAL" = true ] && build_target "ARM macOS" "$MACOS_ARM_TARGET" "--features internal"

# Build other platforms if --all flag is set
if [ "$BUILD_ALL" = true ]; then
    build_target "Intel macOS" "$MACOS_INTEL_TARGET" ""
    [ "$BUILD_INTERNAL" = true ] && build_target "Intel macOS" "$MACOS_INTEL_TARGET" "--features internal"

    build_target "Linux" "$LINUX_TARGET" ""
    [ "$BUILD_INTERNAL" = true ] && build_target "Linux" "$LINUX_TARGET" "--features internal"

    build_target "Windows" "$WINDOWS_TARGET" ""
    [ "$BUILD_INTERNAL" = true ] && build_target "Windows" "$WINDOWS_TARGET" "--features internal"
fi

# Copy artifacts to dist directory
echo -e "${BLUE}Copying artifacts to ${DIST_DIR}...${NC}"
mkdir -p "${DIST_DIR}"

# Helper: copy one binary, appending -internal suffix when needed
copy_binary() {
    local src="$1"       # path inside target/…/release/
    local dest_base="$2" # e.g. radeis_sc2sbom-macos-arm64
    local internal="$3"  # "true" or ""

    local dest_name="$dest_base"
    [ "$internal" = "true" ] && dest_name="${dest_base}-internal"

    if [ -f "$src" ]; then
        cp "$src" "${DIST_DIR}/${dest_name}"
        # Windows .exe already handles its own extension; chmod is a no-op on .exe but harmless
        [[ "$src" != *.exe ]] && chmod +x "${DIST_DIR}/${dest_name}"
        echo -e "${GREEN}✓ ${dest_name}${NC}"
    fi
}

# ARM macOS
copy_binary "target/$MACOS_ARM_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-macos-arm64" ""
[ "$BUILD_INTERNAL" = true ] && copy_binary "target/$MACOS_ARM_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-macos-arm64" "true"

if [ "$BUILD_ALL" = true ]; then
    # Intel macOS
    copy_binary "target/$MACOS_INTEL_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-macos-x86_64" ""
    [ "$BUILD_INTERNAL" = true ] && copy_binary "target/$MACOS_INTEL_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-macos-x86_64" "true"

    # Linux
    copy_binary "target/$LINUX_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-linux" ""
    [ "$BUILD_INTERNAL" = true ] && copy_binary "target/$LINUX_TARGET/release/radeis_sc2sbom" "radeis_sc2sbom-linux" "true"

    # Windows
    copy_binary "target/$WINDOWS_TARGET/release/radeis_sc2sbom.exe" "radeis_sc2sbom-windows.exe" ""
    [ "$BUILD_INTERNAL" = true ] && copy_binary "target/$WINDOWS_TARGET/release/radeis_sc2sbom.exe" "radeis_sc2sbom-windows.exe" "true"
fi

# Copy README to dist directory
if [ -f "BINARY_README.md" ]; then
    cp "BINARY_README.md" "${DIST_DIR}/README.md"
    echo -e "${GREEN}✓ Copied README to ${DIST_DIR}/README.md${NC}"
else
    echo -e "${RED}✗ BINARY_README.md not found. Failing build to avoid shipping binaries without documentation.${NC}" >&2
    exit 1
fi

echo ""

# Helper function to print file info robustly
print_file_info() {
    local file_path="$1"
    if [ -f "$file_path" ]; then
        # Use du for cross-platform human-readable size
        local size
        size=$(du -h "$file_path" 2>/dev/null | awk '{print $1}')
        echo "  $file_path ($size)"
    fi
}

# Show results
echo "======================================"
echo "Build Summary:"
echo "======================================"
echo ""
echo "Artifacts in ${DIST_DIR}:"
echo ""

# Helper: print one row, with optional -internal sibling
print_platform() {
    local label="$1"
    local base="$2"   # e.g. radeis_sc2sbom-macos-arm64
    local ext="${3:-}" # e.g. ".exe" or ""

    echo "$label:"
    if [ -f "${DIST_DIR}/${base}${ext}" ]; then
        print_file_info "${DIST_DIR}/${base}${ext}"
    else
        echo -e "  ${RED}(not built)${NC}"
    fi
    if [ "$BUILD_INTERNAL" = true ]; then
        if [ -f "${DIST_DIR}/${base}-internal${ext}" ]; then
            print_file_info "${DIST_DIR}/${base}-internal${ext}"
        else
            echo -e "  ${RED}(internal not built)${NC}"
        fi
    fi
    echo ""
}

print_platform "ARM macOS" "radeis_sc2sbom-macos-arm64"

if [ "$BUILD_ALL" = true ]; then
    print_platform "Intel macOS" "radeis_sc2sbom-macos-x86_64"
    print_platform "Linux"       "radeis_sc2sbom-linux"
    print_platform "Windows"     "radeis_sc2sbom-windows" ".exe"
fi

echo -e "${GREEN}All builds completed successfully!${NC}"
echo -e "${BLUE}Artifacts available in: ${DIST_DIR}${NC}"
