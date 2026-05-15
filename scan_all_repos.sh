#!/bin/bash
# Scan all repositories in example_target_repos and generate SBOMs with reports
# Supports both production (runtime-only) and complete SBOM modes

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SBOM_BINARY="$SCRIPT_DIR/target/release/radeis_sc2sbom"
REPOS_DIR="$SCRIPT_DIR/example_target_repos"
OUTPUT_DIR="$SCRIPT_DIR/scan_reports/radeis_sc2sbom"

# Default mode: complete (all dependencies)
SCAN_MODE="complete"
PRODUCTION_FLAG=""
INTERNAL_FLAG=""
VULN_FLAG=""

# Color output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Parse arguments
show_usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Scan all repositories in example_target_repos and generate SBOMs.

OPTIONS:
    --production        Generate production SBOMs (runtime + optional dependencies only)
    --complete          Generate complete SBOMs with all dependencies (default)
    --internal          Use internal binary (includes C/C++ SAST scanner)
    --vulnerabilities   Enable vulnerability scanning (requires --internal)
    --help              Show this help message

EXAMPLES:
    # Scan all repos with complete SBOMs (default)
    $0

    # Scan all repos with production SBOMs (runtime-only)
    $0 --production

    # Internal build with SAST and vulnerability scanning
    $0 --internal --vulnerabilities

EOF
    exit 0
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --production)
            SCAN_MODE="production"
            PRODUCTION_FLAG="--production"
            shift
            ;;
        --complete)
            SCAN_MODE="complete"
            PRODUCTION_FLAG=""
            shift
            ;;
        --internal)
            INTERNAL_FLAG="internal"
            shift
            ;;
        --vulnerabilities)
            VULN_FLAG="--check-vulnerabilities true"
            shift
            ;;
        --help|-h)
            show_usage
            ;;
        *)
            echo -e "${RED}Error: Unknown option '$1'${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Select binary: prefer internal if requested and available
if [ -n "$INTERNAL_FLAG" ]; then
    INTERNAL_BINARY="$SCRIPT_DIR/target/release/radeis_sc2sbom-internal"
    if [ -f "$INTERNAL_BINARY" ]; then
        SBOM_BINARY="$INTERNAL_BINARY"
        echo -e "${YELLOW}Using internal binary: $SBOM_BINARY${NC}"
    else
        echo -e "${YELLOW}Internal binary not found, falling back to release binary${NC}"
        INTERNAL_FLAG=""
    fi
fi

# Build the project if needed
if [ ! -f "$SBOM_BINARY" ]; then
    echo -e "${YELLOW}Building radeis_sc2sbom...${NC}"
    cd "$SCRIPT_DIR"
    if [ -n "$INTERNAL_FLAG" ]; then
        cargo build --release --features internal
    else
        cargo build --release
    fi
    echo -e "${GREEN}Build completed!${NC}\n"
fi

# Get tool version from Cargo.toml (reliable; binary doesn't expose --version)
RADEIS_VERSION=$(grep '^version' "$SCRIPT_DIR/Cargo.toml" | head -1 | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || echo "unknown")

echo -e "${BLUE}=== radeis_sc2sbom Multi-Repository Scanner ===${NC}"
echo -e "${BLUE}Version: ${RADEIS_VERSION}${NC}"
echo -e "${BLUE}Mode: ${SCAN_MODE} SBOM${NC}"
[ -n "$INTERNAL_FLAG" ] && echo -e "${BLUE}Build: internal (SAST enabled)${NC}"
[ -n "$VULN_FLAG" ] && echo -e "${BLUE}Vulnerability scanning: enabled${NC}"
echo ""

if [ "$SCAN_MODE" = "production" ]; then
    echo -e "${YELLOW}Production Mode: Generating runtime + optional dependencies only${NC}\n"
else
    echo -e "${YELLOW}Complete Mode: Including all dependencies (runtime, build, test, dev)${NC}\n"
fi

# Create output directory with mode suffix
if [ "$SCAN_MODE" = "production" ]; then
    OUTPUT_DIR="${OUTPUT_DIR}_production"
else
    OUTPUT_DIR="${OUTPUT_DIR}_complete"
fi
mkdir -p "$OUTPUT_DIR"

# Get list of repos: directories only, skip hidden, skip *.zip
REPOS=()
while IFS= read -r -d '' dir; do
    name="$(basename "$dir")"
    REPOS+=("$name")
done < <(find "$REPOS_DIR" -maxdepth 1 -mindepth 1 -type d ! -name ".*" -print0 | sort -z)

echo -e "${BLUE}Found ${#REPOS[@]} repositories to scan:${NC}"
for repo in "${REPOS[@]}"; do
    echo "  - $repo"
done
echo ""

# Track statistics
REPO_NAMES=()
REPO_PKG_COUNTS=()

get_pkg_count() {
    local repo_name="$1"
    for i in "${!REPO_NAMES[@]}"; do
        if [ "${REPO_NAMES[$i]}" = "$repo_name" ]; then
            echo "${REPO_PKG_COUNTS[$i]}"
            return
        fi
    done
    echo "N/A"
}

# Scan each repository
REPO_NUM=0
for repo in "${REPOS[@]}"; do
    REPO_NUM=$((REPO_NUM + 1))
    REPO_PATH="$REPOS_DIR/$repo"

    # Get repo version
    REPO_VERSION="unknown"
    if [ -d "$REPO_PATH/.git" ]; then
        cd "$REPO_PATH"
        REPO_VERSION=$(git describe --tags --always 2>/dev/null || git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        cd "$SCRIPT_DIR"
    fi

    # Output dir uses tool version + build variant
    BUILD_TAG="${RADEIS_VERSION}"
    [ -n "$INTERNAL_FLAG" ] && BUILD_TAG="${RADEIS_VERSION}_internal"
    OUTPUT_SUBDIR="$OUTPUT_DIR/${repo}_${BUILD_TAG}_${SCAN_MODE}_result"

    echo -e "\n${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}[${REPO_NUM}/${#REPOS[@]}] Scanning: ${NC}${YELLOW}$repo${NC}"
    echo -e "${BLUE}Repository Version:${NC} $REPO_VERSION"
    echo -e "${BLUE}SBOM Mode:${NC} $SCAN_MODE"
    echo -e "${BLUE}Output:${NC} $OUTPUT_SUBDIR"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    mkdir -p "$OUTPUT_SUBDIR"

    LOG_FILE="$OUTPUT_SUBDIR/${repo}_scan.log"

    # Build scan command
    SCAN_CMD=(
        "$SBOM_BINARY"
        --path "$REPO_PATH"
        --output "$OUTPUT_SUBDIR"
        --format all
    )
    [ -n "$PRODUCTION_FLAG" ] && SCAN_CMD+=($PRODUCTION_FLAG)
    [ -n "$VULN_FLAG" ] && SCAN_CMD+=($VULN_FLAG)

    "${SCAN_CMD[@]}" > "$LOG_FILE" 2>&1 &
    SCAN_PID=$!

    SCAN_EXIT_CODE=0
    if ! wait $SCAN_PID; then
        SCAN_EXIT_CODE=$?
    fi

    # Extract package count from SPDX JSON (subtract 1 for synthetic root)
    if [ -f "$OUTPUT_SUBDIR/${repo}_spdx.json" ]; then
        PKG_COUNT_RAW=$(jq '.packages | length' "$OUTPUT_SUBDIR/${repo}_spdx.json" 2>/dev/null || echo "N/A")
        if [[ "$PKG_COUNT_RAW" =~ ^[0-9]+$ ]] && [ "$PKG_COUNT_RAW" -gt 0 ]; then
            PKG_COUNT=$((PKG_COUNT_RAW - 1))
        else
            PKG_COUNT="$PKG_COUNT_RAW"
        fi

        VULN_COUNT=$(grep -o "Found [0-9]* vulnerabilities" "$LOG_FILE" 2>/dev/null | grep -o "[0-9]*" || echo "0")

        REPO_NAMES+=("$repo")
        REPO_PKG_COUNTS+=("$PKG_COUNT")

        if [ "${VULN_COUNT:-0}" -gt 0 ] 2>/dev/null; then
            echo -e "${GREEN}✓ Completed:${NC} $repo"
            echo -e "  ${BLUE}Packages:${NC} $PKG_COUNT | ${RED}Vulnerabilities:${NC} $VULN_COUNT"
        else
            echo -e "${GREEN}✓ Completed:${NC} $repo"
            echo -e "  ${BLUE}Packages:${NC} $PKG_COUNT"
        fi
    else
        REPO_NAMES+=("$repo")
        REPO_PKG_COUNTS+=("N/A")
        if [ $SCAN_EXIT_CODE -ne 0 ]; then
            echo -e "${RED}✗ Failed:${NC} $repo (exit $SCAN_EXIT_CODE — check: $LOG_FILE)"
        else
            echo -e "${GREEN}✓ Completed:${NC} $repo (no SPDX output)"
        fi
    fi
done

# Create master index
SCAN_MODE_UPPER=$(echo "$SCAN_MODE" | tr '[:lower:]' '[:upper:]')
BUILD_TAG="${RADEIS_VERSION}"
[ -n "$INTERNAL_FLAG" ] && BUILD_TAG="${RADEIS_VERSION}_internal"

INDEX_FILE="$OUTPUT_DIR/../INDEX_${SCAN_MODE}.md"
cat > "$INDEX_FILE" << EOF
# radeis_sc2sbom Scan Results Index (${SCAN_MODE_UPPER} MODE)

**Generated:** $(date '+%Y-%m-%d %H:%M:%S')
**Tool Version:** radeis_sc2sbom v${RADEIS_VERSION}$([ -n "$INTERNAL_FLAG" ] && echo " (internal)")
**SBOM Mode:** ${SCAN_MODE}
**Total Repositories Scanned:** ${#REPOS[@]}

## Scanned Repositories

| Repository | Version | Packages | Reports |
|------------|---------|----------|---------|
EOF

for repo in "${REPOS[@]}"; do
    REPO_VERSION="unknown"
    if [ -d "$REPOS_DIR/$repo/.git" ]; then
        cd "$REPOS_DIR/$repo"
        REPO_VERSION=$(git describe --tags --always 2>/dev/null || git rev-parse --short HEAD 2>/dev/null || echo "unknown")
        cd "$SCRIPT_DIR"
    fi

    OUTPUT_SUBDIR="${repo}_${BUILD_TAG}_${SCAN_MODE}_result"
    PKG_COUNT=$(get_pkg_count "$repo")

    cat >> "$INDEX_FILE" << EOF
| **$repo** | \`$REPO_VERSION\` | **$PKG_COUNT** | [Report](./radeis_sc2sbom_${SCAN_MODE}/$OUTPUT_SUBDIR/${repo}_report.md) · [SPDX](./radeis_sc2sbom_${SCAN_MODE}/$OUTPUT_SUBDIR/${repo}_spdx.json) · [CDX](./radeis_sc2sbom_${SCAN_MODE}/$OUTPUT_SUBDIR/${repo}_cyclonedx.json) · [Log](./radeis_sc2sbom_${SCAN_MODE}/$OUTPUT_SUBDIR/${repo}_scan.log) |
EOF
done

echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ All Scans Complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"

echo -e "${BLUE}Scan Summary:${NC}"
echo -e "  ${BLUE}Mode:${NC} ${YELLOW}${SCAN_MODE}${NC}"
echo -e "  ${BLUE}Repositories Scanned:${NC} ${#REPOS[@]}"
echo -e "  ${BLUE}Tool Version:${NC} radeis_sc2sbom v${RADEIS_VERSION}$([ -n "$INTERNAL_FLAG" ] && echo " (internal)")"
echo ""

TOTAL_PACKAGES=0
for repo in "${REPOS[@]}"; do
    PKG_COUNT=$(get_pkg_count "$repo")
    if [[ "$PKG_COUNT" =~ ^[0-9]+$ ]]; then
        TOTAL_PACKAGES=$((TOTAL_PACKAGES + PKG_COUNT))
    fi
done

echo -e "${BLUE}Package Statistics:${NC}"
echo -e "┌─────────────────────────────────────┬──────────────┐"
echo -e "│ ${BLUE}Repository${NC}                          │ ${BLUE}Packages${NC}     │"
echo -e "├─────────────────────────────────────┼──────────────┤"
for repo in "${REPOS[@]}"; do
    PKG_COUNT=$(get_pkg_count "$repo")
    printf "│ %-35s │ %12s │\n" "$repo" "$PKG_COUNT"
done
echo -e "├─────────────────────────────────────┼──────────────┤"
printf "│ ${YELLOW}%-35s${NC} │ ${YELLOW}%12s${NC} │\n" "TOTAL" "$TOTAL_PACKAGES"
echo -e "└─────────────────────────────────────┴──────────────┘"
echo ""

echo -e "${BLUE}Output Locations:${NC}"
echo -e "  ${BLUE}Results Directory:${NC} $OUTPUT_DIR"
echo -e "  ${BLUE}Index File:${NC} $INDEX_FILE"
echo ""
