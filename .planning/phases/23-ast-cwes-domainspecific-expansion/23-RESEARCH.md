# Phase 23: ast-cwes-domainSpecific-expansion - Research

**Researched:** 2026-05-12
**Domain:** Rust / tree-sitter-c AST pattern detection, domain-specific C/C++ CWE rules
**Confidence:** HIGH

## Summary

Phase 23 adds 8 domain-specific CWE rules to `ast_scanner.rs`, expanding coverage from 41 to 49
CWEs. All rules target specific named APIs with optional argument checks. The architecture is a
direct continuation of Phases 21–22: new `AstCweRule` entries in `AST_CWE_RULES` plus two new
helper functions (`apply_signal_handler_rules`, `apply_paired_lock_rules`) following the
`apply_division_rules()` pattern.

Four critical findings emerged from fixture inspection:

1. **CWE-762 cannot use AST `delete_expression`** — tree-sitter-c (v0.24.2, name: `'c'`) has no
   `delete_expression` node type and no C++ grammar support. All 6,092 CWE-762 Juliet files are
   `.cpp` with `namespace` keywords, which trigger `has_error() == true` in tree-sitter-c. These
   files fall back to the lexical scanner, which also has no `delete` rule. The planner must choose
   a raw-source text scan approach for CWE-762 rather than an AST `delete_expression` traversal.

2. **CWE-427 Juliet uses PUTENV macro, not direct `putenv()`** — All 560 CWE-427 test files use
   `#define PUTENV putenv` (or `_putenv` on Windows) and call `PUTENV(data)`. tree-sitter-c sees
   the macro name `PUTENV` as the function identifier, not `putenv` or `_putenv`. AnyCall on
   `putenv`/`_putenv`/`setenv` will yield 0 TPs on Juliet. Acceptable under D-11 (same
   rationale as all Windows-API rules).

3. **CWE-591 Juliet uses `malloc()`, not `VirtualAlloc()`** — All 112 CWE-591 files allocate
   sensitive data via `malloc()`. The bad case omits `VirtualLock()`; the good case includes it.
   The D-02 approach (flag `VirtualAlloc` when `VirtualLock` absent) will yield 0 TPs on Juliet.
   Acceptable under D-11. The paired-lock detection logic is still valuable for real Win32 code.

4. **CWE-479 two-pass is fully confirmed** — All 18 CWE-479 Juliet files have the same structure:
   `helperBad()` calls `malloc`/`free`, `helperGood()` assigns a volatile. Both are registered
   via `signal(SIGINT, helperX)` in the same file. The two-pass approach correctly distinguishes
   the bad handler from the good handler.

**Primary recommendation:** Implement all 8 CWEs per CONTEXT.md decisions. For CWE-762, use raw
source text scan (regex-on-raw-bytes in the AST path, or add a lexical fallback rule) rather than
AST `delete_expression`. Add synthetic `.c` fixtures for CWE-762 TPs. Accept 0 TPs for CWE-427
and CWE-591 on Juliet; document in ANALYSIS.md.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** CWE-762 — AnyCall on `delete` and `delete[]` operator-expressions. The planner
  decides exact traversal (new helper, inline in `apply_ast_rules`, or new `ArgCheck::DeleteOperator`
  variant). Matches Juliet CWE-762 bad-sink pattern.
- **D-02:** CWE-591 — flag `VirtualAlloc` when `VirtualLock` absent in same function body.
  `apply_paired_lock_rules()` collects all `call_expression` function names in function scope.
  No cross-function tracking.
- **D-03:** CWE-479 — two-pass via `apply_signal_handler_rules()`:
  - Pass 1: collect function-name identifiers passed as arg-1 to `signal()` calls.
  - Pass 2: for each collected handler name, find its function definition, scan body for
    non-reentrant calls: `malloc`, `free`, `printf`, `fprintf`, `sprintf`, `snprintf`,
    `vprintf`, `vfprintf`, `exit`, `abort`, `syslog`.
- **D-04:** CWE-479 — single-file scope only.
- **D-05:** CWE-284 — `ArgAtIndex(4, &["GENERIC_ALL"])` on `CreateDesktopA` and `CreateDesktopW`.
- **D-06:** CWE-114/272/427/785 (and CWE-591's VirtualAlloc side) use `AnyCall`:
  - CWE-114: `LoadLibraryA`, `LoadLibraryW`, `LoadLibraryExA`, `LoadLibraryExW`
  - CWE-272: `CreateProcessAsUserA`, `CreateProcessAsUserW`
  - CWE-427: `SetDllDirectoryA`, `SetDllDirectoryW`, `putenv`, `_putenv`, `setenv`
  - CWE-785: `PathAppendA`, `PathAppendW`, `realpath`, `_fullpath`
- **D-07:** All Windows-API rules included unconditionally — no platform gate. Document Win32-specific
  CWEs (114/272/284/591/785) in module-level doc comment.
- **D-08:** `apply_signal_handler_rules()` and `apply_paired_lock_rules()` added as helpers in
  `ast_scanner.rs` alongside `apply_division_rules()`. No new module file.
- **D-09:** Both helpers called from `scan_file_ast_or_lexical()`. Planner decides return type
  (`Vec<SastFinding>` vs mutable vec extend).
- **D-10:** Implement all 8 CWEs first, then re-run Juliet benchmark and update
  `benchmark/juliet/ANALYSIS.md` with 8 new per-CWE rows.
- **D-11:** FP gate is ≤40% per ROADMAP Phase 23 success criterion #3. Windows-API rules showing
  0% TPs and 0% FPs on non-Windows fixtures is acceptable.

### Claude's Discretion

- **CWE-762 operator traversal:** Exact implementation of `delete_expression` / `delete[]` AST
  node detection — new helper, inline in `apply_ast_rules`, or new `ArgCheck::DeleteOperator`.
- **CWE-591 paired-check scope:** Function-body scope preferred; planner has discretion.
- **Function lists for CWE-114/272/427/785:** Confirmed by researcher using Juliet fixture
  filenames and CERT C documentation (see Standard Stack section below).

### Deferred Ideas (OUT OF SCOPE)

- CWE-591 cross-function VirtualLock tracking.
- CWE-479 cross-file signal handler.
- CWE-762 `new` + `free()` mismatch detection (requires alloc-origin tracking).
- CWE-427 registry-based search path manipulation (RegSetValueEx).
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CWEXP-03 | Expand AST scanner from 41 to 49 CWEs via narrow domain/API rules (8 new CWEs: 114, 272, 284, 427, 479, 591, 762, 785). All rules validated against Juliet ground truth with ≥1 TP (where Juliet TPs are achievable) and FP% ≤40%. ANALYSIS.md updated with 49-CWE table. | All 8 Juliet directories verified on disk. Per-CWE patterns identified. tree-sitter-c limitations documented. CWE-762 requires text-level approach. CWE-427/591 will yield 0 TPs on Juliet (acceptable per D-11). |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| CWE rule evaluation (CWE-114/272/284/427/785) | AST scanner (API / Backend) | — | AnyCall/ArgAtIndex on named Win32/POSIX APIs; purely call-site detection |
| CWE-479 two-pass signal handler detection | AST scanner | — | Requires function-scope lookup by name; intra-file only |
| CWE-591 paired-lock detection | AST scanner (new helper) | — | Function-scope call presence check; no dataflow |
| CWE-762 delete/calloc mismatch | AST scanner (text-level) | Lexical fallback | tree-sitter-c cannot parse C++ delete; raw text scan needed |
| Juliet benchmark re-run + ANALYSIS.md update | Offline tooling | — | sc2sbom CLI run + manual table update |

## Standard Stack

### Core (already in project)
| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| tree-sitter-c | 0.24.2 | C AST parsing | `#[cfg(feature = "internal")]`; already in Cargo.toml |
| tree-sitter | current | Parser runtime | Already in use |

**tree-sitter-c 0.24.2 verified:** `grep "tree-sitter-c" Cargo.toml` → `version = "0.24"` [VERIFIED: codebase]

**New dependencies:** None. Phase 23 adds no new crate dependencies.

### CWE-762 Implementation Constraint (CRITICAL)

tree-sitter-c v0.24.2 has `name: 'c'` in grammar.js. It supports no C++ keywords (`namespace`,
`class`, `delete`, `new`, `template`). Verified by: [VERIFIED: codebase grep of tree-sitter-c
grammar.js and node-types.json — `delete` absent from both].

Consequence: All 6,092 CWE-762 Juliet `.cpp` files use `namespace` declarations, which produce
`has_error() == true` in tree-sitter-c → these files fall back to lexical scanner → the current
lexical scanner has no `delete` rule → 0 TPs on Juliet.

**Viable approach for CWE-762:** Scan raw source bytes (in the AST code path, before or after
tree-sitter parse) for the `delete` keyword using a simple regex or token check, similar to how
the lexical scanner checks for `/0`. This is a planner discretion item (D-01 says "planner
decides the exact traversal"). A synthetic `.cpp` fixture (namespace-free) could also be added
to tests to achieve ≥1 TP per the success criteria.

## Architecture Patterns

### System Architecture Diagram

```
scan_file_ast_or_lexical(path, ...)
        │
        ├── tree-sitter parse
        │       │
        │       ├── has_error() → lexical_scan_file()
        │       │
        │       └── OK → [AST path]
        │                 │
        │                 ├── apply_ast_rules(root, src, ...)     [existing: AnyCall / ArgAtIndex / FixedSizeBuffer / etc.]
        │                 │       └── AST_CWE_RULES table
        │                 │           new entries: CWE-114, 272, 284, 427, 785 (AnyCall/ArgAtIndex)
        │                 │           new entry: CWE-762 (AnyCall on "delete" or text-level)
        │                 │
        │                 ├── apply_division_rules(root, src, ...) [Phase 21: CWE-369 binary_expression]
        │                 │
        │                 ├── apply_signal_handler_rules(root, src, ...) [NEW: Phase 23, CWE-479]
        │                 │       Pass 1: collect signal() arg-1 identifiers → handler_names
        │                 │       Pass 2: for each handler_name, find fn def, scan for non-reentrant calls
        │                 │
        │                 └── apply_paired_lock_rules(root, src, ...) [NEW: Phase 23, CWE-591]
        │                         collect call_expression names in each function scope
        │                         flag: VirtualAlloc present ∧ VirtualLock absent
        │
        └── all helpers return Vec<SastFinding> → extend all_findings
```

### Recommended Project Structure

No new files. All additions in `src/vulnerability/ast_scanner.rs`:

```
src/vulnerability/ast_scanner.rs
  AST_CWE_RULES          ← +6 new AstCweRule entries (CWE-114, 272, 284, 427, 785 + CWE-762 if
                              feasible via table)
  apply_signal_handler_rules()   ← new helper, CWE-479
  apply_paired_lock_rules()      ← new helper, CWE-591
  scan_file_ast_or_lexical()     ← 2 new helper calls added here
```

### Pattern 1: New AstCweRule entries (CWE-114, 272, 284, 427, 785)

These follow the identical pattern to existing AnyCall/ArgAtIndex rules:

```rust
// Source: existing AST_CWE_RULES in src/vulnerability/ast_scanner.rs
// CWE-114: Process Control via dynamic library load
AstCweRule {
    cwe_id: 114,
    functions: &["LoadLibraryA", "LoadLibraryW", "LoadLibraryExA", "LoadLibraryExW"],
    arg_check: ArgCheck::AnyCall,
},
// CWE-272: Least Privilege Violation via privileged process creation
AstCweRule {
    cwe_id: 272,
    functions: &["CreateProcessAsUserA", "CreateProcessAsUserW"],
    arg_check: ArgCheck::AnyCall,
},
// CWE-284: Improper Access Control via over-privileged desktop creation
AstCweRule {
    cwe_id: 284,
    functions: &["CreateDesktopA", "CreateDesktopW"],
    arg_check: ArgCheck::ArgAtIndex(4, &["GENERIC_ALL"]),
},
// CWE-427: Uncontrolled Search Path Element
AstCweRule {
    cwe_id: 427,
    functions: &["SetDllDirectoryA", "SetDllDirectoryW", "putenv", "_putenv", "setenv"],
    arg_check: ArgCheck::AnyCall,
},
// CWE-785: Path manipulation function without MAX_PATH buffer
AstCweRule {
    cwe_id: 785,
    functions: &["PathAppendA", "PathAppendW", "realpath", "_fullpath"],
    arg_check: ArgCheck::AnyCall,
},
```

### Pattern 2: apply_signal_handler_rules() — CWE-479

Two-pass over the same parsed tree. Model: `apply_division_rules()` from Phase 21.

```rust
// Source: inferred from D-03 + apply_division_rules() pattern
fn apply_signal_handler_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    // Pass 1: walk all call_expression nodes looking for signal() calls
    // Collect arg-1 identifier text as handler function name
    let handler_names: HashSet<String> = collect_signal_handler_names(root, src);

    // Pass 2: for each handler name, find its function_definition in the AST
    // Walk its body for calls to non-reentrant functions
    let mut findings = Vec::new();
    for handler_name in &handler_names {
        if let Some(fn_def) = find_function_definition(root, src, handler_name) {
            if body_calls_non_reentrant(fn_def, src) {
                // flag the signal() call site line (from pass 1)
                findings.push(SastFinding { cwe_id: 479, ... });
            }
        }
    }
    findings
}

// Non-reentrant function set (D-03):
const NON_REENTRANT: &[&str] = &[
    "malloc", "free", "printf", "fprintf", "sprintf", "snprintf",
    "vprintf", "vfprintf", "exit", "abort", "syslog",
];
```

**Key implementation detail:** The finding should be emitted at the `signal()` call site line
(from pass 1), not at the non-reentrant call inside the handler body. This matches the Juliet
oracle expectation and the existing `SastFinding.line` convention.

### Pattern 3: apply_paired_lock_rules() — CWE-591

Per-function scope walk. Collect all function names called in a function body. Flag if
`VirtualAlloc` present but `VirtualLock` absent.

```rust
// Source: inferred from D-02 + find_enclosing_function() pattern
fn apply_paired_lock_rules(
    root: Node,
    src: &[u8],
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
) -> Vec<SastFinding> {
    let mut findings = Vec::new();
    // Walk all function_definition nodes
    visit_function_definitions(root, src, |fn_node| {
        let calls = collect_call_names_in_subtree(fn_node, src);
        if calls.contains("VirtualAlloc") && !calls.contains("VirtualLock") {
            // find the VirtualAlloc call_expression line for accurate reporting
            findings.push(SastFinding { cwe_id: 591, ... });
        }
    });
    findings
}
```

### Pattern 4: CWE-762 — delete/calloc mismatch

**AST approach not viable** via `delete_expression` (tree-sitter-c has no such node type; C++ .cpp
files fail parse → lexical fallback). Two viable options:

**Option A (recommended): Synthetic fixture + text-level scan in test path**
- Add a synthetic `.cpp` file to `tests/fixtures/` that is namespace-free (tree-sitter-c can parse
  it without errors): `void f() { char *p = (char*)calloc(10,1); delete p; }`
- Detect `delete` in AST path via a new `apply_delete_rules()` helper that walks all
  expression_statement nodes and looks for the text `"delete"` as the first token
- OR: Walk nodes of kind `"identifier"` with text `"delete"` (tree-sitter-c treats unknown
  C++ keywords as identifiers in error-tolerant mode when it doesn't produce a full tree error)

**Option B: Lexical-only rule for .cpp files**
- Add a new `ArgCheck::TextPattern(&'static str)` variant or a plain lexical rule in
  `cwe_scanner.rs` that fires on `delete ` token in `.cpp` files with a prior `calloc` in the
  same function — high FP risk without proper scoping

**Recommendation:** Option A with a namespace-free synthetic fixture. The Juliet corpus files will
fall back to lexical (0 TPs), but a synthetic fixture provides the ≥1 TP required by success
criterion #2.

### Anti-Patterns to Avoid

- **Walking `delete` as `call_expression`:** `delete` is a C++ operator, not a function call.
  It will never appear as `call_expression.function.text == "delete"` in tree-sitter-c.
- **Assuming CWE-427 Juliet files call `putenv()` directly:** All 560 files use the `PUTENV`
  preprocessor macro. AnyCall on `putenv` will not match. This is expected and acceptable.
- **Assuming CWE-591 Juliet files call `VirtualAlloc()`:** They use `malloc()`. The
  `apply_paired_lock_rules()` approach only fires on `VirtualAlloc` — 0 TPs is expected.
- **Emitting CWE-479 findings at the handler body line:** Findings should point to the `signal()`
  call site (where the dangerous handler is registered), not the `malloc()`/`free()` line inside
  the handler body.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Function name collection in subtree | Custom recursive walk | Extend `collect_arrays_in_subtree` pattern | Existing pattern already handles fresh cursors per level (Pitfall 1) |
| Signal handler function lookup by name | String map with file scope | Walk root for `function_definition` where `function_declarator` identifier matches | `find_enclosing_function()` already walks ancestors; inverse walk finds definitions |
| Non-reentrant call detection | Per-CWE list | Single `NON_REENTRANT` const slice | Shared across all callers |
| C++ delete detection via grammar | New tree-sitter grammar | Text-level scan or synthetic fixture (see CWE-762 section) | tree-sitter-c v0.24.2 is C-only; C++ grammar is a separate crate not in use |

## Juliet Corpus — Per-CWE Coverage Facts

[VERIFIED: codebase — Juliet fixture directory inspection]

| CWE | Files | Pattern Used in Juliet | Expected Approach TPs | Notes |
|-----|-------|------------------------|----------------------|-------|
| 114 | 672 .c | `LoadLibraryW(data)` direct call | YES — AnyCall on LoadLibraryA/W | Clean AnyCall |
| 272 | 252 .c | `CreateProcessAsUserA/W(...)` direct call | YES — AnyCall on CreateProcessAsUserA/W | ~252 files × 50% bad |
| 284 | 216 .c | `CreateDesktopA/W(..., GENERIC_ALL, ...)` | YES — ArgAtIndex(4, GENERIC_ALL) | 36 files are CreateDesktop; 180 are other sinks |
| 427 | 560 .c | `PUTENV(data)` macro | 0 TPs — macro not expanded; acceptable per D-11 | All files use PUTENV macro |
| 479 | 18 .c | `signal(SIGINT, helperBad)` + `helperBad()` calls `malloc`/`free` | YES — two-pass finds helperBad | Uniform pattern across all 18 files |
| 591 | 112 .c | `malloc()` without `VirtualLock()` | 0 TPs — uses malloc not VirtualAlloc; acceptable per D-11 | Good case has VirtualLock after malloc |
| 762 | 6,092 .cpp | `calloc()` + `delete name` inside class destructor | 0 TPs on Juliet (namespace causes parse error) — need synthetic fixture | Must add namespace-free synthetic `.cpp` |
| 785 | 18 .c | `PathAppendA(path, "AAAAAA")` with `char path[BAD_PATH_SIZE]` | YES — AnyCall on PathAppendA/W | All 18 files use PathAppendA (char variants) |

## Common Pitfalls

### Pitfall 1: delete as call_expression
**What goes wrong:** The planner tries to detect CWE-762 by adding `"delete"` to an `AnyCall`
rule in `AST_CWE_RULES`. At runtime, no findings are emitted.
**Why it happens:** `delete` is a C++ unary operator expression, never a `call_expression` node
in any C grammar. Even if tree-sitter-c parses `delete name` without error, it wouldn't produce
a `call_expression` node.
**How to avoid:** Use a text-level check (token scan on raw source bytes) or walk AST nodes
where the text of an expression_statement starts with `"delete"`.
**Warning signs:** 0 TPs even on the synthetic fixture.

### Pitfall 2: CWE-762 .cpp files always fail tree-sitter-c parse
**What goes wrong:** All Juliet CWE-762 `.cpp` files use `namespace { class { } }` — tree-sitter-c
triggers `has_error() == true` → lexical fallback → 0 TPs.
**Why it happens:** tree-sitter-c v0.24.2 is a pure C grammar; `namespace` is not in its grammar.
**How to avoid:** Use a synthetic fixture without `namespace`/`class` for CWE-762 validation.
The synthetic fixture can be a simple `.c` or namespace-free `.cpp`: `void f() { char *p = (char*)calloc(10,1); delete p; }`.
**Warning signs:** 0 TPs even with a "working" delete rule — check if `has_error()` is triggering.

### Pitfall 3: Signal handler finding emitted at wrong line
**What goes wrong:** Findings for CWE-479 report the line of the `malloc()` call inside
`helperBad()`, not the line of `signal(SIGINT, helperBad)`.
**Why it happens:** Pass 2 walks the handler body and emits a finding at each non-reentrant call
site, rather than at the `signal()` registration site.
**How to avoid:** Retain the line number of the `signal()` call from pass 1. Emit the finding
there, with the CWE-479 tag.
**Warning signs:** Finding lines don't match the `signal()` call in Juliet test output.

### Pitfall 4: FP from signal() calls to well-known safe handlers
**What goes wrong:** A program registers `signal(SIGTERM, SIG_DFL)` or `signal(SIGINT, SIG_IGN)`.
Pass 1 collects `"SIG_DFL"` or `"SIG_IGN"`. Pass 2 tries to find a function definition named
`SIG_DFL` — finds none — no false positive.
**Why it happens:** Not actually a problem; the design handles it correctly as long as pass 2
requires a matching function definition. If no definition is found, no finding is emitted.
**How to avoid:** No action needed; fall through gracefully if the handler name has no matching
`function_definition` in the AST.

### Pitfall 5: apply_paired_lock_rules collecting names from nested function calls
**What goes wrong:** `VirtualAlloc` is called in an outer function; `VirtualLock` is called in a
nested helper function invoked from the same outer function. The paired-lock check sees
`VirtualAlloc` in scope but not `VirtualLock` (which is inside the helper's call_expression
children, not the outer function body's direct calls).
**Why it happens:** The "collect all call names in function body" must include names from ALL
call_expression nodes recursively inside the function body, not just direct children.
**How to avoid:** Walk all descendants of the function body subtree, not just direct
call_expression children.
**Warning signs:** False positives on well-written Win32 code where VirtualLock is called from
a helper.

## Code Examples

### Example: collect_signal_handler_names() — Pass 1
```rust
// Source: inferred from apply_division_rules() pattern + D-03
fn collect_signal_handler_names(root: Node, src: &[u8]) -> HashMap<String, u32> {
    // Returns handler_name → signal() call site line
    let mut result = HashMap::new();
    collect_signal_handlers_rec(root, src, &mut result);
    result
}

fn collect_signal_handlers_rec(node: Node, src: &[u8], out: &mut HashMap<String, u32>) {
    if node.kind() == "call_expression" {
        if let Some(fn_node) = node.child_by_field_name("function") {
            if fn_node.utf8_text(src).ok() == Some("signal") {
                if let Some(args) = node.child_by_field_name("arguments") {
                    let mut cursor = args.walk();
                    let arg_vec: Vec<Node> = args.named_children(&mut cursor).collect();
                    if arg_vec.len() >= 2 && arg_vec[1].kind() == "identifier" {
                        if let Ok(name) = arg_vec[1].utf8_text(src) {
                            let line = (node.start_position().row as u32) + 1;
                            out.insert(name.to_string(), line);
                        }
                    }
                }
            }
        }
    }
    let mut cursor = node.walk(); // fresh cursor per level
    if cursor.goto_first_child() {
        loop {
            collect_signal_handlers_rec(cursor.node(), src, out);
            if !cursor.goto_next_sibling() { break; }
        }
    }
}
```

### Example: find_function_definition() — Pass 2 lookup
```rust
// Source: inferred from find_enclosing_function() inverse pattern
fn find_function_definition<'a>(root: Node<'a>, src: &[u8], name: &str) -> Option<Node<'a>> {
    // Walk root children looking for function_definition with matching declarator identifier
    find_fn_def_rec(root, src, name)
}
```

### Example: ArgAtIndex(4, &["GENERIC_ALL"]) for CWE-284
```rust
// Source: existing ArgCheck::ArgAtIndex from Phase 20
AstCweRule {
    cwe_id: 284,
    functions: &["CreateDesktopA", "CreateDesktopW"],
    arg_check: ArgCheck::ArgAtIndex(4, &["GENERIC_ALL"]),
},
// Implementation in apply_ast_rules() ArgAtIndex arm:
// 1. Get args[4] (0-based 5th argument)
// 2. Walk subtree of args[4] collecting leaf text
// 3. Check token_present_with_boundary(text, "GENERIC_ALL")
// GENERIC_ALL is an identifier node → utf8_text = "GENERIC_ALL" → match ✓
```

## State of the Art

| Old Approach | Current Approach | Notes |
|--------------|------------------|-------|
| ContainsTokens (Phase 18) | ArgAtIndex (Phase 20) | Phase 23 uses ArgAtIndex for CWE-284 |
| apply_ast_rules() only | apply_ast_rules() + structural helpers | apply_division_rules() (P21) + 2 new helpers (P23) |

**Deprecated/outdated for Phase 23:**
- `ArgCheck::ContainsTokens` — deleted in Phase 20; Phase 23 uses `ArgAtIndex` for argument-value checks.
- `delete_expression` AST approach — not viable with tree-sitter-c; use text-level scan instead.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `apply_division_rules()` exists in ast_scanner.rs after Phase 21 executes, returning `Vec<SastFinding>` | Standard Stack, Architecture Patterns | New helpers must match its exact signature; if Phase 21 used a different pattern, helpers need adjustment |
| A2 | `ArgCheck::ArgAtIndex` variant exists after Phase 20 executes | Standard Stack | CWE-284 implementation blocked; must add ArgAtIndex as part of this phase instead |
| A3 | `scan_file_ast_or_lexical()` signature unchanged from Phase 18 | Architecture Patterns | Helper call sites in scan_file_ast_or_lexical must be adjusted if signature changed |
| A4 | tree-sitter-c falls back to lexical on `.cpp` files with `namespace` keyword | CWE-762 Pitfall | Verified via grammar.js inspection; no `namespace` in grammar → `has_error()` fires |

**Note on A1 and A2:** STATE.md indicates Phase 20 execution started 2026-05-12 but Phases 20–22
are not yet complete. If Phase 23 is planned before those phases execute, the planner must
account for the state of ast_scanner.rs at the time Phase 23 begins.

## Open Questions (RESOLVED)

1. **CWE-762 implementation approach: AST text scan vs synthetic fixture only?**
   - What we know: tree-sitter-c cannot parse C++ `delete` operator; all Juliet CWE-762 files use
     `namespace` + `class` → parse error → 0 AST TPs.
   - What's unclear: Should the planner add a raw-text `delete` check in the AST path (for real
     code that might have simpler `.cpp` files), or purely rely on a synthetic fixture for ≥1 TP?
   - Recommendation: Add a synthetic namespace-free `.cpp` fixture for TP validation, AND add a
     lightweight text-level check for `delete` token so the rule fires on real-world `.cpp` files
     that tree-sitter-c can parse (e.g., single-file `.cpp` without namespaces). The planner has
     full discretion per D-01.
   - **RESOLVED:** Both — Plan 02 Task 1 creates a synthetic namespace-free fixture (`tests/fixtures/c/cwe762_delete_bad.c`) AND Plan 02 Task 2 implements a text-level `apply_delete_rules()` helper for real-world `.cpp` files that tree-sitter-c can parse.

2. **CWE-479 finding line: signal() call site vs handler definition?**
   - What we know: Juliet oracle uses file-level TP matching, so the exact line doesn't affect
     TP/FP counting. However, for useful SARIF/markdown output, the signal() call site is more
     actionable.
   - Recommendation: Emit finding at the `signal()` call site line (from pass 1).
   - **RESOLVED:** Emit at the `signal()` call site line (from pass 1). Plan 02 Task 2 explicitly specifies this — findings use the line number stored during pass 1, not the malloc/free line inside the handler body.

## Environment Availability

Step 2.6: No new external dependencies. The phase adds rules to existing Rust code only. Juliet
fixture is at `example_target_repos/juliet-test-suite-c/` — verified present on disk
[VERIFIED: codebase `ls`].

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| tree-sitter-c | All new AST rules | Yes | 0.24.2 | — |
| Juliet corpus | Benchmark re-run | Yes | fixture on disk | — |
| Rust toolchain | Compile/test | Yes | 1.92.0 | — |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` with `#[cfg(feature = "internal")]` |
| Config file | `Cargo.toml` feature gate `internal` |
| Quick run command | `cargo test --features internal -p radeis_sc2sbom -- phase_23` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CWEXP-03 | CWE-114 AnyCall on LoadLibraryA/W | unit | `cargo test --features internal phase_23_cwe114` | No — Wave 0 |
| CWEXP-03 | CWE-272 AnyCall on CreateProcessAsUserA/W | unit | `cargo test --features internal phase_23_cwe272` | No — Wave 0 |
| CWEXP-03 | CWE-284 ArgAtIndex(4, GENERIC_ALL) | unit | `cargo test --features internal phase_23_cwe284` | No — Wave 0 |
| CWEXP-03 | CWE-427 AnyCall on putenv/SetDllDirectory | unit | `cargo test --features internal phase_23_cwe427` | No — Wave 0 |
| CWEXP-03 | CWE-479 two-pass: signal+helperBad→malloc fires | unit | `cargo test --features internal phase_23_cwe479` | No — Wave 0 |
| CWEXP-03 | CWE-479 good: signal+helperGood→no malloc, no finding | unit | `cargo test --features internal phase_23_cwe479` | No — Wave 0 |
| CWEXP-03 | CWE-591 VirtualAlloc without VirtualLock fires | unit | `cargo test --features internal phase_23_cwe591` | No — Wave 0 |
| CWEXP-03 | CWE-591 VirtualAlloc with VirtualLock: no finding | unit | `cargo test --features internal phase_23_cwe591` | No — Wave 0 |
| CWEXP-03 | CWE-762 delete after calloc fires (synthetic fixture) | unit | `cargo test --features internal phase_23_cwe762` | No — Wave 0 |
| CWEXP-03 | CWE-785 AnyCall on PathAppendA/W fires | unit | `cargo test --features internal phase_23_cwe785` | No — Wave 0 |
| CWEXP-03 | No regression on existing 41 CWEs (AUTOSAR fixture) | integration | `cargo test --features internal -- no_regression` | No — Wave 0 |
| CWEXP-03 | ANALYSIS.md updated with 8 new CWE rows | manual | n/a | No |

### Sampling Rate
- **Per task commit:** `cargo test --features internal -- phase_23`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] Unit tests for each new CWE rule in `tests/vulnerability_tests/ast_scanner_tests.rs` (or a new `phase_23_tests.rs`)
- [ ] Synthetic CWE-762 fixture in `tests/fixtures/c/` or `tests/fixtures/cpp/`: namespace-free file with `calloc + delete`
- [ ] Regression test verifying 41 existing CWEs still fire on AUTOSAR or the existing synthetic fixtures

## Security Domain

Phase 23 adds detection rules for security-relevant CWEs. No new code accepts external input; no
new attack surface. Security domain review: N/A for the implementation itself. The rules
_detect_ security issues but do not expose new ones.

## Sources

### Primary (HIGH confidence)
- `/Users/amean_lin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-c-0.24.2/grammar.js` — verified `name: 'c'`; no `delete`, `namespace`, `class` grammar rules [VERIFIED: codebase]
- `/Users/amean_lin/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-c-0.24.2/src/node-types.json` — verified `delete` absent from all node types [VERIFIED: codebase]
- `example_target_repos/juliet-test-suite-c/testcases/CWE479_*/` — 18 files, uniform signal+helperBad/helperGood pattern [VERIFIED: codebase]
- `example_target_repos/juliet-test-suite-c/testcases/CWE762_*/` — 6,092 `.cpp` files, all use `namespace`; no `.c` files [VERIFIED: codebase]
- `example_target_repos/juliet-test-suite-c/testcases/CWE427_*/` — 560 `.c` files, all use `PUTENV` macro; no direct `putenv()` calls [VERIFIED: codebase]
- `example_target_repos/juliet-test-suite-c/testcases/CWE591_*/` — 112 `.c` files, use `malloc()` not `VirtualAlloc()` [VERIFIED: codebase]
- `example_target_repos/juliet-test-suite-c/testcases/CWE284_*/` — 36/216 files are CreateDesktop variants with `GENERIC_ALL` [VERIFIED: codebase]
- `src/vulnerability/ast_scanner.rs` — existing ArgCheck enum, AstCweRule table, helper function patterns [VERIFIED: codebase]
- `.planning/phases/23-ast-cwes-domainspecific-expansion/23-CONTEXT.md` — all locked decisions D-01 through D-11 [VERIFIED: codebase]

### Secondary (MEDIUM confidence)
- `.planning/phases/21-ast-cwes-anycall-argpattern-expansion/21-CONTEXT.md` — `apply_division_rules()` return type and call structure [VERIFIED: codebase]
- `.planning/phases/20-argument-value-ast-migration/20-CONTEXT.md` — `ArgAtIndex` variant design [VERIFIED: codebase]
- `benchmark/juliet/ANALYSIS.md` — existing per-CWE TP/FP table showing 0 TPs for CWE-762 today [VERIFIED: codebase]

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — existing codebase, verified tree-sitter-c grammar source
- Architecture: HIGH — follows established Phase 21/22 patterns; tree-sitter-c limitations verified
- Pitfalls: HIGH — confirmed via direct Juliet fixture inspection and grammar.js grep
- CWE-762 approach: MEDIUM — exact implementation is planner's discretion; text-level approach is well-understood but not yet prototyped

**Research date:** 2026-05-12
**Valid until:** 2026-06-12 (stable domain; Juliet fixture doesn't change)
