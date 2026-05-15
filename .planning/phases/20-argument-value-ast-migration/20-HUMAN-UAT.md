---
status: complete
phase: 20-argument-value-ast-migration
source: [20-VERIFICATION.md]
started: 2026-05-12T00:00:00Z
updated: 2026-05-12T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. ARGVAL-02 baseline diff — scanner output against AUTOSAR_SampleProject_S32K144 and curl
expected: CWE-295/319/732 finding counts match v1.0.17 SARIF baseline (no unexpected additions or drops from the ArgAtIndex migration)
result: pass

**Findings:**
- AUTOSAR_SampleProject_S32K144: CWE-295=0, CWE-319=0, CWE-732=0 (baseline=0, delta=0) ✓
- curl (apples-to-apples, identical flags): CWE-295=0, CWE-319=0, CWE-732=0 (delta=0) ✓
- The stored scan_reports/curl_1.0.17_internal baseline showed CWE-295=1 (openssl.c:3959), but running v1.0.17 itself with default flags also produces CWE-295=0. The stored result was generated with a non-default configuration — user confirmed the v1.0.17 result may not have required precision.
- ArgAtIndex rule for SSL_CTX_set_verify is correct (unit tests pass); the 0-finding result is a pre-existing scanner scoping behavior (component_dirs fallback), not a phase 20 regression.

## Summary

total: 1
passed: 1
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
