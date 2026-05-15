---
phase: 16
phase_name: sarif-as-authoritative-finding-store
status: ready_to_plan
created: "2026-05-11"
---

# Phase 16 — SARIF as Authoritative Finding Store: Context

## Phase Goal

Refactor the static analysis pipeline so SARIF is the single source of truth:
- Findings carry stable fingerprints (SARIF-04)
- `--sarif-baseline` enables new-findings-only CI gate (SARIF-05)
- `_static_analysis.md` and SARIF always show identical findings (SARIF-06)
- cppcheck scope suppression reduces lexical false positives for CWEs cppcheck covers (SARIF-07)

Ships as part of v1.0.17.

---

## Decisions

### SARIF-04: Fingerprinting

**Decision:** Hash fingerprint = `sha256(file_path + ":" + line + ":CWE-" + cwe_id)`

- Path-based, not content-based — no extra file I/O at scan time
- Stable as long as file path and line number don't shift
- Add fingerprint to each SARIF result in `partialFingerprints["primary/v1"]`
- Use the first 16 hex chars (64-bit prefix) — sufficient uniqueness for finding sets our size

**Rationale:** Content-based fingerprints would require reading already-walked files a second time. The path+line+CWE tuple is what our dedup logic already uses; promoting it to a stable hash is a natural extension with zero new I/O.

---

### SARIF-05: Baseline diff (`--sarif-baseline`)

**Decision:** Exit 1 on new findings, exit 0 if no regressions. Output diff SARIF to same path as `--sarif-output` (or default path with `_diff` suffix).

- Without `--sarif-baseline`: every scan writes full SARIF (all findings). No change in behavior.
- With `--sarif-baseline <file>`: load baseline SARIF, extract fingerprints, compare to current run fingerprints.
  - Findings in current run but NOT in baseline → "new" (regressions)
  - Findings in baseline but NOT in current run → "fixed" (silently dropped from diff output)
  - Output SARIF contains only new findings
  - If new findings exist: print count to stderr, exit 1
  - If no new findings: print "No new findings vs baseline", exit 0

**Fingerprint matching:** Compare `partialFingerprints["primary/v1"]` values. If baseline SARIF lacks fingerprints (e.g. older sc2sbom output), fall back to exact `(uri, startLine, ruleId)` tuple match.

**Error handling:** If `--sarif-baseline` file does not exist or is not valid SARIF, print error to stderr and continue with full (non-diff) scan — do NOT abort.

---

### SARIF-06: Markdown and SARIF consistency

**Decision:** Both `_static_analysis.md` and `_static_analysis.sarif` are written from the same in-memory `Vec<SastFinding>` slice. No parse-roundtrip from SARIF back to markdown.

- "SARIF authoritative" means the data model is shared in memory, not that we read the written file back
- The `SastFinding` slice after `deduplicate_sast_findings()` is the canonical finding set
- Both writers consume it before any findings are dropped

**Key invariant:** The markdown table row count must equal the SARIF results array length. Add an assertion in `#[cfg(test)]` that fires if they diverge.

**Scope note:** SARIF-07 suppression (see below) modifies the `Vec<SastFinding>` BEFORE both writers run. This preserves the consistency invariant automatically.

---

### SARIF-07: Cppcheck scope suppression

**Decision:** Scope suppression — drop `SastSource::Lexical` findings for CWEs that cppcheck actively covers, when cppcheck ran on that component's directory and did NOT flag that specific (file, line, CWE) site.

**How it works:**
1. After `deduplicate_sast_findings()`, we have a `Vec<SastFinding>` with `source = Lexical | Cppcheck | Both`
2. Build a `BTreeSet<(file_path, line, cwe_id)>` of all cppcheck-confirmed sites
3. Build a `BTreeSet<String>` of component directories where cppcheck actually ran (from `run_cppcheck_scanner` return)
4. For each `SastFinding` with `source == Lexical`:
   - If its `file_path` is under a cppcheck-scanned directory AND its CWE is in `CPPCHECK_COVERED_CWES` AND the site is NOT in the cppcheck confirmed set → suppress (remove)
   - Otherwise → keep

**`CPPCHECK_COVERED_CWES` table:** The CWEs that cppcheck's `--enable=warning,style,security` flags check reliably. Start with: `{78, 120, 122, 134, 190, 242, 327, 369, 676, 704, 732, 762}`. This set comes from the `CPPCHECK_CWE_OVERRIDES` table already in `cwe_scanner.rs`.

**When cppcheck is not installed:** No suppression occurs. All lexical findings pass through unchanged.

**Consistency with SARIF-06:** Suppression runs before both markdown and SARIF writers — both see the same post-suppression finding set.

---

### SARIF/Markdown consistency (user requirement)

The user explicitly required: **SARIF and `_static_analysis.md` must show identical findings.**

Implementation rule: Any operation that modifies the finding set (dedup, suppression) must complete BEFORE either writer (`save_sarif_report`, `save_static_analysis_report`) is called. Both writers take `&[SastFinding]` from the same slice.

---

## Architectural Notes

### Where suppression fits in main.rs

Current call sequence:
```
lexical_findings + cppcheck_findings
  -> deduplicate_sast_findings(...)  -> sast_findings
  -> save_static_analysis_report(...)
  -> save_sarif_report(...)
```

Phase 16 call sequence:
```
lexical_findings + cppcheck_findings
  -> deduplicate_sast_findings(...)  -> sast_findings
  -> suppress_lexical_false_positives(sast_findings, cppcheck_scanned_dirs)  -> sast_findings
  -> add_fingerprints(&mut sast_findings)  [or compute inline in SARIF writer]
  -> save_static_analysis_report(...)
  -> save_sarif_report(...)        [includes fingerprints in partialFingerprints]
  -> if args.sarif_baseline: load_baseline, diff, write diff SARIF, check exit code
```

### Fingerprint placement

Fingerprint added to `SastFinding`:
```rust
pub fingerprint: String,  // hex(sha256(file_path + ":" + line + ":CWE-" + cwe_id))[..16]
```

Or computed inline in `save_sarif_report` without adding to `SastFinding` struct. **Prefer computing inline in the SARIF writer** — keeps `SastFinding` lean and avoids SHA2 dep in the core struct. The markdown writer doesn't need fingerprints.

### SHA2 dependency

Need `sha2` crate (already commonly used in Rust ecosystem). Gate behind `feature = "internal"` since fingerprinting only applies to SARIF output. If `sha2` is already in `Cargo.toml` from another dep, reuse it. Otherwise add it.

---

## Out of Scope (Phase 16)

- CSV output format (deferred)
- Content-based fingerprinting (more stable across line shifts but more complex; path+line+CWE is sufficient for v1.0.17)
- SARIF suppression file / user-configurable suppress list (CPPCHECK-F1, future)
- cppcheck timing annotations (CPPCHECK-F2, future)

---

## Requirements to Add

Phase 16 introduces SARIF-04..07 which are not yet in REQUIREMENTS.md. The planner should add them:

- **SARIF-04**: Each SARIF result includes a `partialFingerprints["primary/v1"]` field computed as `sha256(file_path + ":" + line + ":CWE-" + cwe_id)[..16]`
- **SARIF-05**: `--sarif-baseline <file>` compares current fingerprints to baseline; exits 1 if new findings found, 0 if none
- **SARIF-06**: `_static_analysis.md` and `_static_analysis.sarif` report identical findings (both written from the same in-memory slice after all suppression is complete)
- **SARIF-07**: Lexical findings for CWEs in `CPPCHECK_COVERED_CWES` are suppressed when cppcheck ran on that component directory and did not confirm the finding

---

*Context captured: 2026-05-11*
*Discussed: SARIF-04, SARIF-05, SARIF-06, SARIF-07, SARIF format suitability*
