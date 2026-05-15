# Phase 11: Lexical Scanner + CycloneDX Output - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Implement a pure-Rust dangerous-function lexical scanner (`src/vulnerability/cwe_scanner.rs`) that:
1. Reads component-mapped C/C++ directories (via new `ScanContext.component_dirs` field)
2. Detects 13 CWEs by function-name/token matching with a static rule table
3. Returns `Vec<SastFinding>` — file path + line number + CWE ID per call site
4. Serializes findings as CycloneDX 1.5 `vulnerabilities[]` entries (one per finding) with bom-ref linkage to owning component, `cwes[]`, `source.name`, `analysis.state`, and `properties` for file+line

All scanner code is behind `#[cfg(feature = "internal")]` (established by Phase 10). SPDX 2.3 output is unchanged.

Requirements in scope: SCAN-01, SCAN-02, SCAN-03, SCAN-04, SCAN-05, CDX-01, CDX-02, CDX-03, CDX-04

</domain>

<decisions>
## Implementation Decisions

### Component-to-Directory Mapping

- **D-01:** Add `component_dirs: HashMap<(String, String), PathBuf>` to `ScanContext`. Key is `(name, ecosystem)` — matches the existing `dep_to_bom_ref` key pattern in `cyclonedx.rs`. Each manifest parser that produces C/C++ dependencies records the manifest's parent directory at discovery time.
- **D-02:** Key choice rationale: `(name, ecosystem)` tuple is consistent with how the existing `vulnerabilities[]` bom-ref lookup works in `cyclonedx.rs` (line ~254). Prevents mis-attribution when the same library name appears in different ecosystems.
- **D-03:** Components where no directory is recorded (e.g., so-scanner discoveries) are skipped by the lexical scanner — no attempt to guess a path.

### Scanner Invocation Point

- **D-04:** The lexical scanner runs **after all enrichment** (after `enrich_cwe_ids`), immediately before the formatting step. Pipeline order in `main.rs`: `scan_directory` → `query_vulnerabilities_batch` → `enrich_cwe_ids` → **`run_lexical_scanner`** → formatters.
- **D-05:** Scanner returns `Vec<SastFinding>`. Findings are passed to the CycloneDX formatter as a **trailing `&[SastFinding]` parameter** — consistent with how `supplier_resolver: Option<&SupplierResolver>` is passed today. No changes to the `Sbom` model.
- **D-06:** SPDX formatter signature is unchanged — it receives no SAST findings (CDX-04 is firm).

### CWE Rule Table

- **D-07:** Rules are a `static` const array `&[CweRule]` defined in `cwe_scanner.rs`. No external files, no config parsing, zero runtime overhead.
- **D-08:** `CweRule` struct: `{ cwe_id: u32, functions: &'static [&'static str], requires_format_heuristic: bool }`. The `requires_format_heuristic` flag is `true` only for CWE-134 — the scanner checks that the format argument is not a string literal when this flag is set (per SCAN-03).

### CycloneDX Vulnerability ID Scheme

- **D-09:** **One entry per finding (file+line)** — each dangerous function call site gets its own `vulnerabilities[]` entry. This gives exact provenance per CDX-03 and is the most actionable for a developer fixing CWE issues.
- **D-10:** bom-ref format: `sast-{cwe_id}-{sanitized_path}-{line}` where `sanitized_path` replaces `/` and `.` with `-`. Example: `sast-120-src-ssl-s3_lib-c-142`. Human-readable without opening JSON; CWE ID and file/line visible in the ref itself.
- **D-11:** `source.name = "radeis_sc2sbom static analysis"` (from CDX-01). `analysis.state = "in_triage"` (from CDX-01). `cwes` is an integer array (e.g., `[120]`). These are firm from REQUIREMENTS.md.

### Claude's Discretion

- Exact `SastFinding` struct field names and types (e.g., whether `file_path` is `PathBuf` or `String`, whether line is `u32` or `usize`).
- Whether `run_lexical_scanner` is a free function or a method on a `LexicalScanner` struct.
- How the CWE-134 next-token heuristic is implemented at the byte/char level — firm that it only fires when the format arg is not a string literal (SCAN-03), flexible on the exact parsing approach.
- Whether to add `gated = true` annotations to scanner tests or gate the entire test module at `#[cfg(feature = "internal")]` (consistent with Phase 10's D-09 decision for vulnerability tests).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Lexical Scanner — SCAN-01..SCAN-05 definitions and acceptance criteria
- `.planning/REQUIREMENTS.md` §CycloneDX Output — CDX-01..CDX-04 definitions and acceptance criteria
- `.planning/ROADMAP.md` §Phase 11 — success criteria (5 items); the success criteria ARE the acceptance test
- `.planning/REQUIREMENTS.md` §Out of Scope — CWE-401, CWE-457, CWE-415/416, CWE-476, SARIF, suppress-list all explicitly out of scope

### Phase 10 Context (prerequisite — read before planning)
- `.planning/phases/10-internal-feature-gate/10-CONTEXT.md` — D-07/D-08: `cwe_scanner.rs` stub already created as the landing zone; D-09: vulnerability tests gated at module level with `#[cfg(feature = "internal")]`

### Source Files — Scanner Landing Zone
- `src/vulnerability/cwe_scanner.rs` — the Phase 10 stub; Phase 11 implements the full scanner here (already `#[cfg(feature = "internal")]` gated)
- `src/vulnerability/mod.rs` — exports `cwe_scanner` module; may need to re-export scanner entry point

### Source Files — CycloneDX Integration
- `src/formats/cyclonedx.rs` — existing `CycloneDXVulnerability` struct (line ~183), `cwes: Vec<u32>`, `affects: Vec<CycloneDXVulnerabilityAffect>`, `properties: Vec<CycloneDXProperty>`, `source: Option<CycloneDXVulnerabilitySource>`, `analysis: Option<CycloneDXVulnerabilityAnalysis>` — reuse these structs directly
- `src/formats/cyclonedx.rs` — `dep_to_bom_ref` pattern (line ~254): `HashMap<(String, String), String>` keyed by `(name, ecosystem)` — the SAST formatter uses the same pattern to resolve `affects[].ref`

### Source Files — Pipeline Integration
- `src/main.rs` — scanner runs after `enrich_cwe_ids` (line ~175 area), findings passed as `&[SastFinding]` to CycloneDX formatter
- `src/models/dependency.rs` — `ScanContext` struct (line ~444): add `component_dirs: HashMap<(String, String), PathBuf>` field here
- `src/scanner/mod.rs` — `scan_directory` builds `ScanContext`; manifest parsers that produce C/C++ deps need to populate `component_dirs`

### C/C++ Manifest Parsers (populate component_dirs)
- `src/parsers/c/makefile.rs` — produces C/C++ deps; parser has access to path
- `src/parsers/cmake/` — CMakeLists.txt + .cmake files
- `src/parsers/c/pkgconfig.rs` — .pc files
- `src/parsers/c/autotools.rs` — configure.ac
- `src/parsers/c/makefile_am.rs` — Makefile.am
- `src/parsers/c/vendored_3rdparty.rs` — vendored C/C++ libs

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CycloneDXVulnerability` struct in `src/formats/cyclonedx.rs` (line ~183): fully spec-compliant with `cwes[]`, `affects[]`, `source`, `properties`, `analysis` — use directly for SAST entries; no new struct needed
- `CycloneDXProperty` struct already exists in `cyclonedx.rs` — reuse for `sc2sbom:finding:file` and `sc2sbom:finding:line` properties (CDX-03)
- `dep_to_bom_ref: HashMap<(String, String), String>` pattern in `cyclonedx.rs` (line ~261) — clone this approach to resolve component bom-refs for SAST `affects[].ref`
- `sha2` crate already a dependency (used for NVD cache keys) — available if needed for finding deduplication

### Established Patterns
- `Option<&SupplierResolver>` as trailing param on formatters (Phase 8 D-11) — SAST findings follow the same trailing-param pattern: `&[SastFinding]`
- `#[cfg(feature = "internal")]` gating established by Phase 10 — all scanner code and its tests must be inside this gate
- `Dependency::default()` + `..Default::default()` pattern for struct construction (Phase 10 D-10) — if ScanContext gains a new field, `Default` derive handles test construction sites

### Integration Points
- `ScanContext` in `src/models/dependency.rs` (line ~444): gains `component_dirs: HashMap<(String, String), PathBuf>`
- `scan_directory` in `src/scanner/mod.rs` (line ~426): must be gated `#[cfg(feature = "internal")]` for the `component_dirs` population logic OR the `component_dirs` field is always present but populated only under the feature flag — researcher should assess which approach minimizes cfg churn
- `main.rs` scanning block: `run_lexical_scanner(&scan_context.component_dirs)` returns `Vec<SastFinding>`, passed as `&sast_findings` to `save_cyclonedx`
- `save_cyclonedx` (or equivalent function in `cyclonedx.rs`): gains `sast_findings: &[SastFinding]` trailing parameter

</code_context>

<specifics>
## Specific Ideas

- bom-ref sanitization: replace `/` and `.` in file paths with `-` to produce clean JSON keys (e.g., `src/ssl/s3_lib.c` → `src-ssl-s3_lib-c`). Simple string replace, no regex needed.
- CWE-134 heuristic: the `requires_format_heuristic` flag means the scanner looks at the token immediately after the function name + `(` — if it starts with `"` it's a string literal (safe); if it starts with anything else (variable, macro, expression) it's a finding. Firm from SCAN-03 / REQUIREMENTS.md.
- xZETA compatibility concern (from STATE.md blockers): does xZETA treat all `vulnerabilities[]` entries as CVEs requiring remediation tracking regardless of `source.name`? This was flagged as an open question. Researcher should validate whether `source.name = "radeis_sc2sbom static analysis"` + `analysis.state = "in_triage"` is sufficient to distinguish SAST from CVE findings in xZETA's ingestion pipeline.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 11-lexical-scanner-cyclonedx-output*
*Context gathered: 2026-05-09*
