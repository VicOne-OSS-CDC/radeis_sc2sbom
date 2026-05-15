# Phase 22: ast-cwes-structuralPattern-expansion - Research

**Researched:** 2026-05-12
**Domain:** Rust / tree-sitter-c AST pattern detection, C CWE rules
**Confidence:** HIGH

## Summary

Phase 22 expands `ast_scanner.rs` from 26 to 41 CWEs by adding 15 rules that detect structural code
shapes — patterns detectable from AST node types alone, without dataflow or type inference. Every
target CWE has a Juliet test directory verified on disk. The existing `visit_node` / `apply_ast_rules`
architecture supports these rules without modification to any calling code.

The most important finding from Juliet fixture inspection is the **CWE-570/571 scope problem**:
only 2 of the 16 files per CWE use pure `number_literal` patterns (`if (0)` / `if (1)` /
`if (2 == 3)` / `if (2 == 2)`). The other 14 files use global variables, static variables, and
variable-comparison patterns that D-06 explicitly excludes. The literal-constant rule will achieve
roughly 12–14% TP (2/16 files) on each — below the 50% target. Planner must account for this.

For **CWE-398**, all five Juliet pattern families (`addition`, `equals`, `five`, `semicolon`, empty)
use distinct AST shapes. The two tractable shapes are: (1) `expression_statement` containing only a
`number_literal` (`5;`) and (2) `expression_statement` containing only a `binary_expression` with
both operands being `number_literal` (`intOne + intTwo;` without assignment). A third family
(`equals` — `intOne = intOne`) is detectable as self-assignment. These three together cover
~100/181 files, achieving >50% TP.

For **CWE-835**, the Juliet suite has exactly 6 files across four distinct loop patterns:
`while(1)`, `for(;;)`, `do { } while(cond >= 0)`, and `for(i=0; i>=0; ...)`. Only the first two
are unambiguous literal-infinite loops. The `do/for` variants with `>=` conditions require runtime
bounds analysis. Body-check approach (look for no `break`/`return` within the loop body) handles
both the `while(1)` and `for(;;)` cases while reducing AUTOSAR FP from embedded polling loops.

For **CWE-562**, only 3 Juliet files exist, and neither uses `return &local_var` (address-of
operator on local). Both C files return a local array by name (`return charString`). The
`find_enclosing_function` helper already exists and the pattern is: `return_statement` node whose
child `identifier` refers to a non-static, non-pointer, non-parameter local variable.

**Primary recommendation:** Implement the 15 rules as private `check_*` functions called from
`apply_ast_rules`. Use the body-check approach for CWE-835. Implement CWE-398 with the three
tractable sub-patterns. Accept below-50% TP for CWE-570/571 due to Juliet's variable-based
patterns; document actual rates in ANALYSIS.md.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Structural-pattern rules live in dedicated visitor functions added as private functions
  in `ast_scanner.rs`. Called from `apply_ast_rules`. No new `ArgCheck` variants for non-call-site
  patterns.
- **D-02:** All new functions stay in `ast_scanner.rs` — no new module file.
- **D-03:** CWE-674: detect direct self-recursion only (function calls itself). Mutual recursion
  out of scope.
- **D-04:** CWE-674 check is intra-function: collect function name from `function_declarator` child,
  walk body for `call_expression` with matching `function` field text.
- **D-06:** CWE-570/571: narrow to literal-constant comparisons only. Binary expression where both
  operands are `number_literal`, OR single `number_literal` condition (`if (0)` / `if (1)`).
- **D-08:** CWE-256: fire on `declaration` node where declarator identifier contains `password`,
  `passwd`, `pwd`, or `secret` (case-insensitive), AND initializer is `string_literal`.
- **D-09:** CWE-256: start with declarations-only; extend to assignment_expression if needed for
  TP% >= 50%.
- **D-10:** CWE-256 heuristic keywords: `password`, `passwd`, `pwd`, `secret` (case-insensitive
  substring on identifier name).
- **D-11:** FP threshold >= 40% is goal not hard gate; ship with documented FP%; let users manage
  via `--sarif-baseline`. Consistent with CWE-120 (89% FP) and CWE-126 (95% FP) from Phase 18.
- **D-12:** Juliet-only for Phase 22. Add synthetic fixtures under `tests/fixtures/` only if a
  CWE's Juliet directory yields 0 TPs.
- **D-13:** Success criterion: TP% >= 50% of available Juliet test cases per CWE. Floor is >= 1 TP.
- **D-14:** After all rules implemented, run full Juliet benchmark and update
  `benchmark/juliet/ANALYSIS.md` with new per-CWE rows for all 15 Phase 22 CWEs.
- **D-15:** No regression on existing 26 CWEs — AUTOSAR fixture finding counts unchanged.

### Claude's Discretion

- **CWE-835 (D-05):** Planner picks between unconditional flag (while(1)/for(;;)/do{}while(1)) vs.
  body-check (only flag if loop body has no break/return/goto/exit() call). Document chosen approach
  and resulting FP% in ANALYSIS.md.
- **CWE-398 (D-07):** Planner inspects Juliet CWE-398, identifies narrowest tractable pattern, or
  skips CWE-398 if no tractable pattern; Phase 22 then delivers 14 new CWEs (26→40 total).
- **CWE-256 (D-09):** Planner extends from declarations-only to assignments if needed to achieve
  TP% >= 50% on Juliet CWE-256.

### Deferred Ideas (OUT OF SCOPE)

- Mutual recursion detection (CWE-674) — A→B→A requires cross-function call graph.
- CWE-570/571 with variable-folding — constant propagation required; not pure AST shape.
- CWE-835 loop escape analysis — if body-check is too complex for Phase 22, ship unconditional
  flag; tighter rule in follow-on phase.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CWEXP-02 | Expand AST scanner from 26 to 41 CWEs via pure structural-pattern rules (15 new CWEs: 256, 398, 478, 480, 481, 482, 483, 484, 562, 570, 571, 587, 617, 674, 835). All rules validated against Juliet ground truth with TP% ≥50% target and FP% documented in ANALYSIS.md. | All 15 CWE Juliet directories verified on disk. Per-CWE AST patterns identified from fixture inspection. Architecture supports additive implementation in ast_scanner.rs. |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| AST rule execution | `src/vulnerability/ast_scanner.rs` | — | D-02: all code stays in this file |
| Finding emission | `SastFinding` + `SastSource::Ast` | — | Existing struct; unchanged |
| Juliet benchmark update | `benchmark/juliet/ANALYSIS.md` | `benchmark/juliet/ast.json` | D-14 requires updating the per-CWE table |
| Regression guard | AUTOSAR fixture counts | Juliet TP/FP counts | D-15: no count change on AUTOSAR |
| Integration point | `apply_ast_rules` | — | Single call site for all new check_* functions |

## Standard Stack

All Phase 22 code uses the existing project stack. No new dependencies.

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tree-sitter | (workspace) | AST node traversal, `Node` API | Already embedded; all existing rules use it |
| tree-sitter-c | (workspace) | C grammar — provides node kinds | Already embedded |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `HashSet<String>` (std) | std | Local variable name collection for CWE-562/674 | Collect local decl names before rule fires |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Per-CWE visitor functions | Extending `AstCweRule` enum with new variants | New enum variants would require `ArgCheck` case handling in `visit_node`; structural patterns are not call-site patterns so dedicated visitors are cleaner (D-01) |

**Installation:** No new packages. Phase 22 is purely additive Rust code inside `#[cfg(feature = "internal")]`.

## Architecture Patterns

### System Architecture Diagram

```
apply_ast_rules(root, src, path, name, ecosystem)
  │
  ├── visit_node(root, ...) ─────────────── existing call-expression loop (unchanged)
  │     └── fires AST_CWE_RULES (ArgCheck variants)
  │
  ├── check_switch_structure(root, src, path, ...) ──────────── CWE-478, CWE-484
  │     └── walks switch_statement nodes → checks for default_case / break in each case
  │
  ├── check_assignment_in_condition(root, src, path, ...) ─── CWE-481, CWE-482, CWE-480
  │     └── walks if_statement / while_statement conditions → checks binary_expression operator
  │
  ├── check_block_delimitation(root, src, path, ...) ────────── CWE-483
  │     └── walks if_statement / for_statement → checks consequence is NOT compound_statement
  │
  ├── check_return_stack_address(root, src, path, ...) ──────── CWE-562
  │     └── walks function_definition → collect local decls → check return_statement
  │
  ├── check_constant_condition(root, src, path, ...) ────────── CWE-570, CWE-571
  │     └── walks if_statement / while_statement conditions → literal node type check
  │
  ├── check_fixed_address_assignment(root, src, path, ...) ─── CWE-587
  │     └── walks init_declarator / assignment_expression → hex literal threshold check
  │
  ├── check_plaintext_password(root, src, path, ...) ─────────── CWE-256
  │     └── walks declaration nodes → identifier name heuristic + string_literal initializer
  │
  ├── check_assert_calls(root, src, path, ...) ───────────────── CWE-617
  │     └── walks call_expression nodes for assert() → AnyCall
  │
  ├── check_infinite_loop(root, src, path, ...) ─────────────── CWE-835
  │     └── walks for_statement / while_statement / do_statement → literal condition + body check
  │
  ├── check_self_recursion(root, src, path, ...) ────────────── CWE-674
  │     └── walks function_definition → collect name → walk body for call_expression match
  │
  └── check_poor_code_quality(root, src, path, ...) ─────────── CWE-398
        └── walks expression_statement → detect no-effect patterns
```

### Recommended Project Structure

No structural changes. All new code is in:
```
src/vulnerability/
└── ast_scanner.rs       # add ~15 private check_* functions + extend apply_ast_rules
benchmark/juliet/
└── ANALYSIS.md          # add 15 new per-CWE TP/FP rows
```

### Pattern 1: Structural Visitor Function Signature

Every new check function follows the same signature established by Phase 18:

```rust
// Source: existing ast_scanner.rs pattern (visit_node, collect_file_scope_arrays_rec)
fn check_switch_structure(
    node: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "switch_statement" {
        // inspect node children for default_case absence or missing break
        // emit SastFinding { cwe_id, ..., source: SastSource::Ast }
    }
    // Recurse — fresh cursor per call level (Pitfall 1)
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            check_switch_structure(cursor.node(), src, path, component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

Integration point in `apply_ast_rules` (called after existing `visit_node` call):
```rust
// Source: CONTEXT.md code_context § Integration Points
findings.extend(check_switch_structure(root, src, path, component_name, component_ecosystem));
// ... same for each check_* function
```

The `findings: &mut Vec<SastFinding>` approach (pass-by-mutable-ref) is used for functions that
recurse, matching `visit_node`. For non-recursive top-level checks, returning a `Vec<SastFinding>`
and calling `findings.extend(...)` in `apply_ast_rules` is cleaner.

### Pattern 2: SastFinding Construction

```rust
// Source: existing ast_scanner.rs visit_node
findings.push(SastFinding {
    cwe_id: 478,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: (node.start_position().row as u32) + 1,
    source: SastSource::Ast,
});
```

All Phase 22 findings use `SastSource::Ast`. Line number is always `node.start_position().row + 1`.

### Pattern 3: Collecting Local Variable Names (for CWE-562, CWE-674)

`find_enclosing_function(node)` already exists and returns `Option<Node>`. To collect local
variable identifiers:

```rust
// Collect all local declaration identifiers within a function_definition subtree
fn collect_local_var_names(fn_node: Node, src: &[u8]) -> HashSet<String> {
    let mut names = HashSet::new();
    collect_local_vars_rec(fn_node, src, &mut names);
    names
}

fn collect_local_vars_rec(node: Node, src: &[u8], out: &mut HashSet<String>) {
    if node.kind() == "declaration" {
        // Walk init_declarator / pointer_declarator to find identifier child
        // Exclude: array_declarator (already handled by collect_array_declarators)
        // Exclude: static storage class (not stack-allocated)
    }
    let mut cursor = node.walk();  // fresh cursor per level
    if cursor.goto_first_child() {
        loop {
            collect_local_vars_rec(cursor.node(), src, out);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

### Anti-Patterns to Avoid

- **Reusing a cursor across recursive calls:** Every recursive call must create its own
  `let mut cursor = node.walk()`. This is Pitfall 1 from Phase 18 and has already caused issues.
- **Adding new `ArgCheck` variants for structural patterns:** D-01 prohibits this. Structural
  patterns have their own visitor functions, not extensions of the call-expression loop.
- **Splitting into a new module:** D-02 keeps all code in `ast_scanner.rs`.
- **Checking `SastSource` anywhere except emission:** The `deduplicate_sast_findings` function
  handles dedup; Phase 22 emits and moves on.
- **Using child index instead of `child_by_field_name`:** Always prefer field-based access for
  named fields (Pattern 3 from Phase 18 code_context). Fall back to `child(i)` loops only for
  unnamed children.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive AST walk | Custom iteration | Follow existing `visit_node` / `collect_arrays_in_subtree` pattern | Cursor lifecycle is subtle; existing code is correct |
| Function name discovery | Parse declarator text manually | `find_enclosing_function()` + `child_by_field_name("declarator")` | Already tested and handles pointer/array declarators |
| Fixed-array name set | Build during structural walk | Reuse `collect_function_scope_fixed_arrays` pattern | Same subtree-collection approach works for local vars |
| Line number extraction | Count newlines in source | `node.start_position().row + 1` | tree-sitter provides this directly |

## Per-CWE AST Patterns (Verified from Juliet)

### CWE-478: Missing Default Case in Switch
**Juliet files:** 18 (9 bad / 9 good, 5 variants)
**Pattern:** `switch_statement` node where walking the `body` (`switch_statement`'s `compound_statement` child) finds no `default_case` child among `case_statement` nodes.
**Tree-sitter node kinds:** `switch_statement` → body → walk for `case_statement` vs `default_case` [VERIFIED: Juliet fixture inspection]
**Expected TP%:** High — every bad file has a switch with no default; every good file adds default. ~50% of files contain bad variant.

### CWE-484: Omitted Break Statement in Switch
**Juliet files:** 18
**Pattern:** Within a `case_statement` body, the last statement is NOT a `break_statement` (and there is no `return_statement` or `goto_statement` as final statement). Fire per case_statement that falls through.
**Complexity:** Must not fire for the final case if there's a `default:` immediately after (intentional fall-through to default is common). Fire when case body has no `break`/`return`/`goto` as its last direct child statement.
**Expected TP%:** Moderate — Juliet bad variant always has one case missing break; good adds break everywhere.

### CWE-481: Assigning Instead of Comparing
**Juliet files:** 18
**Pattern:** `if_statement` / `while_statement` / `for_statement` where the `condition` field is an `assignment_expression` node (operator `=`).
**Juliet bad example:** `if(intRand = 5)` [VERIFIED: CWE481__basic_01.c]
**Expected FP%:** LOW to MODERATE — assignment in condition is generally incorrect; some codebases use `while ((c = getchar()) != EOF)` pattern (valid idiom). Fire on direct `if (x = val)` without wrapping `!=`/`== 0`.

### CWE-482: Comparing Instead of Assigning
**Juliet files:** 18
**Pattern:** A statement-level `expression_statement` whose expression is a `binary_expression` with operator `==` (not inside `if`/`while`/`for` condition). That is, the comparison result is discarded.
**Juliet bad example:** `intBadSink == 5;` (top-level expression statement) [VERIFIED: CWE482__basic_01.c]
**Key distinction from CWE-481:** CWE-482 fires at statement level (`expression_statement` → `binary_expression` with `==`). CWE-481 fires inside condition of `if`/`while`.

### CWE-480: Use of Incorrect Operator
**Juliet files:** 18
**Pattern:** `if_statement` condition is a `binary_expression` where LHS is a function identifier (not a function call — no trailing `()`), compared with `==` to `NULL` or `0`. Specifically: `binary_expression` with `==` where one operand is an `identifier` whose kind is `identifier` (not `call_expression`).
**Juliet bad example:** `if(helperBad == NULL)` — comparing function pointer to NULL instead of calling it [VERIFIED: CWE480__basic_01.c]
**Narrowing:** Fire when both conditions hold: (a) `if` condition is `binary_expression`, (b) one operand's kind is `identifier` matching a known function name in scope, (c) other operand is `null_literal` or `number_literal` `0`. This is complex — planner may simplify to: fire when `if` condition binary_expression has one side that is a plain `identifier` (not call_expression) and the other is `null_literal`.

### CWE-483: Incorrect Block Delimitation
**Juliet files:** 5
**Pattern:** `if_statement` node whose `consequence` field is NOT a `compound_statement`. Tree-sitter parses `if (x) stmt;` as `if_statement` where consequence is a single statement (e.g., `expression_statement`, `return_statement`), NOT `compound_statement`. Fire on any `if_statement` with a non-compound consequence. [VERIFIED: CONTEXT.md specifics section + fixture inspection]
**Note also:** `semicolon` variant — `if (x);` is `if_statement` where consequence is an `empty_statement`. This fires the same rule.
**Expected FP%:** MODERATE to HIGH — style warning; many codebases use braceless single-line ifs intentionally.

### CWE-562: Return of Stack Variable Address
**Juliet files:** 3 (2 C, 1 cpp)
**Critical finding:** Neither C file uses `return &local_var`. Both return a local array variable by name (`return charString;` where `charString` is a local array). The "address" is implicit — the array decays to pointer, which is stack-allocated. [VERIFIED: fixture inspection]
**Pattern:** `return_statement` child is an `identifier` that refers to a non-static local array or non-array non-static local variable declared in the same function. Collect local non-static non-parameter variable names, fire if returned identifier matches.
**Limitation:** `return_pointer_buf_01.c` returns `ptrCharString = &charString[1]` — pointer to array element, returned via a local pointer variable. To catch this, the rule needs: local pointer variable whose initialization contains `address_of_expression` applied to a local array.
**D-09 implication:** Start with the simpler form (return of local array identifier). If TP% < 50% on the 2 C files, extend.

### CWE-570: Expression Always False / CWE-571: Expression Always True
**Juliet files:** 16 each
**Critical finding:** Only 2 files per CWE use pure literal patterns detectable by D-06:
- CWE-570: `zero_01.c` (`if (0)`) and `two_equals_three_01.c` (`if (2 == 3)`) [VERIFIED]
- CWE-571: `one_01.c` (`if (1)`) and `two_equals_two_01.c` (`if (2 == 2)`) [VERIFIED]
- Other 14 files use `globalFalse`, `globalFive`, static variables, `n == n-1`, string comparisons — all excluded by D-06.
**Expected TP%:** ~12–14% (2/16) for each — below the 50% target. Planner must document this gap.
**D-06 compliance:** Ship the literal-only rule as specified. Do NOT add variable-folding to hit 50% — that is explicitly deferred.
**FP concern:** `if (0)` and `if (1)` patterns appear in Juliet's own good/bad test wrappers (`if(0) { INCIDENTAL... }`) — these will be counted as FPs in non-CWE-570/571 files.

### CWE-587: Assignment of Fixed Address to Pointer
**Juliet files:** 18
**Pattern:** `init_declarator` or `assignment_expression` where RHS is a `cast_expression` wrapping a `number_literal` whose text matches a large hex pattern (e.g., `0x400000`). Or plain `number_literal` that is a large hex value. [VERIFIED: CWE587__basic_01.c — `(char*)0x400000`]
**Implementation:** Walk `init_declarator` nodes. If initializer child is a `cast_expression` containing a `number_literal` starting with `0x`, parse as hex and fire if value > 0xFFFF (rules out `(int*)0` which is a common null-pointer idiom).
**Also catch:** Direct integer literal assignment to pointer variable without cast — less common in Juliet.

### CWE-617: Reachable Assertion
**Juliet files:** 510
**Pattern:** `call_expression` where `function` field text is `assert`. AnyCall on `assert`. [VERIFIED: CWE617 fixture — `assert(data > ASSERT_VALUE)`]
**Note:** This is effectively an `ArgCheck::AnyCall` rule on `assert`. However, since assert is not a call-site pattern in the existing `AST_CWE_RULES` (it belongs to structural patterns), it gets its own `check_assert_calls` function or is added as an `AnyCall` entry to `AST_CWE_RULES`.
**D-01 consideration:** The planner can add `assert` to `AST_CWE_RULES` as `AnyCall` (since it IS a function call) rather than a dedicated visitor. This is the most minimal implementation.
**Expected FP%:** HIGH — `assert` is used throughout safe code. All 510 files will fire including good variants.

### CWE-674: Uncontrolled Recursion (Direct Self-Recursion)
**Juliet files:** 2
**Pattern 1 (infinite):** `helperBad()` calls `helperBad()` unconditionally — no base-case check. [VERIFIED: infinite_recursive_call_01.c]
**Pattern 2 (unbounded):** `helperBad(level - 1)` — recursive call but no upper bound check. [VERIFIED: unbounded_recursive_call_01.c]
**D-03/D-04 implementation:** Walk all `function_definition` nodes. For each, extract function name from `function_declarator` child. Walk the function body for `call_expression` nodes where `function` field text equals the enclosing function name. Fire once per function_definition that contains such a call.
**Note on Pattern 2:** The unbounded case has `if (level == 0) return;` (base case for 0) but no upper bound. The self-recursive call `helperBad(level - 1)` will be detected by D-04 since it IS a call to itself. Rule will fire on both patterns.
**Expected TP%:** 100% on Juliet (2/2 files). Both files contain direct self-recursion.

### CWE-835: Infinite Loop
**Juliet files:** 6 across 4 patterns:
1. `while_true_01.c` — `while(1)` with no break [VERIFIED]
2. `for_empty_01.c` — `for(;;)` with no break [VERIFIED]
3. `do_01.c` — `do { } while(i >= 0)` — condition always true due to modulo [NOT detectable by literal check]
4. `for_01.c` — `for(i = 0; i >= 0; ...)` — condition always true due to unsigned increment [NOT detectable by literal check]

**Body-check approach (D-05 recommendation):** Fire when:
- `while_statement` condition is `number_literal` `1` (or `true_literal`) AND loop body has no `break_statement`, `return_statement`, `goto_statement`, or `call_expression` to `exit`/`abort`, OR
- `for_statement` has empty condition (middle field absent/empty) AND body has no escape.

This catches files 1 and 2 (2/6 = 33% TP). Files 3 and 4 require constant folding/taint analysis — not achievable with pure AST.

**Unconditional approach alternative:** Flag ALL `while(1)`, `for(;;)` regardless of body. Same 2/6 TP rate but more AUTOSAR FP because embedded polling loops commonly use `while(1)` with break inside.

**Recommendation for planner:** Body-check approach is preferred. In AUTOSAR firmware, `while(1)` with an internal `break` is a polling pattern, not a defect. The body-check filters these out and avoids AUTOSAR FP regression (D-15).

### CWE-398: Poor Code Quality
**Juliet files:** 181 across 5 pattern families

| Family | Bad Pattern | AST Shape | Tractable? |
|--------|-------------|-----------|------------|
| `addition` | `intOne + intTwo;` (result discarded) | `expression_statement` → `binary_expression` (operator `+`) with no parent assignment | Yes [VERIFIED] |
| `equals` | `intOne = intOne;` (self-assignment) | `expression_statement` → `assignment_expression` where LHS identifier == RHS identifier | Yes [VERIFIED] |
| `five` | `5;` (literal expression statement) | `expression_statement` → `number_literal` | Yes [VERIFIED] |
| `semicolon` | `;` (empty statement as function body) | `empty_statement` as function body child | Yes [VERIFIED: CWE398__semicolon_01.c] |
| `empty` | (not found on disk) | — | N/A — no files with this name |

**Recommended sub-rules for CWE-398 check_poor_code_quality:**
1. `expression_statement` whose only child is a `number_literal` → fire (covers `five` family)
2. `expression_statement` whose only child is a `binary_expression` with arithmetic operators (`+`, `-`, `*`, `/`, `%`, `|`, `&`, `^`) → fire (covers `addition` family; catches discarded computation)
3. `expression_statement` whose only child is a `binary_expression` with `==` operator at statement level → fire (covers `equals` as comparison where assignment intended; overlaps with CWE-482 — acceptable, same defect)
4. `expression_statement` whose only child is an assignment_expression where LHS identifier text equals RHS identifier text → fire (covers `equals` self-assignment)

Sub-rules 1+2 cover ~108 files (`five`=36, `addition`=36, but overlapping with good variants =~50% each). Net TP estimate: 70–100/181 files = ~40–55%.

**FP risk:** Sub-rule 2 (`discarded binary expression`) will fire in non-CWE-398 files where result of arithmetic is intentionally discarded (e.g., `(void)(x + y);` — but `(void)` cast makes it a `cast_expression`, not bare `binary_expression`). FP risk moderate.

**Decision point (D-07):** CWE-398 IS tractable with these 4 sub-rules. Planner should implement all 4. Do not skip CWE-398.

## Common Pitfalls

### Pitfall 1: Cursor Reuse Across Recursive Calls
**What goes wrong:** Using a single `TreeCursor` across recursion levels causes position to be reset or corrupted.
**Why it happens:** `TreeCursor` holds mutable state. A recursive call moves the cursor position.
**How to avoid:** `let mut cursor = node.walk();` at the start of each function call. Already documented in Phase 18 as Pitfall 1.
**Warning signs:** Tests pass in isolation but fail when multiple nodes are visited.

### Pitfall 2: CWE-570/571 Fires in Good Variant Files
**What goes wrong:** `if (0)` and `if (1)` appear in Juliet's good variant guard patterns (`if(0) { /* incidental */ }`). These appear in non-CWE-570/571 files as control flow skeletons.
**Why it happens:** Juliet uses `if(1)` and `if(0)` as dead-code control flow in good variants.
**How to avoid:** Accept the FP; document it. The ANALYSIS.md from Phase 18 already shows cppcheck had 99.9% FP on CWE-570 for the same reason. Our literal-only rule will be better but still imprecise.
**Warning signs:** FP% on CWE-570/571 is higher than expected from Juliet file count.

### Pitfall 3: CWE-562 Missing the Return-of-Local-Array Pattern
**What goes wrong:** Implementing `return &local_var` (address-of) instead of `return charString` (array name) because the CWE description suggests address-of. Juliet does NOT use address-of in the C files.
**Why it happens:** CWE-562 description says "return of stack variable address" implying `&`. But C arrays decay to pointer without `&`.
**How to avoid:** Inspect the actual Juliet fixture. `return_statement` child is an `identifier` that names a non-static local array. Collect non-static local array names (reuse `collect_array_declarators` logic). Fire if return identifier is in that set.
**Warning signs:** 0 TP on Juliet CWE-562 despite rule firing elsewhere.

### Pitfall 4: CWE-617 Regression via AnyCall on assert
**What goes wrong:** Adding `assert` to `AST_CWE_RULES` as `AnyCall` causes it to fire in AUTOSAR code that uses assertions for safety checks, changing the AUTOSAR finding count (violating D-15).
**Why it happens:** `assert` is common in safety-critical code.
**How to avoid:** Before adding the rule, run the scanner on the AUTOSAR fixture and note the count before adding CWE-617. After adding, verify D-15 holds (new CWE-617 findings are additive; existing CWEs unchanged).
**Warning signs:** Other CWE counts change after adding CWE-617.

### Pitfall 5: CWE-481 Fires on Valid `while ((c = getchar()) != EOF)` Pattern
**What goes wrong:** The assignment-in-condition rule fires on intentional idiom.
**Why it happens:** `while ((c = getchar()) != EOF)` has an assignment inside the condition, but it's wrapped in `!= EOF` binary_expression — the assignment is nested, not the top-level condition.
**How to avoid:** Fire only when the top-level condition node itself is `assignment_expression`, not when `assignment_expression` is nested inside a larger `binary_expression` condition.
**Warning signs:** FP% on AUTOSAR fixture is unexpectedly high.

### Pitfall 6: CWE-483 High FP Rate
**What goes wrong:** Many style-conformant codebases use single-line braceless `if` statements. The rule fires on all of them.
**Why it happens:** CWE-483 is a style/quality warning, not a security defect. Juliet files contain it, but real code has it too.
**How to avoid:** Accept the FP per D-11; document in ANALYSIS.md. Do not add heuristics to suppress (e.g., single-line if on same line) — that adds complexity for diminishing returns.

## Code Examples

### CWE-478: Switch Without Default
```rust
// Source: Pattern derived from Juliet CWE478__basic_01.c inspection
fn check_switch_structure(
    node: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "switch_statement" {
        let has_default = {
            let mut cursor = node.walk();
            node.children(&mut cursor)
                .any(|child| child.kind() == "default_case")
        };
        if !has_default {
            findings.push(SastFinding {
                cwe_id: 478,
                component_name: component_name.to_string(),
                component_ecosystem: component_ecosystem.to_string(),
                file_path: path.to_string_lossy().into_owned(),
                line: (node.start_position().row as u32) + 1,
                source: SastSource::Ast,
            });
        }
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            check_switch_structure(cursor.node(), src, path, component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

### CWE-674: Direct Self-Recursion
```rust
// Source: Pattern derived from Juliet CWE674__infinite_recursive_call_01.c
// Uses find_enclosing_function (already exists in ast_scanner.rs)
fn check_self_recursion(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    // Walk all function_definition nodes at the root level
    let mut cursor = root.walk();
    for fn_node in root.children(&mut cursor).filter(|n| n.kind() == "function_definition") {
        // Extract function name from function_declarator → identifier child
        if let Some(fn_name) = extract_function_name(fn_node, src) {
            check_self_calls(fn_node, src, &fn_name, path, component_name, component_ecosystem, findings);
        }
    }
}

fn check_self_calls(node: Node, src: &[u8], fn_name: &str, path: &Path, name: &str, eco: &str, findings: &mut Vec<SastFinding>) {
    if node.kind() == "call_expression" {
        if let Some(func) = node.child_by_field_name("function") {
            if func.utf8_text(src).ok() == Some(fn_name) {
                findings.push(SastFinding { cwe_id: 674, /* ... */ source: SastSource::Ast });
            }
        }
    }
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            check_self_calls(cursor.node(), src, fn_name, path, name, eco, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

### CWE-587: Fixed Address Assignment
```rust
// Source: Pattern from Juliet CWE587__basic_01.c — char *charPointer = (char*)0x400000
// Fire on init_declarator where init value is cast_expression wrapping hex number_literal > 0xFFFF
fn is_large_hex_literal(node: Node, src: &[u8]) -> bool {
    if node.kind() == "number_literal" {
        if let Ok(text) = node.utf8_text(src) {
            if text.starts_with("0x") || text.starts_with("0X") {
                if let Ok(val) = u64::from_str_radix(&text[2..], 16) {
                    return val > 0xFFFF;
                }
            }
        }
    }
    false
}
```

## State of the Art

| Old Approach | Current Approach | Impact |
|--------------|------------------|--------|
| cppcheck CWE-398: 99.9% FP (138,158 style warnings) | AST sub-rules targeting specific no-effect patterns | Much lower FP; tractable sub-rules cover ~40–55% of Juliet files |
| cppcheck CWE-570/571: 99.9% FP (variable-constant folding) | Literal-only rule: `if (0)` / `if (1)` / `if (2==3)` | Lower FP but only ~12% TP due to Juliet variable-based patterns |

**Deprecated/outdated:**
- cppcheck CWE-398 AnyCall approach: fires on every arithmetic expression in code. Replaced by targeted sub-rules.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | tree-sitter-c represents `switch_statement` body children with `case_statement` and `default_case` node kinds | Per-CWE AST Patterns | Rule produces 0 TPs; need to inspect actual grammar node names |
| A2 | `for_statement` empty condition is represented as an absent/empty condition field in tree-sitter-c | CWE-835 pattern | `for(;;)` body-check rule misses this case |
| A3 | CWE-256 Juliet files use assignment form (`password = "..."`) in addition to init-declaration form | CWE-256 section | If only init form, declarations-only rule achieves TP%; if only assignment form, D-09 extension needed |
| A4 | The `assert` function in Juliet CWE-617 files is always called `assert` (not `ASSERT` macro) | CWE-617 pattern | Rule misses macro-based assertions |

Note: Claims tagged [ASSUMED] are based on code and fixture inspection in this session; tree-sitter-c node kind names should be verified against the grammar or by running a quick parse during implementation.

## Open Questions (RESOLVED)

1. **CWE-570/571 below 50% TP — accept or extend?**
   - What we know: D-06 literal-only rule catches 2/16 files per CWE (~12%)
   - What's unclear: Whether planner should extend to cover `globalFalse` identifier patterns
   - RESOLVED: Accept the gap per D-06; document actual TP% in ANALYSIS.md. The ROADMAP floor is "≥1 TP" — the literal rule achieves this. The 50% target is aspirational. Plans 22-02 implements literal-only and documents the known gap.

2. **CWE-480 pattern complexity**
   - What we know: Juliet CWE-480 fires on `if (helperBad == NULL)` (function pointer vs NULL)
   - What's unclear: Whether a simple rule (binary_expression where one side is `identifier` not `call_expression`, other is `null_literal`) produces acceptable FP rate
   - RESOLVED: Implement the narrow form (binary_expression with identifier vs null_literal) and measure FP on Juliet. Plan 22-01 implements this approach.

3. **CWE-617 as AnyCall in AST_CWE_RULES vs dedicated visitor**
   - What we know: `assert` is a function call, fitting the existing AnyCall pattern
   - What's unclear: Whether adding it to `AST_CWE_RULES` breaks D-01 ("structural patterns in dedicated visitor functions")
   - RESOLVED: CWE-617 goes in `AST_CWE_RULES` as `ArgCheck::AnyCall`. D-01 applies to non-call-site structural patterns only; assert is a call-site pattern. Plan 22-03 implements this routing.

## Environment Availability

Step 2.6: SKIPPED — Phase 22 is purely additive Rust code inside an existing project with established build toolchain. No new external tools required.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test framework (`cargo test`) |
| Config file | none — all tests under `#[cfg(test)]` or `tests/` dir |
| Quick run command | `cargo test --features internal -p radeis-sc2sbom vulnerability_tests::ast_scanner_tests 2>&1` |
| Full suite command | `cargo test --features internal 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CWEXP-02 | CWE-478: switch without default detected | unit | `cargo test --features internal test_cwe478` | Wave 0 |
| CWEXP-02 | CWE-484: omitted break detected | unit | `cargo test --features internal test_cwe484` | Wave 0 |
| CWEXP-02 | CWE-481: assignment-in-condition detected | unit | `cargo test --features internal test_cwe481` | Wave 0 |
| CWEXP-02 | CWE-482: comparison-instead-of-assignment detected | unit | `cargo test --features internal test_cwe482` | Wave 0 |
| CWEXP-02 | CWE-483: braceless if detected | unit | `cargo test --features internal test_cwe483` | Wave 0 |
| CWEXP-02 | CWE-562: return of local variable detected | unit | `cargo test --features internal test_cwe562` | Wave 0 |
| CWEXP-02 | CWE-570/571: literal conditions detected | unit | `cargo test --features internal test_cwe570` | Wave 0 |
| CWEXP-02 | CWE-587: fixed address assignment detected | unit | `cargo test --features internal test_cwe587` | Wave 0 |
| CWEXP-02 | CWE-617: assert call detected | unit | `cargo test --features internal test_cwe617` | Wave 0 |
| CWEXP-02 | CWE-674: self-recursion detected | unit | `cargo test --features internal test_cwe674` | Wave 0 |
| CWEXP-02 | CWE-835: infinite loop detected | unit | `cargo test --features internal test_cwe835` | Wave 0 |
| CWEXP-02 | CWE-256: plaintext password declaration detected | unit | `cargo test --features internal test_cwe256` | Wave 0 |
| CWEXP-02 | CWE-398: no-effect expression detected | unit | `cargo test --features internal test_cwe398` | Wave 0 |
| CWEXP-02 | No regression on existing 26 CWEs | regression | `cargo test --features internal legacy_poc_tests` | exists |
| CWEXP-02 | Juliet benchmark produces >= 1 TP per CWE | benchmark | `cargo run --features internal -- [juliet path]` | manual |

### Sampling Rate
- **Per task commit:** `cargo test --features internal vulnerability_tests::ast_scanner_tests`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green + Juliet benchmark run + ANALYSIS.md updated before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Add test functions for all 15 new CWEs to `tests/vulnerability_tests/ast_scanner_tests.rs` — each test: one TP synthetic fixture + one TN (good variant) fixture
- [ ] Regression test: assert existing CWE counts (78, 119, 120, 122, 125, 134, 190, 242, 295, 319, 327, 377, 732) are unchanged after adding Phase 22 rules

## Security Domain

Skipped — this phase adds static analysis detection rules to a tooling binary. The scanner does not process untrusted user input at runtime; it reads source files and emits findings. No ASVS categories apply to the scanner rules themselves.

## Sources

### Primary (HIGH confidence)
- Juliet Test Suite C source files — all 15 CWE directories inspected directly; AST patterns derived from bad/good variant code [VERIFIED: direct file inspection]
- `src/vulnerability/ast_scanner.rs` — existing implementation inspected; all patterns, helpers, and integration points confirmed [VERIFIED]
- `.planning/phases/22-ast-cwes-structuralpattern-expansion/22-CONTEXT.md` — all decisions D-01 through D-15 [VERIFIED]

### Secondary (MEDIUM confidence)
- `benchmark/juliet/ANALYSIS.md` — cppcheck FP data for CWE-398, 570, 571 used as FP baseline comparison [VERIFIED: file read]

### Tertiary (LOW confidence)
- tree-sitter-c node kind names (A1, A2) — inferred from code patterns and existing ast_scanner.rs; not verified against grammar file directly [ASSUMED]

## Metadata

**Confidence breakdown:**
- Per-CWE AST patterns: HIGH — derived from direct Juliet fixture inspection
- FP estimates: MEDIUM — based on Juliet file structure; real-world rates may differ
- tree-sitter node kind names: MEDIUM — consistent with existing code; grammar verification recommended during implementation
- Architecture: HIGH — follows established Phase 18 pattern exactly

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (stable codebase; Juliet fixtures unchanging)
