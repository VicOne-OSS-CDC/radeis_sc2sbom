# Phase 18: ast-scanner-core-and-benchmark — Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the tree-sitter-c PoC (`ast_scanner.rs`) as the default C/C++ scanner under `feature = "internal"`: expand to the full 14-CWE rule set (with per-CWE AST argument inspection for the 11 tractable CWEs), benchmark AST vs cppcheck vs lexical on AUTOSAR_SampleProject_S32K144 and a Juliet subset, and embed the grammar in the static musl binary with no runtime file dependency.

</domain>

<decisions>
## Implementation Decisions

### Feature Flag & Integration

- **D-01:** AST scanner merges into `feature = "internal"` — the separate `feature = "ast-scanner"` is dropped. One build flag for all scanner code.
- **D-02:** AST scanner is the **primary** C/C++ scanner when `internal` is active. The lexical scanner becomes the per-file fallback when tree-sitter fails to parse a file (AST-04). cppcheck continues as a secondary source going into the dedup pipeline (until Phase 19 removes it).
- **D-03:** `SastSource` gets a new `Ast` variant. AST findings use `SastSource::Ast` — distinguishes provenance in dedup logic, SARIF output, and future suppression rules.
- **D-04:** `ast_scanner.rs` emits `Vec<SastFinding>` directly (not `Vec<AstFinding>`). No intermediate type or conversion layer. Drops into the existing dedup + SARIF pipeline unchanged (AST-03).

### Rule Expansion (AST argument inspection)

- **D-05:** Rules use a **data-driven table** (analogous to `CWE_RULES` in `cwe_scanner.rs`). Each rule declares: CWE ID, function name(s), and an `ArgCheck` enum variant describing the AST condition to verify.
- **D-06:** `ArgCheck` is a Rust enum with variants (e.g., `FixedSizeBuffer`, `NotStringLiteralAtIndex(u8)`, `ContainsTokens(&'static [&'static str])`, `AnyCall`). Enum, not closures — exhaustive, no heap allocation, compiler-checked.
- **D-07:** Per-CWE AST argument inspection for the **11 tractable CWEs**: CWE-78, 119, 120, 122, 125, 134, 190, 242, 327, 369, 377, 732. (CWE-119/125 are buffer read overflows — similar pattern to CWE-120.)
- **D-08:** CWE-362 (race condition), CWE-416 (use-after-free), CWE-476 (null deref) — **deferred** from Phase 18. These require dataflow/CFG. They stay in the lexical fallback path. Local-scope heuristics may be added post-benchmark.
- **D-09:** The existing CWE-120 PoC logic (fixed-array destination check) is ported as the `FixedSizeBuffer` ArgCheck variant — not a standalone function.

### Benchmark

- **D-10:** Benchmark is a Rust integration test at `tests/benchmark.rs`, behind `#[cfg(feature = "internal")]`. It is **not** CI-gated — runs locally only.
- **D-11:** Test gracefully skips (with `eprintln!` message) when fixture directories are not present. No hard failure if Juliet fixture is missing.
- **D-12:** Two fixtures: AUTOSAR_SampleProject_S32K144 (primary) and a Juliet Test Suite C/C++ subset (secondary). Juliet files are staged locally, not committed to the repo.
- **D-13:** Benchmark produces a committed `BENCHMARK.md` at the repo root (or `docs/BENCHMARK.md`). This is the artifact Phase 19 planning reads to decide cppcheck fate.
- **D-14:** `BENCHMARK.md` columns: CWE ID | AST TPs | AST FPs | AST FP% | cppcheck TPs | cppcheck FPs | cppcheck FP% | Lexical TPs | Lexical FPs | Lexical FP% — per fixture, per CWE. Plus a summary recommendation row.

### Grammar Embedding

- **D-15:** Try `tree-sitter-c 0.24`'s built-in `build.rs` first with the existing musl-gcc / `musl-tools` CI setup. Only write a custom `build.rs` if the crate's internal build script fails to cross-compile cleanly.
- **D-16:** Grammar must be embedded in the binary (no runtime grammar file dependency). The `cc` crate compile of `parser.c` achieves this. DIST-02 satisfied.
- **D-17:** DIST-01 (license verification): confirm tree-sitter-c MIT license in `Cargo.toml` or a `LICENSE-NOTES.md`. Document in a comment or audit note — not a separate doc unless needed.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` — AST-01..04, BENCH-01, DIST-01, DIST-02 (Phase 18 requirements)
- `.planning/ROADMAP.md` §Phase 18 — Success criteria and phase dependencies

### Existing Scanner Code
- `src/vulnerability/ast_scanner.rs` — PoC to port/expand (CWE-120, `AstFinding` struct, tree-sitter parsing pattern)
- `src/vulnerability/cwe_scanner.rs` — `SastFinding`, `SastSource`, `CWE_RULES` table, `ArgCheck` pattern to mirror
- `src/scanner/mod.rs` — `scan_directory()` call site; where AST scanner wires in as primary

### Feature Flag
- `Cargo.toml` — current `[features]` section: `ast-scanner = ["dep:tree-sitter", "dep:tree-sitter-c"]` must be merged into `internal`

### Build / Distribution
- `.github/workflows/build-release.yml` — musl cross-compile setup (musl-tools, musl-gcc); reference when verifying grammar compilation

### Prior Phase Context
- `.planning/phases/16-sarif-as-authoritative-finding-store-refactor-the-static-ana/16-CONTEXT.md` — SARIF pipeline architecture, suppression logic, `SastSource` variants

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SastFinding` struct + `SastSource` enum (`src/vulnerability/cwe_scanner.rs`): AST scanner emits these directly. Add `SastSource::Ast` variant.
- `deduplicate_sast_findings()` (`src/vulnerability/cwe_scanner.rs`): existing dedup pipeline consumes AST findings unchanged.
- `scan_cwe120()` PoC logic in `ast_scanner.rs`: port as `ArgCheck::FixedSizeBuffer` variant in the new data-driven table.
- `CWE_RULES` static table pattern in `cwe_scanner.rs`: model the AST rule table on this shape.

### Established Patterns
- `feature = "internal"` gate on all scanner code: `#![cfg(feature = "internal")]` at file top.
- Per-file WalkDir + BufRead line scanning pattern in `cwe_scanner.rs`: AST scanner replaces the line scanner but uses the same component_dirs iteration.
- `warn_on_walkdir_err` utility: reuse for any file-access errors during AST scan.

### Integration Points
- `scanner/mod.rs` `scan_directory()`: this is where the call order changes — AST scanner runs first, lexical scanner called only for files where tree-sitter returns `None`.
- `main.rs` SAST pipeline: `lexical_findings + cppcheck_findings → deduplicate → suppress → writers`. Phase 18 changes the first term to `ast_findings + (lexical_fallback_findings) + cppcheck_findings`.
- SARIF writer + markdown report + CycloneDX serializer: these consume `&[SastFinding]` — no changes needed if `SastFinding` shape is preserved (AST-03).

</code_context>

<specifics>
## Specific Ideas

- The user wants per-CWE argument inspection (maximum precision), not a fast name-match-only implementation. Phase 18 is the precision investment phase; benchmark data validates it.
- CWE-362/416/476 local-scope heuristics (ptr=malloc→immediate deref without null check, free(ptr)→subsequent use) are interesting but deferred — ship the 11 tractable CWEs first, let benchmark show if local heuristics are worth adding.
- Benchmark is purely internal (not customer-facing). The `BENCHMARK.md` artifact is the decision artifact for Phase 19 planning. It should be readable by a human in 2 minutes.
- The "try crate's built-in build.rs first" approach: researcher should do a quick local test of `cargo build --features internal --target x86_64-unknown-linux-musl` after adding tree-sitter-c as a dep under `internal`, before designing a custom build.rs.

</specifics>

<deferred>
## Deferred Ideas

- **CWE-416 (use-after-free) local heuristic**: `free(ptr)` → subsequent use of `ptr` in same scope. Feasible with tree-sitter but deferred pending benchmark data.
- **CWE-476 (null deref) local heuristic**: `ptr = malloc(...)` → immediate deref without null check. Feasible with tree-sitter but deferred pending benchmark data.
- **CWE-362 (race condition) local heuristic**: pthread_create/fork + shared global access without adjacent mutex lock. High FP risk — deferred.
- **Content-based fingerprinting**: Discussed in Phase 16, deferred. Still deferred.

</deferred>

---

*Phase: 18-ast-scanner-core-and-benchmark*
*Context gathered: 2026-05-11*
