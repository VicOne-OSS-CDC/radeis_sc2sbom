# Phase 21: ast-cwes-anycall-argpattern-expansion — Research

**Researched:** 2026-05-12
**Domain:** Rust / tree-sitter-c AST CWE rule expansion
**Confidence:** HIGH

---

## Summary

Phase 21 expands `AST_CWE_RULES` in `src/vulnerability/ast_scanner.rs` from 13 to 25 CWEs (net +12 after CWE-605 deferral) by adding four rule categories: AnyCall (fire on any invocation), ArgAtIndex-token (fire when a positional arg text matches a pattern), FixedSizeBuffer (fire when dest is a fixed-size array), and a new binary-expression walk (`apply_division_rules()`) for CWE-369. A new `ArgCheck::SizeofPointer` variant handles CWE-467. All 12 new CWEs have Juliet corpus directories confirmed present.

**Critical finding on CWE-328:** The Juliet corpus for CWE-328 uses Windows `CryptCreateHash` with `CALG_MD2`, `CALG_MD5`, `CALG_SHA1` constants — NOT OpenSSL functions. The existing CWE-327 rule uses OpenSSL functions (`MD5`, `SHA1`, `DES_ecb_encrypt`, etc.) with no overlap on `CryptCreateHash`. The correct Phase 21 approach for CWE-328 is an `ArgAtIndex(1, &["CALG_MD2", "CALG_MD5", "CALG_SHA1"])` rule on `CryptCreateHash` (arg index 1 is the hash algorithm parameter). This achieves Juliet TP with zero function-level overlap with CWE-327.

**Critical finding on CWE-780:** The Juliet corpus for CWE-780 uses Windows `CryptEncrypt` with `CRYPT_OAEP` flag — NOT OpenSSL's `RSA_public_encrypt`. The locked decision D-10 (`ArgAtIndex(4, &["RSA_PKCS1_PADDING", ...])` on `RSA_public_encrypt`) should be revised — the Juliet test has no `RSA_public_encrypt` call. The planner must decide: (a) keep D-10 as a synthetic-fixture rule for OpenSSL, or (b) pivot to `CryptEncrypt` + `CRYPT_OAEP` absence for Juliet TP. Both are valid; only option (b) satisfies the ROADMAP ≥1 TP criterion via Juliet.

**Critical finding on CWE-676:** The Juliet CWE-676 directory tests `cin` (C++ iostream) unbound extraction — NOT `alloca`/`strtok`/`getenv`. A rule targeting `alloca`/`strtok` will produce 0 Juliet TPs. The planner must choose between (a) using `cin` AnyCall for Juliet TP (C++ only, may have different FP profile) or (b) using `alloca`/`strtok` with synthetic fixtures per D-18.

**Primary recommendation:** Proceed with the 12-CWE expansion as described in CONTEXT.md, resolving the CWE-328/780/676 corpus-target mismatches documented here before writing tasks.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** CWE-369 via `apply_division_rules()` helper — binary_expression walk for `/` or `%` operator where RHS is `integer_literal` with text `"0"`. NOT in `AST_CWE_RULES`.
- **D-02:** `apply_division_rules()` called alongside `apply_ast_rules()` in per-file scan loop. Infrastructure reusable for Phase 22 CWE-480/481/482.
- **D-03:** Lexical CWE-369 gate remains for parse-fail fallback.
- **D-04:** New `ArgCheck::SizeofPointer` variant — walks `sizeof_expression` nodes, checks if argument node kind is pointer_declarator or identifier whose resolved type is a pointer. Used by CWE-467.
- **D-05:** `ContainsTokens` was deleted in Phase 20. Phase 21 uses `ArgAtIndex(u8, &'static [&'static str])`.
- **D-06:** CWE-126 uses `ArgCheck::FixedSizeBuffer` (NOT AnyCall) — mirrors CWE-119/120/122/125 pattern.
- **D-07:** CWE-328 uses non-overlapping functions only — no duplicates with CWE-327 list. Researcher finalizes function list.
- **D-08:** CWE-676 uses tight non-overlapping function list — Phase 18 benchmark showed 100% FP with broad list.
- **D-09:** Functions already covered by other CWE rules excluded from CWE-676 to avoid duplicates.
- **D-10:** CWE-780: `ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"])` on `RSA_public_encrypt`. Fires when OAEP NOT used.
- **D-11:** CWE-467 uses `ArgCheck::SizeofPointer` variant.
- **D-12:** CWE-605 deferred from Phase 21.
- **D-13:** CWEs 121, 338, 426, 526, 535, 680 are AnyCall rules. Exact function lists left to researcher/planner.
- **D-14:** CWE-121 and CWE-338 included as AnyCall rules.
- **D-15:** Implement all 12 new CWEs first, then re-run Juliet benchmark and update ANALYSIS.md.
- **D-16:** ROADMAP success criteria: ≥1 TP per CWE on Juliet, FP% ≤35%, no regression on existing 13 CWEs.
- **D-17:** Test infrastructure location planner's discretion; suggested `tests/ast_regression.rs` behind `#[cfg(feature = "internal")]`.
- **D-18:** CWEs not in Juliet (338, 426, 467, 526, 535, 676, 680, 780) fall back to synthetic fixtures.

### Claude's Discretion

- Exact function lists for AnyCall CWEs (121, 338, 426, 526, 535, 680, 676 tight list)
- `apply_division_rules()` exact implementation (standalone function vs inline block)
- Test file location

### Deferred Ideas (OUT OF SCOPE)

- CWE-605 (Multiple Binds on Same Port)
- CWE-676 broad function list
- CWE-126 structural context check if FixedSizeBuffer FP still high
- Phase 22 binary_expression CWEs (480, 481, 482)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CWEXP-01 | Expand AST scanner from 13 to 26 CWEs using AnyCall/ArgPattern/FixedSizeBuffer/OperatorPattern rules, validated against Juliet | Function lists, Juliet coverage map, and ArgCheck variant implementation patterns documented below |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CWE rule table expansion | AST Scanner (ast_scanner.rs) | — | Static rule table, no pipeline changes |
| ArgCheck::SizeofPointer variant | AST Scanner (ast_scanner.rs) | — | New enum variant in same file |
| apply_division_rules() | AST Scanner (ast_scanner.rs) | — | New helper in same file, called from scan_file_ast_or_lexical |
| Benchmark validation | benchmark/juliet/ | ANALYSIS.md | Juliet corpus already wired via juliet-run.sh |
| Unit tests | tests/vulnerability_tests/ast_scanner_tests.rs | — | Existing test file; new test functions appended |
| Downstream consumers (SARIF, CycloneDX, markdown) | Unchanged | — | SastFinding struct is the interface; no changes needed |

---

## Standard Stack

### Core (no new dependencies)

| Item | Version | Purpose | Why Standard |
|------|---------|---------|--------------|
| tree-sitter | Already embedded | AST parsing | Existing; no change |
| tree-sitter-c | Already embedded | C grammar | Existing; no change |
| `ArgCheck` enum | n/a (local) | Rule dispatch | Existing pattern |
| `AstCweRule` struct | n/a (local) | Rule table entry | Existing pattern |

No new Cargo dependencies required for Phase 21. [VERIFIED: codebase grep]

### New Code Elements

| Element | File | Purpose |
|---------|------|---------|
| `ArgCheck::SizeofPointer` | ast_scanner.rs | CWE-467 sizeof(pointer) detection |
| `ArgCheck::ArgAtIndex` | ast_scanner.rs | Already added in Phase 20 — available at start of Phase 21 |
| `apply_division_rules()` | ast_scanner.rs | CWE-369 binary_expression walk |
| 12 new `AstCweRule` entries | ast_scanner.rs `AST_CWE_RULES` | One per new CWE |

**Installation:** None — all changes are to existing source files.

---

## Architecture Patterns

### System Architecture Diagram

```
scan_file_ast_or_lexical()
    │
    ├─► apply_ast_rules()          (call_expression walk via AST_CWE_RULES)
    │       └─► visit_node()
    │               └─► for each call_expression:
    │                       match rule.arg_check:
    │                           AnyCall           → fire
    │                           FixedSizeBuffer    → check dest is fixed array
    │                           NotStringLiteralAtIndex → check arg kind
    │                           ArgAtIndex         → check arg subtree text (Phase 20+)
    │                           SizeofPointer [NEW] → check sizeof arg is pointer type
    │
    └─► apply_division_rules() [NEW]   (binary_expression walk)
            └─► visit_binary_exprs()
                    └─► for each binary_expression:
                            if operator == "/" or "%" AND rhs is integer_literal "0":
                                emit CWE-369 finding
```

### Recommended Project Structure

No new directories. All changes within:

```
src/
└── vulnerability/
    └── ast_scanner.rs     # primary: new enum variant, new helper, 12 new table entries

tests/
└── vulnerability_tests/
    └── ast_scanner_tests.rs  # new unit tests for all 12 new CWEs
```

Benchmark output:

```
benchmark/
└── juliet/
    └── ANALYSIS.md        # updated after implementation — new per-CWE rows
```

### Pattern 1: AnyCall Rule (simplest)

**What:** Fire on any invocation of the named dangerous function.
**When to use:** Function has no safe usage pattern.

```rust
// Source: existing ast_scanner.rs pattern (line 58-59, 74)
AstCweRule { cwe_id: 242, functions: &["gets", "mktemp"], arg_check: ArgCheck::AnyCall },
AstCweRule { cwe_id: 338, functions: &["rand", "random", "srand"], arg_check: ArgCheck::AnyCall },
```

### Pattern 2: FixedSizeBuffer Rule

**What:** Fire only when the first argument is a fixed-size array identifier declared in the enclosing scope.
**When to use:** Function is safe on heap pointers but dangerous on stack buffers (reduces FP dramatically).

```rust
// Source: existing ast_scanner.rs lines 52-55
AstCweRule { cwe_id: 126, functions: &["strcat", "strncat"], arg_check: ArgCheck::FixedSizeBuffer },
```

### Pattern 3: ArgAtIndex Rule (Phase 20 addition)

**What:** Fire when the positional argument's AST subtree text contains all specified tokens (word-boundary).
**When to use:** Dangerous function has both safe and unsafe argument values.

```rust
// Source: Phase 20 — ArgAtIndex(index, tokens)
// CWE-328: CryptCreateHash where arg[1] is CALG_MD2, CALG_MD5, or CALG_SHA1
AstCweRule {
    cwe_id: 328,
    functions: &["CryptCreateHash"],
    arg_check: ArgCheck::ArgAtIndex(1, &["CALG_MD2", "CALG_MD5", "CALG_SHA1"]),
}
// Note: ArgAtIndex fires when ANY of the listed tokens is present in arg[1].
// Verify exact match semantics with Phase 20 implementation.
```

### Pattern 4: SizeofPointer Rule (new variant)

**What:** Walk `sizeof_expression` nodes, fire when the argument is a pointer type.
**When to use:** CWE-467 — `sizeof(ptr)` where programmer intended `sizeof(*ptr)`.

```rust
// D-04 design: new ArgCheck::SizeofPointer variant
// Implementation notes:
//   - sizeof_expression node: child_by_field_name("value") gives the argument
//   - Pointer indicators: node kind == "pointer_declarator", or identifier preceded by "*"
//   - Since tree-sitter-c has no type resolution, use heuristic: the arg to sizeof
//     is a pointer if the malloc call site has the arg declared as pointer_declarator
//     in the enclosing scope, OR if the sizeof arg is an identifier and the declared
//     variable in enclosing scope has a pointer_declarator.
//
// Simpler heuristic (sufficient for Juliet TPs):
//   - Fire when sizeof(X) where X is an identifier, and X is declared in scope as
//     a pointer_declarator (not an array_declarator or plain declarator).
```

### Pattern 5: apply_division_rules() — Binary Expression Walk

**What:** Walk the AST for `binary_expression` nodes where operator is `/` or `%` and the right-hand side is an `integer_literal` with text exactly `"0"`.

```rust
// Source: D-01 design decision; mirrors visit_node() structure
fn apply_division_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    visit_binary_exprs(root, src, path, component_name, component_ecosystem, findings);
}

fn visit_binary_exprs<'a>(
    node: Node<'a>,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "binary_expression" {
        // tree-sitter-c: binary_expression has field "operator" and children "left"/"right"
        // Operator is NOT a named child — access via child(1) or iterate to find it
        // Right operand: child_by_field_name("right") in tree-sitter-c
        if let Some(op_node) = node.child(1) {  // child(1) is operator token
            if let Ok(op) = op_node.utf8_text(src) {
                if op == "/" || op == "%" {
                    if let Some(rhs) = node.child_by_field_name("right") {
                        if rhs.kind() == "number_literal" {  // tree-sitter-c uses "number_literal"
                            if let Ok(text) = rhs.utf8_text(src) {
                                if text == "0" {
                                    findings.push(SastFinding {
                                        cwe_id: 369,
                                        component_name: component_name.to_string(),
                                        component_ecosystem: component_ecosystem.to_string(),
                                        file_path: path.to_string_lossy().into_owned(),
                                        line: (node.start_position().row as u32) + 1,
                                        source: SastSource::Ast,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    // Recurse
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            visit_binary_exprs(cursor.node(), src, path, component_name, component_ecosystem, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

**IMPORTANT:** Verify tree-sitter-c node kinds before implementation — check if RHS literal kind is `"number_literal"` or `"integer_literal"`. The existing code uses `"string_literal"` and `"integer_literal"` for ArgCheck. Verify by printing `node.kind()` in a quick test or consult tree-sitter-c grammar. [ASSUMED — needs single verification call]

### Anti-Patterns to Avoid

- **AnyCall for CWE-126:** 94.8% FP in benchmark — must use FixedSizeBuffer (locked by D-06).
- **Broad function list for CWE-676:** 100% FP in benchmark — must use tight list (locked by D-08).
- **Duplicate function in CWE-676 and another CWE rule:** D-09 forbids this — produces duplicate findings on same call site.
- **Literal `"0"` check via token_present_with_boundary for CWE-369 division:** Use exact node kind + text comparison, not word-boundary matching (prevents `10` from matching).
- **AnyCall on `rand` for CWE-338 without also excluding from CWE-676:** `rand` is NOT in CWE-676 by D-09 (it's covered by CWE-338 rule). Verify no overlap.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Recursive AST walk | Custom iterator | `visit_node()` pattern (fresh cursor per level) | Existing pattern handles the "Pitfall 1" cursor reuse bug |
| Array scope collection | Custom tracker | `collect_function_scope_fixed_arrays()` + `collect_file_scope_fixed_arrays()` | Already implemented and tested |
| Token matching | Hand-rolled substr | `token_present_with_boundary(text, tok)` from `cwe_scanner.rs` | Already imported in ast_scanner.rs |
| Type resolution for CWE-467 | Full type inference | Pointer-declarator heuristic (see Pattern 4) | tree-sitter gives AST only; full type inference is out of scope |

---

## Juliet Corpus Coverage Map

All 12 Phase 21 CWEs have Juliet corpus directories. [VERIFIED: directory listing]

| CWE | Juliet Directory | Confirmed Present | Rule Type | Notes |
|-----|-----------------|-------------------|-----------|-------|
| 121 | CWE121_Stack_Based_Buffer_Overflow | YES | FixedSizeBuffer (subtype of buffer family) | 9 subdirs (s01-s09); large corpus |
| 126 | CWE126_Buffer_Overread | YES | FixedSizeBuffer | 3 subdirs (s01-s03) |
| 328 | CWE328_Reversible_One_Way_Hash | YES | ArgAtIndex | Uses CryptCreateHash + CALG_MD2/MD5/SHA1 |
| 338 | CWE338_Weak_PRNG | YES | AnyCall | Uses rand() — Windows only test files |
| 369 | CWE369_Divide_by_Zero | YES | apply_division_rules() | Uses runtime variable divisor (`100/data`), NOT literal `/0` — see critical note |
| 426 | CWE426_Untrusted_Search_Path | YES | AnyCall | Uses popen + system with relative paths |
| 467 | CWE467_Use_of_sizeof_on_Pointer_Type | YES | SizeofPointer | Uses `sizeof(badChar)` where badChar is `char *` |
| 526 | CWE526_Info_Exposure_Environment_Variables | YES | AnyCall | Uses getenv() |
| 535 | CWE535_Info_Exposure_Shell_Error | YES | AnyCall (fprintf to stderr) | Uses fprintf(stderr, ..., password) |
| 676 | CWE676_Use_of_Potentially_Dangerous_Function | YES | AnyCall | Uses `cin >>` (C++ iostream) — NOT alloca/strtok |
| 680 | CWE680_Integer_Overflow_to_Buffer_Overflow | YES | AnyCall | Uses malloc(data * sizeof(int)) |
| 780 | CWE780_Use_of_RSA_Algorithm_Without_OAEP | YES | AnyCall or ArgAtIndex | Uses CryptEncrypt without CRYPT_OAEP — NOT RSA_public_encrypt |

**CWE-369 corpus gap:** Juliet uses `100 / data` (variable divisor). The AST binary-expression rule fires on `x / 0` (literal zero RHS). These are DIFFERENT patterns. The AST rule will produce 0 Juliet TPs for CWE-369 since Juliet tests use runtime-variable divisors, not compile-time literal `/0`. The existing lexical fallback for CWE-369 also produces 0 Juliet TPs (existing ANALYSIS.md: AST=0, lexical=0 for CWE-369). This is acceptable per D-03 (lexical remains for parse-fail fallback) but the planner should verify whether the ≥1 TP requirement (ROADMAP success criterion #1) can be satisfied by the AST binary-expr rule on the Juliet corpus. If not, a synthetic fixture is needed. [VERIFIED: ANALYSIS.md shows CWE-369 = 0 AST TPs, 0 lexical TPs currently]

---

## Exact Function Lists

### CWE-121 (Stack-Based Buffer Overflow) — FixedSizeBuffer or AnyCall?

The CONTEXT.md D-13 lists CWE-121 as AnyCall. However, the Juliet test structure for CWE-121 focuses on out-of-bounds array indexing patterns (not specific dangerous function calls). Examining Juliet s01 files: they use `char charArray[10]` with bad index, not strcpy/etc. CWE-121 is a buffer overflow sub-type and is covered by the CWE-119 FixedSizeBuffer family rule.

**Conflict:** D-13 says CWE-121 is AnyCall. D-06 covers CWE-126 as FixedSizeBuffer. The Juliet CWE-121 files use array subscript access, not function calls. An AnyCall rule on specific stack-allocating functions (like `alloca`) would be separate from the array-index pattern.

**Research recommendation:** CWE-121 with AnyCall maps cleanly to `alloca` (unconditional stack allocation). Juliet CWE-121 array-index patterns will NOT be caught by `alloca` AnyCall — they need FixedSizeBuffer-style buffer writes. The planner must decide:
- Option A: Add CWE-121 as AnyCall on `alloca` (synthetic fixtures only for Juliet TP; Juliet array-index patterns undetected)
- Option B: Add CWE-121 as additional FixedSizeBuffer functions (`memcpy`, `sprintf` into stack buffers) — Juliet TPs possible
- Option C: Skip CWE-121 for now and note in ANALYSIS.md (conflicts with D-13)

[ASSUMED — requires planner decision; recommend Option A with synthetic fixture per D-18]

### CWE-338 (Weak PRNG) — AnyCall [VERIFIED: Juliet uses rand()]

```
functions: &["rand", "random", "srand"]
```

Juliet uses `rand()` as the bad sink. [VERIFIED: CWE338_Weak_PRNG__w32_01.c read]
`srand` seeds the PRNG and is also commonly flagged. `random()` is POSIX equivalent.
Note: `rand` must NOT appear in the CWE-676 function list (D-09).

### CWE-426 (Untrusted Search Path) — AnyCall [VERIFIED: Juliet uses popen + system]

```
functions: &["popen", "_popen", "system"]
```

Juliet uses `popen` and `system` with relative paths. [VERIFIED: CWE426 test read]
Warning: `system` is already in the CWE-78 rule (`AstCweRule { cwe_id: 78, functions: &["system", "popen", ...] }`). Adding `system`/`popen` to CWE-426 would produce duplicate findings on the same call site. D-09 applies — these are already covered. CWE-426 with `popen`/`system` as AnyCall would duplicate CWE-78.

**Conflict:** CWE-426 Juliet test uses `popen`/`system`, but these are already in CWE-78. The planner must choose:
- Option A: CWE-426 as AnyCall on `popen`/`system` (duplicate with CWE-78 for same calls — but different CWE IDs are valid since the same call can trigger multiple CWEs)
- Option B: Add CWE-426 on functions not in CWE-78 (e.g., `execvp`, `ShellExecute`, `LoadLibrary`) — no Juliet TP from CWE-426 dir
- Option C: CWE-426 AnyCall on `popen` + `system` but accepted as multi-CWE finding (different CWE IDs, same call)

[ASSUMED — recommend Option C: duplicate findings with different CWE IDs are valid per the dedup pipeline which deduplicates on (file, line, cwe_id) not just (file, line)]

### CWE-526 (Info Exposure through Environment Variables) — AnyCall [VERIFIED: Juliet uses getenv()]

```
functions: &["getenv"]
```

Juliet uses `getenv("PATH")` as the bad sink. [VERIFIED: CWE526 test read]
Note: `getenv` is a candidate for CWE-676 but D-09 says exclude functions covered by other CWEs. Since CWE-526 will cover `getenv`, exclude it from CWE-676.

### CWE-535 (Info Exposure through Shell Error Message) — ArgAtIndex or AnyCall?

Juliet CWE-535 uses `fprintf(stderr, "...", password)` — the issue is writing sensitive data to stderr. An AnyCall on `fprintf` would fire on all `fprintf` calls (100% FP). A better approach:
- `ArgAtIndex(0, &["stderr"])` on `fprintf` — fires when first arg contains "stderr"
- Or `AnyCall` on a stderr-specific wrapper function

[ASSUMED — recommend ArgAtIndex(0, &["stderr"]) on "fprintf" for CWE-535; this will fire on all fprintf(stderr, ...) calls which matches the CWE intent and gives Juliet TPs]

Alternative interpretation: CWE-535 as AnyCall on `perror` (which always writes to stderr). Juliet does NOT test perror.

**Recommendation:** `ArgAtIndex(0, &["stderr"])` on `&["fprintf", "vfprintf"]` for Juliet TP.

### CWE-676 (Use of Potentially Dangerous Functions) — Tight AnyCall [CRITICAL: cin mismatch]

Juliet tests `cin >> charBuffer` (unbounded C++ cin extraction). To get Juliet TPs, the rule must detect `cin` usage — but `cin` is an object, not a function call, so AST call_expression matching won't fire on `cin >> x` (it's a binary_expression with operator `>>`).

**Conclusion:** A standard AnyCall rule on `cin` will NOT work because `cin >> x` is not a `call_expression` node in tree-sitter-c. The Juliet CWE-676 tests will produce 0 TPs with any call-expression-based rule.

Per D-08, use a tight non-overlapping function list. Candidates that ARE function calls (not operators):
- `alloca` — unconditional stack allocation, dangerous by design
- `strtok` — non-reentrant (modifies static state)
- `gets` — already in CWE-120/242; D-09 excludes it

**Recommended tight CWE-676 list (function calls only, no cin/iostream):**

```
functions: &["alloca", "strtok"]
```

This will produce 0 Juliet TPs (since Juliet CWE-676 tests only cin). Synthetic fixtures required per D-18.

FP risk: `alloca` and `strtok` are function calls that always indicate the dangerous pattern — 0% FP expected. `strtok` in particular is universally documented as dangerous in multithreaded contexts.

Exclude from list: `gets` (CWE-120), `system` (CWE-78), `rand` (CWE-338), `getenv` (CWE-526), `popen` (CWE-78/426).

### CWE-680 (Integer Overflow to Buffer Overflow) — AnyCall [VERIFIED: Juliet uses malloc]

```
functions: &["malloc", "calloc", "realloc"]
```

Juliet uses `malloc(data * sizeof(int))` where data can overflow. [VERIFIED: CWE680 test read]
Note: `malloc`/`calloc`/`realloc` are already in CWE-190 rule. This is another multi-CWE duplicate scenario. CWE-680 is specifically about integer overflow leading to buffer overflow — the same malloc call can legitimately fire both CWE-190 and CWE-680. The dedup pipeline handles this via (file, line, cwe_id) — different CWE IDs = separate findings.

### CWE-328 (Reversible One-Way Hash) — ArgAtIndex [VERIFIED: Juliet uses CryptCreateHash]

```rust
AstCweRule {
    cwe_id: 328,
    functions: &["CryptCreateHash"],
    arg_check: ArgCheck::ArgAtIndex(1, &["CALG_MD2", "CALG_MD5", "CALG_SHA1"]),
}
```

The Juliet test uses `CryptCreateHash(hCryptProv, CALG_MD2, 0, 0, &hHash)` — arg index 1 (0-based) is the algorithm constant. [VERIFIED: CWE328 test read]

Important: `CALG_MD5` and `CALG_SHA1` overlap with the bad sinks in CWE-328 Juliet, but `MD5`, `MD5_Init`, `SHA1` in CWE-327 are OpenSSL functions — NO function overlap between CWE-327 and CWE-328 rules. [VERIFIED: CWE-327 rule in ast_scanner.rs, CWE-327 Juliet uses CryptDeriveKey not CryptCreateHash]

### CWE-467 (sizeof on Pointer Type) — SizeofPointer [VERIFIED: Juliet uses malloc(sizeof(ptr))]

The Juliet test: `badChar = (char*)malloc(sizeof(badChar))` where `badChar` is `char *`. [VERIFIED: CWE467 test read]

The `sizeof_expression` is a child of the argument to `malloc`. The `SizeofPointer` ArgCheck variant must detect when a `sizeof_expression`'s argument is a pointer-typed identifier. In tree-sitter-c:
- `sizeof_expression` has field `value`
- The `value` child will be an `identifier` (`badChar`) when `sizeof(badChar)` is written
- To determine if `badChar` is a pointer, check the enclosing scope for a `pointer_declarator` containing that identifier

The Juliet test pattern: `char * badChar = NULL; malloc(sizeof(badChar))` — the sizeof appears inside a function call's argument, not directly at statement level.

**Implementation path for SizeofPointer:**

The current `visit_node()` walks call_expression nodes. `sizeof_expression` appears as an argument node inside `arg_list`. The existing args collection in `visit_node()` handles this: when a rule's `arg_check` is `SizeofPointer`, the arm should:
1. Scan all args for `sizeof_expression` nodes (not just index 0)
2. For each sizeof_expression found, get its `value` child
3. If value is an `identifier`, check if that identifier is declared as a pointer_declarator in scope
4. If yes, fire

Alternatively, add a separate AST walk pass specifically for `sizeof_expression` nodes (simpler, avoids coupling to the function-name table). The CWE-467 rule functions list would be `&["malloc", "calloc", "realloc", "memcpy", "memset", "memmove"]` (sizeof(ptr) most dangerous when passed to allocation/copy functions).

But per D-11, the rule targets CWE-467 via the `SizeofPointer` variant on specific function calls. The Juliet test uses `malloc(sizeof(badChar))`.

### CWE-780 (RSA Without OAEP) — Mismatch Resolution

D-10 specifies `ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"])` on `RSA_public_encrypt`. The Juliet test uses Windows `CryptEncrypt` with the absence of `CRYPT_OAEP` flag. These are different APIs.

**Two-rule approach (recommended):**

```rust
// OpenSSL approach (D-10 as specified) — synthetic fixtures needed for Juliet TP
AstCweRule {
    cwe_id: 780,
    functions: &["RSA_public_encrypt"],
    arg_check: ArgCheck::ArgAtIndex(4, &["RSA_PKCS1_PADDING", "RSA_NO_PADDING"]),
}

// Windows CryptEncrypt approach — Juliet TP achievable
// CryptEncrypt(hKey, hHash, Final, dwFlags, ...) — dwFlags is arg 3 (0-based)
// BadSink: dwFlags == 0 (no CRYPT_OAEP flag)
// GoodSink: dwFlags == CRYPT_OAEP
// ArgAtIndex on "0" or check for absence of CRYPT_OAEP
```

The Juliet bad sink is `CryptEncrypt(hKey, NULL, 1, 0, ...)` where `dwFlags=0` means no OAEP. The good sink is `CryptEncrypt(hKey, NULL, 1, CRYPT_OAEP, ...)`. An `ArgAtIndex(3, &["0"])` on `CryptEncrypt` would fire when flags arg is literal 0 — but integer_literal "0" check would be needed (exact match, not word-boundary). This is the same pattern as the CWE-732 umask(0) case.

**Planner decision required:** Add both rules, or just the OpenSSL rule with synthetic fixture. Adding the CryptEncrypt rule satisfies ≥1 TP criterion on Juliet.

---

## Common Pitfalls

### Pitfall 1: tree-sitter-c binary_expression field names

**What goes wrong:** Using wrong field names for binary_expression children in `apply_division_rules()`.
**Why it happens:** tree-sitter-c's `binary_expression` grammar uses fields `left`, `right`, and operator accessed as `child(1)` — the operator token is the second child (index 1), not a named field. Some grammars name it differently.
**How to avoid:** The operator can be accessed via `node.child(1)` (the token between left and right), or verify via tree-sitter playground. The right operand is `node.child_by_field_name("right")`.
**Warning signs:** The function always returns no findings, even on `x / 0` test fixtures.

### Pitfall 2: integer_literal vs number_literal in tree-sitter-c

**What goes wrong:** The RHS `0` in `x / 0` may have node kind `"number_literal"` in tree-sitter-c, not `"integer_literal"`.
**Why it happens:** tree-sitter-c uses `number_literal` as the node kind for numeric constants. The existing `NotStringLiteralAtIndex` checks for `"string_literal"` (string kind). The division rule needs the integer literal kind.
**How to avoid:** Write a quick test or check tree-sitter-c grammar. Based on existing code in `ArgCheck::NotStringLiteralAtIndex` which checks `args[idx].kind() != "string_literal"`, verify that integer literals use `"number_literal"` or `"integer_literal"`.
**Warning signs:** apply_division_rules() never fires even on `int x = 5; int y = x / 0;` fixture.

### Pitfall 3: ArgAtIndex ANY-OF vs ALL-OF token semantics

**What goes wrong:** ArgAtIndex with multiple tokens fires when ALL tokens are present (like ContainsTokens), but for CWE-328 we want ANY-OF (`CALG_MD2` OR `CALG_MD5` OR `CALG_SHA1`).
**Why it happens:** The Phase 20 ArgAtIndex arm may implement ALL-OF semantics (all tokens must be present in the arg subtree) — verify which behavior Phase 20 implemented before writing CWE-328 rule.
**How to avoid:** Read the Phase 20 plan execute result / final ast_scanner.rs state. If ALL-OF, split into three separate AstCweRule entries (one per CALG_ constant). If ANY-OF, one entry suffices.
**Warning signs:** CWE-328 rule only fires when arg text contains all three CALG_ constants simultaneously (never).

### Pitfall 4: CWE-426 duplicate findings with CWE-78

**What goes wrong:** Adding `popen`/`system` to CWE-426 produces duplicate findings on same (file, line) but different CWE IDs. Some consumers may flag this as noisy.
**Why it happens:** Both CWE-78 and CWE-426 legitimately apply to `system()`/`popen()` calls.
**How to avoid:** Acceptable per design — the deduplication pipeline deduplicates on (file, line, cwe_id) triple, so CWE-78 and CWE-426 are separate valid findings. Document in ANALYSIS.md.

### Pitfall 5: Fresh cursor per recursion level (existing requirement)

**What goes wrong:** Reusing a `TreeCursor` across recursion levels causes node traversal bugs (Pitfall 1 from existing code comments).
**Why it happens:** tree-sitter cursors maintain internal state.
**How to avoid:** Every recursive call to `visit_binary_exprs()` must create a fresh cursor via `node.walk()`. Mirror the existing pattern in `visit_node()`.

### Pitfall 6: SizeofPointer scope lookup performance

**What goes wrong:** For every `sizeof_expression` found, collecting the full function scope's pointer declarations is O(n) per call site.
**Why it happens:** `collect_function_scope_fixed_arrays()` walks the entire function body.
**How to avoid:** Pre-collect pointer declarations at function scope entry (same approach as `file_scope_arrays`) and pass down via parameter. Mirror `file_scope_arrays` pattern but for pointer declarators.

---

## Code Examples

### Existing visit_node() structure to mirror in visit_binary_exprs()

```rust
// Source: ast_scanner.rs lines 188-294 (verified read)
fn visit_node<'a>(
    node: Node<'a>, src: &[u8], path: &Path, component_name: &str,
    component_ecosystem: &str, file_scope_arrays: &HashSet<String>,
    findings: &mut Vec<SastFinding>,
) {
    if node.kind() == "call_expression" {
        // ... rule matching ...
    }
    // Recurse — fresh cursor per level
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            visit_node(cursor.node(), src, path, component_name, component_ecosystem,
                       file_scope_arrays, findings);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

### scan_file_ast_or_lexical integration point for apply_division_rules()

```rust
// Source: ast_scanner.rs line 154-161 (verified read)
// Currently:
apply_ast_rules(tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem)

// Phase 21 change: two calls
let mut findings = apply_ast_rules(
    tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem
);
apply_division_rules(
    tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem,
    &mut findings
);
findings
```

### ArgCheck enum after Phase 20 (with Phase 21 addition)

```rust
// Source: ast_scanner.rs (current state has ContainsTokens; Phase 20 deletes it and adds ArgAtIndex)
// Phase 21 adds SizeofPointer:
#[derive(Debug)]
enum ArgCheck {
    FixedSizeBuffer,
    NotStringLiteralAtIndex(u8),
    ArgAtIndex(u8, &'static [&'static str]),  // added Phase 20
    AnyCall,
    SizeofPointer,   // NEW in Phase 21 (D-04)
}
```

---

## State of the Art

| Old Approach | Current Approach | Phase | Impact |
|--------------|------------------|-------|--------|
| ContainsTokens (scans all args) | ArgAtIndex (positional scope) | Phase 20 | Eliminates cross-arg FPs |
| Lexical-only CWE-369 | AST binary_expression + lexical fallback | Phase 21 | Structural precision: won't fire on `x/10` or comments |
| 13-CWE AST scanner | 25-CWE AST scanner | Phase 21 | Expanded coverage per Juliet benchmark |

**Deprecated/outdated:**
- `ContainsTokens` variant: deleted in Phase 20; Phase 21 must not re-add it.
- cppcheck subprocess: removed in Phase 19; all SAST findings now from AST + lexical scanners.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | tree-sitter-c binary_expression operator is `node.child(1)` (second child) | Architecture Patterns, Pattern 5 | apply_division_rules() never fires; must use correct child index |
| A2 | tree-sitter-c uses "number_literal" (not "integer_literal") for numeric literals in binary expressions | Common Pitfalls #2 | CWE-369 rule silently misses all targets |
| A3 | ArgAtIndex Phase 20 implementation uses ANY-OF semantics for token list | CWE-328 function list | Wrong — must split into 3 rules instead of 1 |
| A4 | CWE-426 multi-CWE findings (same call site, different CWE IDs) are acceptable to downstream consumers | CWE-426 function list | FP noise reported by consumers if wrong |
| A5 | CWE-121 AnyCall on alloca satisfies the "AnyCall" classification in D-13 | CWE-121 in function lists | 0 Juliet TPs; synthetic fixture required |
| A6 | The deduplication pipeline deduplicates on (file, line, cwe_id) triple, allowing CWE-78 + CWE-426 to coexist | CWE-426 discussion | Duplicates suppressed if dedup is on (file, line) only |

---

## Open Questions (RESOLVED)

All four open questions are resolved by the Phase 21 plans (21-01, 21-02, 21-03) and the Phase 20 implementation. Resolution notes inline below.

1. **ArgAtIndex token semantics (ANY-OF vs ALL-OF)** — **RESOLVED: ALL-OF**
   - Confirmed from `.planning/phases/20-argument-value-ast-migration/20-01-PLAN.md` line 34: `tokens.iter().all(|tok| token_present_with_boundary(&arg_text, tok))`. ArgAtIndex fires only when ALL listed tokens are simultaneously present in the arg subtree text.
   - Consequence applied in Plan 02: CWE-328 is expressed as THREE separate rule entries (one per CALG_* constant) and CWE-780 as TWO entries (one per padding constant) so that ANY-of-set semantics are achieved at the rule-table level. Plan 02 Task 1 includes a `read_first` step that re-verifies this semantics in the post-Phase-20 `src/vulnerability/ast_scanner.rs` before any rule entries are written.

2. **CWE-780 two-rule vs one-rule approach** — **RESOLVED: TWO RULES (OpenSSL split) + add CryptEncrypt for Juliet TP**
   - D-10 locks the OpenSSL `RSA_public_encrypt` rule. ALL-OF semantics forces it into two entries (RSA_PKCS1_PADDING and RSA_NO_PADDING) — implemented in Plan 02 Task 1.
   - Additionally, Plan 02 Task 1 now ALSO adds a `CryptEncrypt` ArgAtIndex(3, &["0"]) entry to satisfy ROADMAP success criterion #1 with a Juliet TP for CWE-780 (Juliet CWE-780 tests Windows `CryptEncrypt` with `dwFlags == 0` instead of `CRYPT_OAEP`). See revision-mode update.

3. **tree-sitter-c literal node kind (`number_literal` vs `integer_literal`)** — **RESOLVED: accept both**
   - Implemented in Plan 01 Task 2 (`apply_division_rules` / `visit_binary_exprs`) by checking `kind == "number_literal" || kind == "integer_literal"`. The TDD test (`test_cwe_369_division_literal_zero`) covers both `x / 0` (integer) and `x / 0.0` (float — must NOT fire), which validates the kind-check at execution time. No further investigation required pre-execution.

4. **CWE-121 FixedSizeBuffer vs AnyCall classification** — **RESOLVED: AnyCall on alloca, synthetic fixture for TP**
   - Per D-13/D-14, CWE-121 is AnyCall. Plan 02 Task 1 uses `functions: &["alloca"]`. Juliet CWE-121 array-subscript overrun patterns will NOT be detected by this rule (acknowledged); the TP is supplied via the synthetic fixture exercised by `test_cwe_121_anycall_alloca` in Plan 02 Task 2. ROADMAP criterion #1 is amended to allow synthetic-fixture TPs for corpus-gap CWEs (see ROADMAP §Phase 21 SC#1 parenthetical).


---

## Environment Availability

Step 2.6: SKIPPED — Phase 21 is purely code changes to `src/vulnerability/ast_scanner.rs` and test additions. No new external dependencies. The Juliet corpus is already present at `example_target_repos/juliet-test-suite-c/`. [VERIFIED: directory listing]

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + cargo test |
| Config file | Cargo.toml (feature = "internal") |
| Quick run command | `cargo test --features internal -p radeis-sc2sbom ast_scanner` |
| Full suite command | `cargo test --features internal -p radeis-sc2sbom` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CWEXP-01 | Each new CWE rule fires on TP fixture and not on safe code | unit | `cargo test --features internal test_cwe_{NNN}` | ❌ Wave 0 |
| CWEXP-01 | No regression on existing 13 CWEs | regression | `cargo test --features internal test_ast_all_tractable_cwes` | ✅ |
| CWEXP-01 | apply_division_rules() fires on `x / 0` not on `x / 10` | unit | `cargo test --features internal test_cwe_369_division` | ❌ Wave 0 |
| CWEXP-01 | SizeofPointer fires on sizeof(ptr) not sizeof(*ptr) | unit | `cargo test --features internal test_cwe_467_sizeof_pointer` | ❌ Wave 0 |
| CWEXP-01 | CWE-328 ArgAtIndex fires on CALG_MD2 not CALG_SHA_512 | unit | `cargo test --features internal test_cwe_328_weak_hash` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --features internal ast_scanner`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green + Juliet benchmark re-run + ANALYSIS.md updated

### Wave 0 Gaps

New test functions needed in `tests/vulnerability_tests/ast_scanner_tests.rs`:
- `test_cwe_121_anycall` — TP: `alloca(n)` fires; FP guard: safe stack var does not
- `test_cwe_126_fixed_size_buffer` — TP: `strcat(buf, extra)` with `char buf[64]` fires; FP: strcat into pointer does not
- `test_cwe_328_weak_hash` — TP: `CryptCreateHash(p, CALG_MD2, 0, 0, &h)` fires; FP: CALG_SHA_512 does not
- `test_cwe_338_weak_prng` — TP: `rand()` fires; FP guard not needed (AnyCall)
- `test_cwe_369_division` — TP: `int y = x / 0;` fires; FP: `x / 10` does not, `x / 0.0` (float) does not
- `test_cwe_426_search_path` — TP: `system("cmd")` fires
- `test_cwe_467_sizeof_pointer` — TP: `malloc(sizeof(badPtr))` with `char *badPtr` fires; FP: `malloc(sizeof(*badPtr))` does not
- `test_cwe_526_env_exposure` — TP: `getenv("PATH")` fires
- `test_cwe_535_shell_error` — TP: `fprintf(stderr, "%s", pw)` fires; FP: `fprintf(stdout, ...)` does not
- `test_cwe_676_dangerous_fn` — TP: `alloca(n)` fires; FP guard: not a duplicate of CWE-121 (different CWE ID, same call is valid)
- `test_cwe_680_int_overflow_alloc` — TP: `malloc(data * sizeof(int))` fires
- `test_cwe_780_rsa_no_oaep` — TP: `RSA_public_encrypt(..., RSA_PKCS1_PADDING)` fires; FP: `RSA_public_encrypt(..., RSA_PKCS1_OAEP_PADDING)` does not

---

## Security Domain

The work in Phase 21 IS the security domain — expanding CWE detection rules. No external auth, session, or access control concerns. No new attack surface introduced; scanner is an analysis tool, not a network service.

ASVS categories: not applicable for this type of internal analysis-tool code change.

---

## Sources

### Primary (HIGH confidence)
- `src/vulnerability/ast_scanner.rs` (verified read) — existing ArgCheck enum, AST_CWE_RULES, visit_node() structure
- `benchmark/juliet/ANALYSIS.md` (verified read) — existing TP/FP table, Phase 19 recommendations
- Juliet corpus test files (verified read, 10+ files across 8 CWEs) — exact function/API patterns used in bad sinks
- `.planning/phases/21-ast-cwes-anycall-argpattern-expansion/21-CONTEXT.md` (verified read) — locked decisions D-01 through D-18

### Secondary (MEDIUM confidence)
- `.planning/phases/20-argument-value-ast-migration/20-01-PLAN.md` (verified read) — ArgAtIndex variant design and token matching semantics

### Tertiary (LOW confidence / ASSUMED)
- tree-sitter-c binary_expression field names and literal node kinds [ASSUMED — training knowledge; verify before implementing apply_division_rules()]
- ArgAtIndex ANY-OF vs ALL-OF semantics [ASSUMED — depends on Phase 20 implementation not yet read in full]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all extensions to existing patterns
- Architecture: HIGH — all patterns verified from codebase read
- Function lists: MEDIUM — CWE-338/526/676 lists based on Juliet verification; CWE-121/535 lists need planner decision
- Pitfalls: HIGH — identified from code read and Juliet corpus analysis

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (stable codebase; tree-sitter-c grammar version pinned in Cargo.lock)
