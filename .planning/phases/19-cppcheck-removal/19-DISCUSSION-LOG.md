# Phase 19: cppcheck-removal — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 19-cppcheck-removal
**Areas discussed:** Removal vs demotion, suppress_lexical_false_positives fate, Messaging & CLI output, CLI args, Benchmark code, SastSource enum, deduplicate_sast_findings

---

## Removal vs Demotion

| Option | Description | Selected |
|--------|-------------|----------|
| Demote to opt-in (Recommended) | Keep --cppcheck-path as escape hatch; cppcheck removed from default path but available when arg is passed | |
| Hard remove | Delete cppcheck runner entirely; no escape hatch | ✓ |

**User's choice:** Hard remove
**Notes:** Benchmark data showed cppcheck uniquely finds CWE-190/416/476/401/415/590 — user accepted coverage loss in exchange for a clean codebase. Phases 21–23 AST expansion will close the gap.

---

## suppress_lexical_false_positives Fate

| Option | Description | Selected |
|--------|-------------|----------|
| Delete it (Recommended) | Remove function and all cppcheck_confirmed/scanned_dirs scaffolding | ✓ |
| Keep but no-op it | Pass empty sets; preserve for future secondary scanners | |

**User's choice:** Delete it
**Notes:** No secondary confirmation pass needed once AST is authoritative.

---

## Messaging & CLI Output

| Option | Description | Selected |
|--------|-------------|----------|
| Silent — no scanner name (Recommended) | No scanner announcement; SARIF encodes SastSource::Ast per finding | ✓ |
| Announce AST scanner | Print "ℹ Running AST scanner (tree-sitter-c)..." at SAST start | |

**User's choice:** Silent
**Notes:** Output cleanliness preferred; SARIF provenance is sufficient.

---

## CLI Args (--cppcheck-path)

| Option | Description | Selected |
|--------|-------------|----------|
| Delete --cppcheck-path entirely (Recommended) | Remove arg from cli.rs and warning from main.rs | ✓ |
| Keep but warn on use | Retain arg; print "cppcheck support removed — arg ignored" | |

**User's choice:** Delete entirely
**Notes:** Consistent with hard-remove decision.

---

## Benchmark Code

| Option | Description | Selected |
|--------|-------------|----------|
| Keep benchmark, drop cppcheck columns (Recommended) | tests/benchmark.rs stays as AST harness; remove cppcheck calls and columns | |
| Delete benchmark entirely | Remove tests/benchmark.rs; docs/BENCHMARK.md stays as historical artifact | ✓ |

**User's choice:** Delete entirely
**Notes:** Phase 18 benchmark served its decision purpose. Can be re-added when Phase 21+ CWE expansion needs regression tracking.

---

## SastSource Enum

| Option | Description | Selected |
|--------|-------------|----------|
| Delete both Cppcheck and Both variants | Only Lexical and Ast remain | |
| Remove Cppcheck, keep Both (repurposed) | Both = AST∩Lexical finding confirmed by two methods | ✓ |

**User's choice:** Remove Cppcheck only; keep Both with new semantics
**Notes:** User noted that Both can represent a finding detected by both AST and Lexical approaches — meaningful provenance (higher-confidence finding confirmed by two independent methods).

---

## deduplicate_sast_findings Revision

| Option | Description | Selected |
|--------|-------------|----------|
| Delete it entirely | AST scanner returns findings directly; no merge needed | |
| Revise for AST + Lexical dedup | Same logic, new parameter names (ast, lexical); Both = AST∩Lexical | ✓ |

**User's choice:** Revise for AST+Lexical dedup
**Notes:** Dedup logic is still needed since AST scanner produces findings and Lexical fallback (per Phase 18 D-02) also produces findings for unparseable files. Same key: (canonical_file_path, line, cwe_id).

---

## Claude's Discretion

None — all areas had explicit user direction.

## Deferred Ideas

- **Re-adding benchmark for Phase 21–23 regression tracking** — decided to delete now, can re-add if needed.
- **CWE-190, 416, 476, 401, 415, 590 AST coverage** — deferred to Phases 21–23.
