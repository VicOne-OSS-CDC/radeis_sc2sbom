# Phase 12: Static Analysis Report - Research

**Researched:** 2026-05-09
**Domain:** Rust report formatting, Markdown file generation, feature-gated output dispatch
**Confidence:** HIGH

## Summary

Phase 12 is a pure output-formatting phase. The scanner logic and data structures are defined in Phase 11 (`src/vulnerability/cwe_scanner.rs`, `SastFinding`). This phase adds two new artifacts: a standalone `{project}_static_analysis.md` file (RPT-01), a new section appended to the existing `{project}_report.md` (RPT-02), and a stderr disclaimer printed at save time (RPT-03). All code must be wrapped in `#[cfg(feature = "internal")]`.

The implementation pattern is well-established: `save_console_report()` in `src/formats/console.rs` demonstrates the exact file-write idiom — build a `String` buffer with `writeln!`, then call `fs::write(path, output)?`. The new `save_static_analysis_report()` function follows this same pattern verbatim. The integration point in `src/main.rs` follows the same output-dispatch block structure as every other format saver.

All decisions are locked in CONTEXT.md — there are no architectural unknowns. The only open discretion items are the exact function signature and module placement (`console.rs` vs a new `sast_report.rs`), both of which have a clear guidance signal from the CONTEXT.md.

**Primary recommendation:** Add `save_static_analysis_report()` to `src/formats/console.rs` (same module as `save_console_report`, matched file-write pattern). Only create `src/formats/sast_report.rs` if the function body exceeds ~80 lines. Call it from `src/main.rs` inside `#[cfg(feature = "internal")]` after `save_console_report`.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Output filename is `{project}_static_analysis.md` — matches the `{project}_report.md` convention; same output directory as all other scan outputs (`--output-dir`).
- **D-02:** File is **always written** when `--features internal` is active and a static analysis run occurs, even if there are zero findings.
- **D-03:** Zero-findings case: write the file with a "No static analysis findings detected." prose line; the summary table still prints headers but a single note row replaces data rows.
- **D-04:** Per-component CWE summary table uses **one row per CWE per component** (Option B). Columns: `Component | CWE | Name | Count`.
- **D-05:** File:line findings are **grouped by component, then by CWE** — `## libfoo` → `### CWE-120 (Buffer overflow)` → bullet list of `- src/foo.c:42 — strcpy`.
- **D-06:** Zero-findings in the findings section: prose line "No static analysis findings detected." — not an empty grouped structure.
- **D-07:** "Static Analysis Findings" section is placed **after the CVE/vulnerability section** in `_report.md`. CVE section stays intact and first.
- **D-08:** `_report.md` includes only the **summary table** (same format as D-04) — not the full file:line findings list.
- **D-09:** When there are zero findings, the section is **still present** in `_report.md` with a "No static analysis findings" message.
- **D-10:** Disclaimer prints to **stderr when the static analysis report is saved** — inside/alongside the `save_static_analysis_report()` call. Exact text: `Pattern-based — complex data-flow vulnerabilities not covered`.
- **D-11:** Disclaimer is **also embedded** in `_static_analysis.md` as: `> **Note:** Pattern-based — complex data-flow vulnerabilities not covered.` (blockquote below the H1 title).

### Claude's Discretion

- Exact Rust function signature for `save_static_analysis_report()` — pick the form that fits cleanly alongside `save_console_report()` in `src/formats/console.rs` (or a new `src/formats/sast_report.rs` if the function is large enough).
- Whether to add a `--no-static-analysis-disclaimer` CLI flag — only add if trivial; skip if it adds complexity.

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| RPT-01 | Scan produces a separate `_static_analysis.md` report with per-component CWE summary table and file:line findings | `save_static_analysis_report()` writes this file; pattern mirrors `save_console_report()` |
| RPT-02 | Static analysis findings section is integrated into the main markdown scan report alongside CVE findings | `save_console_report()` gains a `sast_findings: &[SastFinding]` parameter; appends "Static Analysis Findings" section after the CVE block |
| RPT-03 | CLI prints disclaimer when static analysis runs: "Pattern-based — complex data-flow vulnerabilities not covered" | `eprintln!()` call inside `save_static_analysis_report()`, matching the `eprintln!("✓ ...")` confirmation pattern |
</phase_requirements>

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Write `_static_analysis.md` | Backend (Rust binary) | — | Same tier as all other output formatters |
| Inject "Static Analysis Findings" into `_report.md` | Backend (Rust binary) | — | `save_console_report()` owns `_report.md` generation; this section is appended there |
| Emit stderr disclaimer | Backend (Rust binary) | — | `eprintln!` at save time, consistent with all other file-save confirmations |
| Feature gating | Compile-time (`#[cfg(feature = "internal")]`) | — | Phase 10 established this for all scanner-related code; Phase 12 is scanner output |

---

## Standard Stack

Phase 12 introduces no new dependencies. All required capabilities are already in the codebase.

### Core (already present)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::fmt::Write` (writeln!) | stdlib | Buffer-building for markdown output | Exact pattern used in `save_console_report()` |
| `std::fs` | stdlib | `fs::write(path, output)?` for atomic file write | Same call as `save_console_report()` line 1857 |
| `anyhow::Result` | in Cargo.toml | Error propagation from file I/O | Project-wide error type |

[VERIFIED: codebase grep — `fs::write`, `writeln!`, `anyhow::Result` all present in `src/formats/console.rs`]

**Installation:** No new packages needed.

---

## Architecture Patterns

### System Architecture Diagram

```
main.rs (output dispatch block)
  └─ #[cfg(feature = "internal")]
       ├─ save_console_report(&sbom, ..., sast_findings)  → {project}_report.md
       │     └─ appends "## Static Analysis Findings" section (summary table only)
       └─ save_static_analysis_report(&project_name, out_dir, sast_findings) → {project}_static_analysis.md
             ├─ writes H1 + disclaimer blockquote
             ├─ writes per-component CWE summary table
             ├─ writes file:line findings grouped by component → CWE
             └─ eprintln! disclaimer to stderr
```

### Recommended Project Structure

No new directories. One new function (possibly in a new module if large):

```
src/
├── formats/
│   ├── console.rs        # save_static_analysis_report() added here (preferred)
│   ├── sast_report.rs    # alternative if function is large (>~80 lines)
│   └── mod.rs            # re-export save_static_analysis_report if sast_report.rs used
└── main.rs               # call site added in #[cfg(feature = "internal")] block
```

### Pattern 1: File Write (replicate exactly from `save_console_report`)

**What:** Build markdown into a `String` buffer with `writeln!`, write atomically with `fs::write`.
**When to use:** Every report file in this project.
**Example:**
```rust
// Source: src/formats/console.rs, lines 1119 and 1857 [VERIFIED: codebase read]
pub fn save_static_analysis_report(
    project_name: &str,
    out_dir: &Path,
    findings: &[SastFinding],
) -> Result<()> {
    let mut output = String::new();
    writeln!(output, "# Static Analysis Report\n")?;
    writeln!(output, "> **Note:** Pattern-based — complex data-flow vulnerabilities not covered.\n")?;
    // ... table and findings sections ...
    let path = out_dir.join(format!("{}_static_analysis.md", project_name));
    fs::write(&path, output)?;
    eprintln!("Pattern-based — complex data-flow vulnerabilities not covered");
    eprintln!("✓ Static analysis report saved to: {}", path.display());
    Ok(())
}
```

### Pattern 2: Passing Findings Into `save_console_report`

**What:** Add `sast_findings: &[SastFinding]` as a trailing parameter to `save_console_report`.
**When to use:** Consistent with how `supplier_resolver: Option<&SupplierResolver>` is threaded into formatters (Phase 8 D-11).
**Example:**
```rust
// Append after the existing CVE vulnerabilities section inside save_console_report
// Source pattern: Phase 11 context D-05 and Phase 8 D-11 [CITED: .planning/phases/11-lexical-scanner-cyclonedx-output/11-CONTEXT.md]
writeln!(output, "## Static Analysis Findings\n")?;
// ... summary table only (D-08) ...
```

### Pattern 3: Summary Table Format (D-04)

```markdown
| Component | CWE | Name | Count |
|-----------|-----|------|-------|
| libfoo | CWE-120 | Buffer overflow | 3 |
| libfoo | CWE-78 | OS command injection | 1 |
| libbar | CWE-242 | Use of inherently dangerous function | 2 |
```

Zero-findings row (D-03):
```markdown
| — | — | No static analysis findings detected. | — |
```

### Pattern 4: Findings Section Format (D-05)

```markdown
## libfoo

### CWE-120 (Buffer overflow)

- src/foo.c:42 — strcpy
- src/foo.c:91 — strcat

### CWE-78 (OS command injection)

- src/foo.c:107 — system
```

### Pattern 5: `main.rs` Dispatch Call Site

**What:** Add the static analysis report call after `save_console_report`, gated behind `#[cfg(feature = "internal")]`.
**Example:**
```rust
// Source pattern: main.rs lines 237–254, 325–335 [VERIFIED: codebase read]
// Inside the Console and All format arms, after save_console_report:
#[cfg(feature = "internal")]
{
    save_static_analysis_report(project_name, out_dir, &sast_findings)?;
}
```

The `sast_findings: Vec<SastFinding>` will be populated by Phase 11's `run_lexical_scanner()`. Phase 12 must assume it arrives as a parameter or a local variable in scope at the dispatch point.

### Anti-Patterns to Avoid

- **Don't stream writes:** All existing reports use buffer-then-write (`String` + `fs::write`). Do not use `BufWriter` or incremental writes to disk.
- **Don't add the section only when findings exist:** D-09 requires the "Static Analysis Findings" section always be present in `_report.md` when the feature is active, even for zero findings. Absence is ambiguous.
- **Don't skip the disclaimer for zero findings:** D-10 says disclaimer emits "when the static analysis report is saved" — the file is always saved (D-02), so the disclaimer always emits.
- **Don't use `println!` for the disclaimer:** All file-save confirmations use `eprintln!`. The disclaimer must also go to stderr.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Markdown table rendering | Custom table formatter | `writeln!` with `|` delimiters inline | Tables are simple ASCII; the rest of the codebase does this inline, no abstraction needed |
| File write with error handling | Custom wrapper | `fs::write(path, output)?` | Exact pattern already in `save_console_report` line 1857 |

**Key insight:** This phase is glue code — the scanner produces the data, the formatter writes strings. There is no complex logic to abstract.

---

## Runtime State Inventory

Step 2.5: SKIPPED — this is a greenfield output feature, not a rename/refactor phase.

---

## Environment Availability

Step 2.6: SKIPPED — no external dependencies. This phase adds Rust code only; the compiler and Cargo are already confirmed working (project is actively building).

---

## Common Pitfalls

### Pitfall 1: `SastFinding` struct not yet defined (Phase 11 dependency)

**What goes wrong:** Phase 12 references `SastFinding` but Phase 11 may not be complete when planning begins. If Phase 11 is not merged, Phase 12 code cannot compile.
**Why it happens:** Phase dependency — 12 depends on 11.
**How to avoid:** Planning should treat `SastFinding` fields as stable per Phase 11 CONTEXT.md D-01 (discretion: field names/types). The planner should record the expected struct shape (`component_name: String`, `file_path: String/PathBuf`, `line: u32/usize`, `cwe_id: u32`, `function_name: String`) and plan around it. If Phase 11 changes the struct, Phase 12 must adapt.
**Warning signs:** Compilation error on `SastFinding` import at Phase 12 implementation time — check `src/vulnerability/cwe_scanner.rs` for the actual struct definition.

### Pitfall 2: `save_console_report` signature change breaks all call sites

**What goes wrong:** Adding `sast_findings: &[SastFinding]` to `save_console_report` breaks all existing callers (there are two call sites in `main.rs`: Console arm and All arm, lines ~244 and ~325).
**Why it happens:** Rust enforces arity at compile time.
**How to avoid:** Update both call sites in `main.rs` simultaneously. In the non-internal build path, `sast_findings` is an empty slice `&[]` (or the parameter is itself feature-gated). The cleanest approach: only add the parameter inside `#[cfg(feature = "internal")]` — provide a separate non-feature signature or pass `&[]` from a non-gated empty `Vec`.
**Warning signs:** Compiler error "expected N arguments, found M" on `save_console_report` calls.

### Pitfall 3: Module not exported after adding `sast_report.rs`

**What goes wrong:** If `save_static_analysis_report` is added to a new `src/formats/sast_report.rs`, the function is invisible to `main.rs` unless re-exported in `src/formats/mod.rs`.
**Why it happens:** Rust module system requires explicit `pub use`.
**How to avoid:** Follow the existing pattern in `src/formats/mod.rs` — add `pub mod sast_report;` and `pub use sast_report::save_static_analysis_report;`. If the function stays in `console.rs`, also add it to the `pub use console::...` line in `mod.rs`.

### Pitfall 4: Feature gate absent on `SastFinding` import

**What goes wrong:** `use crate::vulnerability::cwe_scanner::SastFinding;` at the top of `console.rs` (or `sast_report.rs`) without `#[cfg(feature = "internal")]` causes a compilation error in non-internal builds.
**Why it happens:** `mod vulnerability` is itself feature-gated in `main.rs` and `lib.rs`, so the type doesn't exist in non-internal builds.
**How to avoid:** Gate the import and the entire new function with `#[cfg(feature = "internal")]`. Consistent with Phase 10 D-01 and D-04.

### Pitfall 5: Disclaimer emits twice

**What goes wrong:** If the disclaimer `eprintln!` is placed both inside `save_static_analysis_report()` AND at the call site in `main.rs`, it prints twice per run.
**Why it happens:** D-10 says "inside/alongside the `save_static_analysis_report()` call" — "alongside" could be misread as "in main.rs too".
**How to avoid:** Single `eprintln!` inside `save_static_analysis_report()` only. The confirmation line (`✓ Static analysis report saved to: ...`) follows it immediately.

---

## Code Examples

Verified patterns from the codebase:

### File-write pattern (from `save_console_report`)
```rust
// Source: src/formats/console.rs lines 1119, 1857 [VERIFIED: codebase read]
let mut output = String::new();
writeln!(output, "# SBOM Report\n")?;
// ... build content ...
fs::write(path, output)?;
Ok(())
```

### Save confirmation pattern (from `main.rs`)
```rust
// Source: src/main.rs line 254 [VERIFIED: codebase read]
eprintln!("✓ Console report saved to: {}", out_path.display());
```

### Project name derivation (from `main.rs`)
```rust
// Source: src/main.rs lines 238–239 [VERIFIED: codebase read]
let project_name = sbom.project_path.file_name()
    .and_then(|n| n.to_str()).unwrap_or("sbom");
```

### Output path construction (from `main.rs`)
```rust
// Source: src/main.rs lines 240–242 [VERIFIED: codebase read]
let out_dir = Path::new(out);
std::fs::create_dir_all(out_dir)?;
let out_path = out_dir.join(format!("{}_report.md", project_name));
```

### Feature gate block pattern (from `main.rs`)
```rust
// Source: src/main.rs lines 174–212 [VERIFIED: codebase read]
#[cfg(feature = "internal")]
{
    // ... scanner / formatter code ...
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| N/A — new feature | Buffer-then-write markdown | Phase 1+ | No streaming; single `fs::write` |

**No deprecated patterns apply to this phase.**

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `SastFinding` will have fields: component name, file path, line number, CWE ID, function name | Pitfall 1, Code Examples | Report formatter must match the actual struct fields; if Phase 11 uses different field names, the formatter code will not compile |
| A2 | Phase 11 will be complete and merged before Phase 12 is executed | Common Pitfalls | Phase 12 will not compile without `SastFinding` defined in `cwe_scanner.rs` |

[A1 is ASSUMED from Phase 11 CONTEXT.md D-01 discretion language — field names are explicitly at Claude's discretion in Phase 11, so the exact names are not locked]
[A2 is ASSUMED from the ROADMAP phase ordering — no verified signal that Phase 11 is complete]

---

## Open Questions

1. **`SastFinding` struct field names**
   - What we know: Phase 11 CONTEXT.md D-01 leaves field names at Claude's discretion. Required information: component name, file path, line number, CWE ID, function name.
   - What's unclear: Exact Rust field names and types (e.g., `file_path: PathBuf` vs `file_path: String`, `line: u32` vs `line: usize`).
   - Recommendation: The plan should include a Wave 0 task to read `src/vulnerability/cwe_scanner.rs` and confirm the struct definition before writing report formatter code. The planner can note the expected fields and flag the task as requiring struct verification before code is written.

2. **`save_console_report` parameter threading for non-internal builds**
   - What we know: Adding `sast_findings: &[SastFinding]` to `save_console_report` requires the type to be available, but `SastFinding` doesn't exist in non-internal builds.
   - What's unclear: Whether to (a) feature-gate the entire `save_console_report` function with two versions, (b) pass an opaque `&dyn Any` slice, or (c) make the SAST section addition purely additive inside a `#[cfg]` block within the function body without changing the signature.
   - Recommendation: Approach (c) — add the SAST section inside the function body using `#[cfg(feature = "internal")]` without changing the function signature. This avoids signature divergence and is the least-invasive change.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + cargo test harness |
| Config file | none (cargo.toml test harness is implicit) |
| Quick run command | `cargo test --features internal -- static_analysis` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| RPT-01 | `save_static_analysis_report()` writes a file with correct summary table and file:line findings | unit | `cargo test --features internal -- test_save_static_analysis_report` | No — Wave 0 |
| RPT-01 | Zero-findings case writes "No static analysis findings detected." prose | unit | `cargo test --features internal -- test_static_analysis_report_zero_findings` | No — Wave 0 |
| RPT-02 | `save_console_report()` includes "Static Analysis Findings" section after CVE block | unit | `cargo test --features internal -- test_console_report_includes_sast_section` | No — Wave 0 |
| RPT-03 | Disclaimer appears on stderr when report is saved | unit | `cargo test --features internal -- test_static_analysis_disclaimer` | No — Wave 0 |

Tests should live in `tests/format_tests/sast_report_tests.rs` following the existing `cyclonedx_tests.rs` pattern, and be added to `tests/format_tests/mod.rs`.

### Sampling Rate
- **Per task commit:** `cargo test --features internal -- static_analysis`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** `cargo test --features internal` full suite green before `/gsd-verify-work`

### Wave 0 Gaps
- [ ] `tests/format_tests/sast_report_tests.rs` — covers RPT-01, RPT-02, RPT-03
- [ ] Add `pub mod sast_report_tests;` to `tests/format_tests/mod.rs`

---

## Security Domain

This phase writes markdown files to the local filesystem at a user-specified `--output-dir`. No network calls, no auth, no user input reflected into shell. ASVS categories:

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | no | — |
| V3 Session Management | no | — |
| V4 Access Control | no | — |
| V5 Input Validation | low | File path comes from `--output-dir` CLI arg, already used by all other formatters — no new attack surface |
| V6 Cryptography | no | — |

No new threat patterns introduced. The disclaimer text and CWE names are all hardcoded strings — no user-controlled content is written to the output files except file paths and line numbers from the scanner, which originate from the local filesystem (not user input).

---

## Sources

### Primary (HIGH confidence)
- `src/formats/console.rs` lines 1109–1859 — `save_console_report` implementation, file-write pattern, markdown rendering idiom [VERIFIED: codebase read]
- `src/main.rs` lines 235–335 — output dispatch block, `project_name` derivation, call sites for all formatters [VERIFIED: codebase read]
- `src/formats/mod.rs` — module re-export pattern for formatters [VERIFIED: codebase read]
- `.planning/phases/12-static-analysis-report/12-CONTEXT.md` — all locked decisions [VERIFIED: codebase read]
- `.planning/phases/11-lexical-scanner-cyclonedx-output/11-CONTEXT.md` — `SastFinding` expected shape, scanner invocation pattern [VERIFIED: codebase read]
- `.planning/phases/10-internal-feature-gate/10-CONTEXT.md` — `#[cfg(feature = "internal")]` gating rules [VERIFIED: codebase read]

### Secondary (MEDIUM confidence)
- `tests/format_tests/cyclonedx_tests.rs` — test pattern to replicate for new `sast_report_tests.rs` [VERIFIED: codebase read]

### Tertiary (LOW confidence)
- None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — no new dependencies; all libraries already in project
- Architecture: HIGH — one new function following a verified existing pattern; integration points explicitly documented in CONTEXT.md
- Pitfalls: HIGH — all pitfalls derive from verified code inspection (module exports, feature gates, call-site counts)

**Research date:** 2026-05-09
**Valid until:** Until Phase 11 is executed and `SastFinding` struct is confirmed — A1 assumption should be re-verified against the actual struct before implementing the report formatter.
