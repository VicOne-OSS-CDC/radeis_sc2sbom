# Documentation Index

Complete guide to radeis_sc2sbom documentation.

## Quick Start

**New users start here:**
1. [../README.md](../README.md) - Project overview and quick start
2. [EXAMPLES.md](EXAMPLES.md) - Common usage patterns
3. [CLI.md](CLI.md) - Command-line reference

## User Guides

### Getting Started
- **[EXAMPLES.md](EXAMPLES.md)** - Usage examples and patterns
  - Quick start examples
  - SBOM generation
  - C/C++, ROS, Python projects
  - CI/CD integration

- **[CLI.md](CLI.md)** - Command-line interface reference
  - All command-line options
  - Output formats
  - SBOM modes
  - Vendor modes

- **[USAGE.md](USAGE.md)** - Detailed usage guide
  - Workflow examples
  - CI/CD integration patterns
  - Best practices

### SBOM Modes
- **[sbom_modes_guide.md](sbom_modes_guide.md)** - SBOM mode documentation
  - Use cases and examples
  - CI/CD integration

## Performance & Comparisons

- **[BENCHMARKS.md](BENCHMARKS.md)** - Performance benchmarks
  - vs BlackDuck (ROS, npm)
  - Scan speed metrics
  - Feature comparison matrix
  - When to use which tool

## Technical Documentation

### Ecosystem Support
- **[FORMATS.md](FORMATS.md)** - Ecosystem documentation
  - Supported file formats
  - Parser capabilities
  - Ecosystem-specific features

### Implementation
- **[ARCHITECTURE.md](ARCHITECTURE.md)** - Architecture overview
  - System design
  - Module structure
  - Data models

- **[BUILD.md](BUILD.md)** - Build instructions
  - Compilation
  - Dependencies
  - Platform-specific notes

### Development
- **[TESTING.md](TESTING.md)** - Testing guide
  - Test structure
  - Running tests
  - Writing new tests

## Release Notes

- **[WHATS_NEW.md](WHATS_NEW.md)** - Latest features and changes
  - v1.0.14 - Reliability & SBOM quality
  - v1.0.13 - Multimodal submodel components
  - Earlier releases — see WHATS_NEW.md

## Additional Resources

### Visual Assets
- **images/** - Project logos and diagrams
  - icon.png - Project logo
  - flowchart.png - Scan pipeline diagram

### Archive
- **[archive/](archive/)** - Historical documentation
  - Version-specific reports
  - Project timeline

---

## Documentation Standards

### File Organization

```
docs/
├── README.md                    # This file - Documentation index
├── EXAMPLES.md                  # Usage examples (12KB)
├── CLI.md                       # Command reference (8KB)
├── USAGE.md                     # User guide (11KB)
├── BENCHMARKS.md               # Performance comparisons (13KB)
├── WHATS_NEW.md                # Release notes (16KB)
├── FORMATS.md                  # Ecosystem support (25KB)
├── ARCHITECTURE.md             # Technical architecture (14KB)
├── BUILD.md                    # Build instructions (9KB)
├── TESTING.md                  # Testing guide (11KB)
├── sbom_modes_guide.md         # SBOM modes (6KB)
├── archive/                    # Historical documents
│   └── README.md               # Archive index
└── images/                     # Visual assets
    ├── icon.png
    └── flowchart.png
```

### Best Practices

**For Contributors:**
- Keep documents focused and single-purpose
- Use clear headings and sections
- Include code examples where applicable
- Update cross-references when moving content

**For Users:**
- Start with README.md and EXAMPLES.md
- Consult CLI.md for command reference
- Check BENCHMARKS.md for performance comparisons
- Use FORMATS.md for ecosystem-specific details

---

**Last Updated:** 2026-05-09
