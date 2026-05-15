# Phase 24: tune-high-fp-cwe-rules-from-phases-19-23 — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-13
**Phase:** 24-tune-high-fp-cwe-rules-from-phases-19-23
**Areas discussed:** Scope, Fix Strategy, Regression & Validation, ANALYSIS.md Policy, Fallback Strategy

---

## Scope: Which CWEs to Fix

| Option | Description | Selected |
|--------|-------------|----------|
| Fix everything >35% FP | Attempt a fix for all 17. Document and leave if can't reach <35%. | ✓ |
| Skip 'accepted by design' CWEs | Skip CWE-570/571/338/426/676/680 — explicitly accepted in prior phases. | |
| User-defined threshold | Custom threshold (e.g., only >50% FP or only obvious fixes). | |

**User's choice:** Fix everything >35% FP — maximize coverage improvement.

**Follow-up — scope of phases:**

| Option | Description | Selected |
|--------|-------------|----------|
| Include all CWEs from phases 19–23 | All 17 targets (from ROADMAP Phase 24 list) in scope. | ✓ |
| Phases 19–21 targets only | Strict reading of "phases 19-21" in ROADMAP title. | |

**Notes:** User confirmed all 17 CWEs from phases 19–23 are in scope.

---

## Fix Strategy: AnyCall / Function List Restrictions

### CWE-338/676/680 approach

| Option | Description | Selected |
|--------|-------------|----------|
| Restrict function list | Narrow to most dangerous variants. | Discussed (per-CWE) |
| Add ArgAtIndex / context guard | Promote to ArgAtIndex or add co-occurrence guard. | Discussed |
| Accept high FP — no change | Leave as-is, document. | Not selected |

**User's choice:** Hybrid — tradeoff analysis requested before deciding.

**Tradeoff analysis provided:**
- CWE-338: No approach preserves rand() TPs — removing rand/srand to keep only drand48/lrand48/random/mrand48 drops Juliet TPs to 0. User accepted 0 TPs.
- CWE-676: Drop alloca (already CWE-121), keep strtok. 0 Juliet TPs regardless (corpus mismatch).
- CWE-680: Multiplication-expression size-arg guard is clearly better — targets overflow-prone pattern specifically.

**Decision:** CWE-338: restrict to drand48/lrand48/random/mrand48 (0 TPs acceptable). CWE-676: drop alloca, keep strtok. CWE-680: mul-expr size-arg guard.

### Initial "Accept" decisions overridden by user

User requested all four initially-accepted CWEs be flipped to Fix:
- CWE-338 → Fix (0 TPs acceptable)
- CWE-676 → Fix (drop alloca)
- CWE-535 → Fix (combine with CWE-134 non-literal-fmt guard)
- CWE-126 → Fix (attempt FixedSizeBuffer context guard)

### CWE-256 removal decision

| Option | Description | Selected |
|--------|-------------|----------|
| Accept 0 Juliet TPs — keep unit-test validation | Corpus mismatch; keep rule; unit tests confirm TP. | |
| Remove CWE-256 entirely | 100% FP, 0 Juliet TPs — cleaner to remove. | ✓ |

**User's choice:** Remove CWE-256 — 48 CWEs with better precision beats 49 with noise.

### CWE-571/570 loop-context restriction

| Option | Description | Selected |
|--------|-------------|----------|
| Remove CWE-571 entirely | CWE-835 subsumes infinite loop case. | |
| Restrict to non-loop contexts (if/ternary) | Remove while(1)/for(;;) — covered by CWE-835. | ✓ |

**User's choice:** Restrict both CWE-570 and CWE-571 to if-condition only. Loop patterns removed (CWE-835 covers them).

### CWE-426 function list change

| Option | Description | Selected |
|--------|-------------|----------|
| Switch to dlopen/LoadLibraryExA/W | Canonical CWE-426 functions — removes oracle mismatch FPs. | ✓ |
| Accept as-is — oracle mismatch inherent | Leave popen/system even though oracle maps them to CWE-78. | |

**User's choice:** Switch to dlopen/LoadLibraryExA/W.

### CWE-126 fix approach

| Option | Description | Selected |
|--------|-------------|----------|
| Change to FixedSizeBuffer | Same as CWE-119/120/125 — fires when dest is char buf[N]. | |
| ArgAtIndex + FixedSizeBuffer combined | Fire only when dest is fixed-size AND size arg is not sizeof(). | ✓ |

**User's choice:** More aggressive: new `ArgCheck::FixedSizeBufferWithoutSizeArg(size_arg_index)` variant.

---

## Structural Fix Specifics

### CWE-480 (func-ptr null compare)

| Option | Description | Selected |
|--------|-------------|----------|
| Walk enclosing function declarations, check for () in type | Heuristic: function pointer if type contains '('. | ✓ |
| Identifier name heuristics (_fn, _cb, _handler) | Simpler but misses generic names. | |

### CWE-483 (braceless if)

| Option | Description | Selected |
|--------|-------------|----------|
| Don't fire on return/break/continue body | Common safe single-statement patterns. | ✓ |
| Indentation-level heuristic | Whitespace not in tree-sitter AST — not feasible. | |

### CWE-562 (return of stack var address)

| Option | Description | Selected |
|--------|-------------|----------|
| Fire only when local var is a plain scalar (not array/struct) | Matches Juliet bad-sink; arrays/structs by reference are common. | ✓ |
| Fire only when return type is a pointer type | Less precise; many false negatives with void*. | |

---

## Regression & Validation Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Re-run Juliet oracle after all fixes (single pass) | Efficient; one oracle run at the end. | ✓ |
| Re-run Juliet after each CWE fix (incremental) | Catches regressions earlier but slower. | |

**CWEs with 0 Juliet TPs after tuning:** Validate via synthetic unit tests (inline C string in `#[cfg(test)]`). Pattern consistent with CWE-121/369/426/676 from prior phases.

**AUTOSAR check:** Run after human verification of Juliet oracle delta. Sequence: implement → Juliet oracle → human review → AUTOSAR regression.

---

## ANALYSIS.md Update Policy

| Option | Description | Selected |
|--------|-------------|----------|
| In-place row edits + Phase 24 changelog section | Update FP% column per CWE + Phase 24 Notes section. | |
| Full re-run: regenerate all rows | oracle.sh fresh run; all rows regenerated. | ✓ |

**Follow-up:** Also add `## Phase 24 Notes` section documenting changes and rationale. **Selected: Yes.**

---

## Fallback Strategy

**User's decision:** If a fix attempt still leaves FP% >35%, create a human-review item in the Phase 24 verification checklist with before/after FP% documented. Decision to remove, demote, or accept is made after reviewing the re-run results together — not pre-decided.

---

## Phase 24 Success Criteria

**User's decision:** All 17 fix attempts completed, Juliet oracle re-run, ANALYSIS.md updated with full regenerated table + Phase 24 Notes, human reviews Juliet delta, AUTOSAR regression confirms no regressions.

**Coverage after Phase 24:** 48 CWEs (49 minus CWE-256 removed). User confirmed 48 with better precision is better than 49 with noise.

---

## Claude's Discretion

- CWE-126 `strcat` (2-arg, no size arg): whether to use one `AstCweRule` entry with two modes or two separate entries. Planner decides.
- CWE-680 guard implementation: new `ArgCheck::SizeArgIsMultiplication(u8)` variant vs. inline logic. Planner decides.
- CWE-570/571 loop-context exclusion: exact tree-sitter node types to check for loop context. Planner decides.

## Deferred Ideas

- CWE-338 context-aware detection (rand() in security-sensitive context only) — requires dataflow. Deferred.
- CWE-256 replacement rule using a different detection approach. Deferred — remove first, reconsider in future phase if coverage gap matters.
