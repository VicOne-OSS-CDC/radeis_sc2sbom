---
phase: "10"
phase_name: internal-feature-gate
status: partial
fix_scope: critical_warning
findings_in_scope: 8
fixed: 6
skipped: 2
iteration: 1
date: "2026-05-09"
---

## Code Review Fix Report — Phase 10

### Fixed (6/8)

**CR-01** (`2c17477`) — Removed file-level `#![cfg(feature = "internal")]` from `cyclonedx_tests.rs`, `spdx_tests.rs`, `production_mode_e2e_tests.rs`, and `safetensors_tests.rs`. Replaced with item-level `#[cfg(feature = "internal")]` on `SbomMode`/`Vulnerability` imports and call sites. Public formatter paths now have test coverage. Test count increased from 214 to 295 in public build.

**CR-02** (`92548c8`) — Updated `scripts/strip_vulnerability.sh` to match the two-line cfg-guarded form `#[cfg(feature = "internal")]\npub mod vulnerability;\n` in addition to the bare form.

**CR-03** (`96807dd`) — Added `build-public` CI job that runs `cargo build --release` and `cargo test` (no features) on every CI run. Added `build-public` to `release` job's `needs` list.

**WR-02** (`f947ab9`) — Changed `pub mod cwe_scanner;` to private `mod cwe_scanner;` in `src/vulnerability/mod.rs` — stub is internal until Phase 11 implements its API.

**WR-03** (`0a94471`) — Updated strip script section 8e to also remove `#[cfg(feature = "internal")] ` prefixes from `&args.sbom_mode` arguments.

**WR-05** (`c88a705`) — Added `has_vulns` guard before `enrich_cwe_ids` call in `src/main.rs` to skip NVD API calls when no vulnerabilities were found.

### Skipped (2/8)

**WR-01** — Redundant explicit field names in test construction sites. Stylistic only; no correctness impact. Broad refactor deferred.

**WR-04** — Stripped-binary SBOM validation in CI. Complex multi-step addition; CR-03's `build-public` job covers the primary regression risk.

### Verification

- `cargo test` (public build): 295 passed, 0 failed
- `cargo build --release` (public build): exits 0
- `cargo build --release --features internal` (internal build): exits 0
