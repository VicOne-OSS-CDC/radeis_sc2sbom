# SBOM Output Formats

Complete guide to SPDX and CycloneDX output formats supported by `radeis_sc2sbom`.

## Overview

The tool generates standards-compliant SBOMs in three formats:

| Format | Specification | Version | Use Case |
|--------|--------------|---------|----------|
| **SPDX JSON** | SPDX 2.3 | JSON | Machine-readable, compliance tools |
| **SPDX Tag-Value** | SPDX 2.3 | Text | Human-readable, version control |
| **CycloneDX JSON** | CycloneDX 1.5 | JSON | Modern SBOM, dependency graphs |

## SPDX 2.3 Format

**Specification:** https://spdx.github.io/spdx-spec/v2.3/

### Generating SPDX SBOMs

**JSON Format:**
```bash
./target/release/radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json
```

**Tag-Value Format:**
```bash
./target/release/radeis_sc2sbom --path . --format spdx-tag-value > sbom.spdx
```

### SPDX Document Structure

#### Document Level

| Field | Value | Description |
|-------|-------|-------------|
| `spdxVersion` | `SPDX-2.3` | Specification version |
| `dataLicense` | `CC0-1.0` | SPDX document license (required) |
| `SPDXID` | `SPDXRef-DOCUMENT` | Document identifier |
| `name` | `{project}-sbom` | Document name |
| `documentNamespace` | `https://sbom.example.com/{project}/{timestamp}` | Unique URI |
| `created` | ISO 8601 timestamp | Creation time |
| `creators` | `Tool: radeis_sc2sbom-0.1.0` | Tool identifier |

#### Package Level

| Field | Value | Description |
|-------|-------|-------------|
| `SPDXID` | `SPDXRef-Package-{name}-{uuid}` | UUID-based package ID (v0.8.0+) |
| `name` | Package name | Dependency name |
| `versionInfo` | Version string | From manifest file |
| `downloadLocation` | Registry URL or `NOASSERTION` | Ecosystem-specific package URL (v0.8.0+) |
| `filesAnalyzed` | `false` | No file-level analysis |
| `licenseConcluded` | SPDX identifier or `NOASSERTION` | Extracted license (95%+ coverage in v0.8.0) |
| `licenseDeclared` | SPDX identifier or `NOASSERTION` | Extracted license (95%+ coverage in v0.8.0) |
| `supplier` | `Organization: {registry}` | Package registry organization (v0.8.0+) |
| `originator` | `Person: {author}` | Author/maintainer information (v0.8.0+) |
| `sourceInfo` | Detection source path | Audit trail (98%+ coverage in v0.8.0) |
| `copyrightText` | `NOASSERTION` | Copyright not extracted |

#### External References

**v0.8.0+**: Each package includes both Package URLs (purl) and CPE identifiers:

##### Package URLs (purl)

| Ecosystem | purl Type | Example |
|-----------|-----------|---------|
| npm | `npm` | `pkg:npm/express@4.17.1` |
| Cargo | `cargo` | `pkg:cargo/serde@1.0` |
| pip | `pypi` | `pkg:pypi/django@3.2.0` |
| Go Modules | `golang` | `pkg:golang/github.com/gin-gonic/gin@v1.7.0` |
| RubyGems | `gem` | `pkg:gem/rails@6.1.0` |
| Composer | `composer` | `pkg:composer/symfony/console@5.3` |
| Maven | `maven` | `pkg:maven/group/artifact@1.0` |
| ROS | `ros` | `pkg:ros/rclcpp@16.0.0` |

**Specification:** https://github.com/package-url/purl-spec

##### CPE Identifiers (v0.8.0+)


| Ecosystem | CPE Format | Example |
|-----------|------------|---------|
| npm | `cpe:2.3:a:npm:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:npm:axios:1.0.0:*:*:*:*:*:*:*` |
| npm (scoped) | `cpe:2.3:a:{scope}:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:types:node:18.0.0:*:*:*:*:*:*:*` |
| Cargo | `cpe:2.3:a:rust:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:rust:serde:1.0.195:*:*:*:*:*:*:*` |
| pip | `cpe:2.3:a:python:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:python:requests:2.28.0:*:*:*:*:*:*:*` |
| Composer | `cpe:2.3:a:{vendor}:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:symfony:http_foundation:6.0.0:*:*:*:*:*:*:*` |
| Go | `cpe:2.3:a:{org}:{product}:{version}:*:*:*:*:*:*:*` | `cpe:2.3:a:stretchr:testify:1.8.0:*:*:*:*:*:*:*` |

**Specification:** https://cpe.mitre.org/specification/
**Use Case:** Integration with NIST NVD and security scanning tools

### SPDX JSON Example (v0.8.0+)

```json
{
  "spdxVersion": "SPDX-2.3",
  "dataLicense": "CC0-1.0",
  "SPDXID": "SPDXRef-DOCUMENT",
  "name": "my-project-sbom",
  "documentNamespace": "https://sbom.example.com/my-project/2026-01-27T10:00:00Z",
  "creationInfo": {
    "created": "2026-01-27T10:00:00Z",
    "creators": ["Tool: radeis_sc2sbom-0.8.0"],
    "licenseListVersion": "3.21"
  },
  "packages": [
    {
      "SPDXID": "SPDXRef-Package-axios-d4c3b2a1-e5f6-7890-ab12-cd34ef567890",
      "name": "axios",
      "versionInfo": "1.6.2",
      "downloadLocation": "https://registry.npmjs.org/axios/-/axios-1.6.2.tgz",
      "filesAnalyzed": false,
      "licenseConcluded": "MIT",
      "licenseDeclared": "MIT",
      "supplier": "Organization: npmjs",
      "originator": "Person: Matt Zabriskie",
      "sourceInfo": "Identified by the javascript/packagelockjson extractor from /path/to/package-lock.json",
      "copyrightText": "NOASSERTION",
      "externalRefs": [
        {
          "referenceCategory": "PACKAGE-MANAGER",
          "referenceType": "purl",
          "referenceLocator": "pkg:npm/axios@1.6.2"
        },
        {
          "referenceCategory": "SECURITY",
          "referenceType": "cpe23Type",
          "referenceLocator": "cpe:2.3:a:npm:axios:1.6.2:*:*:*:*:*:*:*"
        }
      ]
    }
  ],
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-Package-main"
    },
    {
      "spdxElementId": "SPDXRef-Package-main",
      "relationshipType": "CONTAINS",
      "relatedSpdxElement": "SPDXRef-Package-axios-d4c3b2a1-e5f6-7890-ab12-cd34ef567890"
    },
    {
      "spdxElementId": "SPDXRef-Package-axios-d4c3b2a1-e5f6-7890-ab12-cd34ef567890",
      "relationshipType": "CONTAINS",
      "relatedSpdxElement": "NOASSERTION"
    }
  ]
}
```

**v0.8.0 Enhancements**:
- ✅ UUID-based SPDX IDs (better uniqueness)
- ✅ License information extracted (MIT vs NOASSERTION)
- ✅ Download location URLs (registry links)
- ✅ Supplier and originator fields populated
- ✅ Source tracking information (sourceInfo)
- ✅ CPE identifiers for security correlation
- ✅ Hierarchical relationships (DESCRIBES → CONTAINS structure)

### SPDX Tag-Value Example

```
SPDXVersion: SPDX-2.3
DataLicense: CC0-1.0
SPDXID: SPDXRef-DOCUMENT
DocumentName: my-project-sbom
DocumentNamespace: https://sbom.example.com/my-project/2026-01-22T10:00:00Z
Creator: Tool: radeis_sc2sbom-0.1.0
Created: 2026-01-22T10:00:00Z

##### Package: express

PackageName: express
SPDXID: SPDXRef-Package-npm-1
PackageVersion: ^4.17.1
PackageDownloadLocation: NOASSERTION
FilesAnalyzed: false
PackageLicenseConcluded: NOASSERTION
PackageLicenseDeclared: NOASSERTION
PackageCopyrightText: NOASSERTION
ExternalRef: PACKAGE-MANAGER purl pkg:npm/express@^4.17.1
```

### ROS Multi-Package Support

For ROS/ROS2 workspaces, SPDX includes relationship information:

```json
{
  "relationships": [
    {
      "spdxElementId": "SPDXRef-DOCUMENT",
      "relationshipType": "DESCRIBES",
      "relatedSpdxElement": "SPDXRef-ROS-Package-my_robot"
    },
    {
      "spdxElementId": "SPDXRef-ROS-Package-my_robot",
      "relationshipType": "DEPENDS_ON",
      "relatedSpdxElement": "SPDXRef-Package-pip-1"
    }
  ]
}
```

## CycloneDX 1.5 Format

**Specification:** https://cyclonedx.org/docs/1.5/

### Generating CycloneDX SBOMs

```bash
./target/release/radeis_sc2sbom --path . --format cyclonedx-json > sbom.cdx.json
```

### CycloneDX Document Structure

#### Document Level

| Field | Value | Description |
|-------|-------|-------------|
| `bomFormat` | `CycloneDX` | Format identifier |
| `specVersion` | `1.5` | Specification version |
| `serialNumber` | `urn:uuid:{uuid}` | Unique BOM identifier (RFC-4122 UUID) |
| `version` | `1` | BOM version |
| `metadata.timestamp` | ISO 8601 timestamp | Creation time |
| `metadata.tools` | Tool information | Generator details |
| `metadata.component` | Root component | Project being analyzed |

#### Component Structure

| Field | Value | Description |
|-------|-------|-------------|
| `type` | `application` or `library` | Component type |
| `bom-ref` | Unique reference | Component identifier |
| `name` | Package name | Dependency name |
| `version` | Version string | From manifest file |
| `purl` | Package URL | Standard package identifier |
| `properties` | Additional metadata | Dev dependencies, scope, source |

#### Properties

Custom properties for additional metadata:

| Property | Values | Description |
|----------|--------|-------------|
| `dev-dependency` | `true` | Development dependency marker |
| `dependency-source` | `manifest`, `lock-file`, `import-scan` | Source of detection |
| `dependency-scope` | `direct`, `transitive` | Dependency relationship |

### CycloneDX JSON Example

```json
{
  "bomFormat": "CycloneDX",
  "specVersion": "1.5",
  "serialNumber": "urn:uuid:12345678-1234-1234-1234-123456789abc",
  "version": 1,
  "metadata": {
    "timestamp": "2026-01-22T10:00:00Z",
    "tools": {
      "components": [
        {
          "type": "application",
          "name": "radeis_sc2sbom",
          "version": "1.0.8"
        }
      ]
    },
    "component": {
      "type": "application",
      "bom-ref": "project-my-app",
      "name": "my-app"
    }
  },
  "components": [
    {
      "type": "library",
      "bom-ref": "dep-npm-1",
      "name": "express",
      "version": "4.17.1",
      "purl": "pkg:npm/express@4.17.1",
      "properties": [
        {
          "name": "dependency-source",
          "value": "lock-file"
        },
        {
          "name": "dependency-scope",
          "value": "direct"
        }
      ]
    },
    {
      "type": "library",
      "bom-ref": "dep-npm-2",
      "name": "jest",
      "version": "27.0.0",
      "purl": "pkg:npm/jest@27.0.0",
      "properties": [
        {
          "name": "dev-dependency",
          "value": "true"
        },
        {
          "name": "dependency-source",
          "value": "manifest"
        },
        {
          "name": "dependency-scope",
          "value": "direct"
        }
      ]
    }
  ],
  "dependencies": [
    {
      "ref": "project-my-app",
      "dependsOn": ["dep-npm-1", "dep-npm-2"]
    }
  ]
}
```

### ROS Multi-Package Support

For ROS workspaces, each package becomes a top-level component:

```json
{
  "components": [
    {
      "type": "application",
      "bom-ref": "ros-package-1",
      "name": "my_robot_package",
      "version": "1.0.0",
      "purl": "pkg:ros/my_robot_package@1.0.0"
    }
  ],
  "dependencies": [
    {
      "ref": "ros-package-1",
      "dependsOn": ["dep-pip-1", "dep-pip-2"]
    }
  ]
}
```

## Validation

### SPDX Validation

**Using spdx-tools (Python):**
```bash
pip install spdx-tools
pyspdxtools -i sbom.spdx.json
```

**Online Validator:**
https://tools.spdx.org/app/validate/

### CycloneDX Validation

**Using cyclonedx-cli:**
```bash
npm install -g @cyclonedx/cyclonedx-cli
cyclonedx-cli validate --input-file sbom.cdx.json
```

**Online Validator:**
https://cyclonedx.github.io/cyclonedx-editor-validator/

## Format Comparison

| Feature | SPDX 2.3 | CycloneDX 1.5 |
|---------|----------|---------------|
| **Maturity** | Established (2010) | Modern (2017) |
| **Focus** | License compliance | Supply chain security |
| **Relationships** | Explicit (DESCRIBES, DEPENDS_ON) | Dependency graph |
| **Metadata** | Minimal | Rich (properties) |
| **Dev Dependencies** | Via relationships | Via properties |
| **License Info** | Structured fields | Component-level |
| **Adoption** | Government, compliance | DevSecOps, containers |

## Use Cases

### Supply Chain Security
```bash
./target/release/radeis_sc2sbom --path . --format cyclonedx-json > sbom.cdx.json
# Upload to Dependency-Track, Grype, or similar tools
```

### License Compliance
```bash
# Generate SPDX for license audits
./target/release/radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json
# Use with FOSSology, ScanCode, or similar tools
```

### Regulatory Compliance
```bash
# Generate both formats for comprehensive compliance
./target/release/radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json
./target/release/radeis_sc2sbom --path . --format cyclonedx-json > sbom.cdx.json
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Generate SBOMs

on: [push, release]

jobs:
  sbom:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install tool
        run: |
          cargo install --path .

      - name: Generate SPDX SBOM
        run: radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json

      - name: Generate CycloneDX SBOM
        run: radeis_sc2sbom --path . --format cyclonedx-json > sbom.cdx.json

      - name: Upload SBOMs
        uses: actions/upload-artifact@v3
        with:
          name: sboms
          path: |
            sbom.spdx.json
            sbom.cdx.json
```

### GitLab CI

```yaml
generate-sboms:
  stage: build
  script:
    - cargo install --path .
    - radeis_sc2sbom --path . --format spdx-json > sbom.spdx.json
    - radeis_sc2sbom --path . --format cyclonedx-json > sbom.cdx.json
  artifacts:
    paths:
      - sbom.spdx.json
      - sbom.cdx.json
```

## Known Limitations

### Current Limitations (v1.0.8)

1. **Copyright Text**: Not extracted (marked as NOASSERTION in SPDX)
2. **Checksums**: Not calculated for components
3. **File-Level Analysis**: Not performed (filesAnalyzed: false)
4. **Import-scanned packages**: Version is unknown by definition; PURL version is omitted and `downloadLocation` is `NOASSERTION` (correct per spec)

### Recently Implemented

✅ **License Detection**: Now extracts from package metadata (95%+ coverage)
✅ **Download URLs**: Determines from package registries (ecosystem-specific)
✅ **Supplier/Originator**: Extracts author/maintainer information (90%+ coverage)
✅ **Source Tracking**: Full audit trail with sourceInfo field (98%+ coverage)
✅ **UUID-based IDs**: Better uniqueness than sequential IDs
✅ **v1.0.8 — Spec Compliance**: Sentinel version strings (`detected`, `unspecified`) no longer appear in PURLs, CycloneDX component versions, or SPDX `downloadLocation` URLs

### Planned Future Enhancements

1. **Copyright Text Extraction**: Parse LICENSE files for copyright information
2. **Checksums**: Add SHA-256/SHA-512 hashes for package integrity
3. **SBOM Signing**: Digital signatures for SBOM integrity verification
4. **Enhanced Transitive Dependencies**: Full dependency graph with all levels

## Standards and References

### SPDX
- [SPDX 2.3 Specification](https://spdx.github.io/spdx-spec/v2.3/)
- [SPDX License List](https://spdx.org/licenses/)
- [SPDX Tools](https://github.com/spdx/tools)

### CycloneDX
- [CycloneDX 1.5 Specification](https://cyclonedx.org/docs/1.5/)
- [CycloneDX JSON Schema](https://github.com/CycloneDX/specification/tree/master/schema)
- [CycloneDX Use Cases](https://cyclonedx.org/use-cases/)

### Package URLs
- [purl Specification](https://github.com/package-url/purl-spec)
- [purl Ecosystem Types](https://github.com/package-url/purl-spec/blob/master/PURL-TYPES.rst)

### Compliance
- [NTIA Minimum Elements for SBOM](https://www.ntia.gov/sbom)
- [Executive Order 14028 (US)](https://www.federalregister.gov/documents/2021/05/17/2021-10460/improving-the-nations-cybersecurity)
- [EU Cyber Resilience Act](https://digital-strategy.ec.europa.eu/en/policies/cyber-resilience-act)

## Ecosystem Support Details

### Comprehensive Ecosystem Table

| Ecosystem | Files | Packages | Trees | License | Author | Metadata Source | Status |
|-----------|-------|----------|-------|---------|--------|----------------|--------|
| **npm** | package.json, package-lock.json, yarn.lock | 690 | ✅ Full | ✅ 100% | ✅ 90% | 🌐 Registry API | 🏆 Industry-leading |
| **Cargo** | Cargo.toml, Cargo.lock | All | ✅ Full | ✅ SPDX | ✅ | 🌐 crates.io API | Production-ready |
| **pip** | requirements.txt, poetry.lock, Pipfile.lock | All | ✅ Poetry | ✅ | ✅ | 🌐 PyPI API | Production-ready |
| **ROS/ROS2** | package.xml, setup.py | All | ❌ | ✅ Normalized | ✅ + Maintainers | 📝 Local XML | Production-ready |
| **PHP** | composer.json, composer.lock | All | ❌ | ✅ | ✅ | 🌐 Packagist API | Production-ready |
| **Ruby** | Gemfile, Gemfile.lock | All | ❌ | ✅ | ✅ | 🌐 RubyGems API | Production-ready |
| **Go** | go.mod, go.sum | All | ❌ | ⚠️ N/A | ⚠️ N/A | ⚠️ N/A | Supported |

**Metadata Legend:**
- ✅ = Full extraction with coverage percentage
- ✅ SPDX = SPDX-compliant license identifiers
- ✅ Normalized = Converted to SPDX format (e.g., "Apache License 2.0" → "Apache-2.0")
- 🌐 = Network API with parallel batch fetching (v0.9.0)
- 📝 = Local file extraction only
- ⚠️ N/A = Not available in ecosystem manifest format (Go design limitation)

### Metadata Extraction Details

**v0.9.0 Performance**: All network-based metadata extraction uses parallel batch fetching for 10-27x speedup.

#### npm Ecosystem
**Metadata Sources:**
1. **Local (Fast)**: Reads `node_modules/{package}/package.json` for installed packages
2. **Network (Fallback)**: npm registry API `https://registry.npmjs.org/{package}/{version}`
3. **Parallel Processing**: Fetches 689 packages in 22.6 seconds (27x speedup)

**Extracted Metadata:**
- License: 100% coverage (689/689 packages)
- Author: 90% coverage (622/689 packages)
- Maintainers: Array of maintainer information
- Download URLs: `https://registry.npmjs.org/{package}/-/{file}.tgz`
- Homepage: Package homepage URLs
- Repository: GitHub/GitLab URLs

**Example:**
```json
{
  "SPDXID": "SPDXRef-Package-axios-{uuid}",
  "name": "axios",
  "versionInfo": "1.6.2",
  "licenseDeclared": "MIT",
  "supplier": "Organization: npmjs",
  "originator": "Person: Matt Zabriskie",
  "downloadLocation": "https://registry.npmjs.org/axios/-/axios-1.6.2.tgz"
}
```

#### Cargo (Rust) Ecosystem
**Metadata Sources:**
1. **Local (Fast)**: Reads `Cargo.toml` [package] section
2. **Network (Fallback)**: crates.io API `https://crates.io/api/v1/crates/{package}/{version}`
3. **Parallel Processing**: Fetches 100-500 packages in 25 seconds (17-36x speedup)

**Extracted Metadata:**
- License: SPDX-compliant format (e.g., "MIT OR Apache-2.0")
- Authors: Array format with emails
- Repository: Git repository URLs
- Download URLs: `https://crates.io/api/v1/crates/{package}/{version}/download`

**Example:**
```json
{
  "licenseDeclared": "MIT OR Apache-2.0",
  "originator": "Person: Alice <alice@example.com>",
  "downloadLocation": "https://crates.io/api/v1/crates/serde/1.0.195/download"
}
```

#### Python Ecosystem (v0.9.3+)
**Supported Package Managers:**
- **pip** - requirements.txt (manifest)
- **Poetry** - poetry.lock (lock file), pyproject.toml (manifest)
- **Pipenv** - Pipfile.lock (lock file), Pipfile (manifest) [v0.9.3+]
- **PDM** - pyproject.toml (manifest) [v0.9.3+]
- **setuptools** - setup.py, pyproject.toml (manifest)

**Manifest Files (Direct Dependencies):**
1. **requirements.txt** - pip format with version specs
2. **Pipfile** - TOML format with [packages] and [dev-packages] sections [v0.9.3+]
3. **pyproject.toml** - Modern Python standard (PEP 517/518) [v0.9.3+]
   - PEP 621 format ([project] section)
   - Poetry format ([tool.poetry] section)
   - PDM format ([tool.pdm] section)
4. **setup.py** - Legacy setuptools format (regex-based parsing)

**Lock Files (Complete Dependency Tree):**
1. **poetry.lock** - TOML format with exact versions + SHA256 checksums
2. **Pipfile.lock** - JSON format with exact versions + SHA256 checksums [v0.9.3+]
   - Parses `default` section (production dependencies)
   - Parses `develop` section (development dependencies)
   - Extracts SHA256 hashes from hashes array
   - Detects direct dependencies via `index` field
   - Includes transitive dependencies

**Metadata Sources:**
1. **Local (Fast)**: Parses manifest and lock files
2. **Network (Fallback)**: PyPI API `https://pypi.org/pypi/{package}/{version}/json`
3. **Parallel Processing**: Fetches 100-500 packages in 30 seconds (20-40x speedup)

**Extracted Metadata:**
- License: From `license` field or classifiers
- Authors: From `author` field with email
- Download URLs: `https://pypi.org/project/{package}/{version}/`
- **Checksums**: SHA256 from Pipfile.lock and poetry.lock [v0.9.3+]
- **Direct dependency flags**: From lock file `index` field [v0.9.3+]

**Priority Order (Highest → Lowest):**
1. Lock files (e.g., Pipfile.lock, poetry.lock: exact versions + checksums)
2. Manifest files (e.g., pyproject.toml, requirements.txt, Pipfile: version specs)
3. Import scanning (fallback, no versions)

**Note:** Within each category (lock files or manifests), all sources are treated equally. Deduplication uses `DependencySource` enum (LockFile > Manifest > ImportScan) without sub-prioritization among files of the same type.

#### ROS/ROS2 Ecosystem
**Metadata Sources:**
- **Local Only**: Parses `package.xml` files (no network API)
- License normalization to SPDX format

**Extracted Metadata:**
- License: Normalized to SPDX (e.g., "Apache License, Version 2.0" → "Apache-2.0")
- Maintainers: With email addresses
- Authors: With email addresses
- Description: Package description

**License Normalization:**
```rust
"Apache License 2.0" → "Apache-2.0"
"BSD" → "BSD-3-Clause"
"MIT" → "MIT"
"GPLv3" → "GPL-3.0-only"
```

#### PHP (Composer) Ecosystem
**Metadata Sources:**
1. **Local (Fast)**: Reads `composer.json`
2. **Network (Fallback)**: Packagist API `https://repo.packagist.org/p2/{package}.json`
3. **Parallel Processing**: Fetches 10-200 packages in 20 seconds (6-21x speedup)

**Extracted Metadata:**
- License: String or array format
- Authors: Array with names and emails
- Repository: Source repository URLs
- Download URLs: `https://packagist.org/packages/{package}`

#### Ruby (Gems) Ecosystem
**Metadata Sources:**
1. **Local (Fast)**: Parses `Gemfile`, scans for `.gemspec` files
2. **Network (Fallback)**: RubyGems API `https://rubygems.org/api/v2/rubygems/{gem}/versions/{version}.json`
3. **Parallel Processing**: Fetches 5-50 gems in 10 seconds (6-12x speedup)

**Extracted Metadata:**
- License: From gemspec `spec.license`
- Authors: From gemspec `spec.authors` array
- Homepage: Package homepage
- Source code: Repository URL

#### Go Modules Ecosystem
**Metadata Sources:**
- **go.mod file only** (no license/author metadata available)

**Design Limitation:**
Go modules do not store license or author metadata in `go.mod` files. This information lives in:
- Repository `LICENSE` files
- Repository `README.md` files
- Go package documentation

**Future Enhancement**: Optional GitHub API integration for metadata lookup from repositories.

### Download Location URLs

**v0.8.0+**: Ecosystem-specific package registry URLs for verification and reproducible builds.

| Ecosystem | URL Format | Example |
|-----------|------------|---------|
| **npm** | `https://registry.npmjs.org/{package}/-/{file}.tgz` | `https://registry.npmjs.org/axios/-/axios-1.6.2.tgz` |
| **npm (scoped)** | `https://registry.npmjs.org/{scope}/{name}/-/{file}.tgz` | `https://registry.npmjs.org/@types/node/-/node-18.0.0.tgz` |
| **PyPI** | `https://pypi.org/packages/source/{first}/{name}/{file}.tar.gz` | `https://pypi.org/packages/source/r/requests/requests-2.28.0.tar.gz` |
| **Cargo** | `https://crates.io/api/v1/crates/{name}/{version}/download` | `https://crates.io/api/v1/crates/serde/1.0.0/download` |
| **Composer** | `https://packagist.org/packages/{vendor}/{package}` | `https://packagist.org/packages/symfony/http-foundation` |
| **RubyGems** | `https://rubygems.org/gems/{name}` | `https://rubygems.org/gems/rails` |
| **Go** | Repository URL (from go.mod) | `https://github.com/gin-gonic/gin` |

### Source Tracking

**v0.8.0+**: Full audit trail showing which extractor and manifest file detected each package (98%+ coverage).

**Pattern**: `"Identified by the {extractor_type} extractor from {absolute_path}"`

**Examples:**
- `"Identified by the javascript/packagelockjson extractor from /path/to/package-lock.json"`
- `"Identified by the rust/cargo extractor from /path/to/Cargo.lock"`
- `"Identified by the python/poetry extractor from /path/to/poetry.lock"`
- `"Identified by the ros/packagexml extractor from /path/to/package.xml"`

**Extractor Type Mappings:**
| Ecosystem | Extractor Type |
|-----------|----------------|
| npm package.json | `javascript/packagejson` |
| npm package-lock.json | `javascript/packagelockjson` |
| npm yarn.lock | `javascript/yarnlock` |
| Cargo.toml | `rust/cargo` |
| Cargo.lock | `rust/cargolock` |
| requirements.txt | `python/requirements` |
| poetry.lock | `python/poetry` |
| **Pipfile.lock** | **`python/pipfilelock`** [v0.9.3+] |
| **Pipfile** | **`python/pipfile`** [v0.9.3+] |
| **pyproject.toml** | **`python/pyproject`** [v0.9.3+] |
| package.xml | `ros/packagexml` |
| composer.json | `php/composer` |
| Gemfile | `ruby/gemfile` |
| go.mod | `go/gomod` |

## Support

For format-specific questions:
- SPDX issues: [SPDX mailing list](https://lists.spdx.org/)
- CycloneDX issues: [CycloneDX GitHub](https://github.com/CycloneDX)
- Tool issues: Open an issue on the repository
