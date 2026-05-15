# Phase 22: ast-cwes-structuralPattern-expansion — Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Add 15 new CWE detection rules to `ast_scanner.rs` using pure AST structural-shape queries — no dataflow or cross-function type analysis. Rules detect code shapes like missing switch default/break, assignment-in-condition, return of stack address, always-true/false constant expressions, infinite loops, plaintext password storage by identifier name, and direct self-recursion. Total CWE coverage expands from 26 (post-Phase 21) to 41. All rules validated against the Juliet C test suite with TP% ≥50% of available test cases per CWE and FP% documented in `benchmark/juliet/ANALYSIS.md`.

</domain>

<decisions>
## Implementation Decisions

### Rule Architecture

- **D-01:** Structural-pattern rules live in **dedicated visitor functions** (`check_switch_structure`, `check_assignment_in_condition`, `check_return_stack_address`, etc.) added as private functions in `ast_scanner.rs`. These are called from `apply_ast_rules` alongside the existing `call_expression` loop. No new `ArgCheck` variants needed for non-call-site patterns.
- **D-02:** All new functions stay in `ast_scanner.rs` — no new module file. Consistent with the Phase 18 pattern; the file may grow but splitting adds module overhead that isn't justified yet.

### CWE-674: Uncontrolled Recursion

- **D-03:** Detect **direct self-recursion only** — a `function_definition` node that contains a `call_expression` to its own function name. Mutual recursion (A→B→A) is out of scope for Phase 22; it requires call-graph analysis across the file.
- **D-04:** The check is purely intra-function: collect the function name from the `function_declarator` child, then walk the function body for `call_expression` nodes with a matching `function` field text.

### CWE-835: Infinite Loop

- **D-05:** **Planner decides** — planner should assess the Juliet CWE-835 fixture and choose between:
  - Flagging `while(1)` / `for(;;)` / `do { } while(1)` unconditionally (simple, potentially high FP in embedded/AUTOSAR code with legitimate polling loops), OR
  - Flagging only if the loop body contains no `break`, `return`, `goto`, or `exit()` call (lower FP but more complex body traversal).
  - Document the chosen approach and its resulting FP% in ANALYSIS.md.

### CWE-570 / CWE-571: Always-False / Always-True Expressions

- **D-06:** Narrow to **literal-constant comparisons only**: fire when an `if` condition (or loop condition) is a `binary_expression` where both operands are `number_literal` nodes, OR a single `number_literal` where the literal is `0` (always-false) or a non-zero integer (always-true). This is far more precise than cppcheck's broad approach (which produced 99.9%+ FP). Document actual FP% in ANALYSIS.md.

### CWE-398: Poor Code Quality

- **D-07:** **Planner decides** — inspect the Juliet `CWE-398` test fixture to identify the narrowest detectable AST pattern. If no tractable single structural shape exists (the CWE is extremely broad), skip CWE-398 in Phase 22, document it as "deferred — no tractable AST pattern found", and Phase 22 delivers 14 new CWEs (coverage 26→40) instead of 15. Do not ship a rule with >40% FP just to hit the count.

### CWE-256: Plaintext Password Storage

- **D-08:** Fire using an **identifier name heuristic**: a `declaration` node where the declarator identifier name contains `password`, `passwd`, `pwd`, or `secret` (case-insensitive), AND the initializer is a `string_literal` node.
- **D-09:** Starting scope: **declarations only** (`char *password = "abc"`). Planner should assess the Juliet `CWE-256` fixture — if TP% ≥50% is achievable from declarations alone, stop there. If the fixture uses post-declaration assignments (`password = "abc"`), extend to `assignment_expression` nodes where the LHS identifier matches the heuristic and the RHS is a `string_literal`.
- **D-10:** The heuristic keywords are: `password`, `passwd`, `pwd`, `secret` (case-insensitive substring match on the identifier name).

### FP Threshold Policy

- **D-11:** ≤40% FP is the **goal** for all 15 CWEs, not a hard gate. If a rule exceeds 40% FP, **document and ship anyway** — record the actual FP% in `benchmark/juliet/ANALYSIS.md` and let users manage noise via `--sarif-baseline`. This is consistent with how CWE-120 (89% FP) and CWE-126 (95% FP) were handled in Phase 18.

### Validation Strategy

- **D-12:** **Juliet-only** for Phase 22. All 15 CWEs have Juliet test directories. Only add synthetic fixtures under `tests/fixtures/` if a CWE's Juliet directory is empty or yields 0 TPs with a reasonable rule.
- **D-13:** Success criterion for each CWE: **TP% ≥50%** of available Juliet test cases (file-level oracle: scanner CWE matches Juliet directory CWE family). This is stricter than the ROADMAP "≥1 TP" wording — the ROADMAP criterion is the floor; 50% is the target.
- **D-14:** After all rules are implemented, run the full Juliet benchmark and update `benchmark/juliet/ANALYSIS.md` with a new per-CWE TP/FP row for each of the 15 Phase 22 CWEs.
- **D-15:** No regression on existing 26 CWEs — AUTOSAR fixture finding counts must be unchanged before and after Phase 22 changes.

### Claude's Discretion

- CWE-835 (D-05): Planner picks the most practical approach (unconditional flag vs. body-check) based on Juliet CWE-835 fixture structure and expected embedded-code FP impact.
- CWE-398 (D-07): Planner inspects Juliet CWE-398, identifies the narrowest tractable pattern, or skips CWE-398 if none exists.
- CWE-256 (D-09): Planner extends from declarations-only to assignments if needed to achieve TP% ≥50% on Juliet CWE-256.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/ROADMAP.md` §Phase 22 — goal, success criteria (15 CWEs, ≤40% FP, regression check, ANALYSIS.md update), and phase dependencies
- `.planning/ROADMAP.md` §Phase 21 — Phase 22 depends on Phase 21 completing; confirms the 26-CWE baseline this phase expands from

### Primary Code Files
- `src/vulnerability/ast_scanner.rs` — primary file for all Phase 22 additions: new visitor functions, called from `apply_ast_rules`
- `src/vulnerability/cwe_scanner.rs` — NOT modified in Phase 22 (structural patterns are AST-only; no lexical scanner changes)
- `src/vulnerability/mod.rs` — check for any exports that need updating after Phase 22 additions

### Benchmark & Validation
- `benchmark/juliet/ANALYSIS.md` — existing per-CWE TP/FP table; Phase 22 must add rows for all 15 new CWEs
- `benchmark/juliet/ast.json` — raw AST scanner output from the last Juliet run; use as regression baseline

### Prior Phase Context
- `.planning/phases/20-argument-value-ast-migration/20-CONTEXT.md` — D-01 through D-04 define ArgCheck enum state entering Phase 22 (ArgAtIndex added, ContainsTokens removed)
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — D-02 (AST primary + lexical fallback), D-03 (SastSource::Ast), established visitor pattern

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `visit_node` recursive walker in `ast_scanner.rs` — the established pattern for walking the AST. New structural visitor functions should follow the same `fn check_X(node: Node, src: &[u8], ..., findings: &mut Vec<SastFinding>)` signature called from `apply_ast_rules`.
- `find_enclosing_function(node: Node) -> Option<Node>` — already exists; CWE-674 (self-recursion) needs this to identify the current function name.
- `SastFinding` struct + `SastSource::Ast` — unchanged; all Phase 22 findings emit `SastSource::Ast`.
- `#[cfg(feature = "internal")]` gate — all new code must be inside this feature, consistent with all other SAST code.

### Established Patterns
- `apply_ast_rules` calls `visit_node` for the call_expression pass. Phase 22 adds separate calls to structural visitor functions at the root level — not nested inside `visit_node`. Each new visitor function does its own recursive walk over the relevant node types.
- Fresh cursor per call level (`let mut cursor = node.walk()`) — Pitfall 1 from Phase 18; all new recursive walkers must follow this.
- `SastFinding { cwe_id, component_name, component_ecosystem, file_path, line, source: SastSource::Ast }` — copy this exact construction pattern.

### Integration Points
- `apply_ast_rules` is the single integration point: add `findings.extend(check_switch_structure(root, src, path, ...))` etc. at the end of `apply_ast_rules`, after the existing `visit_node` call.
- No changes to `run_ast_scanner`, `scan_file_ast_or_lexical`, or the fallback path — Phase 22 is purely additive.
- `deduplicate_sast_findings` (Phase 19/20) — structural findings from Phase 22 pass through dedup unchanged; no new dedup logic needed.

</code_context>

<specifics>
## Specific Ideas

- CWE-483 (Incorrect Block Delimitation — `if (x) stmt1; stmt2;` where only stmt1 is guarded): this is one of the trickier structural patterns — tree-sitter parses this as a valid `if_statement` with a single-statement body. The AST shape is an `if_statement` where the `consequence` is NOT a `compound_statement`. This is structurally detectable with a single node-type check. Planner should start here as a reference implementation for the pattern.
- CWE-562 (Return of Stack Variable Address — `return &local_var`): fire when a `return_statement` contains an `address_of` expression (`&` unary operator) applied to an identifier declared as a local variable in the enclosing function. Requires: (a) walk the enclosing function's local declarations to collect non-pointer, non-array local variable names, (b) check if return `&X` where X is in that set.
- CWE-587 (Assignment of Fixed Address to Pointer — `ptr = (int*)0xDEADBEEF`): fire when an `assignment_expression` or `init_declarator` assigns a `cast_expression` or `integer_literal` with a large hex value (e.g., >0xFFFF) to a pointer-type variable. Planner should inspect Juliet CWE-587 fixture for the exact pattern used.

</specifics>

<deferred>
## Deferred Ideas

- **Mutual recursion detection (CWE-674)** — A→B→A pattern requires cross-function call graph analysis within a file. Deferred beyond Phase 22; direct self-recursion is the tractable subset.
- **CWE-570/571 with variable-folding** — detecting `const int x = 0; if (x)` (always-false via constant propagation) requires simple intra-function constant folding, not pure AST shape. Deferred; Phase 22 covers only literal-in-condition patterns.
- **CWE-835 loop escape analysis** — if the body-check approach (D-05) is too complex for Phase 22, the simpler unconditional flag ships first; a tighter rule can be added in a follow-on phase.

</deferred>

---

*Phase: 22-ast-cwes-structuralPattern-expansion*
*Context gathered: 2026-05-12*
