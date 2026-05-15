# Phase 12: Static Analysis Report - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 12-static-analysis-report
**Areas discussed:** Report file naming + trigger, Report content structure, Main report integration, Disclaimer placement

---

## Report File Naming + Trigger

| Option | Description | Selected |
|--------|-------------|----------|
| `{project}_static_analysis.md` | Matches `{project}_report.md` convention; same output dir | ✓ |
| `_static_analysis.md` (no prefix) | Literal from REQUIREMENTS.md; no namespace by project | |
| `{project}_sast_report.md` | Explicit about SAST; new term not used elsewhere | |

**User's choice:** `{project}_static_analysis.md`
**Notes:** —

| Option | Description | Selected |
|--------|-------------|----------|
| Always write the file | Even zero findings; downstream scripts can always stat/read | ✓ |
| Only write when findings exist | Skip on clean scan; breaks scripts expecting file always present | |

**User's choice:** Always write

| Option | Description | Selected |
|--------|-------------|----------|
| Same output dir as `_report.md` | Consistent with all other scan outputs in `--output-dir` | ✓ |
| Subdirectory (e.g., `out/sast/`) | Separates SAST from SBOM; adds dir creation logic | |

**User's choice:** Same output dir

---

## Report Content Structure

| Option | Description | Selected |
|--------|-------------|----------|
| One row per component (Component \| CWEs \| Count) | Compact; CWEs comma-joined | |
| One row per CWE per component (Component \| CWE \| Name \| Count) | Filterable; room for CWE name | ✓ |

**User's choice:** Option B (one row per CWE per component)
**Notes:** User requested pros/cons before deciding. Key factors: filterability per CWE and space for CWE name outweighed the extra rows for large projects.

| Option | Description | Selected |
|--------|-------------|----------|
| Grouped by component, then by CWE | `## libfoo` → `### CWE-120` → bullet file:line list | ✓ |
| Flat list sorted by file path | One entry per finding sorted by file; loses grouping | |

**User's choice:** Grouped by component, then by CWE

| Option | Description | Selected |
|--------|-------------|----------|
| Write file with 'No findings' message | `No static analysis findings detected.`; file always present | ✓ |
| Write file with empty table headers only | Looks broken without explanatory note | |

**User's choice:** Write file with "No findings" message

---

## Main Report Integration

| Option | Description | Selected |
|--------|-------------|----------|
| After CVE/vulnerability section | CVE section intact first; SAST appended after | ✓ |
| Before CVE/vulnerability section | Scanner runs before network; natural order for some | |
| At end of report | SAST as appendix; users must scroll past dependency tree | |

**User's choice:** After CVE/vulnerability section

| Option | Description | Selected |
|--------|-------------|----------|
| Show section with 'No static analysis findings' | Confirms scanner ran; no findings is a meaningful outcome | ✓ |
| Omit section entirely when no findings | Clean report; ambiguous whether scanner ran | |

**User's choice:** Always show section (with "No findings" when clean)

| Option | Description | Selected |
|--------|-------------|----------|
| Summary table only in `_report.md` | Full file:line in `_static_analysis.md`; keeps report scannable | ✓ |
| Full findings in `_report.md` too | Convenient; bloats report; `_static_analysis.md` becomes redundant | |

**User's choice:** Summary table only in `_report.md`

---

## Disclaimer Placement

| Option | Description | Selected |
|--------|-------------|----------|
| When static analysis runs and produces the report | Print alongside `save_static_analysis_report()` call | ✓ |
| At scan start | Prints early; appears even if no C/C++ files found | |
| After all output files written | Clear sequencing; could be missed on early exit | |

**User's choice:** When static analysis runs and produces the report

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — include as callout at top of `_static_analysis.md` | Anyone reading file offline sees the caveat | ✓ |
| No — CLI only per RPT-03 | Strictly per spec; file readers miss the caveat | |

**User's choice:** Include in file as blockquote note

---

## Claude's Discretion

- Exact Rust function signature for `save_static_analysis_report()` — fit alongside `save_console_report()` or in a new module if large enough
- Whether to add a `--no-static-analysis-disclaimer` flag — only if trivial; skip if it adds complexity

## Deferred Ideas

None — discussion stayed within phase scope.
