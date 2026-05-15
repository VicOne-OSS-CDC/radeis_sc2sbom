# Phase 12: Static Analysis Report - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Surface lexical scanner findings to users via two artifacts and a CLI signal: a dedicated `{project}_static_analysis.md` report with per-component CWE breakdown and file:line findings, a "Static Analysis Findings" summary section integrated into the existing `{project}_report.md`, and a stderr disclaimer when the scanner runs. All output is gated behind `#[cfg(feature = "internal")]` — this phase only adds user-facing reporting; no scanner logic changes.

Requirements in scope: RPT-01, RPT-02, RPT-03

</domain>

<decisions>
## Implementation Decisions

### Report File — Naming and Trigger

- **D-01:** Output filename is `{project}_static_analysis.md` — matches the `{project}_report.md` convention; same output directory as all other scan outputs (`--output-dir`).
- **D-02:** File is **always written** when `--features internal` is active and a static analysis run occurs, even if there are zero findings. Downstream scripts can always stat/read the file.
- **D-03:** Zero-findings case: write the file with a "No static analysis findings detected." prose line (no empty table headers). The summary table still prints headers but a single note row replaces data rows.

### Report Content Structure

- **D-04:** The per-component CWE summary table uses **one row per CWE per component** (Option B). Columns: `Component | CWE | Name | Count`. Allows per-CWE filtering; room for short CWE description in "Name" column.
- **D-05:** Below the summary table, file:line findings are **grouped by component, then by CWE** — e.g., `## libfoo` → `### CWE-120 (Buffer overflow)` → bullet list of `- src/foo.c:42 — strcpy`. Mirrors the summary table structure; easy to cross-reference.
- **D-06:** Zero-findings case in the findings section: prose line "No static analysis findings detected." — not an empty grouped structure.

### Main Report Integration (_report.md)

- **D-07:** "Static Analysis Findings" section is placed **after the CVE/vulnerability section** in `_report.md`. CVE section stays intact and first; SAST appended immediately after.
- **D-08:** `_report.md` includes only the **summary table** (same format as D-04) — not the full file:line findings list. Full detail lives in `_static_analysis.md`. Keeps `_report.md` scannable.
- **D-09:** When there are zero findings, the section is **still present** in `_report.md` with a "No static analysis findings" message. Confirms scanner ran — absence of section would be ambiguous.

### Disclaimer

- **D-10:** Disclaimer prints to **stderr when the static analysis report is saved** — i.e., inside/alongside the `save_static_analysis_report()` call. Exact text: `Pattern-based — complex data-flow vulnerabilities not covered`. Only emits when scanner actually ran; not on every build.
- **D-11:** Disclaimer is **also embedded** as a callout/note at the top of `_static_analysis.md` so readers who open the file offline see the caveat. Format: `> **Note:** Pattern-based — complex data-flow vulnerabilities not covered.` (blockquote below the H1 title).

### Claude's Discretion

- Exact Rust function signature for `save_static_analysis_report()` — Claude picks the form that fits cleanly alongside `save_console_report()` in `src/formats/console.rs` (or a new `src/formats/sast_report.rs` if the function is large enough to warrant its own module).
- Whether to add a `--no-static-analysis-disclaimer` CLI flag is left to Claude — only add if the implementation is trivial; skip if it adds complexity.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Reporting — RPT-01, RPT-02, RPT-03 definitions and acceptance criteria
- `.planning/ROADMAP.md` §Phase 12 — success criteria (3 items)

### Prior Phase Context
- `.planning/phases/10-internal-feature-gate/10-CONTEXT.md` — feature gate decisions (D-01..D-15); all new code in this phase must be wrapped in `#[cfg(feature = "internal")]`

### Existing Report Writer (understand before adding new report)
- `src/formats/console.rs` — `save_console_report()` at line ~1109; study signature, how it opens output file, writes markdown sections, and returns `Result<()>`. New `save_static_analysis_report()` should follow the same pattern.
- `src/main.rs` — lines ~233–249 and ~317–330 show how `save_console_report()` is called, how `project_name` is derived (`sbom.project_path.file_name()`), and how output paths are constructed. Static analysis report call site follows this same pattern.

### Scanner Output (Phase 11 — MUST understand before writing the report)
- `src/vulnerability/cwe_scanner.rs` — the scanner module (stub in Phase 10, implemented in Phase 11). The report writer consumes findings from this module. Researcher must read this file to understand the finding struct shape before designing the report formatter.

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `save_console_report()` in `src/formats/console.rs`: the exact pattern (file open, `writeln!` into a `String`, write to disk) that `save_static_analysis_report()` should replicate.
- `project_name` derivation in `src/main.rs`: `sbom.project_path.file_name().unwrap_or_default().to_string_lossy()` — reuse to construct `{project}_static_analysis.md` filename.

### Established Patterns
- All output files are written in the same `--output-dir`; filename is `{project}_{type}.{ext}`. No subdirectories.
- Markdown reports use `writeln!(output, "...")` into a `String` buffer, then write to disk atomically. No streaming writes.
- `eprintln!("✓ {type} saved to: {}", path.display())` is the confirmation line pattern for all saved files.

### Integration Points
- `src/main.rs` output dispatch block (lines ~230–330): new `save_static_analysis_report()` call is added here, after `save_console_report()`, inside `#[cfg(feature = "internal")]`.
- `src/formats/console.rs` (or new `src/formats/sast_report.rs`): landing zone for the new report function.
- The "Static Analysis Findings" section in `_report.md` is written inside `save_console_report()` — the function receives scanner findings (or lack thereof) as a parameter.

</code_context>

<specifics>
## Specific Ideas

- Summary table format (per-component, per-CWE, with name): `| Component | CWE | Name | Count |`
- Findings list grouping: `## {component_name}` → `### CWE-{id} ({name})` → `- {file}:{line} — {function}`
- Disclaimer in file: `> **Note:** Pattern-based — complex data-flow vulnerabilities not covered.` as a blockquote immediately after the H1 in `_static_analysis.md`.
- Zero-findings prose: "No static analysis findings detected." (same text in both files and both sections)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 12-static-analysis-report*
*Context gathered: 2026-05-09*
