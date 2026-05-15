---
phase: 15-sarif-output
reviewed: 2026-05-10T00:00:00Z
depth: standard
files_reviewed: 7
files_reviewed_list:
  - src/cli.rs
  - src/formats/console.rs
  - src/formats/mod.rs
  - src/formats/sarif.rs
  - src/main.rs
  - tests/format_tests/mod.rs
  - tests/format_tests/sarif_tests.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 15: Code Review Report

**Reviewed:** 2026-05-10T00:00:00Z
**Depth:** standard
**Files Reviewed:** 7
**Status:** issues_found

## Summary

The phase adds SARIF 2.1 output for static analysis findings (`src/formats/sarif.rs`), a `--sarif-output` CLI flag (`src/cli.rs`), and wires everything together in `src/main.rs`. The SARIF serialisation logic is clean and the test coverage in `sarif_tests.rs` is thorough (empty findings, deduplication, schema fields, custom path, parent-dir creation). No security vulnerabilities were found.

Three correctness/robustness warnings were identified:

1. `--sarif-output` is silently ignored for `SpdxJson`, `SpdxTagValue`, and `CyclonedxJson` output formats — the user can pass the flag with no effect and receive no warning.
2. The `SarifArtifactLocation.uri` field emits raw OS file paths (e.g. `src/libfoo/buffer.c`) without the `file://` URI scheme required by the SARIF 2.1 spec, which means consumers that validate against the JSON schema will reject the output.
3. `console.rs` counts direct dependencies inconsistently between `print_summary_section` (includes `is_direct && is_dev` in `direct_count`) and `save_console_report` (excludes `is_direct && is_dev` from `direct_count`). This is a pre-existing bug but is in-scope because `save_console_report` is modified by this phase (SAST section was added).

---

## Warnings

### WR-01: `--sarif-output` silently ignored for SpdxJson / SpdxTagValue / CyclonedxJson formats

**File:** `src/main.rs:323-365`
**Issue:** `save_sarif_report` is called only inside the `OutputFormat::Console` arm (line 310) and the `OutputFormat::All` arm (line 396). When the user chooses `--format spdx-json`, `--format spdx-tag-value`, or `--format cyclonedx-json` together with `--sarif-output /some/path`, the flag is silently ignored — no file is written and no warning is emitted. This is confusing: the user asked for a file and nothing happens.

**Fix:** Either call `save_sarif_report` for all format arms, or emit a warning to stderr when `--sarif-output` is set but the selected format does not produce a SARIF file:
```rust
// After the match block, or at the start of each non-SARIF arm:
#[cfg(feature = "internal")]
if args.sarif_output.is_some() {
    match args.format {
        OutputFormat::SpdxJson | OutputFormat::SpdxTagValue | OutputFormat::CyclonedxJson => {
            eprintln!(
                "Warning: --sarif-output has no effect with --format {}; \
                 use --format console or --format all",
                // format name here
            );
        }
        _ => {}
    }
}
```
Alternatively, factor `save_sarif_report` into a shared helper called after every format arm.

---

### WR-02: SARIF `artifactLocation.uri` emits raw OS paths instead of URI-scheme paths

**File:** `src/formats/sarif.rs:127`
**Issue:** The `uri` field is populated directly from `f.file_path` (a raw filesystem path string such as `src/libfoo/buffer.c` or `/abs/path/file.c`). The SARIF 2.1 spec (§3.4.3) requires `artifactLocation.uri` to be a URI. Absolute paths must be encoded as `file:///abs/path/file.c`; relative paths are valid URIs as-is but absolute paths without the `file://` scheme are not. Many SARIF-consuming tools (GitHub Code Scanning, Visual Studio, SARIF Viewer) will fail to resolve the location or reject the document.

```rust
// Current (line 127):
uri: f.file_path.clone(),

// Fix — normalise to a URI:
uri: {
    let p = std::path::Path::new(&f.file_path);
    if p.is_absolute() {
        format!("file://{}", f.file_path.replace('\\', "/"))
    } else {
        f.file_path.clone()
    }
},
```

---

### WR-03: Inconsistent `direct_count` between `print_summary_section` and `save_console_report`

**File:** `src/formats/console.rs:939-951` (print_summary_section) vs `src/formats/console.rs:1173-1179` (save_console_report)
**Issue:** In `print_summary_section`, a dependency that is both `is_direct` and `is_dev` is counted once in `dev_count` **and** once in `direct_count` (line 948 increments `direct_count` unconditionally when `dep.is_direct`). In `save_console_report`, the same dependency is counted in `dev_count` only — it is explicitly excluded from `direct_count` by the `if dep.is_direct && !dep.is_dev` guard (line 1174). The printed console output and the saved markdown report therefore show different direct-dependency totals for the same SBOM. This phase introduced the SAST section into `save_console_report`, touching the surrounding counting logic and making this pre-existing inconsistency visible at review time.

**Fix:** Align both code paths. The most common convention is: a dependency that is both direct and dev contributes to `direct_count` **and** to `dev_count` (matching npm/cargo behaviour). Apply the `save_console_report` guard to `print_summary_section`, or vice versa — but make both agree.

```rust
// In print_summary_section, change lines 947-951 to match save_console_report:
if dep.is_direct {
    if !dep.is_dev {
        direct_count += 1;
    }
} else {
    transitive_count += 1;
}
```

---

## Info

### IN-01: `SarifRule` is missing the `shortDescription` field recommended by SARIF consumers

**File:** `src/formats/sarif.rs:40-44`
**Issue:** The SARIF 2.1 schema marks `shortDescription` as recommended for rules (§3.49.9). GitHub Code Scanning surfaces `shortDescription.text` in the UI. Without it, the rule name is empty in the GitHub annotations panel. The `name` field on `SarifRule` (which maps to the rule's programmatic name, not its human-readable description) does not fill this role.

**Fix:** Add a `short_description` field:
```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifRule {
    id: String,
    name: String,
    short_description: SarifMessage,  // { text: "..." }
    help_uri: String,
}
// and populate it with cwe_name(id)
```

---

### IN-02: Dead comment block in `src/main.rs` lines 439-471

**File:** `src/main.rs:439-471`
**Issue:** Lines 439–471 consist entirely of comment-only documentation stubs (e.g. `/// Parse ROS/ROS2 package.xml…`, `/// Normalize Python package name…`) with no associated function bodies. These appear to be leftover doc-comment fragments that no longer have corresponding functions in this file. They add noise and mislead readers into thinking functions exist nearby.

**Fix:** Remove the orphaned comment block. The actual functions presumably live in `src/parsers/`.

---

_Reviewed: 2026-05-10T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
