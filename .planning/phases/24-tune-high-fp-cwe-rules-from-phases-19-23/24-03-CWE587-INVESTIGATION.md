# CWE-587 Phase 24 D-18 Investigation

## Current threshold

`is_large_hex_literal()` at src/vulnerability/ast_scanner.rs line 1072: `val > 0xFFFF`

## D-18 directive

Raise threshold to > 0xFFFF (65535).

## Outcome

- [x] Threshold was already > 0xFFFF — D-18 is a no-op. (No code change applied.)
- [ ] Threshold needed to change from <prior> to > 0xFFFF — code change applied at line <N>.

The `is_large_hex_literal` function at line 1066–1078 already implements `val > 0xFFFF` exactly as D-18 requires. The function:
1. Confirms the node is a `number_literal` or `integer_literal`
2. Strips integer suffixes (u, U, l, L)
3. Requires the literal to be hex (`0x` or `0X` prefix)
4. Parses the hex value and returns `val > 0xFFFF`

No code change was applied.

## Root cause of 73.9% FP (hypothesis)

Based on code inspection (no oracle re-run was performed in this plan — that is Plan 04's responsibility):

- **Non-hex literals not caught.** `is_large_hex_literal` explicitly requires a `0x`/`0X` prefix. Decimal large integers (e.g., `int *p = (int*)4194304;`) and octal literals assigned to pointers are not caught — this is intentional (the D-18 directive scoped the fix to hex literals only).
- **Cast expression shape mismatch (most likely root cause).** The visitor fires only when the assignment RHS is a `cast_expression` wrapping a large hex literal. In Juliet CWE-587 fixtures, the assignment may use patterns like `(int *)LARGE_HEX` where `LARGE_HEX` is a `#define`-expanded identifier, not a literal node. The tree-sitter AST sees an `identifier` node inside the cast, not a `number_literal`, so `is_large_hex_literal` returns false. This would suppress the TP, but the 73.9% FP rate suggests the rule is firing too broadly rather than too narrowly — meaning the fixture analysis in RESEARCH.md's D-18 entry should be re-examined.
- **Re-examination of FP rate attribution.** The 73.9% FP figure comes from benchmark/juliet/ANALYSIS.md. Since the threshold is already correct, the remaining FPs may be due to false-positive fixtures where benign embedded hardware register assignments (e.g., peripheral base addresses in AUTOSAR BSP code) legitimately contain large hex literals in cast expressions. These represent correct CWE-587 detections from the rule's perspective, but are false positives in the Juliet oracle context because Juliet's "bad" fixtures for CWE-587 are coded differently.

## Recommendation for Plan 04 ANALYSIS.md

- No code change was applied in this plan. CWE-587 becomes a **human-review item** in Phase 24 Notes per D-24.
- The existing TP test (`test_cwe587_fixed_hex_address`) and TN test (`test_cwe587_null_cast_no_finding`) both pass, confirming the threshold logic is correct.
- Plan 04 should re-examine the Juliet CWE-587 FP rate with focus on whether oracle mismatch (fixture structure vs. detection pattern) is the root cause rather than an incorrect threshold.

## Test evidence

Current CWE-587 test results:

```
running 2 tests
test vulnerability_tests::ast_scanner_tests::test_cwe587_null_cast_no_finding ... ok
test vulnerability_tests::ast_scanner_tests::test_cwe587_fixed_hex_address ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 445 filtered out; finished in 0.03s
```

Both tests pass. No code change applied — D-18 was already satisfied before this investigation.
