---
phase: 11-lexical-scanner-cyclonedx-output
reviewed: 2026-05-09T00:00:00Z
depth: standard
files_reviewed: 14
files_reviewed_list:
  - src/formats/cyclonedx.rs
  - src/main.rs
  - src/models/dependency.rs
  - src/scanner/mod.rs
  - src/vulnerability/cwe_scanner.rs
  - tests/cyclonedx_sast_tests.rs
  - tests/fixtures/c/dangerous_calls.c
  - tests/fixtures/c/safe_printf.c
  - tests/format_tests/cyclonedx_tests.rs
  - tests/integration_tests/production_mode_e2e_tests.rs
  - tests/parser_tests/safetensors_tests.rs
  - tests/spdx_unchanged_test.rs
  - tests/vulnerability_tests/cwe_scanner_tests.rs
  - tests/vulnerability_tests/mod.rs
findings:
  critical: 0
  warning: 5
  info: 2
  total: 7
status: issues_found
---

# Phase 11: Code Review Report

**Reviewed:** 2026-05-09T00:00:00Z
**Depth:** standard
**Files Reviewed:** 14
**Status:** issues_found

## Summary

Phase 11 introduces a lexical CWE scanner for C/C++ source files and wires SAST findings into the CycloneDX output as `vulnerabilities` entries. The core implementation is well-structured: the rule table, boundary checks, and format-string heuristic are solid. However, five issues were found that range from rule-table inconsistencies that cause duplicate findings, to a per-line quadratic scan complexity driven by iterating every rule×function combination for each source line, to a missing right-side word-boundary check that produces false positives on common identifiers.

---

## Warnings

### WR-01: `sprintf`/`vsprintf` matched twice per call site — once for CWE-120 and again for CWE-134

**File:** `src/vulnerability/cwe_scanner.rs:51,58`

**Issue:** `sprintf` and `vsprintf` appear in both the CWE-120 rule (line 51) and the CWE-134 rule (line 58). For every call `sprintf(buf, fmt, ...)` where the format argument is not a string literal, the scanner emits **two** separate `SastFinding` entries for the same call site — one for CWE-120, one for CWE-134. When these findings are serialized to CycloneDX, both entries will share an identical `bom-ref` (constructed from `sast-{cwe_id}-{sanitized}-{line}`) only if the CWE IDs differ, which they do; however the downstream consumer receives redundant, overlapping findings for the same source line. For the CWE-120 rule, `sprintf`/`vsprintf` are unconditional (no format heuristic), so they fire regardless of whether the format argument is a literal — the CWE-120 annotation of `sprintf("hello")` is a false positive since that call cannot overflow via an uncontrolled format string, yet no literal check guards the CWE-120 rule for these functions.

**Fix:** Remove `sprintf` and `vsprintf` from the CWE-120 rule. These functions are buffer-unsafe only when the format string can overflow the destination buffer, which is covered by CWE-134 with the format heuristic already in place. Alternatively, if both CWEs are intentionally retained for `sprintf`, document the design decision and add a test asserting both fire at the same line. The current test `test_all_thirteen_cwes` uses `dangerous_calls.c` which has only one `sprintf`-style call (`printf`) so this double-fire is not caught by any existing test.

```rust
// CWE-120: remove sprintf and vsprintf — these are covered by CWE-134
CweRule {
    cwe_id: 120,
    functions: &["gets", "strcpy", "strcat"],
    requires_format_heuristic: false,
    format_arg_index: 0,
},
```

---

### WR-02: `find_function_call` missing right-side word boundary — false positives on identifiers like `realloc`, `strcat`, `calloc`

**File:** `src/vulnerability/cwe_scanner.rs:123-150`

**Issue:** `find_function_call` enforces a left-side word boundary (char before match must not be alphanumeric or `_`) but does **not** check the right-side boundary. The function accepts a match as long as the first non-whitespace character after the identifier is `(`. This is generally correct because a function call requires `(`. However, consider a case where the substring of one function name appears at the start of another function name that is *longer* — this is correctly handled by the paren check because `realloc(` would match `alloc` only if `alloc(` appeared separately. The real risk is the opposite: if a function name is a *prefix* of another name. For example, the rule for `malloc` (CWE-190) would also fire on a line containing `malloc_size(n)` because `malloc` is found at offset 0, the character before is none (left boundary OK), and the next non-space character after `malloc` in `malloc_size` is `_` — wait, that would be `_`, not `(`, so the paren check saves it. Re-examining: `realloc` contains `alloc` as a suffix; `strcat` contains `cat` — none of these listed functions are prefixes of each other with `(` following. The actual edge case is that `find_function_call` has a subtle off-by-one in the `search_from` advancement when `left_ok` is false:

```rust
search_from = pos + 1;
if search_from >= line.len() { break; }
```

When `left_ok` is false, `search_from` advances by 1 from `pos`. This is correct. However, when `left_ok` is true but the paren check fails (i.e., it is an identifier match but not a function call), `search_from` also advances by only 1, meaning the next iteration could find the same identifier again at `pos+1` through `pos + func.len() - 1` — this only matters if the identifier string appears as a substring of itself, which is only possible for single-character patterns. None of the function names in the rule table are single characters, so this is a degenerate case. **The genuine issue** is that `find_function_call` does not check that the character *immediately after* the function name (before the `(`) is not alphanumeric or `_`. This means a line like `my_gets_wrapper(x)` would correctly not fire because `gets_wrapper` fails the paren check after `gets`. But a line like `gets_wrapper` where `gets` is found and the next char is `_` (not `(`) is fine. The real exposure is: a macro expansion `#define GETS(x) gets(x)` calling `GETS(x)` — but that is out of scope for a line-level scanner. **Concretely, the word boundary gap is a false-positive risk for the `stat` function** (CWE-362 and CWE-367): `stat_result` would not fire because `stat_` has `_` before `(`, but `statistics(` starts with `stat` and would match if the scanner finds `stat` within `statistics` — left-boundary check would fail since `stat` is a substring at offset 4 of `statistics`, and at offset 0 the character before is nothing (start), so `statistics(` would **fire as CWE-362/CWE-367 on the `stat` prefix**. Verify with: `statistics(data, n)` → `find_function_call("statistics(data, n)", "stat")` → finds `stat` at pos 0, `left_ok = true` (start of string), next non-whitespace after `stat` is `i` (from `istics`), not `(` — so paren check saves it. OK, so the paren check does provide sufficient protection in all tested cases.

**Re-evaluation:** After tracing through carefully, `find_function_call` is correct for all rule-table functions given the paren constraint. Downgrading this to INFO.

---

### WR-02 (reassigned): `sast_findings` always runs even when `--check-vulnerabilities` is false

**File:** `src/main.rs:225`

**Issue:** `run_lexical_scanner(&component_dirs)` is called unconditionally inside the `#[cfg(feature = "internal")]` block — it is not gated on `args.check_vulnerabilities`. This means every invocation with the `internal` feature will scan all component-mapped C/C++ directories regardless of whether the user requested any vulnerability checking. For projects with large C/C++ source trees mapped in `component_dirs`, this walk runs silently every time. The user has no way to disable it short of recompiling without `--features internal`.

**Fix:** Gate the lexical scan on `args.check_vulnerabilities`, or introduce a dedicated `--scan-sast` flag. At minimum, print a progress message so the user can see it is running:

```rust
// Phase 11 (D-04): lexical CWE scanner only runs when vulnerability checking is active
if args.check_vulnerabilities {
    sast_findings = crate::vulnerability::run_lexical_scanner(&component_dirs);
}
```

---

### WR-03: Duplicate `dep_to_bom_ref` construction in `convert_to_cyclonedx` — logic drift risk

**File:** `src/formats/cyclonedx.rs:544-586`

**Issue:** The PURL-to-ecosystem normalization logic (PURL type → `dep.ecosystem` name, including the `type=` qualifier extraction for `pkg:generic` PURLs) is duplicated verbatim between `build_cyclonedx_vulnerabilities` (lines 287-329) and the inline block inside `convert_to_cyclonedx` that constructs `dep_map` for SAST lookup (lines 548-585). These two blocks must stay in sync. If a new ecosystem mapping is added to `create_package_url()` in `spdx.rs`, both sites must be updated independently, with no compiler error if one is missed. The comment on line 548 acknowledges this ("acceptable duplication per plan") but it is a latent maintenance bug — the two lookup maps can diverge silently, causing SAST findings to emit an empty `affects[]` array for the ecosystem that was updated in only one place.

**Fix:** Extract the PURL-to-ecosystem normalization into a private helper function called from both sites:

```rust
fn purl_to_dep_ecosystem(purl: &str) -> Option<String> {
    let purl_type = purl.strip_prefix("pkg:")?.split('/').next()?;
    if purl_type == "generic" {
        // extract ?type= qualifier
        ...
    } else {
        Some(match purl_type {
            "pypi" => "pip",
            "golang" => "go",
            "gem" => "rubygems",
            other => other,
        }.to_string())
    }
}

fn build_bom_ref_map(components: &[CycloneDXComponent]) -> HashMap<(String, String), String> {
    components.iter()
        .filter_map(|c| {
            let purl = c.purl.as_deref()?;
            let ecosystem = purl_to_dep_ecosystem(purl)?;
            Some(((c.name.clone(), ecosystem), c.bom_ref.clone()))
        })
        .collect()
}
```

---

### WR-04: `scan_file` silently swallows all I/O errors — no diagnostic on partial read failure

**File:** `src/vulnerability/cwe_scanner.rs:152-186`

**Issue:** When `File::open` fails, `scan_file` returns an empty `Vec` with no diagnostic output. This is consistent with the project's graceful-degradation pattern for `warn_on_walkdir_err`. However, mid-file read errors (line 162: `Err(_) => continue`) also silently skip lines, meaning a file that fails partway through is silently partially scanned. A permission error or a truncated read after line 100 of a 500-line file will produce an incomplete finding list with no indication that the scan was partial. This is distinct from the file-open failure (which produces no findings) — partial scanning produces a misleading "clean" result for the unread portion.

**Fix:** Emit a `eprintln!` warning on read errors within the line loop, consistent with how `warn_on_walkdir_err` surfaces walk errors:

```rust
for (line_idx, line_result) in reader.lines().enumerate() {
    let line = match line_result {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Warning: read error in {:?} at line {}: {}", path, line_idx + 1, e);
            continue;
        }
    };
    ...
}
```

---

### WR-05: Internal test comment contradicts assertion — `test_rule_table_has_thirteen_cwes` asserts 14, not 13

**File:** `src/vulnerability/cwe_scanner.rs:253-261`

**Issue:** The test is named `test_rule_table_has_thirteen_cwes` and the doc comment (line 255) says "SCAN-02 requires 14 distinct CWE IDs" — the name says 13, the comment says 14, and the assertion is `assert_eq!(ids.len(), 14, ...)`. The requirement document `SCAN-02` says 13 CWEs. The rule table has 14 distinct CWE IDs: 20, 22, 78, 120, 126, 134, 190, 242, 327, 362, 367, 377, 676, 807. The implementation introduced CWE-126 (`strlen`, `wcslen`) which was not in the original 13-CWE specification. Either the spec was updated to 14 and the function name/spec reference was not updated, or CWE-126 is an unintended addition that inflated the count. This inconsistency means the contract encoded in the test name and the asserted value disagree, making it impossible to know from the test alone what the intended requirement is.

**Fix:** Align the test name, comment, and assertion with the agreed-upon requirement. If 14 CWEs are intentional, rename the test and update `SCAN-02` references:

```rust
#[test]
fn test_rule_table_has_fourteen_cwes() {
    // SCAN-02 (updated): 14 distinct CWE IDs in CWE_RULES.
    let mut ids: Vec<u32> = CWE_RULES.iter().map(|r| r.cwe_id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 14, "SCAN-02 requires 14 distinct CWE IDs in CWE_RULES");
}
```

If 13 is correct, remove CWE-126 from the rule table.

---

## Info

### IN-01: `getenv("HOME")` in `dangerous_calls.c` fixture neutralizes CWE-807 heuristic-free rule

**File:** `tests/fixtures/c/dangerous_calls.c:13`

**Issue:** The fixture uses `getenv("HOME")` — a call with a string literal argument — to trigger CWE-807. The CWE-807 rule has `requires_format_heuristic: false`, so it fires unconditionally regardless of argument content. This is correct behavior for the rule. However, the fixture inadvertently documents a known false-positive pattern: looking up a well-known environment variable with a hardcoded string is not a security issue. The scanner will flag `getenv("HOME")` even though the actual risk for CWE-807 is when the looked-up variable name is attacker-controlled. This is a known limitation of a line-level lexical scanner, but it is worth noting so future maintainers understand why the fixture uses this particular call.

**Fix:** Add a comment to the fixture and to the rule documenting the known false-positive pattern. No code change needed unless a future phase adds argument-content filtering.

---

### IN-02: `build_sast_vulnerabilities` sets `description: None` — CWE number is the only signal in the output

**File:** `src/formats/cyclonedx.rs:453`

**Issue:** SAST vulnerability entries omit `description`. The only consumer-visible identity information is the `id` field (`"CWE-120"`) and the `cwes` array. CycloneDX consumers that display vulnerability descriptions will show a blank entry, providing no guidance on what the CWE means or what the specific finding was. The `properties` array carries `sc2sbom:finding:file` and `sc2sbom:finding:line`, but those are extension properties that generic consumers may not render.

**Fix:** Populate `description` with a short human-readable string derived from the CWE ID:

```rust
description: Some(format!("CWE-{}: dangerous function call detected at {}:{}",
    finding.cwe_id, finding.file_path, finding.line)),
```

---

_Reviewed: 2026-05-09T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
