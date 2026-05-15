---
status: complete
phase: 12-static-analysis-report
source: [12-VERIFICATION.md]
started: 2026-05-10T00:00:00Z
updated: 2026-05-10T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. Stderr disclaimer emitted at runtime (RPT-03)

expected: Running `cargo run --features internal -- --output /tmp/sast_smoke <c-project-path> 2>&1 | grep "Pattern-based"` prints `Pattern-based — complex data-flow vulnerabilities not covered` on stderr, followed by `✓ Static analysis report saved to: ...`
result: pass

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
