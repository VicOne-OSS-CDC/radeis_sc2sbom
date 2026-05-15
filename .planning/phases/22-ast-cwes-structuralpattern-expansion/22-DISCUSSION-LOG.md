# Phase 22: ast-cwes-structuralPattern-expansion — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 22-ast-cwes-structuralPattern-expansion
**Areas discussed:** New rule shapes needed, FP threshold realism, Juliet coverage gaps, CWE-256 approach

---

## New Rule Shapes Needed

### Q1: Approach for structural patterns

| Option | Description | Selected |
|--------|-------------|----------|
| New visitor passes | Separate functions (check_switch_structure, etc.) called from apply_ast_rules — no ArgCheck extension needed | ✓ |
| Extend ArgCheck enum | Add new ArgCheck variants and fold structural checks into existing dispatch | |
| You decide | Planner chooses per CWE | |

**User's choice:** New visitor passes  
**Notes:** Keep structural patterns separate from call-site ArgCheck dispatch. All new functions stay in ast_scanner.rs.

---

### Q2: Location of new visitor functions

| Option | Description | Selected |
|--------|-------------|----------|
| Same ast_scanner.rs file | Private functions added to existing file, called from apply_ast_rules | ✓ |
| Separate structural_patterns.rs module | New file with public check fn, imported into ast_scanner.rs | |

**User's choice:** Same ast_scanner.rs file

---

### Q3: CWE-835 Infinite Loop detection scope

| Option | Description | Selected |
|--------|-------------|----------|
| Flag unconditionally | Fire on while(1)/for(;;)/do-while(true) regardless of body content | |
| Flag only if no break/return/goto inside | Walk loop body — reduces FP for embedded polling loops | |
| You decide | Planner assesses Juliet CWE-835 fixture | ✓ |

**User's choice:** You decide  
**Notes:** AUTOSAR/embedded code legitimately uses polling loops; planner should pick the approach that best balances FP on the Juliet fixture.

---

### Q4: CWE-674 Uncontrolled Recursion scope

| Option | Description | Selected |
|--------|-------------|----------|
| Direct self-recursion only | Function calls itself — pure intra-file AST check | ✓ |
| Direct + one-level mutual | A→B where B→A in same file — requires two-pass | |

**User's choice:** Direct self-recursion only

---

## FP Threshold Realism

### Q1: ≤40% FP threshold policy

| Option | Description | Selected |
|--------|-------------|----------|
| ≤40% for all, with best-effort | Keep 40% as goal; document-and-ship if exceeded | ✓ |
| Strict ≤40% — drop CWEs that can't meet it | Remove rules that exceed threshold | |
| Relax to ≤60% for structural-only CWEs | Higher allowance for noise-prone patterns | |

**User's choice:** ≤40% for all, with best-effort

---

### Q2: Behavior when a CWE rule exceeds 40% FP

| Option | Description | Selected |
|--------|-------------|----------|
| Document and ship anyway | Record FP% in ANALYSIS.md, ship the rule | ✓ |
| Gate behind a flag | Off by default; opt-in via --noisy-rules | |
| Drop from Phase 22 | Remove rule, reduce phase scope | |

**User's choice:** Document and ship anyway  
**Notes:** Consistent with how CWE-120 (89% FP) and CWE-126 (95% FP) were handled in Phase 18.

---

### Q3: CWE-398 detection approach

| Option | Description | Selected |
|--------|-------------|----------|
| Empty exception handler pattern | Fire on empty catch/error-handling blocks | |
| Unused variable / dead assignment | Requires intra-function liveness — complex | |
| You decide / skip CWE-398 | Planner inspects Juliet CWE-398 fixture; skip if nothing tractable | ✓ |

**User's choice:** You decide / skip CWE-398

---

## Juliet Coverage Gaps

### Q1: TP success threshold per CWE

| Option | Description | Selected |
|--------|-------------|----------|
| ≥1 TP per CWE | Matches ROADMAP as written | |
| Minimum 5 TPs per CWE | Stricter absolute count | |
| TP% ≥50% of available Juliet cases | Relative threshold based on fixture size | ✓ |

**User's choice:** TP% ≥50% of available Juliet cases

---

### Q2: Synthetic fixture strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Juliet-only for Phase 22 | All 15 CWEs have Juliet directories — only add synthetics if a directory is empty | ✓ |
| Add synthetic fixtures for all 15 | One TP + FP-guard per CWE under tests/fixtures/ | |
| Synthetic fixtures only for CWEs with <5 Juliet cases | Targeted supplement | |

**User's choice:** Juliet-only for Phase 22

---

## CWE-256 Approach

### Q1: Detection mechanism

| Option | Description | Selected |
|--------|-------------|----------|
| Identifier name heuristic | Variable name contains password/passwd/pwd/secret + initializer is string_literal | ✓ |
| String literal content scan | Entropy heuristic or pattern on the string value — high FP | |
| Skip CWE-256 in Phase 22 | Defer to domain-specific phase | |

**User's choice:** Identifier name heuristic

---

### Q2: Scope of identifier heuristic

| Option | Description | Selected |
|--------|-------------|----------|
| Declarations only | fire on declaration nodes with string_literal initializer | |
| Declarations + direct assignments | Also fire on assignment_expression nodes | |
| You decide | Planner assesses Juliet CWE-256 to pick narrowest rule achieving TP% ≥50% | ✓ |

**User's choice:** You decide  
**Notes:** Start with declarations; extend to assignments if Juliet CWE-256 fixture requires it.

---

## Claude's Discretion

- **CWE-835:** Planner picks unconditional flag vs. loop-body check based on Juliet CWE-835 fixture and embedded-code FP impact.
- **CWE-398:** Planner inspects Juliet CWE-398 to find narrowest tractable pattern, or skips if none exists.
- **CWE-256 scope:** Planner extends from declarations-only to assignments if needed to achieve TP% ≥50% on Juliet.

## Deferred Ideas

- Mutual recursion detection (CWE-674) — cross-function call graph analysis beyond Phase 22 scope
- CWE-570/571 with constant propagation (variable-folding) — requires simple intra-function constant folding, not pure AST shape
- Tighter CWE-835 loop escape analysis — if unconditional flag ships, a body-check improvement can follow in a later phase
