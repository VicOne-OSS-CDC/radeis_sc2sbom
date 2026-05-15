# Phase 23: ast-cwes-domainSpecific-expansion — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-12
**Phase:** 23-ast-cwes-domainSpecific-expansion
**Areas discussed:** ArgCheck strategy per CWE, Windows-API rule inclusion, CWE-479 signal handler detection, Validation scope

---

## ArgCheck Strategy per CWE

### CWE-762 (Mismatched Memory Management Routines)

| Option | Description | Selected |
|--------|-------------|----------|
| AnyCall on delete/free | Flag delete and delete[] as CWE-762 AnyCall — 0% FP per cppcheck benchmark | |
| Skip CWE-762 | Too hard — requires cross-call tracking. Ship 7 CWEs instead. | |
| You decide | Claude picks based on FP tradeoff and AST-only tractability | ✓ |

**User's choice:** You decide
**Notes:** Claude decided: AnyCall on `delete`/`delete[]` operator-expressions. In mixed C/C++ code, `delete` on a C-allocated pointer is the anomaly. Mirrors cppcheck's 0% FP approach.

---

### CWE-591 (Sensitive Data in Improperly Locked Memory)

| Option | Description | Selected |
|--------|-------------|----------|
| AnyCall on VirtualAlloc | Flag every VirtualAlloc() call. High TP, FP risk on non-sensitive allocs. | |
| AnyCall on VirtualAlloc missing paired VirtualLock | Flag VirtualAlloc only if no VirtualLock in same function scope. New structural check. | ✓ |
| You decide | Claude picks based on implementation cost vs FP rate | |

**User's choice:** AnyCall on VirtualAlloc missing paired VirtualLock
**Notes:** Paired function-scope check: `apply_paired_lock_rules()` helper. No dataflow needed — same-function presence/absence check only.

---

## Windows-API Rule Inclusion

### Platform scope decision

| Option | Description | Selected |
|--------|-------------|----------|
| Include all — AnyCall | Win32 code appears in AUTOSAR MCAL/BSW. 0 TPs on Linux-pure fixtures acceptable. | ✓ |
| Cross-platform CWEs only | Skip Win32. Only ship CWE-427 and CWE-479. Phase 23 = 2 CWEs. | |
| Include but document as Win32-only | Include all 8, add code comment on each Win32 rule. No gate. | |

**User's choice:** Include all — AnyCall
**Notes:** All 8 CWEs included unconditionally. Module-level doc comment will note Win32-specific CWEs.

---

### CWE-284 ArgCheck precision

| Option | Description | Selected |
|--------|-------------|----------|
| AnyCall for both | CreateDesktopA/CreateProcessAsUser are rarely called safely. Simple. | |
| ArgAtIndex for CWE-284 | ArgAtIndex(4, &["GENERIC_ALL"]) on CreateDesktopA/W for tighter precision. CWE-272 uses AnyCall. | ✓ |

**User's choice:** ArgAtIndex for CWE-284
**Notes:** CWE-284 uses `ArgAtIndex(4, &["GENERIC_ALL"])` — matches Juliet bad-sink exactly. CWE-272 uses `AnyCall`.

---

## CWE-479 Signal Handler Detection

### Detection depth

| Option | Description | Selected |
|--------|-------------|----------|
| Two-pass: track signal-registered functions | Pass 1: collect handler names from signal() arg 1. Pass 2: scan those function bodies. Precise, low FP. | ✓ |
| AnyCall on malloc/free inside signal handler files | Flag any malloc/free in files with signal() call. Simple but high FP. | |
| You decide | Claude picks — two-pass preferred if tractable | |

**User's choice:** Two-pass: track signal-registered functions
**Notes:** Single-file scope. No cross-file tracking. Juliet fixtures have handler and registration in same file.

---

### Non-reentrant function list

| Option | Description | Selected |
|--------|-------------|----------|
| Core non-reentrant set | malloc, free, printf, fprintf, sprintf, snprintf, vprintf, vfprintf, exit, abort, syslog | ✓ |
| Expanded set | Core + calloc, realloc, fopen, fclose, fread, fwrite, getenv, setenv, strtok, rand, srand | |
| Minimal: malloc/free only | Only malloc and free. Matches Juliet bad-sink exactly. | |

**User's choice:** Core non-reentrant set (Recommended)

---

## Validation Scope

### Phase pattern

| Option | Description | Selected |
|--------|-------------|----------|
| Implement + validate in same phase | Run Juliet after implementation, update ANALYSIS.md. Matches Phase 21 pattern. | ✓ |
| Separate validation plan | Split into Plan A (implement) + Plan B (validate). Cleaner git history. | |

**User's choice:** Yes — implement + validate in same phase

---

### Structural helper location

| Option | Description | Selected |
|--------|-------------|----------|
| Same file, new helper functions | add apply_signal_handler_rules() and apply_paired_lock_rules() in ast_scanner.rs | ✓ |
| New module: ast_scanner_structural.rs | Extract structural checks into sibling module | |

**User's choice:** Same file, new helper functions

---

## Claude's Discretion

- **CWE-762 operator traversal:** Exact implementation of `delete_expression`/`delete[]` AST node detection (new helper vs inline vs new ArgCheck variant). Planner decides.
- **CWE-591 paired-check scope:** Full function body vs immediate block scope. Function-body preferred.
- **Function lists for CWE-114/272/427/785:** Researcher confirms from Juliet fixture filenames + CERT C documentation.

## Deferred Ideas

- CWE-591 cross-function VirtualLock tracking (interprocedural)
- CWE-479 cross-file signal handler detection
- CWE-762 malloc+free(new-allocated) mismatch without C++ delete
- CWE-427 registry-based DLL search path manipulation (RegSetValueEx)
