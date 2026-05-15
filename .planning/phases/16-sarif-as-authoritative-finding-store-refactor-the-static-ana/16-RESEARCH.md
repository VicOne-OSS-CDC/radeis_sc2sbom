# Phase 16: SARIF as Authoritative Finding Store — Research

**Researched:** 2026-05-11
**Domain:** Rust SARIF serialisation, SHA-2 fingerprinting, baseline diffing, cppcheck suppression
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**SARIF-04 — Fingerprinting**
- Hash = `sha256(file_path + ":" + line + ":CWE-" + cwe_id)`, first 16 hex chars
- Written to each SARIF result in `partialFingerprints["primary/v1"]`
- Computed inline in `save_sarif_report` — NOT added to `SastFinding` struct
- `sha2` crate already in `Cargo.toml`; no new dependency needed

**SARIF-05 — `--sarif-baseline`**
- CLI flag `--sarif-baseline <file>` (internal-gated)
- Load baseline SARIF, extract `partialFingerprints["primary/v1"]`; fall back to `(uri, startLine, ruleId)` tuple when fingerprints absent
- New findings (in current, not in baseline) → write to diff SARIF, print count to stderr, exit 1
- Fixed findings (in baseline, not in current) → silently omitted from diff output
- Bad baseline file → warn to stderr, continue with full scan (do NOT abort)
- Diff SARIF written to same path as `--sarif-output`, or default path with `_diff` suffix

**SARIF-06 — Markdown and SARIF consistency**
- Both `_static_analysis.md` and `_static_analysis.sarif` written from the same `&[SastFinding]` slice
- No parse-roundtrip; "authoritative" means shared in-memory slice, not read-back-from-file
- Assertion in `#[cfg(test)]`: markdown row count == SARIF results array length
- Suppression (SARIF-07) must complete before both writers are called

**SARIF-07 — Cppcheck scope suppression**
- Drop `SastSource::Lexical` findings for CWEs in `CPPCHECK_COVERED_CWES` when cppcheck ran on that component dir and did NOT confirm that `(file, line, CWE)` site
- `CPPCHECK_COVERED_CWES`: `{78, 120, 122, 134, 190, 242, 327, 369, 676, 704, 732, 762}`
- CWE set derived from existing `CPPCHECK_CWE_OVERRIDES` table in `cwe_scanner.rs`
- When cppcheck not installed: no suppression, all lexical findings pass through unchanged
- `run_cppcheck_scanner` must also return the set of directories it actually scanned

### Claude's Discretion

None specified beyond the locked decisions above.

### Deferred Ideas (OUT OF SCOPE)

- CSV output format
- Content-based fingerprinting
- SARIF suppression file / user-configurable suppress list (CPPCHECK-F1)
- cppcheck timing annotations (CPPCHECK-F2)
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SARIF-04 | Each SARIF result includes `partialFingerprints["primary/v1"]` computed as `sha256(file_path + ":" + line + ":CWE-" + cwe_id)[..16]` | `sha2 = "0.10.9"` already in `Cargo.lock`; inline computation in `save_sarif_report` |
| SARIF-05 | `--sarif-baseline <file>` compares fingerprints; exits 1 on new findings, 0 if none | `serde_json` already available; SARIF is plain JSON; new CLI flag needs `#[cfg(feature = "internal")]` |
| SARIF-06 | `_static_analysis.md` and `_static_analysis.sarif` report identical findings (both written from same post-suppression slice) | Both writers already accept `&[SastFinding]`; call order in main.rs must be: suppress → both writers |
| SARIF-07 | Lexical findings for CWEs in `CPPCHECK_COVERED_CWES` suppressed when cppcheck ran on that dir and did not confirm the site | `run_cppcheck_scanner` currently returns `Vec<SastFinding>`; must also return `BTreeSet<PathBuf>` of scanned dirs |
</phase_requirements>

---

## Summary

Phase 16 promotes SARIF from a secondary output format to the single authoritative finding representation. The four requirements form a dependency chain: SARIF-07 (suppression) modifies the `Vec<SastFinding>` slice, SARIF-04 (fingerprinting) annotates SARIF results during serialisation, SARIF-06 (consistency) requires both writers share the suppressed slice, and SARIF-05 (baseline diff) loads a prior SARIF and compares fingerprints post-serialisation.

All required Rust crates are already in `Cargo.toml`. The `sha2 = "0.10.9"` crate is already locked. The `serde_json` and `serde` crates are already present and cover both SARIF write and SARIF read (baseline load). The `clap` derive macro is already used for all other CLI flags; adding `--sarif-baseline` is straightforward. No new dependencies are needed.

The highest-risk change is the `run_cppcheck_scanner` signature change: it currently returns `Vec<SastFinding>` but SARIF-07 needs it to also return a `BTreeSet<PathBuf>` of directories where cppcheck actually ran. This touches the call site in `main.rs` and the function signature in `cwe_scanner.rs`. Everything else is additive.

**Primary recommendation:** Implement in wave order: (1) add suppression + scanner dir return, (2) add fingerprinting to SARIF writer, (3) add baseline diff CLI flag, (4) add consistency assertion tests. Each wave is independently testable.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| SHA-2 fingerprinting | `formats/sarif.rs` (writer) | — | Fingerprint needed only in SARIF output; keep `SastFinding` struct lean per CONTEXT.md |
| Cppcheck scope suppression | `vulnerability/cwe_scanner.rs` | `main.rs` (call site) | Suppression logic belongs with the scanner that produces the finding sets |
| Baseline diff (`--sarif-baseline`) | `main.rs` (orchestrator) + `formats/sarif.rs` | `cli.rs` (flag) | Diff is a post-scan comparison step; SARIF reader/writer live in `formats/sarif.rs` |
| Markdown/SARIF consistency | `main.rs` (call ordering) | test assertion | Consistency is enforced by call order, not by a library |
| CLI flag `--sarif-baseline` | `cli.rs` | — | All other CLI flags live here; `#[cfg(feature = "internal")]` gate required |

---

## Standard Stack

### Core (already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `sha2` | 0.10.9 [VERIFIED: Cargo.lock] | SHA-256 digest for fingerprint | Already locked; RustCrypto standard |
| `serde_json` | 1.0 [VERIFIED: Cargo.toml] | SARIF read (baseline load) + write | Already used for all JSON output |
| `clap` derive | 4.5 [VERIFIED: Cargo.toml] | `--sarif-baseline` CLI flag | Already drives all CLI flags |

### No New Dependencies Required

All capabilities for Phase 16 are available through existing crate dependencies. [VERIFIED: Cargo.toml]

**Version verification:**
```
sha2: 0.10.9 (Cargo.lock — confirmed present)
serde_json: 1.0.x (Cargo.toml — confirmed present)
clap: 4.5.x (Cargo.toml — confirmed present)
```

---

## Architecture Patterns

### System Architecture Diagram

```
component_dirs
    |
    v
run_lexical_scanner()       run_cppcheck_scanner()
    |                           |           |
    |                    Vec<SastFinding>   BTreeSet<PathBuf>  <- NEW: scanned_dirs return
    |                           |
    +---------> deduplicate_sast_findings(lexical, cppcheck)
                        |
                        v
               Vec<SastFinding>  (source = Lexical|Cppcheck|Both)
                        |
                        v
        suppress_lexical_false_positives(          <- NEW (SARIF-07)
            sast_findings,
            cppcheck_scanned_dirs,
            cppcheck_confirmed_sites
        )
                        |
                        v
               Vec<SastFinding>  (post-suppression, CANONICAL slice)
                        |
           +------------+------------+
           |                         |
           v                         v
  save_static_analysis_report()   save_sarif_report()
  (markdown writer —               (SARIF writer —
   unchanged, just receives         computes fingerprint
   post-suppression slice)          inline per finding,
                                    writes partialFingerprints)
                                         |
                                         v
                            if args.sarif_baseline.is_some():
                                load_baseline_sarif()
                                diff_by_fingerprint()
                                write_diff_sarif()
                                exit(1) if new findings, else exit(0)
```

### Recommended Project Structure (changes only)

```
src/
├── cli.rs                   -- add: sarif_baseline: Option<String> (cfg internal)
├── formats/
│   └── sarif.rs             -- add: fingerprint computation, baseline load/diff
└── vulnerability/
    └── cwe_scanner.rs       -- add: suppress_lexical_false_positives(),
                             --      CPPCHECK_COVERED_CWES const,
                             --      run_cppcheck_scanner() returns (Vec<SastFinding>, BTreeSet<PathBuf>)

tests/
└── vulnerability_tests/
    └── cwe_scanner_tests.rs -- add: suppression tests
    (new file or extend)     -- add: sarif fingerprint / baseline diff tests
```

### Pattern 1: SHA-256 Fingerprint (inline in SARIF writer)

```rust
// Source: sha2 crate docs — RustCrypto standard pattern
use sha2::{Digest, Sha256};

fn compute_fingerprint(file_path: &str, line: u32, cwe_id: u32) -> String {
    let input = format!("{}:{}:CWE-{}", file_path, line, cwe_id);
    let hash = Sha256::digest(input.as_bytes());
    // First 16 hex chars = 64-bit prefix
    format!("{:x}", hash)[..16].to_string()
}
```

The `sha2` crate is already imported via `Cargo.toml` (confirmed present). [VERIFIED: Cargo.toml, Cargo.lock]

### Pattern 2: `partialFingerprints` in SARIF result struct

The SARIF 2.1.0 spec defines `partialFingerprints` as `Map<String, String>` on each result. [ASSUMED: SARIF 2.1.0 spec — matches standard usage pattern]

Add to `SarifResult` in `formats/sarif.rs`:

```rust
use std::collections::HashMap;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SarifResult {
    rule_id: String,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
    partial_fingerprints: HashMap<String, String>,  // NEW
}
```

Populate in the `results` iterator:

```rust
let mut pf = HashMap::new();
pf.insert(
    "primary/v1".to_string(),
    compute_fingerprint(&f.file_path, f.line, f.cwe_id),
);
SarifResult {
    rule_id: ...,
    message: ...,
    locations: ...,
    partial_fingerprints: pf,
}
```

### Pattern 3: Baseline load and fingerprint extraction

```rust
fn load_baseline_fingerprints(path: &Path) -> Option<HashSet<String>> {
    let content = std::fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let results = json["runs"][0]["results"].as_array()?;
    let mut fps = HashSet::new();
    for r in results {
        // Try fingerprint first
        if let Some(fp) = r["partialFingerprints"]["primary/v1"].as_str() {
            fps.insert(fp.to_string());
        } else {
            // Fallback: (uri, startLine, ruleId) tuple as string key
            let uri = r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"]
                .as_str().unwrap_or("");
            let line = r["locations"][0]["physicalLocation"]["region"]["startLine"]
                .as_u64().unwrap_or(0);
            let rule = r["ruleId"].as_str().unwrap_or("");
            fps.insert(format!("{}:{}:{}", uri, line, rule));
        }
    }
    Some(fps)
}
```

The fallback key format must match what the current run produces for old-format baselines.

### Pattern 4: Suppression function signature

```rust
// In cwe_scanner.rs
const CPPCHECK_COVERED_CWES: &[u32] = &[78, 120, 122, 134, 190, 242, 327, 369, 676, 704, 732, 762];

pub fn suppress_lexical_false_positives(
    findings: Vec<SastFinding>,
    cppcheck_scanned_dirs: &BTreeSet<PathBuf>,
    cppcheck_confirmed_sites: &BTreeSet<(String, u32, u32)>, // (file, line, cwe)
) -> Vec<SastFinding> {
    if cppcheck_scanned_dirs.is_empty() {
        return findings;  // cppcheck did not run — no suppression
    }
    findings.into_iter().filter(|f| {
        if f.source != SastSource::Lexical {
            return true; // keep cppcheck and both findings
        }
        if !CPPCHECK_COVERED_CWES.contains(&f.cwe_id) {
            return true; // CWE not in cppcheck's scope
        }
        let under_scanned_dir = cppcheck_scanned_dirs.iter().any(|dir| {
            Path::new(&f.file_path).starts_with(dir)
        });
        if !under_scanned_dir {
            return true; // file not in a cppcheck-scanned dir
        }
        // In scope + CWE covered: keep only if cppcheck confirmed it
        let key = (f.file_path.clone(), f.line, f.cwe_id);
        cppcheck_confirmed_sites.contains(&key)
    }).collect()
}
```

### Pattern 5: `run_cppcheck_scanner` signature change

Current signature:
```rust
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&OsStr>,
) -> Vec<SastFinding>
```

New signature (return a tuple):
```rust
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&OsStr>,
) -> (Vec<SastFinding>, BTreeSet<PathBuf>)
```

The second element is the set of `PathBuf` directories where cppcheck actually ran (preflight succeeded AND component dir exists AND cppcheck ran without unexpected exit). Call sites in `main.rs` must be updated to destructure the tuple.

### Pattern 6: `main.rs` call sequence after Phase 16

```rust
let (cppcheck_findings, cppcheck_scanned_dirs) =
    crate::vulnerability::run_cppcheck_scanner(&component_dirs, cppcheck_bin);

sast_findings =
    crate::vulnerability::deduplicate_sast_findings(lexical_findings, cppcheck_findings);

// SARIF-07: build confirmed sites set from cppcheck findings (before suppression)
use std::collections::BTreeSet;
let cppcheck_confirmed: BTreeSet<(String, u32, u32)> = sast_findings.iter()
    .filter(|f| f.source == SastSource::Cppcheck || f.source == SastSource::Both)
    .map(|f| (f.file_path.clone(), f.line, f.cwe_id))
    .collect();

sast_findings = crate::vulnerability::suppress_lexical_false_positives(
    sast_findings,
    &cppcheck_scanned_dirs,
    &cppcheck_confirmed,
);

// Both writers receive the same post-suppression slice (SARIF-06)
save_static_analysis_report(project_name, out_dir, &sast_findings)?;
save_sarif_report(project_name, out_dir, &sast_findings, args.sarif_output.as_deref())?;

// SARIF-05: baseline diff
if let Some(ref baseline_path) = args.sarif_baseline {
    // load baseline, diff, write diff SARIF, check exit code
}
```

### Anti-Patterns to Avoid

- **Adding fingerprint to `SastFinding` struct:** Fingerprint is a SARIF-only concern; keeping it in the struct pollutes the data model and requires SHA computation before writers are called. Compute inline in `save_sarif_report`. (Per CONTEXT.md locked decision.)
- **Parse-roundtrip for consistency:** Reading the written SARIF file back to generate markdown would introduce a latency and error-prone step. Both writers consume the in-memory slice directly (per SARIF-06 decision).
- **Aborting on bad baseline:** If `--sarif-baseline` path doesn't exist or is invalid SARIF, warn and continue with full scan. Never `bail!` or `?`-propagate here (per SARIF-05 decision).
- **Suppressing `SastSource::Both` findings:** Only suppress `Lexical` findings. `Both` means cppcheck confirmed the site — it must not be suppressed.
- **Building confirmed sites from the pre-dedup cppcheck list:** Build confirmed sites from the deduplicated `sast_findings` (post-dedup, pre-suppression), filtering on source == Cppcheck or Both. This ensures path normalization from `deduplicate_sast_findings` is applied consistently.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| SHA-256 | Custom hash | `sha2::Sha256::digest()` | Already in Cargo.toml; RustCrypto is audited, constant-time |
| JSON parsing for baseline | Manual SARIF text parsing | `serde_json::from_str` / `serde_json::Value` | Already used everywhere; handles nested structure safely |
| Hex encoding | `format!("{:x}", byte)` loop | `format!("{:x}", hash)` on `GenericArray` | sha2 digest implements `LowerHex` directly |

**Key insight:** Everything needed already exists in the dependency tree. This phase is pure logic orchestration, not new library integration.

---

## Common Pitfalls

### Pitfall 1: Confirmed sites set built from wrong snapshot

**What goes wrong:** Building `cppcheck_confirmed` from the raw cppcheck findings (before `deduplicate_sast_findings`) means path normalization won't match the normalized paths stored in the deduped `sast_findings` vec. Suppressions will silently fail.

**Why it happens:** `deduplicate_sast_findings` calls `normalize_path()` on file paths. If the confirmed sites set uses un-normalized paths, `contains()` returns false even for genuine cppcheck hits.

**How to avoid:** Build `cppcheck_confirmed` from `sast_findings` (post-dedup) by filtering on `source == Cppcheck || source == Both`. Those entries already have normalized paths from the dedup pass.

**Warning signs:** Lexical findings not being suppressed even when cppcheck confirms the same site.

### Pitfall 2: `starts_with` path comparison requires canonical paths

**What goes wrong:** `Path::new(&f.file_path).starts_with(dir)` fails when `f.file_path` is relative but `dir` is absolute (or vice versa), even for the same file.

**Why it happens:** cppcheck findings are absolutized in `run_cppcheck_scanner` already (existing WR-02 fix). Lexical scanner findings may be absolute or relative depending on how WalkDir resolved them.

**How to avoid:** In `suppress_lexical_false_positives`, normalize both paths before `starts_with`. Use the existing `normalize_path()` helper already in `cwe_scanner.rs`.

**Warning signs:** Zero suppressions even with cppcheck installed and running.

### Pitfall 3: Diff SARIF path when `--sarif-output` is not set

**What goes wrong:** When the user does not pass `--sarif-output`, the diff SARIF must go to a default path with `_diff` suffix (e.g., `{out_dir}/{project_name}_static_analysis_diff.sarif`), not the same path as the full SARIF (which would overwrite it).

**Why it happens:** CONTEXT.md says "same path as `--sarif-output` OR default path with `_diff` suffix". The "OR" case needs explicit handling.

**How to avoid:** In the baseline diff logic, resolve the diff output path as: if `args.sarif_output.is_some()`, use that path; else use `out_dir.join(format!("{}_static_analysis_diff.sarif", project_name))`.

**Warning signs:** Full SARIF overwritten by diff SARIF when `--sarif-output` is not set.

### Pitfall 4: `SarifResult.partialFingerprints` serializes as `partial_fingerprints`

**What goes wrong:** The `#[serde(rename_all = "camelCase")]` attribute on `SarifResult` converts `partial_fingerprints` → `partialFingerprints` correctly. But if you add the field WITHOUT the struct-level `rename_all`, the JSON key will be snake_case and SARIF consumers will not find it.

**Why it happens:** The struct already uses `rename_all = "camelCase"` — the new field is covered automatically if added to the same struct.

**How to avoid:** Verify the serialized field name in a unit test: assert the JSON contains `"partialFingerprints"`.

### Pitfall 5: `run_cppcheck_scanner` signature change breaks `mod.rs` re-export

**What goes wrong:** `vulnerability/mod.rs` re-exports `run_cppcheck_scanner` by name. After the return type changes, any call site that ignored the return value (or expected only `Vec<SastFinding>`) will fail to compile.

**Why it happens:** The only call site is `main.rs` which currently pattern-matches the return directly.

**How to avoid:** Update both the function signature in `cwe_scanner.rs` and the `pub use` in `mod.rs`. The compiler will catch all mismatched call sites.

### Pitfall 6: Baseline diff exit code interaction with `main()`'s `Result<()>`

**What goes wrong:** Returning `exit(1)` from within `main() -> Result<()>` via `std::process::exit(1)` bypasses the `anyhow` error path and does not print an error message. This is intentional here (it's a CI gate signal), but it must not call `exit(1)` when the scan itself fails with an error.

**How to avoid:** Only call `std::process::exit(1)` after all outputs are written successfully. Place the baseline diff check as the last operation before returning `Ok(())` in the format arms that produce SARIF.

---

## Code Examples

### Fingerprint computation (verified against sha2 0.10 API)

```rust
// Source: sha2 crate — RustCrypto [VERIFIED: Cargo.toml sha2 = "0.10"]
use sha2::{Digest, Sha256};

fn sarif_fingerprint(file_path: &str, line: u32, cwe_id: u32) -> String {
    let input = format!("{}:{}:CWE-{}", file_path, line, cwe_id);
    let digest = Sha256::digest(input.as_bytes());
    // GenericArray<u8, 32> implements LowerHex
    let hex = format!("{:x}", digest);
    hex[..16].to_string()
}
```

### Reading SARIF baseline (serde_json untyped)

```rust
// Using serde_json::Value for flexible SARIF parsing
// No new schema types needed — just navigate the JSON tree [VERIFIED: serde_json pattern]
fn extract_baseline_fingerprints(path: &Path) -> HashSet<String> {
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
                // Fallback tuple key for pre-Phase-16 SARIF files
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

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SARIF as write-only output | SARIF as authoritative finding store with fingerprints | Phase 16 | Enables stable baseline diffing across runs |
| All lexical findings pass through | Lexical findings suppressed when cppcheck covers the CWE and did not confirm the site | Phase 16 | Reduces false positives for CWEs with strong cppcheck coverage |
| No CI exit code from finding diff | `--sarif-baseline` exits 1 on regressions | Phase 16 | Enables "no new findings" CI gates |

**What Phase 15 shipped:** SARIF-01 (all findings to `.sarif`), SARIF-02 (`--sarif-output`), SARIF-03 (rules section). Phase 16 adds fingerprints, baseline diff, consistency guarantee, and suppression. [VERIFIED: REQUIREMENTS.md traceability table]

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `partialFingerprints` is a valid top-level field on SARIF 2.1.0 result objects | Standard Stack, Code Examples | If field name differs, SARIF consumers will silently ignore fingerprints; unit test will catch wrong key |
| A2 | `sha2::GenericArray<u8, 32>` implements `LowerHex` via `format!("{:x}", digest)` | Code Examples | Compile error; easily fixed with `hex::encode` if needed (but LowerHex impl is standard in RustCrypto) |

Both A1 and A2 are well-established patterns but not verified via live docs lookup in this session.

---

## Open Questions

1. **Suppression set construction: pre-dedup vs post-dedup cppcheck findings**
   - What we know: `deduplicate_sast_findings` normalizes paths; confirmed sites must use normalized paths for `contains()` to work
   - What's unclear: Should the confirmed-sites set include ALL cppcheck-reported files, or only those that survived dedup?
   - Recommendation: Use the post-dedup `sast_findings` slice filtered on `source == Cppcheck || Both`. This is the safest approach — if a site was confirmed by cppcheck, it survives dedup with source=Cppcheck or Both.

2. **`--sarif-baseline` in which `OutputFormat` arms?**
   - What we know: SARIF output is only written for `Console` (with `--output`) and `All` format arms. `SpdxJson`, `SpdxTagValue`, `CyclonedxJson` already emit a warning that `--sarif-output` has no effect.
   - What's unclear: Should `--sarif-baseline` also emit a warning in non-SARIF format arms, or silently do nothing?
   - Recommendation: Mirror the existing `--sarif-output` warning pattern — emit the same "has no effect" warning for `--sarif-baseline` in non-SARIF format arms.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `sha2` crate | SARIF-04 fingerprinting | ✓ | 0.10.9 (Cargo.lock) | — |
| `serde_json` crate | SARIF-05 baseline load | ✓ | 1.0.x (Cargo.toml) | — |
| `clap` derive | SARIF-05 CLI flag | ✓ | 4.5.x (Cargo.toml) | — |
| `cppcheck` binary | SARIF-07 suppression | ✗ (not on PATH) | — | No suppression (existing graceful degradation) |

**Missing dependencies with no fallback:** None.

**Missing dependencies with fallback:**
- `cppcheck` not installed on this machine — suppression path will not activate. This is expected and handled by the existing graceful degradation in `run_cppcheck_scanner`. Tests for suppression logic should use a mock/fixture approach, not a live cppcheck invocation.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` / integration tests in `tests/` |
| Config file | `Cargo.toml` `[dev-dependencies]` (tempfile, assert_cmd, predicates) |
| Quick run command | `cargo test --features internal 2>/dev/null` |
| Full suite command | `cargo test --features internal --all 2>/dev/null` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SARIF-04 | `partialFingerprints["primary/v1"]` present in SARIF output JSON | unit | `cargo test --features internal fingerprint` | ❌ Wave 0 |
| SARIF-04 | Fingerprint is 16 hex chars | unit | `cargo test --features internal fingerprint` | ❌ Wave 0 |
| SARIF-04 | Same (file, line, CWE) always produces same fingerprint | unit | `cargo test --features internal fingerprint` | ❌ Wave 0 |
| SARIF-05 | Baseline with same fingerprints → exit 0 | integration | `cargo test --features internal baseline_no_new` | ❌ Wave 0 |
| SARIF-05 | Baseline missing a current fingerprint → exit 1 | integration | `cargo test --features internal baseline_new_finding` | ❌ Wave 0 |
| SARIF-05 | Non-existent baseline path → warn + continue full scan | unit | `cargo test --features internal baseline_missing_file` | ❌ Wave 0 |
| SARIF-05 | Invalid SARIF baseline → warn + continue | unit | `cargo test --features internal baseline_invalid` | ❌ Wave 0 |
| SARIF-05 | Old baseline (no fingerprints) → fallback tuple match | unit | `cargo test --features internal baseline_fallback` | ❌ Wave 0 |
| SARIF-06 | Markdown row count == SARIF results length (assertion test) | unit | `cargo test --features internal consistency_assertion` | ❌ Wave 0 |
| SARIF-07 | Lexical CWE in covered set + cppcheck did not confirm → suppressed | unit | `cargo test --features internal suppress_lexical` | ❌ Wave 0 |
| SARIF-07 | Lexical CWE not in covered set → kept even if cppcheck ran | unit | `cargo test --features internal suppress_uncovered_cwe` | ❌ Wave 0 |
| SARIF-07 | `source == Both` finding → never suppressed | unit | `cargo test --features internal suppress_both_kept` | ❌ Wave 0 |
| SARIF-07 | cppcheck not installed → no suppression, all lexical pass through | unit | `cargo test --features internal suppress_no_cppcheck` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --features internal 2>/dev/null`
- **Per wave merge:** `cargo test --features internal --all 2>/dev/null`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

All test files for Phase 16 are new. The existing `tests/vulnerability_tests/cwe_scanner_tests.rs` can be extended, or a dedicated `sarif_tests.rs` can be added.

- [ ] `tests/vulnerability_tests/sarif_tests.rs` — covers SARIF-04 fingerprint unit tests
- [ ] `tests/vulnerability_tests/baseline_tests.rs` — covers SARIF-05 baseline diff unit tests
- [ ] `tests/vulnerability_tests/suppression_tests.rs` — covers SARIF-07 suppression unit tests
- [ ] Extend `tests/vulnerability_tests/cwe_scanner_tests.rs` — test new `run_cppcheck_scanner` tuple return

*(No new framework install needed — existing `tempfile` dev-dep handles temp file fixture creation.)*

---

## Security Domain

> `security_enforcement` not explicitly disabled in config — section included.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | yes | Validate SARIF baseline file before parsing; graceful degradation on malformed input (already locked in SARIF-05 decision) |
| V6 Cryptography | yes (fingerprinting) | `sha2` (RustCrypto, audited); not used for security — used for stable identity, so collision resistance > preimage resistance, but sha256 is appropriate for both |

### Known Threat Patterns for this stack

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via `--sarif-baseline` or `--sarif-output` | Tampering | `std::fs::read_to_string` with path from CLI args — accept any path the OS allows; no user-controlled path construction |
| Malformed SARIF baseline causes panic | Denial of Service | `serde_json::from_str().ok()` + graceful degradation; already in locked SARIF-05 decision |
| Large baseline file causes OOM | Denial of Service | SARIF files for our use case are bounded by finding count × sizeof(result); acceptable risk |

---

## Sources

### Primary (HIGH confidence)

- `Cargo.toml` / `Cargo.lock` — confirmed `sha2 = "0.10.9"`, `serde_json`, `clap 4.5` present [VERIFIED]
- `src/formats/sarif.rs` — current SARIF writer implementation [VERIFIED]
- `src/vulnerability/cwe_scanner.rs` — `SastFinding`, `SastSource`, `deduplicate_sast_findings`, `run_cppcheck_scanner`, `CPPCHECK_CWE_OVERRIDES` [VERIFIED]
- `src/main.rs` — current call sequence for lexical + cppcheck + dedup + writers [VERIFIED]
- `src/cli.rs` — current CLI flag structure, `#[cfg(feature = "internal")]` pattern [VERIFIED]
- `.planning/phases/16-sarif-as-authoritative-finding-store-refactor-the-static-ana/16-CONTEXT.md` — all locked decisions [VERIFIED]

### Secondary (MEDIUM confidence)

- RustCrypto sha2 0.10 API: `Sha256::digest()` returns `GenericArray` implementing `LowerHex` — standard pattern, consistent with existing uses of sha2 in the Rust ecosystem

### Tertiary (LOW confidence / ASSUMED)

- SARIF 2.1.0 `partialFingerprints` field schema — standard SARIF field [ASSUMED] (see Assumptions Log A1)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all crates verified in Cargo.toml/Cargo.lock
- Architecture: HIGH — based on direct code reading of existing pipeline
- Pitfalls: HIGH — derived from existing dedup/path normalization code and locked decisions
- SARIF spec details (partialFingerprints): MEDIUM — well-known field, assumed schema matches standard

**Research date:** 2026-05-11
**Valid until:** 2026-06-11 (stable Rust codebase; no fast-moving external dependencies)
