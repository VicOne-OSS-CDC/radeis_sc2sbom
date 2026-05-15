# Requirements: radeis_sc2sbom

**Defined:** 2026-05-11
**Milestone:** v1.0.18 — Tree-sitter AST Scanner
**Core Value:** Accurate, spec-compliant SBOM output that downstream consumers (xZETA, compliance tools) can ingest without errors.

## v1.0.18 Requirements

### AST Scanner Core

- [ ] **AST-01**: User can run sc2sbom with embedded tree-sitter-c AST scanner as the default C/C++ analysis path (no external cppcheck install required)
- [ ] **AST-02**: AST scanner detects all 14 CWEs from the v1.0.17 rule set (CWE-78, 119, 120, 122, 125, 134, 190, 295, 319, 362, 367, 369, 416, 476, 732)
- [ ] **AST-03**: AST scanner produces `SastFinding` output compatible with existing SARIF writer, markdown report, and CycloneDX serializer — no downstream changes required
- [ ] **AST-04**: When tree-sitter fails to parse a C file (malformed/generated/partial TU), scanner falls back to lexical regex scan for that file with a warning logged

### Benchmarking

- [ ] **BENCH-01**: AST scanner results benchmarked against cppcheck on AUTOSAR_SampleProject_S32K144 and at least one additional fixture — false-positive rates documented before cppcheck fate is decided

### cppcheck Removal

- [ ] **CPP-01**: cppcheck subprocess removed or demoted (to `--features cppcheck` escape hatch) based on Phase A benchmark data; graceful-degradation messaging updated to reflect new default

### Argument-Value Migration

- [x] **ARGVAL-01**: CWE-295 (SSL_VERIFY_NONE), CWE-319 (CURLOPT_USE_SSL), and CWE-732 (umask/DACL) argument-value rules migrated from paren-bound string scanning to AST argument node inspection
- [x] **ARGVAL-02**: Migrated argument-value rules produce no new false positives vs v1.0.17 baseline on AUTOSAR_SampleProject_S32K144

### Distribution

- [ ] **DIST-01**: tree-sitter-c grammar license verified as MIT-compatible for static musl linking; documented in Cargo.toml or a license audit note
- [ ] **DIST-02**: Binary compiled with tree-sitter-c grammar embedded — no runtime file system dependency on grammar files; static musl build verified

## Future Requirements

### Extended AST Rules

- **XAST-01**: CWE-362 (race condition via `pthread_create`/`fork` proximity) — may require dataflow; deferred pending tree-sitter capability assessment
- **XAST-02**: CWE-457 (uninitialized variable) — requires liveness analysis; deferred
- **XAST-03**: CWE-401 (memory leak) — requires control-flow graph; deferred

## Out of Scope

| Feature | Reason |
|---------|--------|
| Full dataflow / taint analysis | Scope exceeds SBOM tool; separate SAST product concern |
| CWE-415/416 (double free / UAF) via dataflow | Requires full dataflow graph; tree-sitter gives AST only |
| Supporting languages beyond C/C++ in AST scanner | Python/Rust/Java have separate parsers; AST scanner is C/C++ only |
| Piping cppcheck native SARIF directly | cppcheck SARIF schema differs from ours; normalize via SastFinding |
| Tree-sitter grammars for other languages | Out of v1.0.18 scope; c grammar only |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| AST-01 | Phase 18 | Pending |
| AST-02 | Phase 18 | Pending |
| AST-03 | Phase 18 | Pending |
| AST-04 | Phase 18 | Pending |
| BENCH-01 | Phase 18 | Pending |
| DIST-01 | Phase 18 | Pending |
| DIST-02 | Phase 18 | Pending |
| CPP-01 | Phase 19 | Pending |
| ARGVAL-01 | Phase 20 | Complete |
| ARGVAL-02 | Phase 20 | Complete |

**Coverage:**
- v1.0.18 requirements: 10 total
- Mapped to phases: 10/10 ✓
- Unmapped: 0

---
*Requirements defined: 2026-05-11*
*Last updated: 2026-05-11 after roadmap creation (phases 18–20)*
