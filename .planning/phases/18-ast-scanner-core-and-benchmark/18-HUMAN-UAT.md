---
status: partial
phase: 18-ast-scanner-core-and-benchmark
source: [18-VERIFICATION.md]
started: 2026-05-12T00:00:00Z
updated: 2026-05-12T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Run benchmark with real AUTOSAR fixture
expected: Stage `AUTOSAR_SampleProject_S32K144` at `../AUTOSAR_SampleProject_S32K144` and run `cargo test --features internal --test benchmark -- --nocapture`. `docs/BENCHMARK.md` should be updated with per-CWE rows for AST, cppcheck, and lexical scanners. This closes ROADMAP SC #4.
result: [pending]

### 2. Verify 14-CWE union coverage on AUTOSAR fixture
expected: After running the benchmark, confirm the union of AST + lexical scanner findings on the AUTOSAR fixture covers all 14 ROADMAP CWEs including D-08-deferred ones (CWE-362, 367, 416, 476) via lexical fallback. This closes ROADMAP SC #2.
result: [pending]

## Summary

total: 2
passed: 0
issues: 0
pending: 2
skipped: 0
blocked: 0

## Gaps
