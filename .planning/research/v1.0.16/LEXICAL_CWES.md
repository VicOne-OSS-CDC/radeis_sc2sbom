# Lexically Detectable CWEs for C/C++ Source Code

**Project:** radeis_sc2sbom v1.0.16
**Research date:** 2026-05-08
**Confidence:** HIGH (Flawfinder source verified; RATS XML verified; CASTLE benchmark cross-checked)

## Method and Scope

This file catalogues every CWE detectable by pure lexical/token-based analysis of C/C++ source
code — no dataflow analysis, no taint tracking, no AST construction. The canonical reference is
the Flawfinder ruleset (david-a-wheeler/flawfinder, `flawfinder.py`, `c_ruleset` dict, ~89 named
rules covering ~219 function/macro variants). RATS (`rats-c.xml`) and ITS4 were cross-checked for
additional coverage gaps.

Detection method definitions used throughout:
- **Function-name match** — any occurrence of the token as a call site or macro use.
- **Format-arg heuristic** — format-family function whose first non-file argument is not a string
  literal token (i.e., the token immediately following `(` or `,fd,` is not `"`).
- **Static-array declaration** — `char`, `TCHAR`, or `wchar_t` followed by `[` without `*`.
- **Return-value check** — presence of an allocation call whose result is not immediately assigned
  or compared (requires 1-token look-ahead; achievable without full AST).

---

## Master Table: CWE → Pattern → Automotive False-Positive Risk → Recommendation

| # | CWE ID | CWE Name | Trigger Pattern(s) | Detection Method | FP Risk (Automotive) | v1.0.16? |
|---|--------|----------|--------------------|-----------------|----------------------|----------|
| 1 | CWE-120 | Classic Buffer Overflow | `strcpy`, `strcat`, `sprintf`, `vsprintf`, `gets`, `scanf`, `fscanf`, `sscanf`, `memcpy`, `strncpy`, `strncat`, `streadd`, `strecpy`, `strtrns`, `cuserid`, plus ~40 Win32/wchar_t variants | Function-name match | **Low** — these are banned in automotive BSW; any hit is real signal | **Yes** |
| 2 | CWE-134 | Use of Externally-Controlled Format String | `printf`, `vprintf`, `fprintf`, `vfprintf`, `snprintf`, `vsnprintf`, `syslog`, `wprintf`, `fwprintf` where format arg is non-literal | Function-name match + format-arg heuristic | **Med** — format literal in same TU is safe; non-literal is true positive in ~60% of cases | **Yes** |
| 3 | CWE-119 | Improper Restriction of Memory Buffer Operations | `char[N]`, `TCHAR[N]`, `wchar_t[N]` static array declarations | Static-array declaration | **Med** — many legitimate fixed-size buffers in automotive; flag only, do not fail | **Yes** |
| 4 | CWE-20 | Improper Input Validation | `gets`, `scanf`, `readlink`, `getenv`, `recv`, `recvfrom`, `fread`, `getchar`, `getopt` | Function-name match | **Med** — in bare-metal ECUs most network I/O is CAN frames, not stdin; calibrate by target | **Yes** |
| 5 | CWE-78 | OS Command Injection | `system`, `popen`, `execl`, `execlp`, `execle`, `execv`, `execvp`, `WinExec`, `ShellExecute`, `CreateProcess`, `CreateProcessAsUser`, `CreateProcessWithLogon` | Function-name match | **Low in deeply embedded** — presence of `system()` in AUTOSAR stack is almost always wrong | **Yes** |
| 6 | CWE-377 | Insecure Temporary File | `tmpnam`, `tempnam`, `mktemp`, `tmpfile`, `mkstemp`, `GetTempFileName` | Function-name match | **Low** — these functions don't exist in typical AUTOSAR/embedded; any hit is noteworthy | **Yes** |
| 7 | CWE-362 | Race Condition (TOCTOU) | `access`, `chown`, `chgrp`, `chmod`, `vfork`, `readlink`, `fopen`, `open`, `creat`, `rename`, `unlink`, `mkdir`, `rmdir`, `lstat`, `stat` | Function-name match | **Med** — POSIX calls in automotive IVI/Linux platforms are real; bare-metal platforms don't use them | **Yes** |
| 8 | CWE-367 | TOCTOU Race Condition (check-use gap) | `access` specifically (check without corresponding `open` in same scope) | Function-name match (subset of CWE-362) | **Low** — `access()` call is almost universally a TOCTOU | **Yes** |
| 9 | CWE-327 | Use of Broken Cryptographic Algorithm | `crypt`, `crypt_r`, `EVP_des_ecb`, `EVP_des_cbc`, `EVP_desx_cbc`, `EVP_rc4_40`, `EVP_rc2_40_cbc`, `EVP_rc2_64_cbc`, `drand48`, `erand48`, `jrand48`, `lrand48`, `mrand48`, `nrand48`, `random`, `srand`, `srandom`, `srand48`, `seed48`, `lcong48`, `initstate`, `setstate`, `g_rand_*`, `strfry`, `memfrob` | Function-name match | **Low** — any use of DES or 48-bit PRNG in safety-critical code is a true finding | **Yes** |
| 10 | CWE-190 | Integer Overflow or Wraparound | `atoi`, `atol`, `atoll`, `_wtoi`, `_wtoi64` | Function-name match | **Low-Med** — `atoi` in parsing boot parameters is common; still a real concern | **Yes** |
| 11 | CWE-126 | Buffer Over-Read | `strlen`, `wcslen`, `_tcslen`, `_mbslen`, `equal`, `mismatch`, `is_permutation` | Function-name match | **Med** — strlen on a properly terminated string is safe; flag only non-null-guaranteed contexts | **Yes** |
| 12 | CWE-676 | Use of Potentially Dangerous Function | `getpw`, `getpass`, `gsignal`, `ssignal`, `memalign`, `ulimit`, `usleep` (obsolete/deprecated) | Function-name match | **Low** — these are effectively dead in modern automotive C; any occurrence is legacy risk | **Yes** |
| 13 | CWE-242 | Use of Inherently Dangerous Function | `gets`, `_getts` (subset of CWE-120 — no safe usage possible) | Function-name match | **Low** — no valid automotive usage; always flag | **Yes** |
| 14 | CWE-732 | Incorrect Permission Assignment | `umask`, `SetSecurityDescriptorDacl`, `AddAccessAllowedAce` | Function-name match | **Low on target** — Linux-based IVI units may call these; bare-metal never | **Maybe** |
| 15 | CWE-250 | Execution with Unnecessary Privileges | `chroot`, `RpcImpersonateClient`, `ImpersonateLoggedOnUser`, `SetThreadToken`, `RevertToSelf`, `ImpersonateSelf`, `InitializeCriticalSection` (Windows privilege APIs) | Function-name match | **Low relevance** — Windows-only; Linux IVI possible but rare | **Maybe** |
| 16 | CWE-22 | Path Traversal | `chroot` (without follow-up `chdir`), `realpath`, `getwd` | Function-name match | **Low** — any use of `chroot` without `chdir` is classic path-traversal setup; `realpath` needs buffer check | **Yes** |
| 17 | CWE-807 | Reliance on Untrusted Inputs in Security Decision | `getenv`, `curl_getenv`, `g_get_home_dir`, `g_get_tmp_dir`, `getlogin`, `cuserid` | Function-name match | **Med** — `getenv` used legitimately in build-time or host tools embedded in firmware toolchains | **Yes** |
| 18 | CWE-829 | Inclusion of Functionality from Untrusted Control Sphere | `LoadLibrary`, `LoadLibraryEx` | Function-name match | **Low relevance** — Windows-only; not an AUTOSAR concern | **Maybe** |
| 19 | CWE-248 | Uncaught Exception | `InitializeCriticalSection` (can throw under low memory on Windows) | Function-name match | **Low relevance** — Windows-only | **No** |
| 20 | CWE-400 | Uncontrolled Resource Consumption | `InitializeCriticalSection` (Windows lock that can raise exception) | Function-name match | **Low relevance** — Windows-only | **No** |
| 21 | CWE-785 | Use of Path Manipulation Function without Max-sized Buffer | `realpath` without explicit buffer size check | Function-name match + buffer-arg heuristic | **Med** — present in Linux IVI; `realpath(path, NULL)` is safe but `realpath(path, buf)` is not | **Maybe** |
| 22 | CWE-401 | Memory Leak (Missing Release) | `malloc`, `calloc`, `realloc` calls where assignment result is not eventually passed to `free` in same function scope | Return-value heuristic (very limited without CFG) | **High** — genuinely requires dataflow; lexical hits are unreliable; MISRA bans heap altogether | **No** |
| 23 | CWE-415 | Double Free | `free(p)` followed by `free(p)` on same identifier within ~10 lines | Token-pair proximity match | **High** — non-adjacent frees and pointer aliasing cause massive FP/FN; not reliable lexically | **No** |
| 24 | CWE-416 | Use After Free | `free(p)` followed by dereference of `p` within ~10 lines | Token-pair proximity match | **High** — same aliasing problem as CWE-415 | **No** |
| 25 | CWE-476 | NULL Pointer Dereference | `malloc`/`calloc` result used without NULL check in next statement | Return-value proximity heuristic | **High** — reliable detection requires dataflow; lexical approximation is very noisy | **No** |
| 26 | CWE-369 | Divide by Zero | Literal `/0` or `%0` in expression | Literal-denominator match | **Med** — catches only the trivial constant case; variable denominators need dataflow | **Maybe** |
| 27 | CWE-457 | Use of Uninitialized Variable | Variable declared without initializer then used | Declaration + use token proximity | **High** — requires scope tracking and variable liveness; pure token proximity is unreliable | **No** |
| 28 | CWE-590 | Free of Memory Not on the Heap | `free()` on stack-allocated or global variable | Token context (free of `&var` or array name) | **Med** — `free(&local_var)` pattern is detectable lexically when `&` is visible | **Maybe** |
| 29 | CWE-252 | Unchecked Return Value | `malloc`, `fopen`, `open`, `read`, `write` result not assigned or result discarded with cast `(void)` | Return-value assignment check | **High** — pervasive in automotive code intentionally; too many FPs without dataflow | **No** |

---

## Pattern Category Groupings

### Category 1: Dangerous Function Name — Direct Match
**CWEs covered:** 120, 134 (partial), 20 (partial), 78, 377, 327, 190, 676, 242, 22, 807
**Detection:** Exact token match against a fixed keyword set.
**Reliability:** HIGH. Zero ambiguity on whether the function was called. Only false positive
source is `#define` aliases or dead code — both are minor in automotive practice.
**Automotive FP rate:** Low. Automotive C coding standards (MISRA C:2012, AUTOSAR C++14) explicitly
ban most of these functions; any occurrence is a compliance finding regardless of exploitability.

### Category 2: Format String Non-Literal Argument
**CWEs covered:** 134
**Detection:** Match `printf`/`fprintf`/`syslog`/`snprintf` family; inspect next token after
opening `(` (or after `fd,` for `fprintf`). If it is not a string literal `"..."`, flag it.
**Reliability:** MEDIUM. Non-literal format strings are genuinely dangerous ~60% of the time.
Common false-positive source: `printf(msg)` where `msg` is a compile-time constant injected via
`#define` — not detectable at token level without macro expansion.
**Automotive FP rate:** Medium. Logging macros wrapping `printf` are common in diagnostic code.

### Category 3: Static Fixed-Size Array Declaration
**CWEs covered:** 119, 120 (partial)
**Detection:** `char|TCHAR|wchar_t` token followed by identifier token followed by `[`.
**Reliability:** MEDIUM-LOW. Every fixed-size character array is a candidate; most are fine.
Primarily useful for surfacing audit targets, not as direct vulnerability assertions.
**Automotive FP rate:** High. AUTOSAR and MISRA code is full of fixed-size arrays by design
(avoids dynamic allocation). Treat as informational only; do not count toward severity score.

### Category 4: Crypto / PRNG Weakness
**CWEs covered:** 327
**Detection:** Function-name match against DES OpenSSL EVP functions and `rand()`/`drand48()`
family.
**Reliability:** HIGH. No legitimate security use of DES-56 or 48-bit LFSR PRNG exists.
**Automotive FP rate:** Low. Any cryptographic function in an automotive context is either a
deliberate choice (flag for review) or dead code.

### Category 5: Integer Conversion Without Range Check
**CWEs covered:** 190
**Detection:** Token match on `atoi`, `atol`, `atoll`, `_wtoi`.
**Reliability:** MEDIUM. `atoi` is definitionally unsafe on overflow but is widely used in
non-security-critical parsing (e.g., reading config files on host tools).
**Automotive FP rate:** Low-Medium. ECU firmware rarely uses `atoi`; host-side tooling does.

### Category 6: Race Condition (POSIX File Operations)
**CWEs covered:** 362, 367
**Detection:** Function-name match on POSIX filesystem + privilege functions.
**Reliability:** MEDIUM. On Linux-based IVI platforms these are real TOCTOU candidates. On
bare-metal AUTOSAR targets the functions don't exist; hits indicate porting layer issues.
**Automotive FP rate:** Context-dependent. Low for IVI/Linux; effectively zero FP on bare-metal
(any hit = wrong code).

### Category 7: Temporary File Insecurity
**CWEs covered:** 377
**Detection:** Function-name match on `tmpnam`, `tempnam`, `mktemp`, `GetTempFileName`.
**Reliability:** HIGH. `tmpnam`/`mktemp` have no safe usage mode.
**Automotive FP rate:** Low. These functions should not appear in safety-critical firmware.

### Category 8: Shell / Process Execution
**CWEs covered:** 78
**Detection:** Function-name match on `system`, `popen`, exec family.
**Reliability:** HIGH for automotive context.
**Automotive FP rate:** Low. `system()` in an ECU firmware binary is almost always an oversight
or debugging artifact.

---

## Automotive / AUTOSAR Specific Notes

**Dynamic memory ban (MISRA C:2012 Rules 21.3 and 21.4):**
`malloc`, `calloc`, `realloc`, `free` are prohibited in MISRA-compliant automotive code. Their
presence is therefore a compliance violation before being a security finding. Lexically flagging
them maps to CWE-401 / CWE-415 / CWE-416 risk indicators — not as false positives, but as
"banned function" hits. Recommended: report as CWE-676 (potentially dangerous function) with
note "MISRA C:2012 Rule 21.3 violation" rather than claiming CWE-401/415/416 without dataflow
evidence.

**Privileged/OS-level functions in bare-metal targets:**
`chmod`, `chown`, `chgrp`, `umask`, `LoadLibrary`, `CreateProcess`, `RpcImpersonateClient` cannot
execute on an AUTOSAR Classic ECU. Any hit in a bare-metal binary is either:
(a) dead code in a ported open-source library, or
(b) host-side tool code accidentally mixed in.
Both are worth flagging even if not directly exploitable.

**`rand()` / `srand()` in automotive:**
Deterministic PRNGs are used legitimately in automotive for non-security purposes (e.g.,
jitter in retry logic, test harnesses). Flag as CWE-327 but with a note distinguishing
"security-relevant randomness" from "non-security use." Let the human reviewer decide.

**`atoi` in embedded toolchains:**
ECU software build chains embed host-side C utilities. Flawfinder-style `atoi` hits in
`tools/` or `utils/` directories have different risk profiles than hits in `src/` or `bsp/`.
Recommended: include file-path context in the output.

---

## ITS4 / Additional Legacy Scanner Coverage

ITS4 (predecessor to Flawfinder, circa 2000) detected a subset of CWE-120/242 via the same
function-name list. It adds no CWE coverage not already in Flawfinder.

RATS `rats-c.xml` adds one coverage gap compared to Flawfinder:
- **`creat`, `mknod`, `mkfifo`, `pathconf`, `scandir`, `dirname`, `basename`** as TOCTOU race
  candidates (CWE-362) — Flawfinder only flags `fopen` and `open` for this; RATS' list is broader.
  Recommended for v1.0.16: add `creat`, `mknod`, `mkfifo` to the CWE-362 match set.

---

## Recommended v1.0.16 Implementation Set (Yes + Maybe candidates)

Priority 1 — High confidence, low FP, covers most impactful CWEs:

| CWE | Trigger Count | Expected FP Rate |
|-----|---------------|-----------------|
| CWE-120 | ~55 function/macro tokens | Low |
| CWE-242 | 2 tokens (`gets`, `_getts`) | Near-zero |
| CWE-78 | 12 tokens | Low |
| CWE-327 | ~28 tokens | Low |
| CWE-377 | 6 tokens | Low |
| CWE-190 | 4 tokens | Low-Med |
| CWE-134 | 9 tokens + format-arg check | Med |
| CWE-22 | 3 tokens (`chroot`, `realpath`, `getwd`) | Low |
| CWE-807 | 6 tokens | Med |
| CWE-362/367 | ~18 tokens | Med |

Priority 2 — Worth including but require path-context or platform flag:

| CWE | Condition for inclusion |
|-----|------------------------|
| CWE-676 | Flag deprecated/obsolete functions with low severity |
| CWE-126 | Flag `strlen` family with informational severity |
| CWE-119 | Static char arrays — informational only |
| CWE-732 | Only for Linux/IVI target profiles |
| CWE-590 | `free(&local)` pattern — detectable lexically, add if trivial |
| CWE-369 | Literal `/0` denominator only |

Not recommended for v1.0.16 (require dataflow — lexical FP rate > 50%):

CWE-401, CWE-415, CWE-416, CWE-476, CWE-457, CWE-252, CWE-248, CWE-400

---

## Complete Flawfinder CWE Coverage Reference

The following CWE IDs appear in the verified Flawfinder `c_ruleset`:

CWE-20, CWE-22, CWE-119, CWE-120, CWE-126, CWE-134, CWE-190, CWE-242, CWE-248, CWE-250,
CWE-327, CWE-362, CWE-367, CWE-377, CWE-400, CWE-676, CWE-732, CWE-785, CWE-807, CWE-829

Additional CWEs not in Flawfinder but detectable lexically (with caveats):

CWE-369 (literal denominator only), CWE-590 (`free(&var)` pattern)

CWEs that appear detectable but require dataflow and are NOT reliably lexical:

CWE-401, CWE-415, CWE-416, CWE-457, CWE-476

---

## Sources

- Flawfinder source `c_ruleset`: https://github.com/david-a-wheeler/flawfinder/blob/master/flawfinder.py
- RATS `rats-c.xml`: https://github.com/andrew-d/rough-auditing-tool-for-security/blob/master/rats-c.xml
- Flawfinder CWE compatibility: https://cwe.mitre.org/compatible/questionnaires/28.html
- CASTLE benchmark (25 CWEs across 13 SAST tools): https://arxiv.org/html/2503.09433v1
- MISRA C:2012 dynamic memory prohibition: https://industrialmonitordirect.com/blogs/knowledgebase/embedded-c-memory-management-why-avoid-malloccalloc
- CWE-190 atoi / strtol: https://wiki.sei.cmu.edu/confluence/display/c/ERR34-C.+Detect+errors+when+converting+a+string+to+a+number
- CodeQL format string query (CWE-134): https://codeql.github.com/codeql-query-help/cpp/cpp-non-constant-format/
- Empirical study on Flawfinder false positive rate: https://arxiv.org/html/2407.12241v1
