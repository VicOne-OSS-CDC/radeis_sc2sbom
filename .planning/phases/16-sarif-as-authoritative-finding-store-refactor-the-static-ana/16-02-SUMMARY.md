---
phase: 16
plan: "02"
subsystem: sarif
tags: [sarif, fingerprint, sha256, baseline, tdd]
dependency_graph:
  requires: []
  provides: [sarif_fingerprint, extract_baseline_fingerprints, partial_fingerprints_field]
  affects: [src/formats/sarif.rs, src/formats/mod.rs]
tech_stack:
  added: [sha2 (already in deps), HashSet, HashMap]
  patterns: [sha256 fingerprinting, SARIF partialFingerprints, baseline diffing]
key_files:
  created:
    - tests/vulnerability_tests/sarif_fingerprint_tests.rs
  modified:
    - src/formats/sarif.rs
    - src/formats/mod.rs
    - tests/vulnerability_tests/mod.rs
decisions:
  - Added fingerprint_matches_known_value test beyond plan spec to validate sha256 preimage correctness
  - Used pub fn (not pub(crate)) for sarif_fingerprint per plan instruction to support Plan 03 cross-module access
  - Re-exported both helpers from formats/mod.rs under #[cfg(feature = "internal")] for clean public API
metrics:
  duration: "~8 minutes"
  completed: "2026-05-11"
  tasks_completed: 2
  files_changed: 4
---

# Phase 16 Plan 02: SARIF-04 Stable Fingerprints Summary

## One-liner

SHA-256-based 16-hex-char stable fingerprints in SARIF `partialFingerprints["primary/v1"]` with baseline-loading helper for cross-run diffing.

## What Was Built

### Task 1: sarif_fingerprint helper + partial_fingerprints field
- Added `pub fn sarif_fingerprint(file_path: &str, line: u32, cwe_id: u32) -> String` to `src/formats/sarif.rs`
- Computes `sha256("{file_path}:{line}:CWE-{cwe_id}")[..16]` as lowercase hex
- Added `partial_fingerprints: HashMap<String, String>` field to `SarifResult` struct (serializes as `"partialFingerprints"` via `rename_all = "camelCase"`)
- Populated `partialFingerprints["primary/v1"]` in the results iterator for every finding
- sha2/HashMap/HashSet imports added to sarif.rs

### Task 2: extract_baseline_fingerprints + unit tests
- Added `pub fn extract_baseline_fingerprints(path: &Path) -> HashSet<String>` to `src/formats/sarif.rs`
- Returns empty HashSet (with stderr warning) on missing or malformed file — never panics
- Falls back to `"<uri>:<line>:<ruleId>"` tuple key for old SARIF baselines without `partialFingerprints`
- Re-exported both `sarif_fingerprint` and `extract_baseline_fingerprints` from `src/formats/mod.rs`
- Created `tests/vulnerability_tests/sarif_fingerprint_tests.rs` with 12 tests
- Registered test module in `tests/vulnerability_tests/mod.rs`

## Verification

- `cargo build --features internal`: clean (0 errors)
- `cargo test --features internal sarif_fingerprint`: 12 passed, 0 failed
- Saved SARIF JSON contains `"partialFingerprints"` (camelCase), not `"partial_fingerprints"`
- Same (file, line, cwe) tuple always produces same 16-hex-char fingerprint (deterministic)

## Deviations from Plan

### Minor: 12 tests instead of 11

- **Found during:** Task 2 implementation
- **Issue:** Task 1's `<behavior>` section listed `fingerprint_matches_known_value` as a required test, but Task 2's `<verify>` expected exactly 11 tests. Adding both sets resulted in 12 tests.
- **Fix:** Included `fingerprint_matches_known_value` test that validates the sha256 preimage exactly. This is additional correctness coverage, not a regression.
- **Files modified:** `tests/vulnerability_tests/sarif_fingerprint_tests.rs`
- **Impact:** Positive — stronger validation of the fingerprint hash function correctness.

## Known Stubs

None — all functionality is fully implemented and wired.

## Threat Flags

None — no new network endpoints, auth paths, or trust boundary changes introduced.

## Self-Check

Files exist:
- [x] src/formats/sarif.rs
- [x] src/formats/mod.rs
- [x] tests/vulnerability_tests/sarif_fingerprint_tests.rs
- [x] tests/vulnerability_tests/mod.rs

Commits exist:
- [x] c504783 — feat(16-02): add sarif_fingerprint helper and partial_fingerprints to SarifResult
- [x] 74c872b — feat(16-02): add extract_baseline_fingerprints and SARIF-04 unit tests

## Self-Check: PASSED
