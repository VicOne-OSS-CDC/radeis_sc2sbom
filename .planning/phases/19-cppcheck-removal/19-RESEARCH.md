# Phase 19: cppcheck-removal — Research

**Researched:** 2026-05-12
**Domain:** Rust codebase surgery — dead code removal and pipeline simplification
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Hard remove — cppcheck is deleted entirely. No opt-in escape hatch, no `--features cppcheck` flag, no `--cppcheck-path` arg retained.
- **D-02:** Delete `run_cppcheck_scanner()` and `parse_cppcheck_xml()` from `src/vulnerability/cwe_scanner.rs` entirely.
- **D-03:** Delete the `--cppcheck-path` CLI arg from `src/cli.rs` and the associated warning in `main.rs`.
- **D-04:** Remove `run_cppcheck_scanner` and `parse_cppcheck_xml` from the `pub use` re-exports in `src/vulnerability/mod.rs`.
- **D-05:** Delete `SastSource::Cppcheck` — it becomes dead code with no cppcheck runner.
- **D-06:** Keep `SastSource::Both` — repurposed to represent a finding detected by BOTH AST and Lexical fallback scanners.
- **D-07:** Revise `deduplicate_sast_findings` to merge AST findings with Lexical fallback findings. Rename parameters from `(lexical, cppcheck)` to `(ast, lexical)`. Same dedup key: `(canonical_file_path, line, cwe_id)`.
- **D-08:** `main.rs` pipeline simplifies: `ast_findings → deduplicate(ast, lexical_fallback) → sast_findings`. Delete the `cppcheck_confirmed` BTreeSet and all related code.
- **D-09:** Delete `suppress_lexical_false_positives()` entirely.
- **D-10:** Remove `suppress_lexical_false_positives` from the `pub use` export in `src/vulnerability/mod.rs`.
- **D-11:** No scanner announcement printed in default CLI output. The "⚠ cppcheck not found" message is deleted with the runner.
- **D-12:** Delete `tests/benchmark.rs` entirely. `docs/BENCHMARK.md` can remain as a historical artifact.

### Claude's Discretion

None specified.

### Deferred Ideas (OUT OF SCOPE)

- Re-adding benchmark for Phase 21–23 AST regression tracking.
- CWE-190, 416, 476, 401, 415, 590 AST coverage — deferred to Phases 21–23.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CPP-01 | cppcheck subprocess removed or demoted based on Phase 18 benchmark data; graceful-degradation messaging updated to reflect new default | All locked decisions (D-01 through D-12) implement hard removal. Benchmark completed in Phase 18. No escape hatch retained. |
</phase_requirements>

## Summary

Phase 19 is a pure removal/simplification phase with no new feature development. The entire scope is: delete cppcheck-related code from five files, revise `deduplicate_sast_findings` to merge AST vs Lexical (instead of Lexical vs cppcheck), and delete the Phase 18 benchmark test file.

The codebase is fully inventoried. All deletion targets are confirmed present in the current state of the repo (branch `feature/v1.0.17-autosar-sast-sarif`). No new dependencies are introduced. The AST scanner (`run_ast_scanner`) already exists and already returns `Vec<SastFinding>` with `SastSource::Ast` — the pipeline simplification in `main.rs` is a mechanical substitution.

The main complication is the test file `tests/vulnerability_tests/suppression_tests.rs` — it tests `suppress_lexical_false_positives` and uses `SastSource::Cppcheck`. Both must be deleted along with the function they test. No other external test files reference cppcheck functions.

**Primary recommendation:** Execute as three atomic commits: (1) delete cppcheck functions and update `cwe_scanner.rs` + `mod.rs`, (2) revise `main.rs` + `cli.rs` pipeline, (3) delete test files. Compile-check after each commit.

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SAST scanning | API/Backend (Rust binary) | — | All scanner logic lives in `src/vulnerability/` |
| CLI argument parsing | API/Backend (`src/cli.rs`) | — | clap-based args; `--cppcheck-path` is the deletion target |
| Finding deduplication | API/Backend (`src/vulnerability/cwe_scanner.rs`) | — | `deduplicate_sast_findings` is revised in-place |
| Pipeline orchestration | API/Backend (`src/main.rs`) | — | cppcheck block replaced with simpler `deduplicate(ast, lexical)` call |
| Output formatting | API/Backend (`src/formats/`) | — | SARIF/CycloneDX/console writers consume `&[SastFinding]` unchanged |

## Standard Stack

This phase uses no new libraries. All existing dependencies remain.

### Unchanged Dependencies
| Library | Purpose | Affected by Phase 19? |
|---------|---------|----------------------|
| `quick_xml` | cppcheck XML parsing (currently imported in `cwe_scanner.rs`) | Import deleted with `parse_cppcheck_xml` |
| `indicatif` | Progress spinner for cppcheck scan | Import deleted with `run_cppcheck_scanner` |
| `tree_sitter` / `tree_sitter_c` | AST scanner — stays | No change |
| `walkdir` | File walking — stays | No change |
| `serde` | Serialization — stays | No change |

**Cargo cleanup note:** After removing `quick_xml::*` and `indicatif::*` imports from `cwe_scanner.rs`, `cargo check --features internal` will confirm whether these crates are still used elsewhere. [VERIFIED: grep of `src/` shows `quick_xml` and `indicatif` are only used in `cwe_scanner.rs`] — both become unused transitive dependencies but removing them from `Cargo.toml` is out of scope for this phase unless `cargo check` emits unused-dependency warnings that block compilation.

## Architecture Patterns

### System Architecture: Before vs. After

**Before (current):**
```
component_dirs
    → run_ast_scanner()           → ast_findings (SastSource::Ast)
    → run_cppcheck_scanner()      → cppcheck_findings + cppcheck_scanned_dirs
    → deduplicate_sast_findings(ast_findings, cppcheck_findings)
    → cppcheck_confirmed BTreeSet build
    → suppress_lexical_false_positives()
    → sast_findings
    → SARIF / CycloneDX / console writers
```

**After (target):**
```
component_dirs
    → run_ast_scanner()           → ast_findings (SastSource::Ast + lexical fallback already inside)
    → deduplicate_sast_findings(ast_findings, [])   -- or just use ast_findings directly
    → sast_findings
    → SARIF / CycloneDX / console writers
```

**Important clarification on `lexical_fallback_findings`:** Per Phase 18 D-02, the lexical fallback already runs *inside* `run_ast_scanner` on a per-file basis when tree-sitter fails to parse. The fallback findings are returned as part of the `ast_findings` Vec with `SastSource::Lexical` (the fallback scanner sets `source: SastSource::Lexical`). There is no separate `lexical_fallback_findings` Vec passed into `main.rs` — the entire output of `run_ast_scanner` IS the combined AST + per-file-fallback findings.

This means D-07's `deduplicate_sast_findings(ast, lexical)` should be called as `deduplicate_sast_findings(ast_findings, vec![])` OR the function can be simplified to a single-input dedup, OR the function is retained as a two-arg form for future use (Phases 21–23 may re-introduce a second scanner). The planner must choose — the decision is **Claude's discretion** since CONTEXT.md doesn't specify the call shape.

**Recommendation:** Retain the two-arg signature `deduplicate_sast_findings(ast: Vec<SastFinding>, lexical: Vec<SastFinding>)` for forward compatibility with Phases 21–23. In `main.rs`, call it as `deduplicate_sast_findings(ast_findings, vec![])`. This matches D-07's intent while keeping the API clean.

### Deletion Targets — Exact Inventory

**`src/vulnerability/cwe_scanner.rs`:**
- Lines 16–25: Trim unused imports (`quick_xml`, `indicatif`, `OsStr`, `BTreeSet`, `Command`, `Stdio`, `Process`) — only remove what becomes unused after deletions
- Lines 30–36: `SastSource` enum — delete `Cppcheck` variant (line 32); keep `Lexical`, `Both`, `Ast`
- Lines 456–483: `CPPCHECK_COVERED_CWES` const and `CPPCHECK_CWE_OVERRIDES` static — delete entirely
- Lines 493–583: `parse_cppcheck_xml()` function — delete entirely
- Lines 596–725: `run_cppcheck_scanner()` function — delete entirely
- Lines 757–783: `deduplicate_sast_findings()` — revise in-place (rename params, update doc comment, remove `SastSource::Cppcheck` branch → replace with `SastSource::Lexical` branch for the second input)
- Lines 785–818: `suppress_lexical_false_positives()` — delete entirely
- Update `SastSource::Both` doc comment to reflect AST∩Lexical semantics (not AST∩cppcheck)

**`src/vulnerability/mod.rs` line 16:**
```rust
// Before:
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, parse_cppcheck_xml, run_cppcheck_scanner, run_lexical_scanner, SastFinding, SastSource, suppress_lexical_false_positives};

// After:
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource};
```

**`src/cli.rs` lines 273–276:**
- Delete `cppcheck_path: Option<PathBuf>` field and its `#[cfg(feature = "internal")]` + `#[arg(long)]` annotations (3 lines)

**`src/main.rs` lines 197–276:**
- Lines 197–201: Delete the `--cppcheck-path` warning block (WR-01 comment + if block)
- Lines 250–276: Delete the cppcheck block: `cppcheck_bin` var, `run_cppcheck_scanner` call, `cppcheck_confirmed` BTreeSet, `suppress_lexical_false_positives` call
- Line 246–260: Replace the current multi-step pipeline with `let sast_findings_pre = crate::vulnerability::run_ast_scanner(&component_dirs);` then `sast_findings = crate::vulnerability::deduplicate_sast_findings(sast_findings_pre, vec![]);`

**`tests/benchmark.rs`:** Delete file entirely.

**`tests/vulnerability_tests/suppression_tests.rs`:** Delete file entirely (tests `suppress_lexical_false_positives` which is being removed).

### Anti-Patterns to Avoid

- **Leaving dead imports:** After deleting the cppcheck functions, `cwe_scanner.rs` will have unused imports for `quick_xml`, `indicatif`, `Command`, `Stdio`, `OsStr`, `BTreeSet`. Delete all that become unused — `cargo check` will identify them precisely.
- **Leaving `SastSource::Cppcheck` as dead variant:** A dead enum variant won't cause a compile error, but it creates confusion for Phases 21–23. Delete it.
- **Forgetting `suppression_tests.rs`:** This test file uses `suppress_lexical_false_positives` and `SastSource::Cppcheck` — it must be deleted, not just commented out. Leaving it causes compile failure under `--features internal`.
- **Over-simplifying `deduplicate_sast_findings` to single-input:** Retaining the two-arg form is the safer choice for Phase 21+ extensibility.

## Don't Hand-Roll

This phase is pure deletion. No new code is being built.

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Confirming unused imports after deletion | Manual inspection | `cargo check --features internal` — compiler lists all unused imports precisely |
| Verifying compilation after each step | Inference | `cargo check --features internal` after each file edit |
| Running the test suite | Manual review | `cargo test --features internal` — catches any missed references |

## Runtime State Inventory

This is a pure Rust code removal phase. No runtime state carries "cppcheck" as a stored name.

| Category | Items Found | Action Required |
|----------|-------------|-----------------|
| Stored data | None — cppcheck is a subprocess invocation, not a stored key | None |
| Live service config | None — no external service configuration references cppcheck | None |
| OS-registered state | None — cppcheck is called via `Command::new`, not a registered service | None |
| Secrets/env vars | None — no env vars name cppcheck | None |
| Build artifacts | None — cppcheck binary is a system tool, not a project artifact | None |

## Common Pitfalls

### Pitfall 1: Forgetting `suppression_tests.rs`
**What goes wrong:** The file `tests/vulnerability_tests/suppression_tests.rs` imports `suppress_lexical_false_positives` and constructs `SastSource::Cppcheck` findings. After removing those items, `cargo test --features internal` fails with a compile error.
**Why it happens:** Test files are not listed in CONTEXT.md's canonical refs — easy to overlook.
**How to avoid:** Delete `suppression_tests.rs` in the same commit or task as the function deletion.
**Warning signs:** `error[E0425]: cannot find function 'suppress_lexical_false_positives'` in test output.

### Pitfall 2: Orphaned `quick_xml` / `indicatif` imports
**What goes wrong:** `cwe_scanner.rs` currently imports `quick_xml::events::Event`, `quick_xml::Reader`, `indicatif::{ProgressBar, ProgressStyle}`, `std::process::{Command, Stdio}`, `std::ffi::OsStr`. These are exclusively used by the cppcheck functions. After deletion they become unused.
**Why it happens:** Rust's `unused_imports` lint is a warning by default but can be an error in CI (`-D warnings`).
**How to avoid:** Run `cargo check --features internal` after function deletion; the compiler identifies exactly which imports to remove.
**Warning signs:** `warning: unused import` for any of the above.

### Pitfall 3: `main.rs` still references `args.cppcheck_path`
**What goes wrong:** After deleting the `cppcheck_path` field from `cli.rs`, `main.rs` lines 199–201 (the warning block) still reference `args.cppcheck_path.is_some()` — compile error.
**Why it happens:** The two deletions are in different files and can be missed if done piecemeal.
**How to avoid:** Delete both the CLI field and the `main.rs` warning block in the same task.

### Pitfall 4: `deduplicate_sast_findings` doc comment refers to cppcheck
**What goes wrong:** The doc comment on `deduplicate_sast_findings` says "union of lexical and cppcheck findings" and references `SastSource::Both` as `D-12` (the old cppcheck-era decision). Leaving it creates misleading documentation.
**How to avoid:** Update the doc comment as part of the function revision (D-07).

### Pitfall 5: `benchmark.rs` imports `run_cppcheck_scanner` 
**What goes wrong:** `tests/benchmark.rs` line 16 imports `run_cppcheck_scanner` from `radeis_sc2sbom::vulnerability`. After `mod.rs` removes it from `pub use`, this import fails. The file must be deleted before or at the same time as the `mod.rs` change.
**How to avoid:** Delete `tests/benchmark.rs` first (or in the same task as `mod.rs` changes).

## Code Examples

### Revised `deduplicate_sast_findings` signature and doc comment

```rust
// Source: [ASSUMED] — derived from existing function, D-07
/// Deduplicate the union of AST and Lexical fallback findings by
/// `(normalized_file_path, line, cwe_id)`. When the same key appears
/// in both inputs, the surviving entry has its `source` set to
/// `SastSource::Both` (indicating higher-confidence dual-detected finding);
/// the ast entry's other fields are kept as the base.
///
/// Path normalization uses `normalize_path` (resolves `.`/`..` without
/// filesystem access) so dedup works even when source files are absent
/// at call time (CI, temp dirs, archives).
pub fn deduplicate_sast_findings(
    ast: Vec<SastFinding>,
    lexical: Vec<SastFinding>,
) -> Vec<SastFinding> {
    // ... same logic, first loop processes `ast`, second loop processes `lexical`
    // second loop: when key found in seen → promote to SastSource::Both
    //              when key not found → push as-is (SastSource::Lexical)
}
```

### Revised `main.rs` pipeline

```rust
// Source: [ASSUMED] — derived from CONTEXT.md D-08
// Phase 18 (D-02): AST scanner is primary; lexical fallback runs per-file inside run_ast_scanner.
let ast_findings = crate::vulnerability::run_ast_scanner(&component_dirs);
sast_findings = crate::vulnerability::deduplicate_sast_findings(ast_findings, vec![]);
```

### Revised `SastSource` enum

```rust
// Source: [ASSUMED] — derived from CONTEXT.md D-05, D-06
/// Origin of a SAST finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SastSource {
    /// Finding produced by the lexical regex scanner (fallback when AST parse fails).
    Lexical,
    /// Finding confirmed by both AST scanner and Lexical fallback scanner — higher confidence.
    Both,
    /// Phase 18: finding produced by tree-sitter-based AST scanner.
    Ast,
}
```

### Revised `mod.rs` re-export line

```rust
// Source: [VERIFIED: src/vulnerability/mod.rs line 16]
#[cfg(feature = "internal")]
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, run_lexical_scanner, SastFinding, SastSource};
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| Lexical scanner + cppcheck subprocess merger | AST scanner (tree-sitter) + per-file lexical fallback | Eliminates external binary dependency; faster scans; embedded grammar |
| `SastSource::Both` = AST∩cppcheck | `SastSource::Both` = AST∩Lexical fallback | Semantic repurpose; no code change to the variant itself |

## Open Questions

1. **Does `deduplicate_sast_findings` need to be called at all post-Phase 19?**
   - What we know: `run_ast_scanner` already returns a single Vec of combined AST + per-file lexical fallback findings. The lexical fallback in `scan_file_ast_or_lexical` is per-file exclusive (either AST or lexical, never both for the same file). So there is no overlap possible within `ast_findings` itself.
   - What's unclear: Whether any use case produces duplicate `(file, line, cwe)` entries within the AST findings alone (e.g., two rules matching the same site).
   - Recommendation: Retain `deduplicate_sast_findings(ast_findings, vec![])` as a safety measure and to preserve the two-arg API for Phase 21+. The cost is negligible.

2. **`docs/BENCHMARK.md` contains "run this test" instructions — does it confuse users?**
   - What we know: CONTEXT.md says "leave untouched unless it contains stale 'run this test' instructions."
   - What's unclear: Whether `docs/BENCHMARK.md` currently contains cargo-test invocation instructions that reference `tests/benchmark.rs`.
   - Recommendation: The planner should read `docs/BENCHMARK.md` and, if it contains a "how to run" section referencing `tests/benchmark.rs`, add a task to update that section to say the benchmark test was removed in v1.0.18.

## Environment Availability

Step 2.6: SKIPPED — this phase is pure code/config changes within the existing Rust project. No new external tools or services are required.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` |
| Config file | none — standard Cargo test runner |
| Quick run command | `cargo test --features internal` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CPP-01 | cppcheck code absent — project compiles without cppcheck binary | compile check | `cargo check --features internal` | N/A — compile pass IS the test |
| CPP-01 | `SastSource::Cppcheck` variant absent from enum | compile check | `cargo check --features internal` | N/A |
| CPP-01 | `run_cppcheck_scanner` not exported from `vulnerability` mod | compile check | `cargo check --features internal` | N/A |
| CPP-01 | `--cppcheck-path` CLI arg absent | unit | `cargo test --features internal -- cli` | existing test suite |
| CPP-01 | `deduplicate_sast_findings(ast, lexical)` correctly merges and deduplicates | unit | `cargo test --features internal -- deduplicate` | existing inline tests in `cwe_scanner.rs` |
| CPP-01 | Pipeline produces AST-only findings end-to-end | integration | `cargo test --features internal` | existing scanner integration tests |

### Sampling Rate
- **Per task commit:** `cargo check --features internal`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

A new unit test should be added to `cwe_scanner.rs` inline tests to verify the revised `deduplicate_sast_findings` with `(ast, lexical)` semantics:

- [ ] `test_deduplicate_ast_and_lexical_merge` — verifies that when same `(file, line, cwe)` appears in both ast and lexical inputs, result has `SastSource::Both`
- [ ] `test_deduplicate_ast_only_passthrough` — verifies that `deduplicate_sast_findings(ast_findings, vec![])` returns findings unchanged with `SastSource::Ast`

The existing `run_lexical_scanner` dedup test (`test_run_lexical_scanner_dedups_by_file_line_cwe`) is unaffected and continues to pass.

## Security Domain

This phase performs no security-relevant changes. It removes a subprocess invocation (`Command::new("cppcheck")`), which reduces the attack surface slightly (no external process execution at runtime). No new attack vectors are introduced.

ASVS categories V2/V3/V4 are not applicable. V5 (input validation) is unchanged — `cwe_scanner.rs` input validation logic is untouched. No cryptography changes.

## Sources

### Primary (HIGH confidence)
- [VERIFIED: direct file read] `src/vulnerability/cwe_scanner.rs` — full inventory of all functions, types, and imports to be deleted/revised
- [VERIFIED: direct file read] `src/vulnerability/mod.rs` — confirmed exact `pub use` line to revise
- [VERIFIED: direct file read] `src/cli.rs` lines 273–276 — `cppcheck_path` field confirmed present
- [VERIFIED: direct file read] `src/main.rs` lines 197–276 — cppcheck pipeline block confirmed present
- [VERIFIED: direct file read] `tests/benchmark.rs` — confirmed present, imports `run_cppcheck_scanner`
- [VERIFIED: direct file read] `tests/vulnerability_tests/suppression_tests.rs` — confirmed present, uses `suppress_lexical_false_positives` and `SastSource::Cppcheck`
- [VERIFIED: direct file read] `src/formats/sarif.rs`, `src/formats/cyclonedx.rs`, `src/formats/console.rs` — none pattern-match on `SastSource`; safe from this phase's changes
- [VERIFIED: direct file read] `src/vulnerability/ast_scanner.rs` — `run_ast_scanner` returns combined AST+fallback findings; lexical fallback is embedded per-file

### Secondary (MEDIUM confidence)
- [VERIFIED: grep] `quick_xml` and `indicatif` are used only in `cwe_scanner.rs` within `src/` — both imports become unused after function deletion

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `deduplicate_sast_findings(ast_findings, vec![])` is the correct call shape in `main.rs` after Phase 19 | Code Examples | If wrong, planner may choose a simpler single-arg form; low risk since two-arg form is a superset |
| A2 | No `quick_xml` or `indicatif` usage exists in other `src/` files besides `cwe_scanner.rs` | Standard Stack | If wrong, removing those imports from `cwe_scanner.rs` would still compile fine; cargo would retain the crate deps |
| A3 | `docs/BENCHMARK.md` does not contain "run this test" instructions requiring update | Open Questions | If wrong, requires an additional doc-update task |

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all changes are within files that were fully read in this session
- Architecture: HIGH — deletion targets verified by direct inspection; pipeline shape derived from Context.md locked decisions
- Pitfalls: HIGH — identified by direct code inspection (import lists, test file contents, cross-file references)

**Research date:** 2026-05-12
**Valid until:** Until branch state changes (this research targets the specific branch `feature/v1.0.17-autosar-sast-sarif`)
