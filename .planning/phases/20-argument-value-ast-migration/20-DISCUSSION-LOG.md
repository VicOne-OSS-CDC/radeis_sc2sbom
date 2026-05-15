# Phase 20: argument-value-ast-migration — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 20-argument-value-ast-migration
**Areas discussed:** ArgCheck variant design, CWE-732 precision fix, Lexical scanner parity, CWE-319 rule consolidation, CWE-295 wolfSSL rule, ArgAtIndex arg count guard, Juliet manifest tooling, ContainsTokens retirement

---

## ArgCheck Variant Design

### What does Phase 20 add beyond ContainsTokens?

| Option | Description | Selected |
|--------|-------------|----------|
| Nested expression detection | ContainsTokens misses args buried in binary/cast expressions | |
| Positional arg index | Check only the Nth argument, not all args indiscriminately | |
| Both: index + nested walk | ArgAtIndex(u8, &[&str]): check arg N, walk its nested AST subtree | ✓ |

**User's choice:** Both: index + nested walk

### ArgAtIndex structure

| Option | Description | Selected |
|--------|-------------|----------|
| ArgAtIndex(u8, &'static [&'static str]) | Positional index + token slice, matches ContainsTokens shape | ✓ |
| Separate AstArgRule struct | New struct alongside AST_CWE_RULES | |
| You decide | Leave to planner | |

**User's choice:** `ArgAtIndex(u8, &'static [&'static str])` as enum variant

---

## CWE-732 Precision Fix

### umask rule exact check

| Option | Description | Selected |
|--------|-------------|----------|
| integer_literal == "0" only | Arg 0 is integer_literal AND text is exactly "0" | ✓ |
| Zero-value family ("0", "0x0", "0X0", "0b0") | Match all zero-valued literal representations | |
| Numeric equality via parse | Parse as u64, check == 0 | |

**User's choice:** `integer_literal == "0"` only

### SetSecurityDescriptorDacl positional fix

| Option | Description | Selected |
|--------|-------------|----------|
| ArgAtIndex(2, &["NULL"]) — check pDacl arg | Positionally scoped to arg 2 | ✓ |
| Keep ContainsTokens | NULL anywhere in arg list is sufficient | |
| You decide | Leave to planner | |

**User's choice:** `ArgAtIndex(2, &["NULL"])`

---

## Lexical Scanner Parity

### What happens to arg_value_contains in cwe_scanner.rs?

| Option | Description | Selected |
|--------|-------------|----------|
| Keep in lexical scanner | Parse-fail files still get CWE-295/319/732 detection | |
| Remove from lexical scanner | AST is authoritative; simpler to delete | ✓ |
| You decide | Leave to planner | |

**User's choice:** Remove from lexical scanner

### Validation strategy for CWE-295/319/732

| Option | Description | Selected |
|--------|-------------|----------|
| New unit tests per rule | #[test] with inline C, mirrors test_argval_* pattern | |
| SARIF diff on full Juliet corpus | Run sc2sbom before/after, diff SARIF | |
| Synthetic fixture files | Small .c files under tests/fixtures/ for each CWE | ✓ |

**User's choice:** Synthetic fixture files
**Notes:** User noted that Juliet test suite is the ground-truth source for most CWEs, but CWE-295/319/732 are not present in Juliet (Juliet focuses on buffer overflows/string handling). Synthetic fixtures are the appropriate validation path for API-misuse rules.

---

## CWE-319 Rule Consolidation

### Rule structure after migration

| Option | Description | Selected |
|--------|-------------|----------|
| 3 separate ArgAtIndex rules | Each rule positionally scoped per option constant + value | ✓ |
| 1 merged rule with multi-token check | Single rule fires on any dangerous combination | |
| New ArgAtTwoIndices variant | Dedicated variant for two-arg AND-logic | |

**User's choice:** 3 separate ArgAtIndex rules

### Nested walk strategy for ArgAtIndex

| Option | Description | Selected |
|--------|-------------|----------|
| text-contains with word boundary (Recommended) | Collect all leaf texts in arg subtree, apply token_present_with_boundary | ✓ |
| Exact node-kind check for identifiers/literals | Walk subtree, check identifier/integer_literal nodes exactly | |
| You decide | Leave to planner | |

**User's choice:** text-contains with word boundary, reusing `token_present_with_boundary`

---

## CWE-295 wolfSSL Rule

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — add to AST rules | ArgAtIndex(1, &["SSL_VERIFY_NONE"]) for wolfSSL_CTX_set_verify | ✓ |
| No — lexical fallback covers it | wolfSSL is less common | |
| You decide | Leave to planner | |

**User's choice:** Add `wolfSSL_CTX_set_verify` to AST rules
**Notes:** Was present in lexical scanner (cwe_scanner.rs:93) but missing from AST_CWE_RULES. Closing the gap before lexical arg_value_contains rules are removed.

---

## ArgAtIndex Arg Count Guard

| Option | Description | Selected |
|--------|-------------|----------|
| Skip silently — no finding | Mirror NotStringLiteralAtIndex: if idx >= args.len(), return false | ✓ |
| Fall back to ContainsTokens | Safety net scan across all args | |
| Emit warning and skip | Log warning, then skip | |

**User's choice:** Skip silently — no finding

---

## Juliet Manifest Tooling

**User's question:** Are CWE-295/319/732 present in the Juliet test suite?

**Resolution:** They are not. Juliet primarily covers CWE categories related to buffer overflows, integer overflows, and string handling. CWE-295 (certificate validation), CWE-319 (cleartext transmission), and CWE-732 (permissions) require OpenSSL/curl/POSIX API context that Juliet doesn't provide.

**Decision:** Validation via synthetic fixture files only. No manifest.xml converter needed for Phase 20.

---

## ContainsTokens Retirement

| Option | Description | Selected |
|--------|-------------|----------|
| Keep ContainsTokens for now | Phases 21–23 may reuse it | |
| Delete ContainsTokens in Phase 20 | Clean removal once all uses replaced | ✓ |
| You decide | Leave to planner | |

**User's choice:** Delete `ContainsTokens` in Phase 20. Phases 21–23 should use `ArgAtIndex` from the start for any argument-value rules.

---

## Claude's Discretion

- **CWE-319 AND-logic implementation**: if AND-ing two positional args requires a new enum variant (`ArgAtTwoIndices`), the planner may introduce it. Preference is to avoid it unless needed for FP prevention.
- **Exact implementation of ArgAtIndex nested walk**: text-concatenation approach specified, but the planner decides the exact Rust implementation (collect via `utf8_text` on leaf nodes or on the subtree root).

## Deferred Ideas

- **Juliet CWE-295/319/732 coverage**: if Juliet corpus is extended in the future, Phase 18's manifest→truth converter can be reused.
- **`ArgAtTwoIndices` enum variant**: deferred to Phase 21 if the same two-arg AND pattern arises again.
- **CWE-732 zero-value family (0x0, 0b0)**: exact `"0"` check is sufficient for now; extended zero-value matching deferred.
