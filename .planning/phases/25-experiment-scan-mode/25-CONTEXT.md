# Phase 25: experiment-scan-mode — Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Gate the 17 high-FP CWE rules behind a `--experiment-scan` CLI flag. Default scan runs only the 22 high-confidence CWEs (Clean + Good + AUTOSAR-confirmed). Passing `--experiment-scan` is additive — both sets run and findings are combined into the same output formats.

This phase is purely structural: no rule logic changes, no new CWEs. It reclassifies existing rules into two tiers by annotating `AST_CWE_RULES` entries and modifying the scan dispatch path.

</domain>

<decisions>
## Implementation Decisions

### CWE Split

**D-01: Default scan — 22 CWEs (always active)**
- Clean tier (FP% ≤10%, 16 CWEs): CWE-114, 134, 242, 272, 284, 328, 479, 481, 482, 484, 526, 587, 617, 762, 785, 835
- Good tier (FP% 11–35%, 4 CWEs): CWE-78, 119, 125, 398
- AUTOSAR-confirmed no-signal (0 Juliet hits but confirmed TP on AUTOSAR): CWE-362, CWE-367

**D-02: Experimental scan — 17 CWEs (active only when `--experiment-scan` is passed, additive)**
- Marginal (1): CWE-467 (65.4% FP — oracle artifact; real incidental bugs in Juliet CWE-122 files)
- Poor (12): CWE-120, 122, 126, 190, 480, 483, 562, 570, 571, 676, 680, 780
- No-signal unconfirmed (4): CWE-338, 426, 478, 535 (0 AUTOSAR hits; unit-test TP only)

**D-03: No signal CWE-362/367 stay in default** because they fire correctly on AUTOSAR (race condition / TOCTOU — 1 finding each, confirmed TP). Juliet 0 hits is a corpus mismatch, not a false signal.

**D-04: No signal CWE-338/426/478/535 go to experimental** because they produce 0 findings on both Juliet AND AUTOSAR. Unit tests confirm the rule logic is correct, but there is no real-codebase evidence of signal yet.

### CLI Flag

**D-05: New flag `--experiment-scan` (boolean, no value)**
- When absent: run only the 22 default CWEs
- When present: run all 39 CWEs (22 default + 17 experimental), combined output
- Flag lives in the existing CLI arg struct alongside `--sarif-output`, `--sarif-baseline`

**D-06: Output format unchanged** — experimental findings go into the same SARIF/CycloneDX/markdown output as default findings. No separate file, no tagging of individual findings as "experimental" in this phase. The distinction is at the invocation level, not per-finding. Future phases may add per-finding tags.

**D-07: `run_ast_scanner` signature change** — add `experiment_mode: bool` parameter. The scan function filters `AST_CWE_RULES` by `experimental` flag before iterating. Structural visitor functions (`check_*`, `apply_*`) are unchanged — filtering happens at rule selection, not at rule execution.

### Rule Annotation

**D-08: Annotate `AST_CWE_RULES` entries** — add an `experimental: bool` field to the `AstCweRule` struct. Default `false` (active by default). Experimental rules set `experimental: true`. The `apply_ast_rules` function skips entries where `rule.experimental && !experiment_mode`.

**D-09: Structural `check_*` functions are not gated** — they are only called from `apply_ast_rules` (or `apply_*` helpers) which already skip the rule. No changes to individual check function signatures.

### Testing

**D-10: Regression — default scan must not change** on both Juliet and AUTOSAR vs current baseline:
- Juliet oracle on default 22 CWEs must match current per-CWE TP/FP counts for those CWEs
- AUTOSAR regression test must still pass (3 findings: CWE-362 × 1, CWE-367 × 1, CWE-369 × 1)

**D-11: New unit tests:**
- `experiment_scan_false_excludes_experimental_cwe` — run scanner without flag on a fixture that fires a Poor CWE; assert no finding
- `experiment_scan_true_includes_experimental_cwe` — run scanner with flag on same fixture; assert finding present
- `default_scan_includes_clean_cwe` — run scanner without flag; assert a Clean CWE still fires

**D-12: CWE-369 remains in default** — it appears in the AUTOSAR regression baseline (1 finding), but it's not in the AST_CWE_RULES table (it's a division-rule handled by `apply_division_rules`). The division rules are not gated by experiment mode — they always run. CWE-369 is not in the 17 experimental CWEs list.

### Documentation

**D-13: Update `--help` output** to describe `--experiment-scan` with a note that it enables 17 additional high-FP rules.

**D-14: Update `benchmark/juliet/ANALYSIS.md`** tier table to mark experimental CWEs with a note that they require `--experiment-scan`.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Code File
- `src/vulnerability/ast_scanner.rs` — `AstCweRule` struct, `AST_CWE_RULES` static, `apply_ast_rules()`, `run_ast_scanner()` signature

### CLI
- `src/cli.rs` (or `src/main.rs`) — existing flag definitions; add `--experiment-scan` here

### Benchmark Truth
- `benchmark/juliet/ANALYSIS.md` — authoritative per-CWE tier classification; D-01/D-02 split sourced from this document
- `tests/autosar_ast_regression.rs` — AUTOSAR baseline: 3 findings (CWE-362/367/369); must not regress

### Prior Phase Context
- `.planning/phases/24-tune-high-fp-cwe-rules-from-phases-19-23/24-CONTEXT.md` — ArgCheck variants, check_* function signatures, AST_CWE_RULES structure
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — run_ast_scanner() integration with main dispatch

</canonical_refs>

<specifics>
## Specific Ideas

- `AstCweRule` struct currently has fields: `cwe_id`, `check_fn` (or similar), `rule_entries`. Add `experimental: bool` — default `false` via `..Default::default()` or explicit field in each entry.
- `run_ast_scanner(dirs, experiment_mode: bool)` — filter rules before the scan loop: `let active_rules: Vec<_> = AST_CWE_RULES.iter().filter(|r| !r.experimental || experiment_mode).collect();`
- The 17 experimental entries need `experimental: true` set. The remaining 22 keep `experimental: false` (or omit if default).
- CLI call site passes `args.experiment_scan` boolean into `run_ast_scanner`.

</specifics>

<deferred>
## Deferred Ideas

- **Per-finding `experimental` tag in SARIF/CycloneDX output** — useful for tooling that wants to filter post-scan. Deferred to a follow-up phase.
- **`--experiment-only` mode** (run experimental rules but not default) — not requested; deferred.
- **Further tuning of experimental CWEs** (CWE-190 SizeArgIsArithmetic, CWE-122 memcpy drop, etc.) — deferred to a future tuning phase.

</deferred>

---

*Phase: 25-experiment-scan-mode*
*Context gathered: 2026-05-13 (from conversation decisions)*
