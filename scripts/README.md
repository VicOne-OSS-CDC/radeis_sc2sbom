# Build Scripts

This directory contains utility scripts for the radeis_sc2sbom project.

## generate_binary_readme_pdf.sh

Converts BINARY_README.md to a VicOne-styled PDF using the xnexus-md2pdf-tool.

### Prerequisites

1. **xnexus-md2pdf-tool** must be cloned at `../xnexus-md2pdf-tool` (relative to the project root)
2. **Python 3.11+** must be installed
3. The xnexus-md2pdf-tool must have its dependencies installed (the script will set this up automatically if needed)

### Usage

```bash
# Generate PDF with current version from Cargo.toml
./scripts/generate_binary_readme_pdf.sh

# Generate PDF for specific version
./scripts/generate_binary_readme_pdf.sh 1.0.6
```

### Output

The script generates:
- `release_assets/README.pdf` - VicOne-styled PDF of the binary distribution guide

### What It Does

1. Validates that xnexus-md2pdf-tool exists
2. Sets up Python virtual environment if needed
3. Converts BINARY_README.md to PDF with:
   - VicOne cover page and styling
   - Version-specific title
   - Professional formatting with headers/footers
   - Current date generation timestamp
4. Places output in `release_assets/` directory

### Integration with CI/CD

This script is automatically run during the GitHub Actions release workflow. When you create a new release tag (e.g., `v1.0.6`), the workflow:

1. Builds binaries for all platforms
2. Checks out xnexus-md2pdf-tool
3. Generates README.pdf from BINARY_README.md
4. Includes README.pdf in the release assets alongside:
   - Binary files (macOS ARM/Intel, Linux, Windows)
   - README.md (markdown version)
   - checksums.txt

### Manual Testing

To test PDF generation locally:

```bash
# Run the script
./scripts/generate_binary_readme_pdf.sh

# Check the output
ls -lh release_assets/README.pdf
open release_assets/README.pdf  # macOS
```

### Troubleshooting

**Error: xnexus-md2pdf-tool not found**
```bash
# Clone the tool in the parent directory
cd ..
git clone git@github.com:VicOne-RD/xnexus-md2pdf-tool.git
cd radeis_sc2sbom
```

**Error: Python dependencies missing**
```bash
# The script will automatically install dependencies
# If manual setup is needed:
cd ../xnexus-md2pdf-tool
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
deactivate
```

**PDF generation fails**
```bash
# Run with verbose logging
cd ../xnexus-md2pdf-tool
source .venv/bin/activate
python build_all.py \
  --md ../radeis_sc2sbom/BINARY_README.md \
  --outdir ../radeis_sc2sbom/release_assets \
  --out-pdf "README.pdf" \
  --title "Test PDF" \
  --log-level DEBUG
```

### Notes

- The PDF uses VicOne styling (cover page, headers, footers) from xnexus-md2pdf-tool
- Charts and dashboard features are disabled (`--no-dashboard`, `--no-inline-charts`) since BINARY_README.md is documentation, not a report
- The generated PDF is self-contained with the full MIT License text (no external references needed)
