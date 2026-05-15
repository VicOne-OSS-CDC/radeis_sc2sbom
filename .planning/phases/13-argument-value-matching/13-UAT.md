---
status: complete
phase: 13-argument-value-matching
source: [13-01-SUMMARY.md]
started: 2026-05-10T00:00:00Z
updated: 2026-05-10T00:00:00Z
---

## Current Test

[testing complete]

## Tests

### 1. CWE-295 SSL_VERIFY_NONE detection
expected: When scanning C/C++ code containing `SSL_CTX_set_verify(ctx, SSL_VERIFY_NONE, NULL)`, the scanner reports a CWE-295 finding. Code using SSL_VERIFY_PEER does NOT produce a finding.
result: pass
verified_by: test_argval_cwe295_ssl_verify_none, test_argval_cwe295_secure_does_not_fire (both ok)

### 2. CWE-319 curl insecure options detection
expected: Scanning code with `curl_easy_setopt(curl, CURLOPT_SSL_VERIFYPEER, 0)` reports CWE-319. The secure variant `CURLOPT_SSL_VERIFYPEER, 2` does NOT fire. CURLOPT_SSL_VERIFYHOST with value 1 also fires; value 2 does not.
result: pass
verified_by: test_argval_cwe319_curl_verifypeer_zero, test_argval_cwe319_curl_use_ssl_none, test_argval_cwe319_curl_verifyhost_one, test_argval_cwe319_secure_does_not_fire (all ok)

### 3. CWE-732 permissive umask detection
expected: `umask(0)` triggers CWE-732. `umask(0077)` does NOT fire (non-zero octal). The digit boundary prevents false positives on octal values containing 0 as a digit.
result: pass
verified_by: test_argval_cwe732_umask_zero, test_argval_cwe732_umask_octal_does_not_fire, test_argval_cwe732_setsd_dacl_null (all ok)

### 4. CWE-369 division-by-literal-zero detection
expected: Code containing `int x = a / 0;` or `int y = b % 0;` generates a CWE-369 finding. Division by a non-zero literal (`/ 10`, `/ 0.5`) does NOT fire. The scanner is separate from the CWE_RULES table (CWE-369 is not a named-function rule).
result: pass
verified_by: test_argval_cwe369_div_by_zero, test_argval_cwe369_modulo_by_zero, test_argval_cwe369_zero_in_number_does_not_fire, test_argval_cwe369_decimal_zero_does_not_fire (all ok)

### 5. Duplicate finding deduplication
expected: When the same vulnerability appears in multiple scanner passes (e.g., file reachable via two component entries), the output contains only ONE finding per (file, line, CWE) triple — no duplicates in the SAST section of the SBOM/report.
result: pass
verified_by: test_run_lexical_scanner_dedups_by_file_line_cwe (ok)

### 6. Backward compatibility — existing CWE rules unchanged
expected: CWE detections from earlier phases (e.g., CWE-78 command injection, CWE-120 buffer overflow) still fire correctly after this change. The 14 pre-existing CWE_RULES entries continue to produce findings for their trigger patterns.
result: pass
verified_by: test_argval_05_name_only_rule_unaffected + full suite 328 passed, 0 failed

## Summary

total: 6
passed: 6
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
