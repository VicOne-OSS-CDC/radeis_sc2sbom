# Phase 11: Lexical Scanner + CycloneDX Output - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 11-lexical-scanner-cyclonedx-output
**Areas discussed:** Component-to-directory mapping, Scanner invocation point, CWE rule table structure, CycloneDX vulnerability ID scheme

---

## Component-to-Directory Mapping

| Option | Description | Selected |
|--------|-------------|----------|
| source_file parent dir | Use existing Dependency.source_file; zero new data; skips so-scanner/vendored components | |
| Full tree + nearest manifest | Walk all C/C++ files, attribute by directory proximity; violates SCAN-05 spirit | |
| Explicit HashMap in ScanContext | Add component_dirs: HashMap<(String,String), PathBuf> populated by manifest parsers; explicit and testable | ✓ |

**User's choice:** Explicit HashMap in ScanContext

**Follow-up — HashMap key:**

| Option | Description | Selected |
|--------|-------------|----------|
| (name, ecosystem) tuple | Consistent with dep_to_bom_ref pattern in cyclonedx.rs; handles same name in different ecosystems | ✓ |
| name only | Simpler; breaks if same name appears in multiple ecosystems | |

**User's choice:** (name, ecosystem) tuple
**Notes:** User asked for pros/cons before deciding. Key choice was driven by consistency with existing bom-ref lookup pattern.

---

## Scanner Invocation Point

| Option | Description | Selected |
|--------|-------------|----------|
| After scan_directory, before OSV | Runs early; independent of CVE data | |
| Mutate Dependency.vulnerabilities | Conflates CVEs and SAST findings; not preferred | |
| After all enrichment | Runs last before formatters; all CVE data available if ever needed | ✓ |

**User's choice:** After all enrichment

**Follow-up — findings passing mechanism:**

| Option | Description | Selected |
|--------|-------------|----------|
| Extra parameter: &[SastFinding] | Trailing param; consistent with SupplierResolver pattern | ✓ |
| Store on Sbom struct | Expands SBOM model with CycloneDX-specific data | |

**User's choice:** Extra parameter: &[SastFinding]
**Notes:** User confirmed lexical scanner is independent of SBOM dependency graph — it enriches source code findings, not the package list.

---

## CWE Rule Table Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Static const array in Rust | &[CweRule] slice; zero runtime cost; compile-time checked | ✓ |
| Embedded TOML/JSON string | Data-logic separation; adds startup parsing | |
| External YAML rule file | Out of scope per REQUIREMENTS.md (EXT-02 is v1.0.17+) | |

**User's choice:** Static const array in Rust

**Follow-up — CweRule struct fields:**

| Option | Description | Selected |
|--------|-------------|----------|
| cwe_id + functions[] + heuristic flag | Simple; covers all 13 CWEs; requires_format_heuristic flag for CWE-134 | ✓ |
| cwe_id + functions[] + custom matcher fn | Flexible but complex type | |
| Flat HashMap<fn_name, cwe_id> | Dead simple; loses heuristic flag; makes CWE-134 implicit | |

**User's choice:** cwe_id + functions[] + heuristic flag

---

## CycloneDX Vulnerability ID Scheme

| Option | Description | Selected |
|--------|-------------|----------|
| One entry per finding (file+line) | Fine-grained provenance; maps to CDX-03; potentially large for noisy codebases | ✓ |
| One entry per CWE per component | Smaller output; properties become unordered bag of pairs | |

**User's choice:** One entry per finding (file+line)

**Follow-up — bom-ref format:**

| Option | Description | Selected |
|--------|-------------|----------|
| sast-{cwe_id}-{sanitized_path}-{line} | Human-readable; CWE + file + line visible in ref | ✓ |
| sast-{cwe_id}-{component}-{idx} | Shorter; requires reading properties to trace | |
| sast-{cwe_id}-{sha256} | Opaque; stable across re-runs | |

**User's choice:** sast-{cwe_id}-{sanitized_path}-{line}
**Notes:** User asked which format best helps a developer fix CWE findings. Path-embedded form was selected as most actionable. User also asked about CWE spec compliance — clarified that bom-ref format is a CycloneDX concept, not CWE-spec-mandated.

---

## Claude's Discretion

- `SastFinding` struct field names and types (PathBuf vs String, u32 vs usize for line)
- Whether `run_lexical_scanner` is a free function or method on a struct
- Exact byte-level implementation of CWE-134 next-token heuristic
- Whether scanner tests gate at module level or per-test — researcher to assess consistency with Phase 10's D-09

## Deferred Ideas

None — discussion stayed within phase scope.
