# Phase 14: cppcheck-integration - Context

**Gathered:** 2026-05-10
**Status:** Ready for planning

<domain>
## Phase Boundary

Invoke cppcheck as an external subprocess on each component-mapped C/C++ directory; parse its XML output; resolve CWE IDs from the XML `cwe` attribute (primary) with a static override table (secondary); deduplicate findings against the lexical scanner results by `(file, line, cwe)` after path canonicalization; merge dual-detected findings into a single `SastFinding` with a new `source` field; surface the combined `Vec<SastFinding>` through the existing CycloneDX + static analysis report pipeline.

When cppcheck is not found on PATH and no `--cppcheck-path` is given, emit a one-line stderr warning and complete with lexical-only results. When a per-component invocation fails, warn and continue.

Requirements in scope: CPPCHECK-01, CPPCHECK-02, CPPCHECK-03, CPPCHECK-04, CPPCHECK-05

</domain>

<decisions>
## Implementation Decisions

### CWE Mapping

- **D-01:** CWE resolution order: (1) read the `cwe` attribute from the cppcheck XML `<error>` element; (2) if absent or 0, look up the cppcheck error `id` in a static `&[(&str, u32)]` override table compiled into `cwe_scanner.rs`. If neither yields a non-zero CWE, skip the finding silently.
- **D-02:** The static override table corrects known mis-mappings and fills gaps for error IDs that cppcheck does not annotate with a `cwe` attribute. It is NOT the primary source — the XML attribute is.
- **D-03:** Include any cppcheck error ID that has a real CWE mapping, regardless of cppcheck severity tier (security, warning, or style). The CWE integer is the filter.
- **D-04:** Findings with no resolvable CWE (no XML attribute, not in override table) are silently dropped — never emitted with a sentinel value.

### Subprocess Invocation

- **D-05:** Invoke `cppcheck --xml --xml-version=2 --enable=warning,style,security` once per entry in `component_dirs`, in the same loop structure as `run_lexical_scanner`. Each finding is directly attributed to the component name/ecosystem from the map key.
- **D-06:** `run_cppcheck_scanner(component_dirs, cppcheck_bin)` is called sequentially after `run_lexical_scanner` inside the same `#[cfg(feature = "internal")]` block in `main.rs`. Deduplication happens in the same block before `sast_findings` is finalized.
- **D-07:** Show an `indicatif` progress bar while cppcheck runs across components, consistent with the progress-bar pattern already used in the lexical scanner.
- **D-08:** After all components finish, emit a single completion line to stderr: `"cppcheck: {N} findings from {M} components"`.
- **D-09:** If cppcheck binary is not found (PATH lookup and `--cppcheck-path` both fail): `eprintln!("⚠ cppcheck not found — lexical-only results. Install cppcheck or use --cppcheck-path.")` and return an empty Vec. No abort, exit code 0.
- **D-10:** If a per-component cppcheck invocation exits non-zero or writes to stderr: `eprintln!` the component name and cppcheck's stderr, skip that component, continue. Consistent with broken-symlink tolerance pattern.

### Deduplication

- **D-11:** Deduplication key: `(canonical_file_path, line, cwe_id)`. Call `Path::canonicalize()` on `file_path` strings from both sources before building the `HashSet` key, to handle relative vs. absolute path differences.
- **D-12:** When a `(file, line, cwe)` tuple appears in both lexical and cppcheck results: keep one `SastFinding` with `source = SastSource::Both`. The lexical finding's other fields (component attribution) are preserved as the base.

### SastFinding Struct Extension

- **D-13:** Add `source: SastSource` field to `SastFinding`. Enum: `Lexical | Cppcheck | Both`. Lexical scanner findings set `Lexical`; cppcheck-only findings set `Cppcheck`; deduped duplicates set `Both`.
- **D-14:** All downstream consumers (`cyclonedx.rs`, `console.rs`, static analysis report) that pattern-match or construct `SastFinding` must be updated for the new field. `source` need not be surfaced in current outputs — it is metadata for Phase 15 (SARIF) and future use.

### CLI Flag

- **D-15:** Add `--cppcheck-path <PATH>` CLI flag (gated behind `#[cfg(feature = "internal")]`). When provided, use that binary path instead of PATH lookup. Follows the same `#[arg(long)]` pattern as other internal flags.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §cppcheck Subprocess (Track A) — CPPCHECK-01 through CPPCHECK-05, the full requirement text and out-of-scope items

### Prior Phase Context (Scanner Architecture)
- `.planning/milestones/v1.0.16-phases/11-lexical-scanner-cyclonedx-output/11-CONTEXT.md` — D-01 (component_dirs key pattern), D-05 (scanner invocation point), D-08 (inline cfg gating pattern)
- `.planning/milestones/v1.0.16-phases/12-static-analysis-report/12-CONTEXT.md` — D-01 through D-06 (report format, zero-findings case, static analysis report structure)
- `.planning/milestones/v1.0.16-phases/10-internal-feature-gate/10-CONTEXT.md` — D-01 through D-08 (feature gate rules, what's gated, cfg patterns)

### Source Files
- `src/vulnerability/cwe_scanner.rs` — `SastFinding` struct (lines 23–31), `run_lexical_scanner` signature (line 215), `CweRule` pattern to follow for cppcheck invocation
- `src/main.rs` — lines 191–239: `sast_findings` declaration, `component_dirs` fallback, `run_lexical_scanner` call site — cppcheck call goes immediately after line 239

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `SastFinding` struct (`src/vulnerability/cwe_scanner.rs:23`) — extend with `source: SastSource` field; all other fields reused as-is
- `run_lexical_scanner(component_dirs)` (`cwe_scanner.rs:215`) — signature and loop structure to mirror for `run_cppcheck_scanner`
- `indicatif::ProgressBar` — already imported in `cwe_scanner.rs`; reuse the same spinner/bar pattern
- `warn_on_walkdir_err` utility — precedent for the warn-and-continue error handling pattern

### Established Patterns
- `#[cfg(feature = "internal")]` at module level (`cwe_scanner.rs:13`) — entire cppcheck module gets the same gate
- `#[cfg(feature = "internal")]` per-parameter inline gating on formatter functions — `sast_findings` param already uses this; no changes needed to formatter call sites beyond the struct field addition
- `(name, ecosystem)` tuple as `component_dirs` map key — cppcheck runner receives the same map, attributes findings using the key directly
- `eprintln!` for warnings — used throughout the codebase; no structured logging

### Integration Points
- `main.rs:239` — lexical scanner call; cppcheck call goes immediately after, still inside the `#[cfg(feature = "internal")]` block
- `src/vulnerability/mod.rs` — exports `SastFinding` and `run_lexical_scanner`; add `run_cppcheck_scanner` and `SastSource` exports here
- `src/formats/cyclonedx.rs`, `src/formats/console.rs` — consume `&sast_findings`; will need `SastFinding` struct update but no behavioral change for this phase

</code_context>

<specifics>
## Specific Ideas

- cppcheck XML format uses `--xml-version=2`; parse `<results><errors><error id="..." cwe="..." ...><location file="..." line="..."/></error></errors></results>` structure
- `SastSource::Both` is primarily useful for SARIF output (Phase 15) where dual detection can be noted in the `partialFingerprints` or `message` fields
- The `--cppcheck-path` flag naming follows the `--supplier-config` / `--vulnerability-output` convention in the existing CLI (kebab-case, full words)

</specifics>

<deferred>
## Deferred Ideas

- **Suppress-list support** (CPPCHECK-F1 in REQUIREMENTS.md) — user-configurable file to silence known FP findings; future milestone
- **CI timing annotation** (CPPCHECK-F2) — log per-component cppcheck duration; future milestone
- **Surfacing `source` field in static analysis report** — `SastSource::Both` metadata not shown in current report format; Phase 15 (SARIF) is the right place to expose it

</deferred>

---

*Phase: 14-cppcheck-integration*
*Context gathered: 2026-05-10*
