#!/bin/bash
#
# Generate PDF from BINARY_README.md using xnexus-md2pdf-tool
#
# Usage:
#   ./scripts/generate_binary_readme_pdf.sh [version]
#
# Example:
#   ./scripts/generate_binary_readme_pdf.sh 1.0.5
#
# Requirements:
#   - xnexus-md2pdf-tool must exist at ../xnexus-md2pdf-tool
#   - Python 3 with virtualenv
#

set -e  # Exit on error

# Get script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MD2PDF_TOOL="$PROJECT_ROOT/../xnexus-md2pdf-tool"

# Default values
VERSION="${1:-$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')}"
INPUT_MD="$PROJECT_ROOT/BINARY_README.md"
OUTPUT_DIR="$PROJECT_ROOT/release_assets"
OUTPUT_PDF="README.pdf"

# Validate xnexus-md2pdf-tool exists
if [ ! -d "$MD2PDF_TOOL" ]; then
    echo "❌ ERROR: xnexus-md2pdf-tool not found at: $MD2PDF_TOOL"
    echo "Please ensure the tool is cloned at ../xnexus-md2pdf-tool relative to this project"
    echo ""
    echo "To clone the tool, run:"
    echo "  cd $PROJECT_ROOT/.."
    echo "  git clone git@github.com:VicOne-RD/xnexus-md2pdf-tool.git"
    exit 1
fi

# Convert to absolute path after validation
MD2PDF_TOOL="$(cd "$MD2PDF_TOOL" && pwd)"

if [ ! -f "$MD2PDF_TOOL/src/build_all.py" ]; then
    echo "❌ ERROR: build_all.py not found in xnexus-md2pdf-tool/src/"
    exit 1
fi

# Validate input markdown exists
if [ ! -f "$INPUT_MD" ]; then
    echo "❌ ERROR: BINARY_README.md not found at: $INPUT_MD"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "=========================================="
echo "Generating PDF from BINARY_README.md"
echo "=========================================="
echo "Version: v$VERSION"
echo "Input: $INPUT_MD"
echo "Output: $OUTPUT_DIR/$OUTPUT_PDF"
echo "Tool: $MD2PDF_TOOL"
echo ""

# Check if virtual environment exists in md2pdf tool
if [ ! -d "$MD2PDF_TOOL/.venv" ]; then
    echo "⚠️  Virtual environment not found in xnexus-md2pdf-tool"
    echo "Setting up Python environment..."
    cd "$MD2PDF_TOOL"
    python3 -m venv .venv
    source .venv/bin/activate
    pip install -r requirements.txt
    deactivate
    cd "$PROJECT_ROOT"
    echo "✅ Python environment setup complete"
    echo ""
fi

# Activate virtual environment and run conversion
echo "🔄 Converting markdown to PDF..."
cd "$MD2PDF_TOOL"
source .venv/bin/activate

python src/build_all.py \
  --md "$INPUT_MD" \
  --outdir "$OUTPUT_DIR" \
  --out-pdf "$OUTPUT_DIR/$OUTPUT_PDF" \
  --title "radeis_sc2sbom v${VERSION}" \
  --generated "Generated: $(date '+%B %Y')" \
  --product-name "radeis_sc2sbom" \
  --no-dashboard \
  --no-inline-charts \
  --log-level INFO

deactivate
cd "$PROJECT_ROOT"

# Verify output
if [ -f "$OUTPUT_DIR/$OUTPUT_PDF" ]; then
    FILE_SIZE=$(du -h "$OUTPUT_DIR/$OUTPUT_PDF" | cut -f1)
    echo ""
    echo "=========================================="
    echo "✅ PDF generated successfully!"
    echo "=========================================="
    echo "File: $OUTPUT_DIR/$OUTPUT_PDF"
    echo "Size: $FILE_SIZE"
    echo ""
else
    echo ""
    echo "❌ ERROR: PDF generation failed"
    echo "Expected output: $OUTPUT_DIR/$OUTPUT_PDF"
    exit 1
fi

echo ""
echo "✅ Done!"
