---
status: partial
phase: 22-ast-cwes-structuralpattern-expansion
source: [22-VERIFICATION.md]
started: 2026-05-12T00:00:00Z
updated: 2026-05-12T00:00:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. ROADMAP CWE count (25→40 vs 26→41)
expected: ROADMAP says Phase 22 expands from 26→41 CWEs; ANALYSIS.md documents 25→40. Confirm 40 total CWEs is acceptable or update ROADMAP.
result: [pending]

### 2. CWE-256 and CWE-674 zero Juliet TPs
expected: SC#1 requires ≥1 TP on Juliet corpus. Both CWEs have 0 corpus TPs (documented corpus mismatch) but have synthetic unit test TPs. Confirm unit-test fallback satisfies SC#1.
result: [pending]

### 3. 8 of 15 CWEs exceed 40% FP gate
expected: SC#2 requires FP% ≤40%. 8 CWEs exceed this (478, 480, 483, 562, 570, 571, 587, 256). Task 4 was approved and these are tracked in backlog 999.2. Confirm approval covers SC#2 exceptions.
result: [pending]

## Summary

total: 3
passed: 0
issues: 0
pending: 3
skipped: 0
blocked: 0

## Gaps
