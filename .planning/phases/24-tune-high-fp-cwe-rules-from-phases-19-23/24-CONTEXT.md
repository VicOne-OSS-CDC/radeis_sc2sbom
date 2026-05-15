# Phase 24: tune-high-fp-cwe-rules-from-phases-19-23 — Context

**Gathered:** 2026-05-13
**Status:** Ready for planning

<domain>
## Phase Boundary

Audit and tighten all 17 CWE rules from phases 19–23 that exceed the 35% FP gate on the Juliet corpus. All 17 targets receive a fix attempt — no pure "accept" exceptions. Net result: 48 CWEs (CWE-256 removed; 16 tightened; no new CWEs added). Phase 24 is purely subtractive/tightening — no new `AstCweRule` entries, no new downstream format changes.

</domain>

<decisions>
## Implementation Decisions

### Scope

- **D-01:** All 17 CWE targets from phases 19–23 that exceed 35% FP gate receive a fix attempt. No CWE is pre-accepted without an attempt.
- **D-02:** Phase 24 covers all high-FP CWEs from phases 19–23 (ROADMAP title says 19–23; Phase 24 target list already explicitly names CWEs from all three phases). All 17 are in scope.
- **D-03:** CWE-256 is **removed** from `AST_CWE_RULES` — 100% FP, 0 Juliet TPs, corpus mismatch (Juliet uses Windows API, not string-literal password patterns). Net coverage after Phase 24: 48 CWEs.

### Fix Strategy — AnyCall / Function List Restrictions

- **D-04:** **CWE-338** — Remove `rand()`/`srand()` from function list; keep only `drand48`, `lrand48`, `random`, `mrand48`. Juliet TPs will drop to 0 (Juliet only uses `rand()`). Acceptable — 0 TPs is preferred over 99.9% FP. Validate via synthetic unit test.
- **D-05:** **CWE-676** — Drop `alloca` from function list (already covered by CWE-121); keep only `strtok`. Reduces noise; Juliet TPs will stay 0 (Juliet CWE-676 uses `cin>>`, not `strtok`). Validate via synthetic unit test.
- **D-06:** **CWE-426** — Replace `popen`/`system` (which the oracle maps to CWE-78) with `dlopen`, `LoadLibraryExA`, `LoadLibraryExW`. These are the canonical CWE-426 (untrusted search path) functions. Removes the oracle-mismatch FPs entirely.
- **D-07:** **CWE-780** — Remove `RSA_public_encrypt` from function list (fires in all non-CWE-780 OpenSSL files). Keep only the `CryptEncrypt` `ArgAtIndex(3, &["0"])` rule. This is already present as a separate `AstCweRule` entry; the `RSA_public_encrypt` entries are the FP source.
- **D-08:** **CWE-676 and CWE-338 unit tests** — New synthetic unit tests confirming TP on the tightened function lists. Pattern: inline C string in `#[cfg(test)]` block, matching existing Phase 21/22/23 unit test style.

### Fix Strategy — New ArgCheck Variant

- **D-09:** **CWE-126** — Change from `ArgCheck::AnyCall` to a new `ArgCheck::FixedSizeBufferWithoutSizeArg(size_arg_index)` variant. Fires when: (1) the destination arg is a fixed-size array (`char buf[N]`), AND (2) the size arg at `size_arg_index` is NOT a `sizeof()` expression. For `strcat(buf, src)` there is no size arg (AnyCall semantics apply only to dest check); for `strncat(buf, src, n)` the size arg index is 2. Planner implements this new `ArgCheck` variant in `apply_ast_rules`.
- **D-10:** **CWE-680** — Add a size-arg `binary_expression *` guard: fire only when the size argument is a multiplication expression (e.g., `malloc(n * sizeof(T))`). This targets the actual integer-overflow-in-size-calculation vulnerability. Simple `malloc(sizeof(T))` does NOT fire. Planner decides whether this is a new `ArgCheck` variant or inline logic in `apply_ast_rules`.
- **D-11:** **CWE-467** — Tighten `SizeofPointer` check: fire only when the `sizeof` operand is a pointer type (not a struct or array). Walk the enclosing function's declarations to confirm the operand identifier was declared as a pointer. Planner determines exact tree-sitter node type check.

### Fix Strategy — Structural Visitor Function Guards

- **D-12:** **CWE-480** — In `check_func_ptr_null_compare()`: before firing, walk the enclosing function's declarations and check that the compared identifier's declaration type contains `(` (pointer-to-function syntax heuristic). Only fire when the identifier looks like a function pointer. Reduces the 22,755 FPs from generic null checks.
- **D-13:** **CWE-483** — In `check_block_delimitation()`: add exclusion — do NOT fire when the braceless if-body is a `return_statement`, `break_statement`, or `continue_statement`. These are common single-statement patterns that aren't dangerous.
- **D-14:** **CWE-562** — In `check_return_stack_address()`: restrict to plain scalar variables only (not arrays or structs). If the local variable's declaration is an `array_declarator` or has a struct/union type, skip it. Arrays/structs returned by address are common legitimate patterns.
- **D-15:** **CWE-570** — In `check_constant_condition()`: remove detection in loop conditions (`while`/`for`/`do-while`). Keep only `if`-condition context. Loop-constant patterns (`while(0)`) are common macro artifacts. CWE-835 covers the dangerous infinite loop case.
- **D-16:** **CWE-571** — In `check_constant_condition()`: remove detection in loop conditions (`while(1)`, `for(;;)`). CWE-835 already fires on these via `check_infinite_loop()`. Keep CWE-571 for `if`/ternary always-true conditions only. Eliminates CWE-835 / CWE-571 overlap.
- **D-17:** **CWE-535** — In the `AstCweRule` for `fprintf`/`stderr`: combine with a non-literal format-string guard (same as CWE-134's `ArgCheck::NotStringLiteralAtIndex`). Fire on `fprintf(stderr, non_literal_fmt, ...)` only. This restricts to the actually dangerous pattern (format-string injection into stderr) rather than any `fprintf(stderr, ...)` call. Planner decides whether to extend the existing CWE-134 rule entries or create a new `AstCweRule` for CWE-535 with `NotStringLiteralAtIndex(1)`.
- **D-18:** **CWE-587** — In `check_fixed_address_assignment()`: raise the fixed-address threshold from the current value to `> 0xFFFF` (65535). Hardware peripheral addresses in embedded/AUTOSAR code are typically small (GPIO registers, UART base addresses); attacker-controlled fixed addresses are large hex values. This reduces FPs on embedded fixtures.
- **D-19:** **CWE-478** — In `check_switch_structure()`: add guard — do NOT fire when the switch has ≤2 cases. Trivial 2-case switches (often bool/flag switches) are common and not dangerous. Only fire when the switch has ≥3 cases without a default.
- **D-20:** **CWE-762** — In `apply_delete_rules()`: add co-occurrence guard — fire only when `calloc`, `malloc`, or `realloc` also appears in the same file. This prevents firing on pure C++ files that use `delete` for C++ allocations only. Walk file-level findings or pre-scan for C-alloc function calls before emitting CWE-762 findings.

### Validation Strategy

- **D-21:** Implement all 17 fixes first, then re-run `oracle.sh` once (single-pass validation). ANALYSIS.md updated in one batch after the oracle run. Consistent with Phase 21/22/23 pattern.
- **D-22:** CWEs that drop to 0 Juliet TPs after tightening (CWE-338, CWE-676, possibly CWE-426) are validated via new synthetic unit tests in `#[cfg(test)]` blocks. Pattern: inline C string that exercises the tightened rule, confirming TP on synthetic fixture.
- **D-23:** AUTOSAR regression check (`AUTOSAR_SampleProject_S32K144`) runs **after human verification of the Juliet oracle delta**. Sequence: (1) implement all fixes → (2) run Juliet oracle → (3) human reviews ANALYSIS.md delta → (4) run AUTOSAR regression.
- **D-24:** Any fix attempt that still leaves FP% >35% after the oracle run becomes a **human-review item** in the Phase 24 verification checklist with before/after FP% documented. Decision to remove, demote, or accept is made after reviewing the re-run results together — not decided in advance.

### ANALYSIS.md Update

- **D-25:** Full oracle re-run (`oracle.sh`) after all 17 fixes — regenerate all rows in the Per-CWE TP/FP table. Do not do in-place edits; regenerate from fresh counts.
- **D-26:** Add a `## Phase 24 Notes` section documenting: which CWEs were fixed (before/after FP%), which were removed (CWE-256), and any CWEs with residual FP% >35% that became human-review items.

### Phase 24 Success Criteria

- **D-27:** Phase 24 is complete when: (1) all 17 code changes made, (2) Juliet oracle re-run, (3) ANALYSIS.md updated with full regenerated table + Phase 24 Notes section, (4) human reviews Juliet delta, (5) AUTOSAR regression run confirms no regressions on pre-existing CWEs.
- **D-28:** No hard numeric bar on how many must reach <35% FP. Residual failures become documented human-review items; the phase is not blocked by them.

### Claude's Discretion

- **CWE-126 `strcat` (no size arg):** For `strcat(dest, src)` (2-arg, no size arg), the `FixedSizeBufferWithoutSizeArg` variant should fire on dest-is-fixed-size-array alone (no size-arg check possible). For `strncat(dest, src, n)`, the full combined check applies (dest fixed-size AND size arg not `sizeof`). Planner decides whether to use one rule entry with two modes or two separate `AstCweRule` entries.
- **CWE-680 guard implementation:** Whether the multiplication-expression size-arg check is a new `ArgCheck` variant (`SizeArgIsMultiplication`) or inline logic in `apply_ast_rules`. Planner decides based on code complexity.
- **CWE-570/571 loop-context exclusion:** Exact tree-sitter node types to check (`while_statement`, `for_statement`, `do_statement` vs walking parent chain). Planner decides based on `check_constant_condition()` implementation.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope & Targets

- `.planning/ROADMAP.md` §Phase 24 — goal, depends-on Phase 23, full list of 17 target CWEs with their FP% from phases 19–23
- `benchmark/juliet/ANALYSIS.md` — **MUST READ** — existing per-CWE TP/FP table; Phase 24 regenerates all rows after fixes; Phase 19/21/22/23 notes document the root causes and recommended actions for each high-FP CWE

### Primary Code File

- `src/vulnerability/ast_scanner.rs` — all Phase 24 changes are in this file: `AST_CWE_RULES` table edits (function list changes, ArgCheck changes), `check_*` visitor function guards, `apply_delete_rules()` co-occurrence guard, `apply_ast_rules()` extensions for new ArgCheck variants

### ArgCheck Enum Reference

- `.planning/phases/20-argument-value-ast-migration/20-CONTEXT.md` — D-01 through D-04: full `ArgCheck` enum state entering Phase 24; `ArgAtIndex`, `NotStringLiteralAtIndex`, `SizeofPointer`, `FixedSizeBuffer` variants documented
- `.planning/phases/21-ast-cwes-anycall-argpattern-expansion/21-CONTEXT.md` — D-01 (`apply_division_rules()` model), D-04 (`SizeofPointer` variant), D-14/D-15 (validate-after-implement pattern)

### Prior Phase Context (FP root causes)

- `.planning/phases/22-ast-cwes-structuralpattern-expansion/22-CONTEXT.md` — D-05 (CWE-835 body-check), D-06 (CWE-570/571 literal-only policy, now overridden by D-15/D-16), D-11 (FP threshold policy), D-13 (≥1 TP floor)
- `.planning/phases/23-ast-cwes-domainspecific-expansion/23-CONTEXT.md` — D-01 (CWE-762 delete_expression traversal), D-08 (apply_signal/paired_lock structure), FP Gate Violations section (CWE-762 co-occurrence fix recommendation)

### Benchmark / Validation

- `benchmark/juliet/ANALYSIS.md` §Phase 21 EXCEEDS GATE rationale — root-cause notes for CWE-126/338/426/467/535/676/680/780
- `benchmark/juliet/ANALYSIS.md` §Phase 22 D-11 documented FP exceptions — root-cause notes for CWE-256/478/480/483/562/570/571/587
- `benchmark/juliet/ANALYSIS.md` §Phase 23 FP Gate Violations — CWE-762 root cause and recommended action

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `ArgCheck` enum in `ast_scanner.rs` — add `FixedSizeBufferWithoutSizeArg(u8)` variant for CWE-126; optionally add `SizeArgIsMultiplication(u8)` for CWE-680. Existing variants: `AnyCall`, `FixedSizeBuffer`, `ArgAtIndex`, `NotStringLiteralAtIndex`, `SizeofPointer`.
- `apply_ast_rules()` at line 342 — the integration point for `AstCweRule` table entries. New `ArgCheck` variant handling is added here.
- `check_func_ptr_null_compare()` at line 629 — add function-pointer type declaration check (D-12, CWE-480).
- `check_block_delimitation()` at line 479 — add return/break/continue exclusion (D-13, CWE-483).
- `check_return_stack_address()` at line 758 — add scalar-only guard (D-14, CWE-562).
- `check_constant_condition()` at line 882 — add non-loop-context filter (D-15/D-16, CWE-570/571).
- `check_fixed_address_assignment()` at line 978 — raise hex threshold to >0xFFFF (D-18, CWE-587).
- `check_switch_structure()` at line 390 — add ≤2 case guard (D-19, CWE-478).
- `apply_delete_rules()` at line 1640 — add co-occurrence guard for C-alloc functions (D-20, CWE-762).
- `AST_CWE_RULES` static table at line 67 — function list edits for CWE-338/426/676/780; ArgCheck changes for CWE-126/535/680.

### Established Patterns

- `find_enclosing_function(node: Node) -> Option<Node>` — already exists; reuse for CWE-480 declaration walk and CWE-562 scalar-only check.
- `visit_node` recursive walker — structural checks call their own recursive walks; don't nest inside `visit_node`.
- `#[cfg(feature = "internal")]` gate — all scanner code is behind this; no change.
- Unit test pattern: `run_ast_scanner()` / `apply_ast_rules()` with inline C string in `#[cfg(test)]` block — use for CWE-338/426/676 synthetic TPs.

### Integration Points

- `deduplicate_sast_findings(ast, lexical_fallback)` — Phase 24 changes reduce finding counts; dedup logic unchanged.
- SARIF writer, markdown report, CycloneDX serializer — all consume `&[SastFinding]`; no downstream changes required.
- CWE-570/571 share `check_constant_condition()` — the loop-context exclusion (D-15/D-16) modifies this single function.

</code_context>

<specifics>
## Specific Ideas

- **CWE-780 function list fix:** The current rule has three separate `AstCweRule` entries for CWE-780 (see line ~199–225 in ast_scanner.rs). The `RSA_public_encrypt` entries are the FP source. Remove those entries; keep only the `CryptEncrypt` `ArgAtIndex(3, &["0"])` entry.
- **CWE-571/CWE-835 overlap fix:** After restricting CWE-571 to non-loop contexts, confirm that `check_infinite_loop()` (CWE-835) still fires on `while(1)` — the two rules are now cleanly separated with no overlap.
- **CWE-762 co-occurrence check:** The check scans the file's raw byte content (or pre-scanned call list) for `calloc`, `malloc`, or `realloc` before emitting delete findings. Same file-level scan approach used in `apply_paired_lock_rules()` for CWE-591.
- **ANALYSIS.md full re-run:** Run `benchmark/juliet/oracle.sh` after all code changes. The script is already reproducible (documented in Phase 21 notes). All rows in the Per-CWE TP/FP table are regenerated from the fresh run output.

</specifics>

<deferred>
## Deferred Ideas

- **CWE-338 context-aware detection** — keeping `rand()` but firing only in security-sensitive contexts (key/seed/token usage). Requires data-flow or naming heuristic; too complex for Phase 24 tightening pass.
- **CWE-256 replacement rule** — a more precise plaintext-password rule using a different detection approach (e.g., API call pattern instead of identifier name heuristic). Deferred to a future phase if the coverage gap matters.
- **CWE-570/571 variable-folding** — detecting `const int x = 0; if (x)` (always-false via constant propagation). Requires intra-function constant folding, not pure AST shape. Deferred per Phase 22 D-11.
- **CWE-480 mutual-recursion detection** — A→B→A call-graph analysis. Deferred per Phase 22 D-03.

</deferred>

---

*Phase: 24-tune-high-fp-cwe-rules-from-phases-19-23*
*Context gathered: 2026-05-13*
