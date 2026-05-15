# Requirements: v1.0.16 C/C++ Lexical CWE Scanner

**Milestone:** v1.0.16
**Status:** Active
**Created:** 2026-05-09

---

## v1 Requirements

### Scanner

- [ ] **SCAN-01**: User can scan C/C++ source files (.c, .h, .cpp, .hpp, .cc) for dangerous-function patterns
- [ ] **SCAN-02**: Scanner detects CWE-120 / CWE-242 — buffer overflow and inherently dangerous functions (`strcpy`, `strcat`, `gets`, `sprintf`, `vsprintf`, `memcpy`, `strncpy`, `strncat`, ~55 variants)
- [ ] **SCAN-03**: Scanner detects CWE-134 — use of externally-controlled format string (`printf`, `fprintf`, `vprintf`, `snprintf`, `syslog`, `wprintf` with non-literal format argument)
- [ ] **SCAN-04**: Scanner detects CWE-78 — OS command injection (`system`, `popen`, `execl`, `execlp`, `execle`, `execv`, `execvp`, `WinExec`, `ShellExecute`)
- [ ] **SCAN-05**: Scanner detects CWE-327 — use of broken/weak cryptographic algorithm (DES OpenSSL EVP functions, `rand`, `drand48`, `srand`, `srandom`, `lrand48` family)
- [ ] **SCAN-06**: Scanner detects CWE-377 — insecure temporary file (`tmpnam`, `tempnam`, `mktemp`, `GetTempFileName`)
- [ ] **SCAN-07**: Scanner detects CWE-190 — integer overflow without range check (`atoi`, `atol`, `atoll`, `_wtoi`, `_wtoi64`)
- [ ] **SCAN-08**: Scanner detects CWE-22 — path traversal (`chroot` without follow-up `chdir`, `getwd`, `realpath` without buffer size guard)
- [ ] **SCAN-09**: Scanner detects CWE-676 — use of potentially dangerous / deprecated functions (`getpass`, `memalign`, `usleep`, `gsignal`, `ssignal`, `ulimit`)
- [ ] **SCAN-10**: Scanner detects CWE-362 / CWE-367 — TOCTOU race conditions (`access`, `open`, `creat`, `rename`, `unlink`, `fopen`, `mkdir`, `rmdir`, `stat`, `lstat`, `mknod`, `mkfifo`)
- [ ] **SCAN-11**: Scanner detects CWE-20 — improper input validation (`gets`, `scanf`, `readlink`, `getenv`, `recv`, `recvfrom`, `fread`, `getchar`, `getopt`)
- [ ] **SCAN-12**: Scanner detects CWE-126 — buffer over-read (`strlen`, `wcslen`, `_tcslen`, `_mbslen`)
- [ ] **SCAN-13**: Scanner detects CWE-807 — reliance on untrusted inputs in security decision (`getenv`, `curl_getenv`, `getlogin`, `cuserid`)
- [ ] **SCAN-14**: Scan is scoped to component-mapped C/C++ directories only (not full source tree)
- [ ] **SCAN-15**: Scanner is gated behind `cargo feature = "internal"` — public binary excludes it at compile time

### Output

- [ ] **OUT-01**: CycloneDX 1.5 output includes SAST findings as `vulnerabilities[]` entries with `cwes[]`, `affects[].ref` to component bom-ref, `source.name = "radeis_sc2sbom static analysis"`, and `analysis.state: "in_triage"`
- [ ] **OUT-02**: Each CycloneDX finding includes file path and line number in `properties` (`sc2sbom:finding:file`, `sc2sbom:finding:line`, `sc2sbom:finding:function`)
- [ ] **OUT-03**: User sees a separate `_static_analysis.md` report file with per-component CWE summary table and file:line findings list when `--all` format flag is used
- [ ] **OUT-04**: CLI output includes disclaimer: "Pattern-based — complex data-flow vulnerabilities not covered"
- [ ] **OUT-05**: SPDX 2.3 output is unchanged (no per-finding CWE data; no native model exists)
- [ ] **OUT-06**: Existing `_report.md` (dependency inventory) is unchanged

### Quality

- [ ] **QA-01**: False positive rate benchmarked against xcar-linux corpus before finalizing rule set
- [x] **QA-02**: ~~xZETA ingestion behavior validated~~ — closed; our output is spec-compliant CycloneDX 1.5. Non-CVE `vulnerabilities[]` entries are distinguished by `source.name` per spec. xZETA-specific behavior is their responsibility to implement correctly.

---

## Future Requirements (deferred)

- CWE-416 (use-after-free), CWE-401 (memory leak), CWE-415 (double free), CWE-476 (null pointer dereference) — require dataflow analysis; lexical FP rate > 50%
- cppcheck 2.16+ subprocess integration — optional higher-confidence SAST results (`--with-cppcheck` flag)
- SARIF output — IDE/PR integration format for developer workflow
- Suppress-list support (`sc2sbom-nocheck` annotation or config file)
- Per-finding severity/confidence scoring
- ISO 26262 / ASIL tag on findings

---

## Out of Scope

| Excluded | Reason |
|----------|--------|
| Full dataflow / taint analysis | SAST scope; requires AST + interprocedural analysis — 10x complexity |
| Inter-file analysis | Requires build graph; out of SBOM tool scope |
| Exploitability scoring | Requires runtime context; emit `state: "in_triage"` instead |
| Non-C/C++ file scanning | Different patterns; high FP rate |
| Replacing cppcheck/Flawfinder | This is a compliance metadata enrichment tool, not a SAST replacement |
| Adding CWE data to SPDX output | No native per-finding model in SPDX 2.3 |
| Modifying existing `_report.md` | Inventory and security findings are separate concerns |

---

## Traceability

| REQ-ID | Phase | Status |
|--------|-------|--------|
| SCAN-01 | Phase 11 | Not started |
| SCAN-02 | Phase 11 | Not started |
| SCAN-03 | Phase 11 | Not started |
| SCAN-04 | Phase 11 | Not started |
| SCAN-05 | Phase 11 | Not started |
| SCAN-06 | Phase 11 | Not started |
| SCAN-07 | Phase 11 | Not started |
| SCAN-08 | Phase 11 | Not started |
| SCAN-09 | Phase 11 | Not started |
| SCAN-10 | Phase 11 | Not started |
| SCAN-11 | Phase 11 | Not started |
| SCAN-12 | Phase 11 | Not started |
| SCAN-13 | Phase 11 | Not started |
| SCAN-14 | Phase 11 | Not started |
| SCAN-15 | Phase 11 | Not started |
| OUT-01 | Phase 12 | Not started |
| OUT-02 | Phase 12 | Not started |
| OUT-03 | Phase 12 | Not started |
| OUT-04 | Phase 12 | Not started |
| OUT-05 | Phase 12 | Not started |
| OUT-06 | Phase 12 | Not started |
| QA-01 | Phase 11 | Not started |
| QA-02 | Phase 12 | Closed (spec-compliant; xZETA deferred) |

---

*Created: 2026-05-09 — v1.0.16 milestone*
