---
phase: 10-internal-feature-gate
plan: 01
subsystem: infra
tags: [rust, cargo-features, feature-flags, reqwest, cfg, vulnerability]

# Dependency graph
requires: []
provides:
  - internal Cargo feature declared with dep:reqwest membership
  - reqwest made optional (excluded from public build link graph)
  - vulnerability module gated at all four mount points (lib.rs, main.rs, models/mod.rs)
  - src/vulnerability/ subtree excluded from public compilation
affects:
  - 10-02 (Dependency.vulnerabilities field gating)
  - 10-03 (cli.rs/formatter vulnerability symbol gating)
  - 10-04 (integration verification)

# Tech tracking
tech-stack:
  added: [Cargo dep:reqwest optional feature syntax (Cargo 1.60+)]
  patterns:
    - "#[cfg(feature = \"internal\")] before mod/use/pub use declarations"
    - "#[cfg(feature = \"internal\")] { ... } block wrapper for scanning code"

key-files:
  created: []
  modified:
    - Cargo.toml
    - src/lib.rs
    - src/main.rs
    - src/models/mod.rs

key-decisions:
  - "Use dep:reqwest syntax (Cargo 1.60+) so reqwest is fully excluded from public binary link graph"
  - "Wrap if args.check_vulnerabilities block in #[cfg(feature = \"internal\")] { } rather than gating individual statements — preserves block structure"

patterns-established:
  - "Module-level cfg gate: add #[cfg(feature = \"internal\")] immediately above mod/use declaration"
  - "Block-level cfg gate: wrap multi-statement sections in #[cfg(feature = \"internal\")] { } at function scope"

requirements-completed: [GATE-01, GATE-02, GATE-03]

# Metrics
duration: 11min
completed: 2026-05-09
---

# Phase 10 Plan 01: Internal Feature Gate — Module Level Summary

**Cargo `internal` feature established with optional reqwest; vulnerability module excluded from public build at all four mount points (Cargo.toml, lib.rs, main.rs, models/mod.rs)**

## Performance

- **Duration:** 11 min
- **Started:** 2026-05-09T13:41:50Z
- **Completed:** 2026-05-09T13:52:38Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added `internal = ["dep:reqwest"]` feature to Cargo.toml; made reqwest `optional = true` — reqwest is no longer linked in public binary
- Gated `pub mod vulnerability;` in src/lib.rs, `mod vulnerability;` and `use vulnerability::{...}` in src/main.rs, and both `pub mod vulnerability;` / `pub use vulnerability::{...}` in src/models/mod.rs
- Wrapped the entire `if args.check_vulnerabilities { ... }` scanning block in `#[cfg(feature = "internal")] { }` in main.rs
- Internal build (`cargo build --release --features internal`) compiles cleanly with `--help` showing vulnerability flags
- Public build fails only on downstream consumer references to Vulnerability/FixAction/VulnerabilitySeverity symbols (expected; plan 02 addresses)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add `internal` feature and make reqwest optional in Cargo.toml** - `6b1a36c` (feat)
2. **Task 2: Gate `vulnerability` module declarations and main.rs scanning block** - `1e737cd` (feat)

## Files Created/Modified

- `Cargo.toml` - Added `internal = ["dep:reqwest"]` feature; changed reqwest to `optional = true`
- `src/lib.rs` - Added `#[cfg(feature = "internal")]` above `pub mod vulnerability;`
- `src/main.rs` - Added cfg gates on `mod vulnerability;`, `use vulnerability::{...}`, and scanning block
- `src/models/mod.rs` - Added cfg gates on `pub mod vulnerability;` and `pub use vulnerability::{...}`

## Decisions Made

- Used `dep:reqwest` syntax rather than a plain feature-gated import so that reqwest is completely absent from the public binary's link graph (T-10-02 mitigation)
- Wrapped the scanning block as `#[cfg(feature = "internal")] { if args.check_vulnerabilities { ... } }` rather than gating on `args.check_vulnerabilities` itself — this excludes the entire block from compilation, not just execution, ensuring symbols are absent from the public binary

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None. The `grep -A1 'cfg(feature = "internal")' src/main.rs | grep -q '^{'` verification command in the plan expected `{` at column 0, but the block opener `{` is indented (inside a function body). The code is correct — the verification command does not account for indentation. The structural pattern is present and correct.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Plan 02 can now gate `Dependency.vulnerabilities` field and downstream cli.rs/formatter consumers of vulnerability symbols
- The public build currently fails at those downstream sites — exactly the expected state for plan 02 to address
- No blockers

---
*Phase: 10-internal-feature-gate*
*Completed: 2026-05-09*

## Self-Check: PASSED

- Cargo.toml: FOUND
- src/lib.rs: FOUND
- src/main.rs: FOUND
- src/models/mod.rs: FOUND
- 10-01-SUMMARY.md: FOUND
- Commit 6b1a36c: FOUND
- Commit 1e737cd: FOUND
