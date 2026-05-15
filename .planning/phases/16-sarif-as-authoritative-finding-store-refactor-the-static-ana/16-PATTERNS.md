# Phase 16: SARIF as Authoritative Finding Store — Pattern Map

**Mapped:** 2026-05-11
**Files analyzed:** 5 (4 modified, 1 new function in existing file)
**Analogs found:** 5 / 5

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/formats/sarif.rs` | writer/service | transform (findings → JSON) | `src/formats/sarif.rs` itself (additive) | exact — extend existing file |
| `src/vulnerability/cwe_scanner.rs` | service | transform + filter | `src/vulnerability/cwe_scanner.rs` itself (additive) | exact — extend existing file |
| `src/vulnerability/mod.rs` | config/re-export | — | `src/vulnerability/mod.rs` itself | exact — extend pub use list |
| `src/cli.rs` | config | request-response | `src/cli.rs` existing `--sarif-output` flag | exact — copy adjacent field pattern |
| `src/main.rs` | orchestrator | request-response | `src/main.rs` existing cppcheck + writer call sequence | exact — extend existing call sequence |

---

## Pattern Assignments

### `src/formats/sarif.rs` — SARIF-04 fingerprint + SARIF-05 baseline diff

**Analog:** The file itself (lines 1–163). All additions are additive to this file.

**Imports pattern** (lines 1–10, current state — add `sha2` and `std::collections::HashMap`/`HashSet`):
```rust
#![cfg(feature = "internal")]

use crate::vulnerability::cwe_scanner::SastFinding;
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::console::cwe_name;
```

New imports to add at the top of the file (after existing imports):
```rust
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
```

**Core struct pattern — `SarifResult` with `partialFingerprints`** (lines 47–52, current state — add one field):
```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    // NEW (SARIF-04): partial_fingerprints serialises as "partialFingerprints" via rename_all
    partial_fingerprints: HashMap<String, String>,
}
```

**Fingerprint helper function** (new, insert before `save_sarif_report`):
```rust
fn sarif_fingerprint(file_path: &str, line: u32, cwe_id: u32) -> String {
    let input = format!("{}:{}:CWE-{}", file_path, line, cwe_id);
    let digest = Sha256::digest(input.as_bytes());
    // GenericArray<u8, 32> implements LowerHex; first 16 hex chars = 64-bit prefix
    let hex = format!("{:x}", digest);
    hex[..16].to_string()
}
```

**Core pattern — populate `partial_fingerprints` in results iterator** (lines 117–142, current `results` builder — extend each `SarifResult`):
```rust
let results: Vec<SarifResult> = findings
    .iter()
    .map(|f| {
        let mut pf = HashMap::new();
        pf.insert(
            "primary/v1".to_string(),
            sarif_fingerprint(&f.file_path, f.line, f.cwe_id),
        );
        SarifResult {
            rule_id: format!("CWE-{}", f.cwe_id),
            message: SarifMessage {
                text: cwe_name(f.cwe_id).to_string(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: { /* existing uri logic */ },
                    },
                    region: SarifRegion { start_line: f.line },
                },
            }],
            partial_fingerprints: pf,
        }
    })
    .collect();
```

**SARIF-05 baseline extraction helper** (new function, add after `save_sarif_report`):
```rust
/// Load fingerprints from a SARIF baseline file.
/// Returns an empty set (with a warning to stderr) if the file is missing or invalid — never aborts.
pub fn extract_baseline_fingerprints(path: &Path) -> HashSet<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: could not read baseline {}: {}", path.display(), e);
            return HashSet::new();
        }
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Warning: invalid SARIF baseline {}: {}", path.display(), e);
            return HashSet::new();
        }
    };
    let mut fps = HashSet::new();
    if let Some(results) = json["runs"][0]["results"].as_array() {
        for r in results {
            if let Some(fp) = r["partialFingerprints"]["primary/v1"].as_str() {
                fps.insert(fp.to_string());
            } else {
                // Fallback: (uri, startLine, ruleId) tuple for pre-Phase-16 SARIF files
                let uri = r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                    .as_str().unwrap_or("");
                let line = r["locations"][0]["physicalLocation"]["region"]["startLine"]
                    .as_u64().unwrap_or(0);
                let rule = r["ruleId"].as_str().unwrap_or("");
                fps.insert(format!("{}:{}:{}", uri, line, rule));
            }
        }
    }
    fps
}
```

**SARIF-05 write-diff helper** (new function — mirrors `save_sarif_report` signature):
```rust
/// Write a diff SARIF containing only new findings (those not in the baseline).
/// Returns the count of new findings.
pub fn save_diff_sarif_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
    baseline_fingerprints: &HashSet<String>,
    sarif_output: Option<&str>,
) -> Result<usize> {
    // Filter to new findings only
    let new_findings: Vec<&SastFinding> = findings.iter().filter(|f| {
        let fp = sarif_fingerprint(&f.file_path, f.line, f.cwe_id);
        !baseline_fingerprints.contains(&fp)
    }).collect();

    let count = new_findings.len();
    if count == 0 {
        return Ok(0);
    }

    // Diff SARIF path: same as --sarif-output, or default with _diff suffix
    let diff_path = match sarif_output {
        Some(p) => PathBuf::from(p),
        None => out_dir.join(format!("{}_static_analysis_diff.sarif", project_name)),
    };
    // ... build SarifLog from new_findings and write (same structure as save_sarif_report)
    Ok(count)
}
```

**Error handling pattern** (lines 91–103, existing `save_sarif_report` — copy this pattern for new helpers):
```rust
// existing: path resolution + parent dir creation
let path = match sarif_path {
    Some(p) => PathBuf::from(p),
    None => out_dir.join(format!("{}_static_analysis.sarif", project_name)),
};
if let Some(parent) = path.parent() {
    if !parent.as_os_str().is_empty() {
        fs::create_dir_all(parent)?;
    }
}
// existing: final write
let json = serde_json::to_string_pretty(&log)?;
fs::write(&path, json)?;
eprintln!("SARIF report saved to: {}", path.display());
Ok(())
```

---

### `src/vulnerability/cwe_scanner.rs` — SARIF-07 suppression + SARIF-05 scanner dir return

**Analog:** The file itself. Two changes: (1) `run_cppcheck_scanner` return type change, (2) new `suppress_lexical_false_positives` function.

**Imports pattern** (lines 13–25, current state — `BTreeSet` already imported):
```rust
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
```

**CPPCHECK_COVERED_CWES const** (new, add near `CPPCHECK_CWE_OVERRIDES` at line 454):
```rust
/// CWEs that cppcheck's `--enable=warning,style,security` covers reliably.
/// Derived from the existing CPPCHECK_CWE_OVERRIDES table (D-01).
/// When cppcheck ran on a component dir and did NOT confirm a Lexical finding
/// for one of these CWEs, the lexical finding is suppressed (SARIF-07).
const CPPCHECK_COVERED_CWES: &[u32] = &[78, 120, 122, 134, 190, 242, 327, 369, 676, 704, 732, 762];
```

**`run_cppcheck_scanner` signature change** (lines 583–586, current return `Vec<SastFinding>` → tuple):
```rust
// BEFORE (line 583-586):
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&OsStr>,
) -> Vec<SastFinding>

// AFTER (SARIF-07: also return the set of dirs where cppcheck actually ran):
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&OsStr>,
) -> (Vec<SastFinding>, BTreeSet<PathBuf>)
```

Inside `run_cppcheck_scanner`, track dirs and return them. Add a `scanned_dirs: BTreeSet<PathBuf>` alongside `all_findings`:
```rust
let mut all_findings: Vec<SastFinding> = Vec::new();
let mut scanned_dirs: BTreeSet<PathBuf> = BTreeSet::new();  // NEW
// ... in the Ok(out) arm, after extending all_findings (line ~682):
scanned_dirs.insert(dir.clone());  // NEW: record this dir as scanned
// ... graceful-degradation path (preflight failed, line ~601):
return (Vec::new(), BTreeSet::new());  // was: Vec::new()
// ... at the end of the function (line ~705):
(all_findings, scanned_dirs)  // was: all_findings
```

**`suppress_lexical_false_positives` function** (new, add after `deduplicate_sast_findings`):
```rust
/// SARIF-07: Remove Lexical findings for CWEs that cppcheck covers when
/// cppcheck ran on that component's directory and did NOT confirm the site.
///
/// `cppcheck_scanned_dirs` — dirs where cppcheck actually ran (from run_cppcheck_scanner).
/// `cppcheck_confirmed` — (normalized_file_path, line, cwe_id) tuples that
///    survived dedup with source == Cppcheck or Both.
///
/// When cppcheck was not installed (scanned_dirs is empty), returns findings unchanged.
pub fn suppress_lexical_false_positives(
    findings: Vec<SastFinding>,
    cppcheck_scanned_dirs: &BTreeSet<PathBuf>,
    cppcheck_confirmed: &BTreeSet<(String, u32, u32)>,
) -> Vec<SastFinding> {
    if cppcheck_scanned_dirs.is_empty() {
        return findings; // cppcheck did not run — no suppression
    }
    findings.into_iter().filter(|f| {
        if f.source != SastSource::Lexical {
            return true; // keep Cppcheck and Both findings unconditionally
        }
        if !CPPCHECK_COVERED_CWES.contains(&f.cwe_id) {
            return true; // CWE not in cppcheck's reliable scope
        }
        let normalized = normalize_path(&f.file_path);
        let under_scanned_dir = cppcheck_scanned_dirs.iter().any(|dir| {
            Path::new(&normalized).starts_with(dir)
        });
        if !under_scanned_dir {
            return true; // file not under a cppcheck-scanned dir
        }
        // In scope + CWE covered: keep only if cppcheck confirmed this site
        let key = (normalized, f.line, f.cwe_id);
        cppcheck_confirmed.contains(&key)
    }).collect()
}
```

**Test pattern** (lines 763–970, existing inline `#[cfg(test)]` module — copy this structure for new tests):
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn suppress_lexical_cwe_covered_not_confirmed() {
        // Lexical finding for covered CWE in scanned dir, not confirmed → suppressed
        let dir = tempfile::tempdir().unwrap();
        let mut scanned_dirs = BTreeSet::new();
        scanned_dirs.insert(dir.path().to_path_buf());
        let confirmed: BTreeSet<(String, u32, u32)> = BTreeSet::new();
        let finding = SastFinding {
            cwe_id: 120,
            component_name: "lib".to_string(),
            component_ecosystem: "C/C++".to_string(),
            file_path: dir.path().join("foo.c").to_string_lossy().into_owned(),
            line: 5,
            source: SastSource::Lexical,
        };
        let result = suppress_lexical_false_positives(vec![finding], &scanned_dirs, &confirmed);
        assert!(result.is_empty(), "Expected suppression; got {:?}", result);
    }
}
```

---

### `src/vulnerability/mod.rs` — re-export `suppress_lexical_false_positives`

**Analog:** The file itself (line 12, current `pub use` list).

**Current pub use pattern** (line 12):
```rust
#[cfg(feature = "internal")]
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, parse_cppcheck_xml, run_cppcheck_scanner, run_lexical_scanner, SastFinding, SastSource};
```

**After Phase 16** (add `suppress_lexical_false_positives` to the same pub use):
```rust
#[cfg(feature = "internal")]
pub use cwe_scanner::{deduplicate_sast_findings, has_c_cpp_files, parse_cppcheck_xml,
    run_cppcheck_scanner, run_lexical_scanner, suppress_lexical_false_positives,
    SastFinding, SastSource};
```

---

### `src/cli.rs` — `--sarif-baseline` flag (SARIF-05)

**Analog:** The existing `--sarif-output` field (lines 278–282).

**Exact pattern to copy** (lines 278–282):
```rust
/// SARIF output file path for static analysis findings (v1.0.17)
/// Defaults to {out_dir}/{project_name}_static_analysis.sarif
#[cfg(feature = "internal")]
#[arg(long)]
pub sarif_output: Option<String>,
```

**New field to add immediately after `sarif_output`:**
```rust
/// SARIF baseline file for new-findings-only CI gate (v1.0.17)
/// When provided, compares current scan fingerprints to baseline; exits 1 if
/// new findings found, 0 if none. Bad/missing baseline: warn and continue.
#[cfg(feature = "internal")]
#[arg(long)]
pub sarif_baseline: Option<String>,
```

---

### `src/main.rs` — orchestration changes (SARIF-06, SARIF-07, SARIF-05 call site)

**Analog:** The file itself — extend the existing `#[cfg(feature = "internal")]` block and the `OutputFormat::Console` / `OutputFormat::All` arms.

**Current call sequence pattern** (lines 255–310, the two relevant spots):
```rust
// Existing scanner + dedup call (lines 255-261):
let cppcheck_findings =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);
sast_findings =
    crate::vulnerability::deduplicate_sast_findings(lexical_findings, cppcheck_findings);

// Existing writers (lines 308-310, inside OutputFormat::Console arm):
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;
```

**Phase 16 replacement call sequence** (extend the scanner+dedup block):
```rust
// SARIF-07: destructure new tuple return
let (cppcheck_findings, cppcheck_scanned_dirs) =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);

sast_findings =
    crate::vulnerability::deduplicate_sast_findings(lexical_findings, cppcheck_findings);

// SARIF-07: build confirmed-sites set from post-dedup slice (pitfall: must be post-dedup)
let cppcheck_confirmed: std::collections::BTreeSet<(String, u32, u32)> = sast_findings.iter()
    .filter(|f| f.source == crate::vulnerability::SastSource::Cppcheck
             || f.source == crate::vulnerability::SastSource::Both)
    .map(|f| (f.file_path.clone(), f.line, f.cwe_id))
    .collect();

// SARIF-07: suppress lexical false positives
sast_findings = crate::vulnerability::suppress_lexical_false_positives(
    sast_findings,
    &cppcheck_scanned_dirs,
    &cppcheck_confirmed,
);
// SARIF-06: both writers now see the same post-suppression slice
```

**SARIF-05 baseline diff block** (add after `save_sarif_report` in Console and All arms):
```rust
// SARIF-05: baseline diff (CI gate)
#[cfg(feature = "internal")]
if let Some(ref baseline_path) = args.sarif_baseline {
    let baseline_fps = formats::sarif::extract_baseline_fingerprints(
        std::path::Path::new(baseline_path)
    );
    let new_count = formats::sarif::save_diff_sarif_report(
        project_name,
        out_dir,
        &sast_findings,
        &baseline_fps,
        args.sarif_output.as_deref(),
    )?;
    if new_count > 0 {
        eprintln!("{} new finding(s) vs baseline — CI gate failed", new_count);
        std::process::exit(1);
    } else {
        eprintln!("No new findings vs baseline");
    }
}
```

**`--sarif-baseline` warning pattern** (mirror existing `--sarif-output` warning at lines 324–330):
```rust
// In SpdxJson / SpdxTagValue / CyclonedxJson arms — add alongside existing sarif_output warning:
#[cfg(feature = "internal")]
if args.sarif_baseline.is_some() {
    eprintln!(
        "Warning: --sarif-baseline has no effect with --format spdx-json; \
         use --format console or --format all"
    );
}
```

---

## Shared Patterns

### `#[cfg(feature = "internal")]` guard
**Source:** `src/cli.rs` (lines 122–124, 274–276) and `src/formats/sarif.rs` (line 1)
**Apply to:** Every new function, field, and module added in Phase 16

All new items are internal-only. Pattern:
- New file-level items in `sarif.rs`: already gated by `#![cfg(feature = "internal")]` at top of file — no per-item annotation needed
- New CLI field in `cli.rs`: `#[cfg(feature = "internal")]` attribute on the field
- New functions in `cwe_scanner.rs`: `#![cfg(feature = "internal")]` already at top of file — no per-function annotation needed
- New pub use in `mod.rs`: wrap in `#[cfg(feature = "internal")]` (same as line 11–12)

### `eprintln!` for diagnostic output
**Source:** `src/formats/sarif.rs` line 161; `src/vulnerability/cwe_scanner.rs` lines 598, 691–703
**Apply to:** All new warning and status messages

Pattern: use `eprintln!` for all diagnostic output (warnings, counts, status). Never `println!`. Existing examples:
```rust
eprintln!("SARIF report saved to: {}", path.display());
eprintln!("Warning: --sarif-output has no effect with --format spdx-json; ...");
eprintln!("cppcheck: {} findings from {} components", count, total);
```

### `serde_json::Value` for untyped JSON parsing
**Source:** `Cargo.toml` (`serde_json = "1.0"`)
**Apply to:** `extract_baseline_fingerprints` in `sarif.rs`

Pattern for graceful degradation (never `?`-propagate on baseline parse):
```rust
let json: serde_json::Value = match serde_json::from_str(&content) {
    Ok(v) => v,
    Err(e) => {
        eprintln!("Warning: invalid SARIF baseline {}: {}", path.display(), e);
        return HashSet::new();
    }
};
```

### `BTreeSet` for deterministic ordering
**Source:** `src/formats/sarif.rs` line 6 (`use std::collections::BTreeSet`) and lines 106–114 (rules dedup)
**Apply to:** `cppcheck_scanned_dirs` set in `run_cppcheck_scanner`; `cppcheck_confirmed` set in `main.rs`

Pattern: prefer `BTreeSet` over `HashSet` when ordering must be deterministic (e.g., for reproducible test output).

### `normalize_path` for path comparison
**Source:** `src/vulnerability/cwe_scanner.rs` lines 711–724
**Apply to:** `suppress_lexical_false_positives` path comparison

Pattern: always call `normalize_path(path_str)` before inserting into or looking up from the confirmed-sites set. This is the same normalization `deduplicate_sast_findings` applies (line 744).

---

## No Analog Found

All Phase 16 changes extend existing files. No entirely new files are needed.

| Capability | File | Reason |
|------------|------|--------|
| SHA-2 fingerprint helper | `src/formats/sarif.rs` (new fn, no prior analog) | First use of `sha2` in the formats layer; pattern from RESEARCH.md Code Examples section |
| Diff SARIF writer | `src/formats/sarif.rs` (new fn) | Mirrors `save_sarif_report` exactly; no external analog needed |

---

## Metadata

**Analog search scope:** `src/formats/`, `src/vulnerability/`, `src/cli.rs`, `src/main.rs`
**Files read:** 6 (`sarif.rs`, `cwe_scanner.rs`, `mod.rs`, `cli.rs`, `main.rs` scan, `Cargo.toml`)
**Pattern extraction date:** 2026-05-11
