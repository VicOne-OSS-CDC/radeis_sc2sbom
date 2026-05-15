# Phase 15: sarif-output - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-10
**Phase:** 15-sarif-output
**Areas discussed:** SARIF file location, SARIF schema depth, Module placement, Dependency approach

---

## SARIF File Location

**Q1: Default path when --sarif-output not specified?**

| Option | Description | Selected |
|--------|-------------|----------|
| Same out_dir, default name | `{project_name}_static_analysis.sarif` in out_dir — mirrors .md file | ✓ |
| Only write when --sarif-output given | No default — opt-in only | |
| You decide | Claude picks | |

**Q2: How should --sarif-output interact with --output?**

| Option | Description | Selected |
|--------|-------------|----------|
| Independent flag, any path | --sarif-output is orthogonal to --output | ✓ |
| Relative to --output dir | --sarif-output interpreted relative to --output dir | |
| You decide | Claude picks | |

**Q3: Write SARIF when findings is empty?**

| Option | Description | Selected |
|--------|-------------|----------|
| Always write | Valid SARIF with empty arrays — CI expects artifact to exist | ✓ |
| Skip when empty | Don't write if no findings | |

---

## SARIF Schema Depth

**Q1: Which schema fields beyond the minimum?**

| Option | Description | Selected |
|--------|-------------|----------|
| Minimum viable + file URIs | schema, tool.driver, results[] with URI + startLine | |
| Minimum only | schema, tool.driver.rules[], results[] with ruleId + message only | |
| Full rich output | Add artifactContents, fingerprints, logical locations | ✓ |

**Q2: What rich fields are actually achievable from SastFinding?**

| Option | Description | Selected |
|--------|-------------|----------|
| URI + startLine + ruleId/name/helpUri | Realistic rich for a lexical scanner | ✓ |
| Also add component as logicalLocation | Encode component_name as logicalLocation.fullyQualifiedName | |
| You decide | Claude determines richest achievable schema | |

**Notes:** User initially selected "full rich output" then scoped down to what's achievable from available SastFinding fields — no AST means no function names. Decided: URI + startLine + rules with id/name/helpUri is the right target.

---

## Module Placement

**Q1: Where does the SARIF writer live?**

| Option | Description | Selected |
|--------|-------------|----------|
| New src/formats/sarif.rs | Mirrors console.rs; pub use in formats/mod.rs | ✓ |
| Inside console.rs | Add alongside save_static_analysis_report | |

**Q2: How does main.rs call it?**

| Option | Description | Selected |
|--------|-------------|----------|
| Immediately after .md call | Both call sites (286, 370) call save_static_analysis_report then save_sarif_report | ✓ |
| Single wrapper function | Extract write_analysis_outputs(...) wrapper | |

---

## Dependency Approach

**Q1: How is SARIF JSON produced?**

| Option | Description | Selected |
|--------|-------------|----------|
| serde_json hand-rolled structs | #[derive(Serialize)] structs, serde_json::to_string_pretty, zero new deps | ✓ |
| sarif-rs crate | External crate with spec-validated types | |

**Q2: Where do SARIF structs live?**

| Option | Description | Selected |
|--------|-------------|----------|
| Inside src/formats/sarif.rs | Private structs collocated with writer | ✓ |
| src/models/sarif.rs | Separate public models module | |

---

## Claude's Discretion

- `tool.driver.name` string value
- `tool.driver.version` — source from Cargo.toml or consistent with existing version reporting

## Deferred Ideas

None.
