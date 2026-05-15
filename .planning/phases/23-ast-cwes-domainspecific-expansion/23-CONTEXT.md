# Phase 23: ast-cwes-domainSpecific-expansion — Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Expand `AST_CWE_RULES` in `ast_scanner.rs` from 41 to 49 CWEs by adding 8 domain-specific/API rules: CWE-114 (process control via dynamic library load), CWE-272 (least privilege via CreateProcessAsUser), CWE-284 (access control via overprivileged CreateDesktop), CWE-427 (uncontrolled DLL search path via setenv/SetDllDirectory), CWE-479 (non-reentrant calls inside signal handlers), CWE-591 (sensitive memory not locked via VirtualLock), CWE-762 (mismatched alloc/free — calloc+delete, new+free), CWE-785 (path functions without MAX_PATH buffer — PathAppend/realpath). Add two new structural scan helpers (`apply_signal_handler_rules`, `apply_paired_lock_rules`). Validate all 8 CWEs against Juliet corpus; update `benchmark/juliet/ANALYSIS.md` with final 49-CWE table.

</domain>

<decisions>
## Implementation Decisions

### CWE-762 (Mismatched Memory Management Routines)

- **D-01:** Use `AnyCall` on `delete` and `delete[]` operator-expressions for CWE-762. In C/C++ code that mixes C-alloc (`calloc`, `malloc`) with C++ dealloc (`delete`/`delete[]`), the `delete`/`delete[]` usage is the anomaly. The planner decides the exact operator-expression traversal (similar to `apply_division_rules()` walking `delete_expression` AST nodes). Matches the Juliet CWE-762 bad-sink pattern and mirrors cppcheck's 0% FP approach.

### CWE-591 (Sensitive Data in Improperly Locked Memory)

- **D-02:** Flag `VirtualAlloc` only when `VirtualLock` does NOT appear in the same function body. Implement as a new `apply_paired_lock_rules()` helper in `ast_scanner.rs` that: (1) collects all `call_expression` function names in the current function scope, (2) flags `VirtualAlloc` if `VirtualLock` is absent from that set. No cross-function or cross-file tracking — single function scope only.

### CWE-479 (Signal Handler Non-Reentrant Calls)

- **D-03:** Two-pass detection via a new `apply_signal_handler_rules()` helper:
  - Pass 1: collect all function-name identifiers passed as the 2nd argument to `signal()` calls in the file (these are the registered signal handler functions).
  - Pass 2: for each collected function name, find its function definition in the AST and scan its body for non-reentrant calls from the core set: `malloc`, `free`, `printf`, `fprintf`, `sprintf`, `snprintf`, `vprintf`, `vfprintf`, `exit`, `abort`, `syslog`.
  - No cross-file tracking needed — Juliet test cases have handler and registration in the same file.
- **D-04:** Single-file scope only. If the signal handler function is defined in a different translation unit, it is missed. Acceptable — same scoping constraint as all other AST rules.

### CWE-284 (Improper Access Control)

- **D-05:** Use `ArgAtIndex(4, &["GENERIC_ALL"])` on `CreateDesktopA` and `CreateDesktopW`. The 5th parameter (0-based index 4) is the access rights — firing only on `GENERIC_ALL` is the precise dangerous variant. Mirrors the Juliet bad-sink pattern (`GENERIC_ALL` = over-privileged desktop creation).

### CWE-114/272/427/785 and remaining Windows-API rules

- **D-06:** All 6 Windows-API CWEs (CWE-114, CWE-272, CWE-427, CWE-591, CWE-785 + CWE-284 with ArgAtIndex) use `AnyCall` for their primary functions:
  - CWE-114: `LoadLibraryA`, `LoadLibraryW`, `LoadLibraryExA`, `LoadLibraryExW`
  - CWE-272: `CreateProcessAsUserA`, `CreateProcessAsUserW`
  - CWE-427: `SetDllDirectoryA`, `SetDllDirectoryW`, `putenv`, `_putenv`, `setenv` (cross-platform)
  - CWE-785: `PathAppendA`, `PathAppendW`, `realpath`, `_fullpath`
- **D-07:** All Windows-API rules are included unconditionally — no compile-time platform gate. On Linux/AUTOSAR targets, these APIs won't appear in source and will produce 0 TPs (identical to CWE-362/367 in the current rule set). Document in module-level doc comment that CWEs 114/272/284/591/785 are Win32-specific.

### Structural Helpers

- **D-08:** `apply_signal_handler_rules()` and `apply_paired_lock_rules()` are added as new helper functions in `ast_scanner.rs` alongside `apply_division_rules()` (Phase 21 pattern). All three helpers follow the same per-file call structure. No new module or file — all AST scanner logic stays co-located.
- **D-09:** Both helpers are called from `scan_file_ast_or_lexical()` alongside the existing `apply_ast_rules()` call. The planner decides whether they return `Vec<SastFinding>` (like `apply_division_rules()`) or extend a mutable findings vec.

### Validation Strategy

- **D-10:** Implement all 8 new CWE rules first, then re-run `sc2sbom` against the Juliet corpus and update `benchmark/juliet/ANALYSIS.md` with new per-CWE TP/FP rows. Single-phase pattern matching Phase 21. No separate validation plan.
- **D-11:** FP gate is ≤40% per ROADMAP Phase 23 success criterion #3. Windows-API rules may show 0% TPs and 0% FPs on non-Windows fixtures — that is acceptable, not a failure.

### Claude's Discretion

- **CWE-762 operator traversal:** Exact implementation of `delete_expression` / `delete[]` AST node detection — whether to walk these as a new helper, inline in `apply_ast_rules`, or as a new `ArgCheck::DeleteOperator` variant. Planner decides.
- **CWE-591 paired-check scope:** Whether `apply_paired_lock_rules()` walks the full function body or only the immediate block scope. Function-body scope preferred but planner has discretion.
- **Function lists for CWE-114/272/427/785:** Exact function lists confirmed by researcher using Juliet fixture filenames (already visible) and CERT C documentation.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap

- `.planning/REQUIREMENTS.md` — CWEXP-03 (the requirement for this phase)
- `.planning/ROADMAP.md` §Phase 23 — success criteria: ≥1 TP per CWE on Juliet (where test cases exist), ≤40% FP, no regression on existing 41 CWEs, ANALYSIS.md updated with final 49-CWE table

### Benchmark / Ground Truth

- `benchmark/juliet/ANALYSIS.md` — **MUST READ** — existing Juliet full corpus results per CWE; success criterion #5 requires updating this file with 8 new CWE rows
- `benchmark/juliet/ast.json` — raw AST scanner findings per file (used to compute TP/FP table)

### Juliet Test Case Directories (all 8 CWEs confirmed present)

- `example_target_repos/juliet-test-suite-c/testcases/CWE114_Process_Control/` — LoadLibraryA/W patterns
- `example_target_repos/juliet-test-suite-c/testcases/CWE272_Least_Privilege_Violation/` — CreateProcessAsUserA/W
- `example_target_repos/juliet-test-suite-c/testcases/CWE284_Improper_Access_Control/` — CreateDesktopA/W with GENERIC_ALL
- `example_target_repos/juliet-test-suite-c/testcases/CWE427_Uncontrolled_Search_Path_Element/` — putenv/_putenv/SetDllDirectory
- `example_target_repos/juliet-test-suite-c/testcases/CWE479_Signal_Handler_Use_of_Non_Reentrant_Function/` — signal()+malloc/free bad-sink pattern
- `example_target_repos/juliet-test-suite-c/testcases/CWE591_Sensitive_Data_Storage_in_Improperly_Locked_Memory/` — VirtualAlloc without VirtualLock
- `example_target_repos/juliet-test-suite-c/testcases/CWE762_Mismatched_Memory_Management_Routines/` — calloc+delete, malloc+delete[] patterns
- `example_target_repos/juliet-test-suite-c/testcases/CWE785_Path_Manipulation_Function_Without_Max_Sized_Buffer/` — PathAppendA/W without MAX_PATH

### Primary Code to Modify

- `src/vulnerability/ast_scanner.rs` — primary file: add `apply_signal_handler_rules()`, `apply_paired_lock_rules()`, new `AstCweRule` entries, update module-level doc comment CWE coverage list

### Prior Phase Context

- `.planning/phases/21-ast-cwes-anycall-argpattern-expansion/21-CONTEXT.md` — D-01 (`apply_division_rules()` pattern), D-04 (`SizeofPointer` variant), D-15 (validate-after-implement pattern)
- `.planning/phases/20-argument-value-ast-migration/20-CONTEXT.md` — D-01 (`ArgAtIndex` variant design used in CWE-284 rule)
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — D-06 (ArgCheck enum design), benchmark infrastructure

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets

- `ArgCheck::ArgAtIndex(u8, &'static [&'static str])` in `ast_scanner.rs` — used for CWE-284 (`ArgAtIndex(4, &["GENERIC_ALL"])`). Already present from Phase 20.
- `ArgCheck::AnyCall` — used for CWE-114/272/427/762/785 and CWE-591's VirtualAlloc side.
- `apply_division_rules()` (Phase 21) — model for `apply_signal_handler_rules()` and `apply_paired_lock_rules()`. Same per-file call structure, same `Vec<SastFinding>` return.
- `AST_CWE_RULES` static table — append 6 new `AstCweRule` entries (CWE-114, 272, 284, 427, 785 + CWE-762 if implemented via a new table entry vs operator traversal).
- `args: Vec<Node>` collection in `apply_ast_rules()` — `ArgAtIndex(4, ...)` for CWE-284 indexes into this existing vec.

### Established Patterns

- `#[cfg(feature = "internal")]` gate at file top — all scanner code is behind this; no change.
- Per-CWE table entry format: `AstCweRule { cwe_id: N, functions: &["fn1", "fn2"], arg_check: ArgCheck::Variant }` — uniform for CWE-114/272/284/427/785.
- `SastSource::Ast` on all findings from `ast_scanner.rs` — unchanged.
- Test pattern: `run_ast_scanner()` or `apply_ast_rules()` with inline C string — mirrors for new CWE unit tests.

### Integration Points

- `scan_file_ast_or_lexical()` — the two new helpers (`apply_signal_handler_rules`, `apply_paired_lock_rules`) are called here, alongside `apply_ast_rules()`.
- `deduplicate_sast_findings(ast, lexical_fallback)` from Phase 19 — unchanged; new CWE findings flow through the same pipeline.
- SARIF writer, markdown report, CycloneDX serializer: consume `&[SastFinding]` — no downstream changes required (same as all prior CWE expansions).

</code_context>

<specifics>
## Specific Ideas

- **CWE-479 two-pass implementation:** The Juliet fixture at `CWE479_Signal_Handler_Use_of_Non_Reentrant_Function__basic_01.c` shows: `signal(SIGINT, helperBad)` in `CWE479_...bad()`, and `helperBad()` calls `malloc(10)` and `free(voidPointer)`. The two-pass must collect `"helperBad"` from the signal() arg-1 identifier, then find and scan the `helperBad` function definition. The researcher should verify that all Juliet CWE-479 variants follow this pattern (same-file handler registration).
- **CWE-591 paired check:** The Juliet fixture confirms: bad case = `VirtualAlloc()` without `VirtualLock()` in function body; good case = `VirtualAlloc()` followed by `VirtualLock(password, 100*sizeof(char))`. The `apply_paired_lock_rules()` function walks the current function's body for both calls and fires if `VirtualAlloc` present but `VirtualLock` absent.
- **CWE-284 GENERIC_ALL note:** The Juliet pattern is `CreateDesktopA(name, NULL, NULL, 0, GENERIC_ALL, NULL)` — GENERIC_ALL is at arg index 4 (0-based). The good variant uses a more restrictive access mask (not GENERIC_ALL). `ArgAtIndex(4, &["GENERIC_ALL"])` is precise and matches the Juliet bad-sink exactly.
- **CWE-762 Juliet structure:** Juliet CWE-762 test cases are in `s01/` subdirectory as `.cpp` files (C++ syntax: `calloc()` + `delete name`). The researcher should confirm tree-sitter-c parses `.cpp` files correctly — `is_c_cpp_source()` already includes `.cpp`.

</specifics>

<deferred>
## Deferred Ideas

- **CWE-591 cross-function VirtualLock tracking** — if `VirtualAlloc` and `VirtualLock` are called in different functions (caller + callee pattern), the paired check misses it. Requires interprocedural analysis. Deferred.
- **CWE-479 cross-file signal handler** — if the handler is registered in one TU and defined in another, the two-pass misses it. Deferred — same scoping limitation as all other AST rules.
- **CWE-762 malloc+free mismatch (non-C++ delete)** — mismatching `new` with `free()` in C-style code. The `delete_expression` AST node covers C++ `delete`/`delete[]`. If `free(ptr)` on a `new`-allocated pointer needs detection, that requires tracking alloc origin. Deferred.
- **CWE-427 registry-based search path manipulation** — Windows `RegSetValueEx` for `HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\KnownDLLs`. True DLL hijacking vector but out of scope for a simple call-site rule.

</deferred>

---

*Phase: 23-ast-cwes-domainspecific-expansion*
*Context gathered: 2026-05-12*
