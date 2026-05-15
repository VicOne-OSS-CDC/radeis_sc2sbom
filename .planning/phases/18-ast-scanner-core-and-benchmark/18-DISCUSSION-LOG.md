# Phase 18: ast-scanner-core-and-benchmark — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-11
**Phase:** 18-ast-scanner-core-and-benchmark
**Areas discussed:** Feature flag strategy, Rule expansion approach, Benchmark format & fixture scope, Grammar embedding strategy

---

## Feature Flag Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Merge into `internal` | AST scanner joins `feature = "internal"` — same gate as lexical scanner | ✓ |
| Keep `ast-scanner` separate | Two optional features: `internal` and `ast-scanner` | |
| Make it unconditional | Embed tree-sitter-c in the public binary | |

**User's choice:** Merge into `internal` (Recommended)
**Notes:** Clean single build flag; public binary guarantee preserved.

---

| Option | Description | Selected |
|--------|-------------|----------|
| AST as primary, lexical as parse-fallback | AST runs first; lexical only for files tree-sitter fails to parse; cppcheck secondary | ✓ |
| All three in parallel | Lexical + AST + cppcheck all run | |

**User's choice:** AST as primary, lexical as parse-fallback

---

| Option | Description | Selected |
|--------|-------------|----------|
| Emit `SastFinding` directly | AST scanner returns `Vec<SastFinding>` — no conversion layer | ✓ |
| Keep internal `AstFinding`, convert at call site | Separate type, conversion in scanner/mod.rs | |

**User's choice:** Emit `SastFinding` directly

---

| Option | Description | Selected |
|--------|-------------|----------|
| New `SastSource::Ast` variant | New enum variant alongside Lexical/Cppcheck/Both | ✓ |
| Reuse `SastSource::Lexical` | AST findings tagged as Lexical | |

**User's choice:** New `SastSource::Ast` variant

---

## Rule Expansion Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Name-match + call site | Flag every call — AST confirms it's a real call_expression | |
| Per-CWE argument inspection | Custom AST arg check per CWE | ✓ |
| Hybrid: argument inspection for top 3 | Custom for CWE-134/78/190, name-match for rest | |

**User's choice:** Per-CWE argument inspection
**Notes:** User asked for pros/cons comparison before deciding. Chose maximum precision despite implementation effort.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Name-match only for dataflow-dependent CWEs | Flag call site, no false precision | |
| Skip them in AST scanner | Lexical fallback covers them | |
| Defer all three to post-benchmark | Ship Phase 18 without CWE-362/416/476 in AST scanner | ✓ |

**User's choice:** Defer CWE-362/416/476 to post-benchmark
**Notes:** User asked whether we could do dataflow ourselves. Discussed local-scope heuristics for CWE-476 (ptr=malloc→deref) and CWE-416 (free→use). Decision: defer pending benchmark data; ship 11 tractable CWEs first.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Data-driven table | Extend CWE_RULES style; `ArgCheck` enum variants | ✓ |
| Standalone functions per CWE | Each CWE gets its own `scan_cweN()` fn | |

**User's choice:** Data-driven table (consistent with lexical scanner pattern)

---

| Option | Description | Selected |
|--------|-------------|----------|
| Enum variants | `ArgCheck` enum, exhaustive, no heap allocation | ✓ |
| Closures / fn pointers | `arg_check: fn(Node, &[u8]) -> bool` per rule | |

**User's choice:** Enum variants

---

## Benchmark Format & Fixture Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Markdown comparison table | Per-CWE counts in `_static_analysis.md` benchmark section | |
| SARIF baseline diff | Machine-readable diff SARIF | |
| Test assertions with hardcoded counts | `#[test]` asserting finding counts | |

**User's choice:** Hybrid (both markdown table AND SARIF diffs)
**Notes:** User asked "will this show to customers?" — clarified: benchmark is internal evaluation only, not customer-facing.

---

| Option | Description | Selected |
|--------|-------------|----------|
| Integration test + committed BENCHMARK.md | `tests/benchmark.rs` produces committed markdown | ✓ |
| Shell script | `scripts/benchmark.sh` | |

**User's choice:** Rust integration test + committed `BENCHMARK.md`

---

| Option | Description | Selected |
|--------|-------------|----------|
| Juliet Test Suite (C/C++) subset | Known-TP/FP files per CWE | ✓ |
| xcar-linux repo | Internal real-world automotive Linux C code | |
| Hand-crafted fixture in tests/fixtures/ | Minimal controlled C files | |

**User's choice:** Juliet Test Suite subset

---

| Option | Description | Selected |
|--------|-------------|----------|
| Committed subset in tests/fixtures/juliet/ | Curated subset committed to repo | |
| Downloaded at test time | Network dependency | |

**User's choice:** Stage locally, not committed (local-only benchmark test)
**Notes:** "Keep the repo clean, this test is performed locally only without CI."

---

| Option | Description | Selected |
|--------|-------------|----------|
| Skip gracefully if fixture missing | `eprintln!` + return | ✓ |
| Hard fail if fixture missing | Test fails without fixture | |

**User's choice:** Graceful skip with message

---

| Option | Description | Selected |
|--------|-------------|----------|
| Per-CWE TP/FP/FP-rate table (all three scanners) | Full comparison table | ✓ |
| Total finding counts only | Simple totals | |
| SARIF diff only | No TP/FP labeling | |

**User's choice:** Per-CWE: TP count, FP count, FP rate — AST vs cppcheck vs lexical

---

## Grammar Embedding Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Try crate's built-in build.rs first | tree-sitter-c 0.24 includes its own build.rs | ✓ |
| Write custom build.rs from the start | Full control over compiler flags | |

**User's choice:** Try crate's built-in build.rs first; custom only if musl-gcc fails
**Notes:** User asked for detailed comparison of embedding options. Discussed build.rs + cc crate, using crate's built-in build.rs, and include_bytes! of pre-compiled grammar. Recommended option B (crate's built-in) first.

---

## Claude's Discretion

- Specific `ArgCheck` variant names and the complete variant list (e.g., `FixedSizeBuffer`, `NotStringLiteralAtIndex(u8)`, `ContainsTokens`) — left to researcher/planner
- Exact path for committed `BENCHMARK.md` (repo root vs `docs/`) — left to planner

## Deferred Ideas

- CWE-476 local heuristic: `ptr = malloc(...)` → immediate deref without null check
- CWE-416 local heuristic: `free(ptr)` → subsequent use in same scope
- CWE-362 local heuristic: pthread_create/fork + shared access without mutex (high FP risk)
