---
phase: 14-cppcheck-integration
reviewed: 2026-05-10T12:00:00Z
depth: standard
files_reviewed: 8
files_reviewed_list:
  - src/cli.rs
  - src/main.rs
  - src/vulnerability/cwe_scanner.rs
  - src/vulnerability/mod.rs
  - tests/cyclonedx_sast_tests.rs
  - tests/format_tests/sast_report_tests.rs
  - tests/vulnerability_tests/cppcheck_scanner_tests.rs
  - tests/vulnerability_tests/mod.rs
findings:
  critical: 0
  warning: 4
  info: 2
  total: 6
status: issues_found
---

# Phase 14: Code Review Report

**Reviewed:** 2026-05-10T12:00:00Z
**Depth:** standard
**Files Reviewed:** 8
**Status:** issues_found

## Summary

Phase 14 adds cppcheck SAST integration: `SastSource` enum, XML v2 parser, subprocess driver (`run_cppcheck_scanner`), `--cppcheck-path` CLI flag, and pipeline deduplication (`deduplicate_sast_findings`). The prior CR-01 fix (non-zero exit discarding findings) is confirmed present and correct. WR-01 (non-UTF-8 path fallback), WR-02 (path absolutization for dedup), WR-03 (XML parse error surfaced via eprintln) fixes are all in place.

Four new warnings remain, plus two informational items, detailed below.

---

## Critical Issues

None.

---

## Warnings

### WR-01: SAST scan silently skipped when `--check-vulnerabilities` is not set

**File:** `src/main.rs:198`
**Issue:** Both `run_lexical_scanner` and `run_cppcheck_scanner` are called only inside the `if args.check_vulnerabilities { ... }` block (line 198). A user who passes `--cppcheck-path` without also passing `--check-vulnerabilities` will get no SAST output and no diagnostic explaining why. The `--cppcheck-path` flag is gated under `#[cfg(feature = "internal")]` and appears to be an independent capability, yet it is silently no-op'd without the vulnerability flag. This is a correctness/usability defect — the CLI accepts the flag but produces no effect, with zero user feedback.

**Fix:** Either (a) run SAST scanning unconditionally when `--check-vulnerabilities` is false but `component_dirs` is non-empty (and cppcheck is available), or (b) emit a warning when `cppcheck_path.is_some() && !args.check_vulnerabilities`:
```rust
#[cfg(feature = "internal")]
if args.cppcheck_path.is_some() && !args.check_vulnerabilities {
    eprintln!("Warning: --cppcheck-path has no effect without --check-vulnerabilities");
}
```

---

### WR-02: `completed_components` counts successful spawns, not scanned components — summary is misleading when cppcheck exits non-zero for a valid scan

**File:** `src/vulnerability/cwe_scanner.rs:655-681`
**Issue:** `completed_components` is incremented inside `Ok(out)` regardless of cppcheck's exit code (lines 681). When cppcheck exits with a code other than 0 or 1 (the "unexpected exit" branch at line 657), the warning is printed but the component is still counted as completed because `completed_components += 1` is inside the `Ok(out)` arm unconditionally. The summary line (line 688-692) prints "N findings from M components" where M includes components that were warned as having unexpected exits. This misrepresents coverage.

Separately: `completed_components` is never incremented for components skipped because `!dir.exists()` (line 619) or non-UTF-8 path (line 628), which is correct, but there is no count of skipped components in the summary, so the operator cannot tell how many were dropped vs. scanned.

**Fix:** Move `completed_components += 1` to after the `if !out.status.success()` guard, incrementing only when the exit code is 0 or 1 (both accepted). For unexpected exits, add a separate `skipped_components` counter:
```rust
Ok(out) => {
    let code = out.status.code().unwrap_or(-1);
    if !out.status.success() && code != 1 {
        eprintln!("cppcheck: unexpected exit {} for {}: {}", code, name,
            String::from_utf8_lossy(&out.stderr));
        skipped_components += 1;
        continue;  // do not parse partial/corrupt XML output
    }
    let raw_findings = parse_cppcheck_xml(&out.stderr, name, ecosystem);
    // ... absolutize ...
    all_findings.extend(resolved);
    completed_components += 1;
}
```
The `continue` also prevents parsing stdout/stderr from a failed invocation as valid XML, which could silently produce zero findings when the real problem is a bad flag or permission error.

---

### WR-03: `CURLOPT_SSL_VERIFYHOST` value `1` is deprecated but not actually insecure — false-positive CWE-319 rule

**File:** `src/vulnerability/cwe_scanner.rs:95`
**Issue:** The rule at line 95 fires CWE-319 when `CURLOPT_SSL_VERIFYHOST` is set to `1`. In libcurl, a value of `1` was meaningful in old versions (check that the Common Name exists) but was deprecated as of curl 7.28.1 — setting it to `1` is treated the same as `2` (full verification) in modern curl and explicitly documented as "not a security risk." The correct insecure value is `0`. Flagging `1` produces false positives on all codebases that use the deprecated-but-safe value, with no ability to suppress. This reduces trust in all CWE-319 findings.

**Fix:** Remove the `CURLOPT_SSL_VERIFYHOST = 1` entry from CWE_RULES (line 95) and update the test at line 846-852 accordingly. The only genuinely insecure curl verifyhost setting is `0` (already covered by line 94):
```rust
// Remove this line:
CweRule { cwe_id: 319, functions: &["curl_easy_setopt"], ..., arg_value_contains: Some(&["CURLOPT_SSL_VERIFYHOST", "1"]) },
```
And delete the `test_argval_cwe319_curl_verifyhost_one` test (line 846-852) which validates the false-positive behavior.

---

### WR-04: `deduplicate_sast_findings` uses `Path::canonicalize` which fails for paths that do not exist at dedup time — falls back to raw string comparison, breaking cross-scanner dedup

**File:** `src/vulnerability/cwe_scanner.rs:715-718` and `724-727`
**Issue:** The dedup key is built by calling `std::path::Path::new(&f.file_path).canonicalize()` with an `unwrap_or_else` fallback to the raw path. `canonicalize` on Linux/macOS resolves symlinks and makes the path absolute — but it also **requires the path to exist on disk at call time**. In CI environments, in containerized builds, or when scanning an archive extracted to a temp dir that gets cleaned up, the source files may no longer be present. In that case, canonicalize fails and the fallback uses the raw path string.

The lexical scanner produces absolute paths (via `path.to_string_lossy()` on WalkDir entries). The cppcheck scanner absolutizes relative paths using `dir.join(p)` (line 675). Neither of these goes through `canonicalize`, so the key for the same file could be:
- Lexical: `/repo/src/foo.c` (absolute, no symlinks resolved)
- Cppcheck: `/repo/src/foo.c` (absolute after join)
- Canonicalized (if file exists): `/repo/src/foo.c` (same, assuming no symlinks)

If a symlink is involved — e.g., `/repo/src` is a symlink to `/mnt/src` — canonicalize will produce `/mnt/src/foo.c` for any path that resolves, but the raw fallback would produce `/repo/src/foo.c`. A finding from the lexical scanner (using WalkDir with `follow_links(true)`) may resolve via the physical path while the fallback raw path does not, causing dedup to miss the collision and produce a `Both`-promoted entry as two separate entries.

**Fix:** Use consistent path normalization without filesystem access. A simple approach is to normalize using `std::path::Path::new(s).components().collect::<PathBuf>()`, which eliminates `.` and `..` components without requiring the file to exist:
```rust
fn normalize_path(s: &str) -> String {
    use std::path::{Component, PathBuf};
    let mut out = PathBuf::new();
    for c in std::path::Path::new(s).components() {
        match c {
            Component::ParentDir => { out.pop(); }
            Component::CurDir => {}
            other => out.push(other),
        }
    }
    out.to_string_lossy().into_owned()
}
```
Then use `normalize_path(&f.file_path)` instead of `canonicalize` in both loops.

---

## Info

### IN-01: `parse_cppcheck_xml` handles `Event::Start` for `<error>` but cppcheck emits `<error ... />` (self-closing) — dead branch

**File:** `src/vulnerability/cwe_scanner.rs:496-498`
**Issue:** The match arm at line 497 is `Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.name().as_ref() == b"error"`. Cppcheck XML v2 always emits `<error ... />` (self-closing, `Event::Empty`). The `Event::Start` branch for `<error>` is never reached in practice because start-tag errors have child `<location>` elements and cppcheck always uses the self-closing form for the outer error tag. The `Event::Start` arm is not harmful but is dead code. This is a minor quality issue.

**Fix:** If future cppcheck versions emit non-self-closing `<error>` tags, the current code would correctly set `current_id` / `current_cwe` but then miss the `<location>` because the `Event::Empty` match for `<location>` only fires inside `<error>...</error>` after state was set. The code is accidentally correct for the common case but fragile. Document the assumption:
```rust
// cppcheck XML v2 always emits <error ... /> (self-closing).
// The Event::Start arm is retained for forward-compatibility with any
// cppcheck version that switches to <error>...<location/>...</error> form,
// but has not been observed in practice.
Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
    if e.name().as_ref() == b"error" => { ... }
```

---

### IN-02: `--cppcheck-path` CLI flag is not documented in help text as requiring `--check-vulnerabilities`

**File:** `src/cli.rs:274-276`
**Issue:** The doc comment on `cppcheck_path` (line 274) reads: "Path to cppcheck binary. When provided, uses this binary instead of PATH lookup. (v1.0.17)". It does not mention that `--check-vulnerabilities` must also be set for the flag to have any effect. A user relying on `--help` output cannot discover this dependency.

**Fix:** Update the doc comment:
```rust
/// Path to cppcheck binary. When provided, uses this binary instead of PATH lookup.
/// Has no effect unless --check-vulnerabilities is also specified. (v1.0.17)
#[cfg(feature = "internal")]
#[arg(long)]
pub cppcheck_path: Option<PathBuf>,
```

---

_Reviewed: 2026-05-10T12:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
