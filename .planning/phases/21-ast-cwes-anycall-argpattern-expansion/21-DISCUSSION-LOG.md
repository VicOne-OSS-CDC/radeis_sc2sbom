# Phase 21: ast-cwes-anycall-argpattern-expansion — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 21-ast-cwes-anycall-argpattern-expansion
**Areas discussed:** CWE-369 re-evaluation, ArgCheck variant gaps, Function lists & precision, Test coverage strategy

---

## CWE-369 Re-evaluation

### Phase 20 log check

| Option | Description | Selected |
|--------|-------------|----------|
| Re-check Phase 20 discussion log | User requested to re-check Phase 20 log before deciding | ✓ |

**Notes:** User asked to re-check Phase 20 discussion log before answering. Phase 20 CONTEXT.md confirmed: `ContainsTokens` deleted in Phase 20; Phase 21 uses `ArgAtIndex` from the start for any arg-value rules.

### CWE-369 AST gate vs lexical-only

| Option | Description | Selected |
|--------|-------------|----------|
| Keep lexical-only, drop from Phase 21 | CWE-369 stays out of AST_CWE_RULES; lexical literal-zero gate already handles it | |
| Add AST rule with literal-zero gate | New ArgCheck::LiteralZeroAtDivisor variant for binary_expression nodes | |
| Drop from Phase 21 scope | Remove from target list; lexical-only coverage accepted | |

**User's choice:** User asked follow-up: "Which one has higher precision?"

### AST vs lexical precision comparison

**Claude's analysis:** AST binary-expression gate has higher precision (won't fire on comments, float literals, or parenthesized expressions). However, requires new binary_expression traversal infrastructure not in AST_CWE_RULES table.

### Infrastructure reuse for Phase 22

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add binary-expression infrastructure in Phase 21 | apply_division_rules() sets pattern for Phase 22 CWE-480/481/482 | ✓ |
| No — CWE-369 stays lexical, infrastructure in Phase 22 | Phase 22 adds traversal when needed for 3 CWEs | |

**User's choice:** Yes — add binary-expression infrastructure in Phase 21
**Notes:** User recognized that the infrastructure investment spreads across 4 CWEs total (CWE-369 in Phase 21; CWE-480/481/482 in Phase 22).

---

## ArgCheck Variant Gaps

### CWE-467 sizeof approach

| Option | Description | Selected |
|--------|-------------|----------|
| New ArgCheck::SizeofPointer variant (Recommended) | Walks sizeof_expression nodes, checks if arg is pointer type | ✓ |
| AnyCall approximation on sizeof | Treat sizeof as function call if tree-sitter exposes it as call_expression | |
| Skip CWE-467 in Phase 21 | Defer to Phase 22/23 | |

**User's choice:** New ArgCheck::SizeofPointer variant (Recommended)

### CWE-327/328 overlap handling

| Option | Description | Selected |
|--------|-------------|----------|
| CWE-328 rule with non-overlapping functions only (Recommended) | MD5/SHA1 stay under CWE-327; CWE-328 adds MD4, RC4, etc. | ✓ |
| CWE-328 with full function list, dedup handles it | Both CWE-327 and CWE-328 findings appear for same call | |

**User's choice:** CWE-328 rule with non-overlapping functions only (Recommended)

---

## Function Lists & Precision

### CWE-676 broad vs tight

| Option | Description | Selected |
|--------|-------------|----------|
| Non-overlapping list only (Recommended) | alloca, strtok, getenv-like; no duplicates with other CWEs | ✓ |
| Full CERT C dangerous-function list | Multiple CWEs per call site, noisy | |
| You decide | Leave to planner | |

**User's choice:** Non-overlapping list only (Recommended)
**Notes:** User selected this before benchmark data was surfaced. Benchmark data later confirmed 100% FP with broad list, validating this choice.

### CWE-780 and CWE-605 approach

| Option | Description | Selected |
|--------|-------------|----------|
| CWE-780: ArgAtIndex on padding param (Recommended) | ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"]) | ✓ |
| CWE-780: AnyCall on RSA functions | Any RSA_public_encrypt flagged regardless of padding | |
| CWE-605: defer to Phase 22/23 | Requires fd tracking across calls | |

**User's choice:** CWE-780: ArgAtIndex on padding param (Recommended)

### CWE-605 final decision

| Option | Description | Selected |
|--------|-------------|----------|
| Defer CWE-605 to Phase 22/23 (Recommended) | Not a pure AnyCall/ArgPattern rule; needs cross-call tracking | ✓ |
| AnyCall on bind() as approximation | File-level count heuristic; high FP rate | |

**User's choice:** Defer CWE-605 to Phase 22/23 (Recommended)

### Remaining AnyCall CWEs (121, 126, 338, 426, 526, 535)

| Option | Description | Selected |
|--------|-------------|----------|
| Leave to researcher/planner (Recommended) | Canonical function lists from CERT C; no specific inclusions/exclusions | ✓ |
| I have specific calls to add | User specifies particular function names | |

**User's choice:** Leave to researcher/planner (Recommended)

---

## Test Coverage Strategy

### Primary test pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Inline #[test] in ast_scanner.rs | Phase 18 pattern; fast, self-contained | |
| Synthetic fixture files (tests/fixtures/) | Phase 20 pattern; better for multi-line C context | |
| Mixed: fixtures for complex rules, inline for AnyCall | Structurally complex rules get fixtures | |

**User's choice (freeform):** "We should use ground truth inside of the Juliet testsuite (manifest.xml) to test our implementation."

### Juliet coverage strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Juliet where available, synthetic fixtures as fallback | Research confirms per-CWE Juliet coverage | |
| All Juliet — researcher to verify coverage | Assume Juliet covers all; fail loudly if not | ✓ |
| All synthetic fixtures | Skip Juliet for Phase 21 | |

**User's choice:** All Juliet — researcher to verify coverage

### benchmark.rs deleted — where do tests live?

| Option | Description | Selected |
|--------|-------------|----------|
| New tests/ast_regression.rs integration test file | Dedicated file, graceful skip, #[cfg(feature = "internal")] | |
| Inline in ast_scanner.rs test module | All scanner tests in one file | |
| You decide | Leave to planner | ✓ |

**User's choice:** You decide

### Juliet manifest.xml integration details

User asked about manifest.xml integration details. Claude surfaced existing `benchmark/juliet/ANALYSIS.md` with real data from the Phase 18 benchmark run.

**Key benchmark findings surfaced for Phase 21:**
- CWE-676: 100% FP (Phase 19 recommendation: DROP or tighten)
- CWE-126: 94.8% FP (Phase 19 recommendation: TUNE)
- CWE-369: 0 AST TPs currently (cppcheck was 0% FP)
- CWE-467: 0 AST TPs (cppcheck 54 TPs at 71% FP)

### Re-run strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Re-run after implementation, update ANALYSIS.md | Implement first, then validate; matches ROADMAP success criterion #4 | ✓ |
| Use existing data to pre-screen rules before implementing | Drop CWE-676 now based on 100% FP data | |

**User's choice:** Re-run after implementation, update ANALYSIS.md

### CWE-676 given 100% FP data

| Option | Description | Selected |
|--------|-------------|----------|
| Tighter function list — non-overlapping functions only | alloca, strtok, getenv-like; verify FP% after implementation | ✓ |
| Drop CWE-676 from Phase 21 entirely | 100% FP too high a starting point | |

**User's choice:** Tighter function list — non-overlapping functions only

### CWE-126 given 94.8% FP data

| Option | Description | Selected |
|--------|-------------|----------|
| FixedSizeBuffer for CWE-126 (Recommended) | Mirrors CWE-119/120 pattern; targets FP% ≤35% | ✓ |
| AnyCall on CWE-126, accept high FP | Ships but fails ROADMAP FP gate | |

**User's choice:** FixedSizeBuffer for CWE-126 (Recommended)

---

## Claude's Discretion

- **Exact function lists for AnyCall CWEs (121, 338, 426, 526, 535, 680, CWE-676 tight list):** Left to researcher/planner
- **`apply_division_rules()` exact implementation:** Planner decides function vs. inline block; designs for Phase 22 reuse
- **Test file location:** `tests/ast_regression.rs` suggested; planner has discretion

## Deferred Ideas

- **CWE-605** — requires cross-call socket fd tracking; defer to Phase 22/23
- **CWE-676 broad function list** — benchmark confirms 100% FP; Phase 21 ships tight list only
- **CWE-126 further tuning** — if FixedSizeBuffer still produces high FP, buffer-origin check (source arg type) deferred post-benchmark
- **Phase 22 binary_expression CWEs (480, 481, 482)** — intentionally deferred; `apply_division_rules()` in Phase 21 is the shared infrastructure foundation
