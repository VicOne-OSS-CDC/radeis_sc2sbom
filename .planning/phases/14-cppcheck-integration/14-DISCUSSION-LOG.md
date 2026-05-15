# Phase 14: cppcheck-integration - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 14-cppcheck-integration
**Areas discussed:** CWE mapping strategy, Per-component vs. whole-tree invocation, Deduplication granularity, Warning verbosity / stderr format

---

## CWE Mapping Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Static table in-code | `&[("errorId", cwe_u32)]` array compiled into cwe_scanner.rs. No runtime cost, easy to audit. | |
| Parse cppcheck --errorlist at startup | Run `cppcheck --errorlist` XML at each scan. Always in sync but adds second subprocess call. | |
| External JSON/YAML file | Ship cppcheck-cwe-map.json alongside binary. Allows field-updating without recompile. | |

**User's choice:** Static table in-code  
**Notes:** Followed by additional questions:

- Unmapped IDs → skip silently (not CWE-0 sentinel)
- Scope of table → security + any warning/style ID with a real CWE
- XML `cwe` attribute as PRIMARY source; static table as override/correction layer
- Findings with no XML attribute AND not in table → always skip

| Option | Description | Selected |
|--------|-------------|----------|
| XML cwe attribute as primary, table as override | Future-proof; new cppcheck versions add CWEs automatically. | ✓ |
| Static table only | Predictable but requires manual updates. | |
| Table primary, XML as fallback | Middle ground but inverts what cppcheck itself says. | |

---

## Per-component vs. Whole-tree Invocation

| Option | Description | Selected |
|--------|-------------|----------|
| Once per component dir | Loop over component_dirs, one process per entry. Direct attribution, matches lexical scanner structure. | ✓ |
| Single invocation on scan root | One cppcheck call with --file-filter. Faster startup but requires post-hoc attribution. | |

**User's choice:** Once per component dir

| Option | Description | Selected |
|--------|-------------|----------|
| Sequential in same cfg block | Call after run_lexical_scanner in same #[cfg(feature = "internal")] block. | ✓ |
| Separate cfg block | Second cfg block. Cleaner separation but more fragmentation. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Simple stderr message per component | eprintln! before each call. | |
| No progress output | Silent. | |
| indicatif progress bar | Reuse existing ProgressBar pattern from lexical scanner. | ✓ |

**Notes:** User chose indicatif progress bar for consistency with the lexical scanner UX.

---

## Deduplication Granularity

| Option | Description | Selected |
|--------|-------------|----------|
| Canonicalize paths before dedup | Path::canonicalize() before HashSet key. Handles relative vs. absolute differences. | ✓ |
| String-equality only | Use file_path as-is. Simpler but misses cross-format duplicates. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Lexical finding wins | Simpler; no merge logic. | |
| cppcheck finding wins | Richer context but complex merge. | |
| Merge both | Keep one entry, combine metadata. | ✓ |

**User's choice:** Merge both — user wants dual-detection to be visible in output.

Follow-up: SastFinding has no message field. How to represent "merge both"?

| Option | Description | Selected |
|--------|-------------|----------|
| Add source field to SastFinding | `source: SastSource` enum (Lexical / Cppcheck / Both). On dedup, set Both. | ✓ |
| Keep SastFinding as-is, lexical wins | Don't change the struct. Simpler. | |

**Notes:** `SastSource::Both` is primarily valuable for Phase 15 (SARIF) where dual detection can be surfaced in fingerprints/messages.

---

## Warning Verbosity / Stderr Format

| Option | Description | Selected |
|--------|-------------|----------|
| stderr only, one line | eprintln! with cppcheck-not-found message. | ✓ |
| stderr + note in static analysis report | Same line plus note in _static_analysis.md. | |
| stderr only, structured JSON | JSON diagnostic. | |

| Option | Description | Selected |
|--------|-------------|----------|
| No metadata output | Silent aside from progress bar. | |
| Single stderr line on completion | "cppcheck: {N} findings from {M} components" | ✓ |

| Option | Description | Selected |
|--------|-------------|----------|
| Warn and continue | eprintln! the component + cppcheck stderr, skip, continue. | ✓ |
| Abort the whole scan | Return Err(...). | |

---

## Claude's Discretion

None — all areas had explicit user decisions.

## Deferred Ideas

- Suppress-list support (CPPCHECK-F1) — future milestone
- CI timing annotation (CPPCHECK-F2) — future milestone
- Surfacing `SastSource::Both` in static analysis report — Phase 15 (SARIF) is the right place
