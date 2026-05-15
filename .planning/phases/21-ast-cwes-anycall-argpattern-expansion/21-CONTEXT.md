# Phase 21: ast-cwes-anycall-argpattern-expansion — Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Expand `AST_CWE_RULES` in `ast_scanner.rs` from 13 to 26 CWEs (net: 12 new CWEs after dropping CWE-605) by adding call-site and operator-pattern rules. All new rules use the existing `ArgCheck` enum variants or two new variants (`SizeofPointer`, and a binary-expression gate for CWE-369). Validate each new CWE against the Juliet corpus after implementation; update `benchmark/juliet/ANALYSIS.md` with new per-CWE TP/FP rows. No downstream pipeline changes — SARIF writer, markdown report, and CycloneDX serializer consume `&[SastFinding]` unchanged.

**Effective scope: 12 CWEs** — CWE-605 deferred (requires cross-call fd tracking); CWE-676 included with a tighter function list.

</domain>

<decisions>
## Implementation Decisions

### CWE-369 (Divide by Zero)

- **D-01:** Add CWE-369 via a new AST binary-expression scan pass (`apply_division_rules()` or equivalent helper), NOT as an entry in `AST_CWE_RULES`. The function walks `binary_expression` nodes where the operator is `/` or `%` and the RHS child is an `integer_literal` AST node with text exactly `"0"`.
- **D-02:** `apply_division_rules()` is called alongside `apply_ast_rules()` in the per-file scan loop. Its infrastructure (binary_expression traversal) is intentionally designed for reuse by Phase 22's CWE-480/481/482 (wrong operator detection on binary_expression nodes).
- **D-03:** The lexical CWE-369 gate remains in place — parse-fail files still get CWE-369 detection via lexical fallback.

### New ArgCheck Variants

- **D-04:** Add `ArgCheck::SizeofPointer` — walks `sizeof_expression` nodes, checks if the argument node kind is a pointer_declarator or an identifier whose resolved type is a pointer. Used by CWE-467.
- **D-05:** `ContainsTokens` was deleted in Phase 20. Phase 21 uses `ArgAtIndex(u8, &'static [&'static str])` (inherited from Phase 20) for any new argument-value rules (e.g., CWE-780).

### CWE-126 (Buffer Over-Read)

- **D-06:** CWE-126 uses `ArgCheck::FixedSizeBuffer` — NOT `AnyCall`. The Phase 18 benchmark shows 94.8% FP with AnyCall on `strcat`/`strncat`. Restricting to calls where the destination arg is a fixed-size array declaration brings FP% within the ≤35% ROADMAP gate. Mirrors the existing CWE-119/120/122/125 pattern.

### CWE-327/328 Overlap

- **D-07:** CWE-328 rule uses non-overlapping functions only — MD5, SHA1, and DES variants are already covered by CWE-327 (`AST_CWE_RULES` line 67). CWE-328 adds only weak-hash functions NOT in the CWE-327 list (e.g., MD4, RC4-based functions). Researcher finalizes exact function list. No duplicate findings on the same call site.

### CWE-676 (Use of Potentially Dangerous Functions)

- **D-08:** CWE-676 is included but with a **tight non-overlapping function list** — only functions with no safe usage pattern that are not already covered by another CWE rule (e.g., `alloca`, `strtok`, and similar non-reentrant/dangerous-by-design functions). The Phase 18 benchmark shows 100% FP with a broad list; the researcher must select a minimal, high-signal list and verify FP% ≤35% after implementation.
- **D-09:** Functions already covered by other CWE rules (gets → CWE-120, system → CWE-78, rand → CWE-338) are excluded from CWE-676 to avoid duplicate findings.

### CWE-780 (RSA Without OAEP Padding)

- **D-10:** `ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"])` on `RSA_public_encrypt(flen, from, to, rsa, padding)` — the padding parameter is at index 4 (0-based). Fires when OAEP is NOT used. Uses the `ArgAtIndex` variant from Phase 20.

### CWE-467 (sizeof on Pointer Type)

- **D-11:** Uses new `ArgCheck::SizeofPointer` variant (D-04). Detects `sizeof(ptr)` where `ptr` is a pointer type — a common bug where the programmer intends `sizeof(*ptr)` or `sizeof(struct_type)` but gets pointer size instead.

### CWE-605 (Multiple Binds Same Port)

- **D-12:** **Deferred** from Phase 21. Requires tracking a socket file descriptor across two `bind()` calls — light dataflow, not a pure call-site AST pattern. Defer to Phase 22 or Phase 23.

### Remaining AnyCall CWEs (121, 338, 426, 526, 535, 680)

- **D-13:** CWEs 121, 338, 426, 526, 535, 680 are `AnyCall` rules — fire on any invocation of the named dangerous function. Exact function lists are **left to the researcher/planner** using CERT C documentation and the existing `AST_CWE_RULES` pattern. No user-specified inclusions or exclusions.
- **D-14:** CWE-121 (stack overflow) and CWE-338 (weak PRNG via `rand()`/`random()`/`srand()`) are included as AnyCall rules.

### Validation / Test Strategy

- **D-15:** Implement all 12 new CWE rules first, then re-run `sc2sbom` against the Juliet corpus and update `benchmark/juliet/ANALYSIS.md` with new per-CWE TP/FP rows. This satisfies ROADMAP Phase 21 success criterion #4.
- **D-16:** ROADMAP success criteria to verify: (1) each new CWE produces ≥1 TP on Juliet corpus; (2) FP% ≤35% per CWE using file-level oracle; (3) no regression on existing 13 CWEs.
- **D-17:** Test infrastructure location left to the planner (user said "you decide"). Suggested: a new `tests/ast_regression.rs` integration test file behind `#[cfg(feature = "internal")]`, gracefully skipping if Juliet fixture directory is absent — mirrors Phase 18 benchmark pattern but in a dedicated file.
- **D-18:** Juliet corpus already covers CWEs 78, 119, 120, 121, 122, 125, 126, 190, 242 well. CWEs not in Juliet (338, 426, 467, 526, 535, 676, 680, 780) should fall back to synthetic fixture files. Researcher confirms Juliet coverage per CWE.

### Claude's Discretion

- **Exact function lists for AnyCall CWEs (121, 338, 426, 526, 535, 680, 676 tight list):** Researcher/planner selects based on CERT C documentation and existing `AST_CWE_RULES` patterns.
- **`apply_division_rules()` exact implementation:** Planner decides whether to make it a standalone function or an inline block in the per-file scan loop, and how to share the binary_expression traversal pattern cleanly with Phase 22.
- **Test file location:** `tests/ast_regression.rs` suggested but planner has discretion.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/REQUIREMENTS.md` — CWEXP-01 (the requirement for this phase; note: REQUIREMENTS.md was last updated at Phase 20 milestone; Phase 21 CWE expansion is tracked in ROADMAP.md)
- `.planning/ROADMAP.md` §Phase 21 — success criteria: ≥1 TP per CWE on Juliet, ≤35% FP, no regression on existing 13 CWEs, ANALYSIS.md updated

### Benchmark / Ground Truth

- `benchmark/juliet/ANALYSIS.md` — **MUST READ** — existing Juliet full corpus results; contains Phase 19 recommendations including CWE-676 100% FP finding, CWE-126 94.8% FP finding, CWE-126 tune recommendation; success criterion #4 requires updating this file
- `benchmark/juliet/ast.json` — raw AST scanner findings per file (used to compute TP/FP table)

### Primary Code to Modify

- `src/vulnerability/ast_scanner.rs` — primary file: add `SizeofPointer` ArgCheck variant, add `apply_division_rules()` helper, add 12 new `AstCweRule` entries to `AST_CWE_RULES`, update module-level doc comment CWE coverage list
- `src/vulnerability/cwe_scanner.rs` — no modifications expected; lexical CWE-369 gate stays intact

### Prior Phase Context

- `.planning/phases/20-argument-value-ast-migration/20-CONTEXT.md` — D-01 (ArgAtIndex variant design), D-04 (ContainsTokens deleted), D-21 (ArgAtIndex struct), Phase 20 carried forward into Phase 21
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — D-06 (ArgCheck enum design), D-07 (11 tractable CWEs list), benchmark infrastructure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `ArgCheck` enum in `ast_scanner.rs:26` — add `SizeofPointer` variant here; `ArgAtIndex` is already present after Phase 20
- `AST_CWE_RULES` static table in `ast_scanner.rs:50` — append new `AstCweRule` entries; table is `&'static [AstCweRule]`
- `apply_ast_rules()` function (lines ~160+) — model `apply_division_rules()` on this function's structure; same `visit_node` recursion pattern but matches `binary_expression` nodes instead of `call_expression`
- `ArgCheck::FixedSizeBuffer` arm (lines ~235+) — CWE-126 reuses this existing variant; no new code required, just a new table entry
- `token_present_with_boundary` imported from `cwe_scanner.rs` — available for `ArgAtIndex` nested subtree text-contains check (already used in Phase 20)

### Established Patterns

- `#[cfg(feature = "internal")]` gate at file top — all scanner code is behind this; no change
- Per-CWE table entry format: `AstCweRule { cwe_id: N, functions: &["fn1", "fn2"], arg_check: ArgCheck::Variant }` — uniform for all new rules
- `SastSource::Ast` on all findings from `ast_scanner.rs` — unchanged
- Test pattern in existing `#[cfg(test)]` block: `run_ast_scanner` or `apply_ast_rules` with a small inline C string — mirrors for new CWE unit tests

### Integration Points

- `main.rs` SAST pipeline: consumes `Vec<SastFinding>` from `run_ast_scanner()` — no changes needed
- `deduplicate_sast_findings(ast, lexical_fallback)` from Phase 19 — unchanged; new CWE findings flow through the same pipeline
- SARIF writer, markdown report, CycloneDX serializer: consume `&[SastFinding]` — no changes

</code_context>

<specifics>
## Specific Ideas

- The `apply_division_rules()` helper is explicitly designed to share infrastructure with Phase 22's CWE-480/481/482 (wrong operator detection). The planner should design the binary_expression traversal so Phase 22 can add new operator-pattern rules with minimal new code.
- The CWE-676 100% FP finding in the benchmark is a strong signal — the researcher must choose a very tight function list. `alloca` (unconditional stack allocation risk), `strtok` (non-reentrant state), and potentially `getenv` (unvalidated env read in security context) are candidate-only; the final list must be verified against the Juliet corpus.
- CWE-126 with `FixedSizeBuffer` is expected to behave like CWE-119 (29.9% FP) — the planner should verify this hypothesis by checking the Juliet CWE-126 directory structure matches the CWE-119 family pattern used in the oracle.
- CWE-369 AST binary-expression gate provides structural precision the lexical scanner cannot: it won't fire on `x / 10` or `// comment with 0` or `x / 0.0` (float). The existing lexical CWE-369 remains for parse-fail fallback.

</specifics>

<deferred>
## Deferred Ideas

- **CWE-605 (Multiple Binds on Same Port)** — requires tracking socket fd across two `bind()` calls; light dataflow, not a pure AST pattern. Defer to Phase 22 or 23.
- **CWE-676 broad function list** — the full CERT C dangerous-function list produces 100% FP per benchmark. Deferred; Phase 21 ships with a minimal tight list only.
- **CWE-126 structural context check** — if `FixedSizeBuffer` still produces high FP on CWE-126, a more sophisticated buffer-origin check (source arg type, not dest) may be needed. Deferred post-benchmark.
- **Phase 22 binary_expression CWEs (480, 481, 482)** — intentionally deferred to Phase 22. The `apply_division_rules()` infrastructure added in Phase 21 is the foundation.

</deferred>

---

*Phase: 21-ast-cwes-anycall-argpattern-expansion*
*Context gathered: 2026-05-12*
