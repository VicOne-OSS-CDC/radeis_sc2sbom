---
phase: 15-sarif-output
plan: "02"
subsystem: cli-wiring
tags:
  - sarif
  - cli
  - clap
  - main-wiring
dependency_graph:
  requires:
    - "15-01"
  provides:
    - SARIF-02
  affects:
    - src/cli.rs
    - src/main.rs
    - tests/format_tests/sarif_tests.rs
tech_stack:
  added: []
  patterns:
    - "#[cfg(feature = \"internal\")] field in clap Args struct"
    - "Option<String> -> Option<&str> via .as_deref()"
    - "dual call-site wiring after save_static_analysis_report"
key_files:
  created: []
  modified:
    - src/cli.rs
    - src/main.rs
    - tests/format_tests/sarif_tests.rs
decisions:
  - "sarif_output field type is Option<String> (not PathBuf) to match save_sarif_report's sarif_path: Option<&str> via .as_deref()"
  - "Both main.rs call sites identical in structure; indentation matches surrounding scope"
metrics:
  duration: "~15 minutes"
  completed: "2026-05-10"
  tasks_completed: 2
  tasks_total: 2
---

# Phase 15 Plan 02: CLI Wiring + main.rs Integration Summary

**One-liner:** Wired `--sarif-output <PATH>` CLI flag and two `save_sarif_report` call sites in main.rs, completing SARIF-02 path-override delivery.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add --sarif-output CLI flag | 0694e9d | src/cli.rs |
| 2 (RED) | Add path-override integration tests | 2a9da7c | tests/format_tests/sarif_tests.rs |
| 2 (GREEN) | Wire save_sarif_report at both call sites | ebb234a | src/main.rs |

## CLI Flag Confirmed in --help

Running `cargo run --features internal -- --help` shows:

```
      --sarif-output <SARIF_OUTPUT>
```

Field definition in `src/cli.rs`:
- Under `#[cfg(feature = "internal")]`
- `#[arg(long)]` only — no short flag, no default value
- Type: `Option<String>` (not `PathBuf`) for `.as_deref()` compatibility

## Both main.rs Call Sites Wired

**Import (line 29):**
```rust
#[cfg(feature = "internal")]
use formats::save_sarif_report;
```

**First call site (line 310) — Console format block:**
```rust
#[cfg(feature = "internal")]
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;
```
Follows immediately after `save_static_analysis_report(project_name, out_dir, &sast_findings)?;`

**Second call site (line 396) — All format block:**
```rust
#[cfg(feature = "internal")]
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;
```
Follows immediately after the analogous `save_static_analysis_report` call in the All format path.

Count verification: `grep -c 'save_sarif_report' src/main.rs` = 3 (1 import + 2 calls)

## Tests Added

3 new integration tests appended to `tests/format_tests/sarif_tests.rs`:

1. `test_sarif_custom_path_override` — custom path written; default path NOT written
2. `test_sarif_custom_path_creates_parent_dir` — nested parent dirs auto-created
3. `test_sarif_default_path_when_none` — regression check: None still uses default

**Total SARIF test count: 10** (7 from Plan 01 + 3 new)

`cargo test --features internal sarif` — 10/10 passed

## Full Suite Green

`cargo test --features internal` — all tests pass, zero regressions

Both feature configurations build clean:
- `cargo build --features internal` — Finished dev profile, 0 errors
- `cargo build` (no internal) — Finished dev profile, 0 errors

## Coverage Closure

| Requirement | Where Delivered | Status |
|-------------|-----------------|--------|
| SARIF-01: Write SARIF 2.1 file with all SAST findings | Plan 01 — src/formats/sarif.rs | Done |
| SARIF-02: --sarif-output flag overrides default path | Plan 02 — src/cli.rs + src/main.rs | Done |
| SARIF-03: SARIF runs at same call sites as static analysis report | Plan 02 — main.rs dual wiring | Done |

All decisions D-01, D-02, D-08 fully implemented.

## Deviations from Plan

None — plan executed exactly as written.

The three new tests passed during the TDD RED phase (not failing) because the `save_sarif_report` function's path-override behavior was already fully implemented in Plan 01. The GREEN gate (main.rs wiring) is the new behavior delivered in this plan.

## Self-Check: PASSED

- src/cli.rs modified with sarif_output field: FOUND
- src/main.rs import at line 29: FOUND
- src/main.rs call site at line 310: FOUND
- src/main.rs call site at line 396: FOUND
- tests/format_tests/sarif_tests.rs 3 new tests: FOUND
- Commit 0694e9d (Task 1): FOUND
- Commit 2a9da7c (RED): FOUND
- Commit ebb234a (GREEN): FOUND
