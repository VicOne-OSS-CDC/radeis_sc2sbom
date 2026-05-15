---
title: v1.0.18 Tree-sitter AST Scanner
trigger_when: v1.0.17 ships
planted_during: v1.0.17 milestone — AUTOSAR SAST verification session (2026-05-11)
---

# SEED-003: v1.0.18 — Tree-sitter AST Scanner

## Idea

Replace the cppcheck subprocess with an embedded tree-sitter-c (MIT) AST scanner compiled directly into the binary. The scanner walks the parse tree for C/C++ source files, enabling data-flow-aware CWE detection without any external tool dependency.

## Why This Matters

cppcheck has three friction points discovered during v1.0.17:

1. **External install required** — users without cppcheck get lexical-only results with no upgrade path
2. **False positives** — cppcheck fires on patterns our component-scoped context makes irrelevant; suppression lists add maintenance burden
3. **Subprocess overhead** — ~30s per component dir on large repos; degrades CI experience

tree-sitter-c solves all three: zero install, higher precision via parse-tree context, ships inside the static binary.

## PoC Evidence

A working proof-of-concept was committed during v1.0.17 at `src/vulnerability/ast_scanner.rs` behind `--features ast-scanner`:

- Parses C source with tree-sitter-c grammar
- Detects CWE-120 (`strcpy`, `sprintf`, unsafe buffer ops) by matching call expressions in the AST
- **Zero false positives on pointer params** — the AST disambiguates `strcpy(dst, src)` from safe wrappers in a way the lexical scanner cannot
- Single unit test passes on AUTOSAR_SampleProject_S32K144 fixture

## Proposed Scope for v1.0.18

### Phase A: AST scanner core

- Wire `ast_scanner.rs` into `run_lexical_scanner` as the default code path (replacing regex-based token matching)
- Expand CWE rule set from CWE-120 PoC to full v1.0.17 rule set:
  - CWE-78 (OS command injection), CWE-119/120/122/125 (buffer ops), CWE-134 (format string), CWE-190 (integer overflow), CWE-362/367 (race conditions), CWE-369 (divide-by-zero), CWE-416 (use-after-free), CWE-476 (null deref), CWE-732 (permissions)
- Preserve `SastFinding` struct — no downstream changes to SARIF/markdown output
- Gate behind `--features internal` (same as today)

### Phase B: cppcheck removal (or demotion)

- Remove cppcheck subprocess as default; optionally retain as `--features cppcheck` escape hatch
- Update graceful-degradation messaging

### Phase C: argument-value matching in AST

- Migrate v1.0.17 argument-value rules (CWE-295 `SSL_VERIFY_NONE`, CWE-319 `CURLOPT_USE_SSL`) to use AST argument node inspection instead of paren-bound string scanning
- Higher precision: AST gives typed argument nodes, not raw substring match

## Key Decisions to Make at Milestone Start

1. Ship phases A+B+C together or sequence them? Recommendation: Phase A first (parallel to existing lexical scanner, compare results), then B+C once parity is proven.
2. tree-sitter-c grammar version to pin — check license compatibility for static linking in musl binary.
3. Benchmark: measure AST scan time vs cppcheck on AUTOSAR_SampleProject_S32K144 and ros2cli before committing to removal.
4. Fallback: if tree-sitter parse fails (malformed C), do we fall back to lexical or skip the file?

## Relationship to v1.0.17

- v1.0.17 ships: `SastFinding`, `run_lexical_scanner`, cppcheck subprocess, SARIF output, `--sarif-baseline`, argument-value matching, suppress_lexical_false_positives
- v1.0.18 replaces the cppcheck subprocess path; all downstream consumers (SARIF writer, markdown report, CycloneDX serializer) are unchanged
- PoC (`src/vulnerability/ast_scanner.rs`) committed in v1.0.17 — not behind feature flag in public binary; should be gated before v1.0.18 ships

## Open Questions

1. Does tree-sitter-c handle incomplete/generated AUTOSAR C files (missing headers, partial TUs) without panicking?
2. What is the false-positive rate on the AUTOSAR_SampleProject_S32K144 BSW modules vs the v1.0.17 lexical scanner baseline?
3. Can the AST scanner detect CWE-362 (race condition) reliably, or does that remain a cppcheck-only rule?
