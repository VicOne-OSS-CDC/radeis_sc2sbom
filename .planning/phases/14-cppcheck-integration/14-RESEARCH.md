# Phase 14: cppcheck-integration - Research

**Researched:** 2026-05-10
**Domain:** Rust subprocess invocation, cppcheck XML v2 parsing, SastFinding struct extension, deduplication
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** CWE resolution order: (1) read the `cwe` attribute from the cppcheck XML `<error>` element; (2) if absent or 0, look up the cppcheck error `id` in a static `&[(&str, u32)]` override table compiled into `cwe_scanner.rs`. If neither yields a non-zero CWE, skip the finding silently.
- **D-02:** The static override table corrects known mis-mappings and fills gaps for error IDs that cppcheck does not annotate with a `cwe` attribute. It is NOT the primary source — the XML attribute is.
- **D-03:** Include any cppcheck error ID that has a real CWE mapping, regardless of cppcheck severity tier (security, warning, or style). The CWE integer is the filter.
- **D-04:** Findings with no resolvable CWE (no XML attribute, not in override table) are silently dropped — never emitted with a sentinel value.
- **D-05:** Invoke `cppcheck --xml --xml-version=2 --enable=warning,style,security` once per entry in `component_dirs`, in the same loop structure as `run_lexical_scanner`. Each finding is directly attributed to the component name/ecosystem from the map key.
- **D-06:** `run_cppcheck_scanner(component_dirs, cppcheck_bin)` is called sequentially after `run_lexical_scanner` inside the same `#[cfg(feature = "internal")]` block in `main.rs`. Deduplication happens in the same block before `sast_findings` is finalized.
- **D-07:** Show an `indicatif` progress bar while cppcheck runs across components, consistent with the progress-bar pattern already used in the lexical scanner.
- **D-08:** After all components finish, emit a single completion line to stderr: `"cppcheck: {N} findings from {M} components"`.
- **D-09:** If cppcheck binary is not found (PATH lookup and `--cppcheck-path` both fail): `eprintln!("⚠ cppcheck not found — lexical-only results. Install cppcheck or use --cppcheck-path.")` and return an empty Vec. No abort, exit code 0.
- **D-10:** If a per-component cppcheck invocation exits non-zero or writes to stderr: `eprintln!` the component name and cppcheck's stderr, skip that component, continue. Consistent with broken-symlink tolerance pattern.
- **D-11:** Deduplication key: `(canonical_file_path, line, cwe_id)`. Call `Path::canonicalize()` on `file_path` strings from both sources before building the `HashSet` key, to handle relative vs. absolute path differences.
- **D-12:** When a `(file, line, cwe)` tuple appears in both lexical and cppcheck results: keep one `SastFinding` with `source = SastSource::Both`. The lexical finding's other fields (component attribution) are preserved as the base.
- **D-13:** Add `source: SastSource` field to `SastFinding`. Enum: `Lexical | Cppcheck | Both`. Lexical scanner findings set `Lexical`; cppcheck-only findings set `Cppcheck`; deduped duplicates set `Both`.
- **D-14:** All downstream consumers (`cyclonedx.rs`, `console.rs`, static analysis report) that pattern-match or construct `SastFinding` must be updated for the new field. `source` need not be surfaced in current outputs — it is metadata for Phase 15 (SARIF) and future use.
- **D-15:** Add `--cppcheck-path <PATH>` CLI flag (gated behind `#[cfg(feature = "internal")]`). When provided, use that binary path instead of PATH lookup. Follows the same `#[arg(long)]` pattern as other internal flags.

### Claude's Discretion

None specified — all implementation decisions are locked above.

### Deferred Ideas (OUT OF SCOPE)

- Suppress-list support (CPPCHECK-F1) — user-configurable file to silence known FP findings; future milestone
- CI timing annotation (CPPCHECK-F2) — log per-component cppcheck duration; future milestone
- Surfacing `source` field in static analysis report — Phase 15 (SARIF) is the right place to expose it
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| CPPCHECK-01 | Scanner invokes `cppcheck --xml --enable=warning,style,security` on component-mapped C/C++ dirs when cppcheck is on PATH | PATH lookup via `std::process::Command::new("cppcheck")` preflight; subprocess pattern from `python.rs` and `so_scanner.rs` |
| CPPCHECK-02 | cppcheck XML output is parsed and cppcheck IDs are mapped to CWE IDs; findings are emitted as `SastFinding` entries through the existing CycloneDX pipeline | quick-xml 0.30 `Reader::from_reader` + `try_get_attribute`; static override table for IDs without `cwe` attribute |
| CPPCHECK-03 | If cppcheck binary is not found on PATH, scanner logs a warning and continues with lexical-only results (no abort) | `Command::new("cppcheck").output().is_err()` preflight pattern from `python.rs:162` |
| CPPCHECK-04 | `--cppcheck-path` CLI flag allows specifying an explicit cppcheck binary location | `#[cfg(feature = "internal")] #[arg(long)]` pattern from `cli.rs` |
| CPPCHECK-05 | Findings from cppcheck and the lexical scanner are deduplicated by `(file, line, cwe)` tuple — no duplicate entries in output | `HashSet<(PathBuf, u32, u32)>` with `Path::canonicalize().unwrap_or_else(|_| path.to_path_buf())` pattern from `parsers/` |
</phase_requirements>

---

## Summary

Phase 14 integrates cppcheck as an optional external subprocess scanner into the existing `SastFinding` pipeline. The integration has three distinct technical sub-problems: (1) subprocess invocation with graceful degradation, (2) XML output parsing to extract CWE-annotated findings, and (3) deduplication against lexical scanner results before the combined `Vec<SastFinding>` reaches the formatters.

All three sub-problems have direct precedents in the existing codebase. Subprocess invocation mirrors `python.rs` and `so_scanner.rs`. XML parsing uses the already-vendored `quick-xml 0.30` crate, which is already used in `parsers/ros.rs` with the identical event-loop pattern. Deduplication uses `HashSet` with tuple keys, the same approach used in `parsers/mod.rs` and `parsers/source_scanner.rs`. No new dependencies are required.

The primary structural addition is the `SastSource` enum and the `source` field on `SastFinding`. This is a purely additive change, but it touches every call site that constructs a `SastFinding` — the lexical scanner (line 181 of `cwe_scanner.rs`), and all test fixtures in `tests/vulnerability_tests/cwe_scanner_tests.rs` and `tests/cyclonedx_sast_tests.rs` that directly construct `SastFinding` structs.

**Primary recommendation:** Implement in two sub-units — (A) `SastFinding` struct extension + consumer updates as a Wave 0 structural change, (B) `run_cppcheck_scanner` function + deduplication + CLI flag + tests as Wave 1 functionality.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| PATH lookup / binary discovery | Internal module (`cwe_scanner.rs`) | CLI args (`cli.rs`) | Binary resolution is scanner-local; CLI only passes override path |
| cppcheck subprocess invocation | Internal module (`cwe_scanner.rs`) | — | Scanner owns subprocess lifecycle |
| XML parsing | Internal module (`cwe_scanner.rs`) | — | Parser is co-located with scanner function |
| CWE mapping (XML attr + override table) | Internal module (`cwe_scanner.rs`) | — | Same file as lexical `CWE_RULES` table |
| Deduplication | `main.rs` (inside `#[cfg(feature = "internal")]` block) | — | D-06 specifies dedup happens at call site in main.rs |
| `SastFinding` struct + `SastSource` enum | `cwe_scanner.rs` | `vulnerability/mod.rs` (re-exports) | Struct ownership in scanner module |
| `--cppcheck-path` CLI flag | `cli.rs` | — | D-15 specifies `#[arg(long)]` pattern |
| Progress bar | `run_cppcheck_scanner` function | — | D-07; same `indicatif` pattern as `scan_directory` |
| Downstream output (CycloneDX, console, report) | `formats/cyclonedx.rs`, `formats/console.rs` | — | Struct-level change propagates; no behavioral change needed |

---

## Standard Stack

### Core (all already in Cargo.toml)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `quick-xml` | 0.30 [VERIFIED: Cargo.toml] | Parse cppcheck XML output | Already vendored; used in `parsers/ros.rs` with identical event-loop |
| `std::process::Command` | std | Spawn cppcheck subprocess | Established in `python.rs` and `so_scanner.rs` |
| `std::collections::HashSet` | std | Deduplication keyset | Already used throughout `parsers/` |
| `indicatif` | 0.17 [VERIFIED: Cargo.toml] | Progress bar during cppcheck run | Already imported in `scanner/mod.rs`; D-07 requires same pattern |

### No New Dependencies

No new crate dependencies are required for this phase. The `which` crate (for PATH lookup) is NOT needed — the existing pattern (`Command::new("cppcheck").output().is_err()` + `Command::new("cppcheck").output().ok()?.status.success()`) is sufficient for a preflight check. The binary is found by the OS via PATH when `Command::new("cppcheck")` is called.

**PATH lookup pattern used in project** (`python.rs:162`):
```rust
let check = Command::new("cppcheck").args(["--version"]).output();
if check.is_err() || !check.unwrap().status.success() {
    eprintln!("⚠ cppcheck not found — lexical-only results. Install cppcheck or use --cppcheck-path.");
    return Vec::new();
}
```

**Custom path variant** (when `--cppcheck-path` is provided, use `Command::new(path)` instead of `Command::new("cppcheck")`).

---

## Architecture Patterns

### System Architecture Diagram

```
main.rs (#[cfg(feature = "internal")] block)
  │
  ├── run_lexical_scanner(component_dirs) → Vec<SastFinding{source=Lexical}>
  │
  ├── run_cppcheck_scanner(component_dirs, cppcheck_bin) → Vec<SastFinding{source=Cppcheck}>
  │     │
  │     ├── PATH lookup / validate binary
  │     │     └── if not found: eprintln! warning, return Vec::new()
  │     │
  │     ├── for each (name, ecosystem, dir) in component_dirs:
  │     │     ├── Command::new(cppcheck_bin).args([--xml, --xml-version=2, ...]).output()
  │     │     ├── if exit non-zero: eprintln! component + stderr, continue
  │     │     └── parse_cppcheck_xml(stdout_bytes) → Vec<CppcheckRawFinding>
  │     │           └── for each raw finding:
  │     │                 ├── read cwe attr → u32 (primary)
  │     │                 ├── if cwe==0: lookup id in CPPCHECK_CWE_OVERRIDES
  │     │                 └── if cwe==0: drop silently
  │     │
  │     └── progress bar (indicatif spinner per D-07)
  │
  ├── deduplicate(lexical_findings, cppcheck_findings) → Vec<SastFinding>
  │     ├── HashSet<(PathBuf, u32, u32)> keyed by (canonical_path, line, cwe_id)
  │     ├── lexical findings → insert all, key seen
  │     └── cppcheck findings → if key seen: mark existing as SastSource::Both
  │                              else: insert as SastSource::Cppcheck
  │
  └── sast_findings = deduplicated Vec<SastFinding>
        │
        ├── cyclonedx.rs → build_sast_vulnerabilities(&sast_findings)
        ├── console.rs → save_console_report(..., &sast_findings)
        └── save_static_analysis_report(project_name, out_dir, &sast_findings)
```

### Recommended Project Structure

No new source files required beyond additions to existing files. All cppcheck logic lives in `src/vulnerability/cwe_scanner.rs` under the same `#![cfg(feature = "internal")]` gate.

```
src/vulnerability/
├── cwe_scanner.rs       # add: SastSource enum, source field on SastFinding,
│                        #      run_cppcheck_scanner(), parse_cppcheck_xml(),
│                        #      CPPCHECK_CWE_OVERRIDES table
├── mod.rs               # add: re-export SastSource, run_cppcheck_scanner
src/cli.rs               # add: cppcheck_path: Option<PathBuf> under #[cfg(feature = "internal")]
src/main.rs              # add: run_cppcheck_scanner call + deduplication block
src/formats/cyclonedx.rs # update: SastFinding construction (add source field)
src/formats/console.rs   # update: SastFinding construction (add source field)
tests/vulnerability_tests/
├── cwe_scanner_tests.rs # update: SastFinding{} literals need source field
├── cppcheck_tests.rs    # new: unit tests for parse_cppcheck_xml, dedup
tests/cyclonedx_sast_tests.rs  # update: SastFinding{} literals need source field
```

### Pattern 1: cppcheck XML v2 Structure

**Verified structure** [CITED: cppcheck.sourceforge.io/manual.html]:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<results version="2">
  <cppcheck version="2.x"/>
  <errors>
    <error id="bufferAccessOutOfBounds" severity="error"
           msg="Buffer is accessed out of bounds." verbose="..."
           cwe="788" inconclusive="false">
      <location file="src/foo.c" line="42" info="..."/>
    </error>
    <error id="uninitvar" severity="error"
           msg="Uninitialized variable: x" verbose="..."
           cwe="457">
      <location file="src/bar.c" line="10"/>
    </error>
    <error id="unusedVariable" severity="style"
           msg="Unused variable: y" verbose="...">
      <!-- no cwe attribute — needs override table or drop -->
      <location file="src/baz.c" line="5"/>
    </error>
  </errors>
</results>
```

**Key XML facts** [VERIFIED: cppcheck.sourceforge.io/manual.html + sourceforge.net discussion]:
- `cwe` attribute is present on `<error>` only when cppcheck has a CWE mapping for that error ID. It is absent (not "0") when unknown.
- A single `<error>` can have multiple `<location>` child elements. Use the FIRST `<location>` as the primary file/line for `SastFinding`.
- cppcheck writes its XML output to **stderr**, not stdout. Invoke with `.stderr(Stdio::piped())` and parse `output.stderr`.
- Root element is `<results version="2">`, not `<results>` alone.

### Pattern 2: quick-xml 0.30 Attribute Reading

**Source:** `src/parsers/ros.rs` (existing codebase usage) + Context7 `/tafia/quick-xml` [VERIFIED]

```rust
// Source: Context7 /tafia/quick-xml + existing ros.rs pattern
use quick_xml::events::Event;
use quick_xml::Reader;

fn parse_cppcheck_xml(xml_bytes: &[u8]) -> Vec<CppcheckRawFinding> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut findings = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.name().as_ref() == b"error" =>
            {
                // Read id, cwe attributes from <error>
                let mut error_id = String::new();
                let mut cwe: u32 = 0;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"id"  => error_id = String::from_utf8_lossy(&attr.value).into_owned(),
                        b"cwe" => cwe = String::from_utf8_lossy(&attr.value)
                                         .parse().unwrap_or(0),
                        _ => {}
                    }
                }
                // ... collect location children
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"location" => {
                // file and line attributes
                if let Ok(Some(attr)) = e.try_get_attribute("file") {
                    // attr.value is &[u8]; convert with from_utf8_lossy
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    findings
}
```

**Important:** quick-xml 0.30 uses `read_event_into(&mut buf)` (not `read_event()`), consistent with `ros.rs`. The `try_get_attribute("name")` method returns `Ok(Some(Attribute))` when present. [VERIFIED: Context7]

**cppcheck writes XML to stderr.** Capture with:
```rust
use std::process::Stdio;
let output = Command::new(&cppcheck_bin)
    .args(["--xml", "--xml-version=2", "--enable=warning,style,security",
           dir.to_str().unwrap_or(".")])
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .output()?;
// parse: &output.stderr
```

### Pattern 3: SastFinding Struct Extension

**Current struct** (`cwe_scanner.rs:23`) [VERIFIED: read source]:

```rust
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
}
```

**Extended struct** (D-13):

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SastSource { Lexical, Cppcheck, Both }

#[derive(Debug, Clone)]
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
    pub source: SastSource,   // new field
}
```

**All call sites that construct `SastFinding` literally** [VERIFIED: grep]:
1. `cwe_scanner.rs:181` — `scan_file()` → add `source: SastSource::Lexical`
2. `tests/vulnerability_tests/cwe_scanner_tests.rs` — multiple `SastFinding { ... }` literals
3. `tests/cyclonedx_sast_tests.rs:33` — `SastFinding { cwe_id: 120, ... }`
4. New `run_cppcheck_scanner` function → `source: SastSource::Cppcheck`

`SastFinding` is not destructured with `..` patterns in `cyclonedx.rs` or `console.rs` — those files access fields by name only. Adding a new field therefore does not break those consumers, but the struct update will trigger compile errors at construction sites.

### Pattern 4: Deduplication Logic

**D-11/D-12.** Occurs in `main.rs` inside the `#[cfg(feature = "internal")]` block, after both scanners return.

```rust
// Source: decision D-11, D-12 from CONTEXT.md
use std::collections::HashMap;

let mut deduped: Vec<SastFinding> = Vec::new();
// key: (canonical_path_string, line, cwe_id) → index in deduped
let mut seen: HashMap<(String, u32, u32), usize> = HashMap::new();

for f in lexical_findings {
    let canon = Path::new(&f.file_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&f.file_path));
    let key = (canon.to_string_lossy().into_owned(), f.line, f.cwe_id);
    seen.insert(key, deduped.len());
    deduped.push(f); // source already SastSource::Lexical
}

for f in cppcheck_findings {
    let canon = Path::new(&f.file_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&f.file_path));
    let key = (canon.to_string_lossy().into_owned(), f.line, f.cwe_id);
    if let Some(&idx) = seen.get(&key) {
        deduped[idx].source = SastSource::Both; // D-12
    } else {
        seen.insert(key, deduped.len());
        deduped.push(f); // source already SastSource::Cppcheck
    }
}
sast_findings = deduped;
```

**Note:** `HashMap` (not `HashSet`) is needed because D-12 requires mutating the existing entry (setting `source = Both`). A pure `HashSet` cannot do this.

### Pattern 5: indicatif Progress Bar

**Existing pattern** from `scanner/mod.rs:473–480` [VERIFIED: read source]:

```rust
// Source: src/scanner/mod.rs:473
use indicatif::{ProgressBar, ProgressStyle};

let pb = ProgressBar::new_spinner();
let style = ProgressStyle::default_spinner()
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .template("[cppcheck] Scanning {msg} {spinner}")
    .map_err(|_| /* ignore or eprintln */)?;
pb.set_style(style);
pb.enable_steady_tick(std::time::Duration::from_millis(100));
// in loop:
pb.set_message(format!("{} ({}/{})", name, i+1, total));
// after loop:
pb.finish_and_clear();
```

`run_cppcheck_scanner` is not fallible (it returns `Vec<SastFinding>` not `Result`), so `map_err` on the template should use `unwrap_or_default()` or ignore gracefully. Match `run_lexical_scanner`'s non-Result signature.

### Pattern 6: CLI Flag Addition

**Existing pattern** from `cli.rs` — all internal flags follow this layout [VERIFIED: read source]:

```rust
/// Path to cppcheck binary. When provided, uses this binary instead of PATH lookup. (v1.0.17)
#[cfg(feature = "internal")]
#[arg(long)]
pub cppcheck_path: Option<PathBuf>,
```

Passed from `main.rs` into `run_cppcheck_scanner` as `args.cppcheck_path.as_deref()` → `Option<&Path>`.

### Anti-Patterns to Avoid

- **Parsing cppcheck stdout:** cppcheck writes XML to stderr, not stdout. Piping stdout and parsing it yields empty output. [VERIFIED: cppcheck manual]
- **Asserting `cwe` attribute always present:** The `cwe` attribute is absent (not "0") on many error IDs that cppcheck hasn't mapped. Parse as `Option<u32>` or default to 0, then apply override table.
- **Using `HashSet` for dedup with mutation:** D-12 requires setting `source = Both` on an already-inserted finding. Use `HashMap<key, usize>` (index into Vec) to allow mutation.
- **Calling `Path::canonicalize()` without fallback:** On non-existent paths it returns `Err`. Always `.unwrap_or_else(|_| PathBuf::from(&f.file_path))` — consistent with `parsers/npm.rs:228` pattern [VERIFIED].
- **Implementing separate XML state machine for `<location>` nesting:** cppcheck emits `<location>` as self-closing empty elements inside `<error>`. quick-xml emits these as `Event::Empty`, not `Event::Start`+`Event::End`. Match on `Event::Empty` for location elements.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| XML parsing | Custom byte scanner / regex over XML | `quick-xml 0.30` (already in Cargo.toml) | Handles encoding, escaping, malformed input; already in use |
| Subprocess invocation | `libc::fork`/`exec` | `std::process::Command` | Safe, portable, already in `python.rs` and `so_scanner.rs` |
| PATH binary lookup | Manual `$PATH` env split + iterate | `Command::new("cppcheck").output()` preflight | OS handles PATH resolution; no `which` crate needed |
| CWE override table generation | Runtime fetch from NVD/cppcheck website | Static `&[(&str, u32)]` compiled in | Deterministic, no network, fast |

**Key insight:** No new dependencies are required. All required primitives (XML, subprocess, HashSet, progress bar) are already in `Cargo.toml` and used in the codebase.

---

## Common Pitfalls

### Pitfall 1: cppcheck XML Written to stderr, Not stdout

**What goes wrong:** Piping `output.stdout` and finding it empty; assuming cppcheck failed.
**Why it happens:** cppcheck sends `--xml` output to stderr by design (to allow stderr for errors and stdout for other output modes).
**How to avoid:** Always capture `.stderr(Stdio::piped())` and parse `output.stderr`. Leave stdout as `Stdio::null()` or inherit.
**Warning signs:** Empty XML parse, zero cppcheck findings even when cppcheck finds real issues.
[VERIFIED: cppcheck.sourceforge.io/manual.html]

### Pitfall 2: Missing `cwe` Attribute Is Absent, Not "0"

**What goes wrong:** Parsing `cwe` as `u32` and panicking or failing on absent attribute.
**Why it happens:** cppcheck omits the `cwe` attribute entirely when no mapping exists; it does NOT emit `cwe="0"`.
**How to avoid:** Parse attribute as `Option` or default to 0 when absent. Apply override table when result is 0.
**Warning signs:** `unwrap()` panic on attribute value parse, or override table never consulted.
[VERIFIED: cppcheck.sourceforge.io/manual.html — "only included when the CWE ID is known"]

### Pitfall 3: `<location>` Is an Empty Element (Event::Empty)

**What goes wrong:** Matching only `Event::Start` for `<location>` and never finding it.
**Why it happens:** cppcheck emits `<location file="..." line="..."/>` as self-closing — quick-xml emits `Event::Empty`, not `Event::Start`.
**How to avoid:** Match both `Event::Empty(ref e) if e.name().as_ref() == b"location"` in the loop.
**Warning signs:** All cppcheck findings have empty `file_path` strings.
[VERIFIED: sourceforge.net discussion thread + quick-xml docs]

### Pitfall 4: Test Fixtures Constructing SastFinding Directly Break

**What goes wrong:** Adding `source: SastSource` field causes compile errors in all `SastFinding { ... }` struct literals in tests.
**Why it happens:** Rust struct literals require all fields unless `..` spread is used.
**How to avoid:** Update every literal in `tests/vulnerability_tests/cwe_scanner_tests.rs` and `tests/cyclonedx_sast_tests.rs` to include `source: SastSource::Lexical` (or add `#[non_exhaustive]` — but that's a separate tradeoff). Wave 0 task should be the struct change + all consumer updates.
**Warning signs:** Compile errors on `tests/cyclonedx_sast_tests.rs:33`, `tests/vulnerability_tests/cwe_scanner_tests.rs`.
[VERIFIED: read test source files]

### Pitfall 5: Deduplication Must Use HashMap, Not HashSet

**What goes wrong:** Using `HashSet<DedupeKey>` and losing the ability to mark existing entries as `SastSource::Both`.
**Why it happens:** HashSet only tests membership; it cannot return a mutable reference to the existing element.
**How to avoid:** Use `HashMap<(String, u32, u32), usize>` where value is the index into the `Vec<SastFinding>`. On collision, mutate `deduped[idx].source = SastSource::Both`.
[ASSUMED — derived from D-12 requirement; no alternative was specified]

### Pitfall 6: indicatif template `map_err` in a non-Result function

**What goes wrong:** `ProgressStyle::template()` returns `Result`, but `run_cppcheck_scanner` returns `Vec<SastFinding>` not `Result<...>`.
**Why it happens:** Mismatch between scanner signature (matches `run_lexical_scanner` which is also non-Result) and `indicatif` API.
**How to avoid:** Use `.unwrap_or(ProgressStyle::default_spinner())` instead of `?`. Progress bar failure is non-fatal.
[VERIFIED: indicatif source; scanner/mod.rs uses `map_err` because scan_directory returns Result<_>]

---

## Code Examples

### Verified: Subprocess invocation pattern from codebase

```rust
// Source: src/parsers/python.rs:162 (verified read)
let check = Command::new("cppcheck").args(["--version"]).output();
if check.is_err() || !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
    eprintln!("⚠ cppcheck not found — lexical-only results. Install cppcheck or use --cppcheck-path.");
    return Vec::new();
}
```

### Verified: Subprocess with stderr capture from codebase

```rust
// Source: src/parsers/c/so_scanner.rs:141 pattern (verified read), adapted for stderr
use std::process::Stdio;
let output = Command::new(&cppcheck_bin)
    .args(["--xml", "--xml-version=2", "--enable=warning,style,security",
           dir.to_str().unwrap_or(".")])
    .stdout(Stdio::null())
    .stderr(Stdio::piped())
    .output();

match output {
    Err(e) => {
        eprintln!("cppcheck: failed to spawn for {}: {}", component_name, e);
        continue;
    }
    Ok(out) if !out.status.success() => {
        eprintln!("cppcheck: non-zero exit for {}: {}",
            component_name,
            String::from_utf8_lossy(&out.stderr));
        continue;
    }
    Ok(out) => {
        // parse out.stderr as XML
    }
}
```

### Verified: quick-xml attribute reading pattern from codebase

```rust
// Source: src/parsers/ros.rs:66 + Context7 /tafia/quick-xml (verified)
for attr in e.attributes().flatten() {
    match attr.key.as_ref() {
        b"id"  => error_id = String::from_utf8_lossy(&attr.value).into_owned(),
        b"cwe" => cwe = String::from_utf8_lossy(&attr.value).parse::<u32>().unwrap_or(0),
        b"file" => file = String::from_utf8_lossy(&attr.value).into_owned(),
        b"line" => line = String::from_utf8_lossy(&attr.value).parse::<u32>().unwrap_or(0),
        _ => {}
    }
}
```

### Verified: canonicalize with fallback from codebase

```rust
// Source: src/parsers/npm.rs:228 pattern (verified read)
let canon = Path::new(&f.file_path)
    .canonicalize()
    .unwrap_or_else(|_| PathBuf::from(&f.file_path));
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| cppcheck XML v1 (flat attributes) | cppcheck XML v2 (`--xml-version=2`) | cppcheck 1.xx | v2 adds `cwe` attribute, multi-location; D-05 specifies v2 explicitly |
| No CWE attribute in XML | `cwe` attribute added for known IDs | cppcheck 1.73 | Override table needed for IDs cppcheck hasn't mapped |

**Deprecated/outdated:**
- `--xml-version=1`: Legacy format, missing CWE attribute. Do not use. D-05 specifies `--xml-version=2`.
- `which` crate for PATH lookup: Not needed; `Command::new(bin).output()` performs PATH resolution natively.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `HashMap<key, usize>` (index-into-Vec) is the correct dedup structure to satisfy D-12 (mutate source to Both) | Deduplication Pattern | If D-12 is revised to allow a two-pass approach, a HashSet could work; impact is minor (refactor dedup block) |
| A2 | `run_cppcheck_scanner` should have the same non-Result return signature as `run_lexical_scanner` | Architecture | If caller needs to distinguish cppcheck failure modes, a Result return would be cleaner; impact is minor |
| A3 | cppcheck `--enable=warning,style,security` (as specified in D-05) does not require `--enable=all`; the CWE override table covers IDs in those three categories | Standard Stack | If important CWEs only surface under `--enable=all`, some findings would be missed; verify with test fixture |

---

## Open Questions

1. **CWE override table completeness**
   - What we know: D-01/D-02 specify a static `&[(&str, u32)]` override table for cppcheck error IDs without `cwe` attributes. No specific entries are listed in CONTEXT.md.
   - What's unclear: Which specific cppcheck error IDs need entries? This requires a cppcheck run on representative C/C++ code to audit which important IDs lack the `cwe` attribute.
   - Recommendation: The planner should include a task to audit cppcheck output on a representative fixture and populate the initial override table. Even an empty initial table is valid (D-04: findings without CWE are silently dropped).

2. **Progress bar visibility gating**
   - What we know: D-07 says "show an indicatif progress bar while cppcheck runs across components."
   - What's unclear: Should the progress bar be suppressed when there is only 1 component (same convention as lexical scanner)? The lexical scanner has no progress bar at all.
   - Recommendation: Show the bar unconditionally. It finishes quickly for small inputs and is consistent with the scan_directory spinner pattern.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| cppcheck binary | CPPCHECK-01 | No (not installed on dev machine) | — | D-09: graceful degradation to lexical-only with warning |
| Rust / cargo | Build | Yes | (system Rust) | — |
| `quick-xml` 0.30 | XML parsing | Yes (Cargo.toml) | 0.30 | — |
| `indicatif` 0.17 | Progress bar | Yes (Cargo.toml) | 0.17 | — |

**Missing dependencies with no fallback:** None that block development. cppcheck absence is handled by D-09 at runtime.

**Missing dependencies with fallback:** cppcheck binary — not installed on this machine, but D-09 specifies the graceful degradation path. Tests that exercise the actual subprocess should be marked `#[ignore]` or use a mock approach; unit tests for XML parsing and deduplication use in-process fixture data and do not require cppcheck.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | None — feature gate: `--features internal` |
| Quick run command | `cargo test --features internal -p radeis_sc2sbom -- cppcheck` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| CPPCHECK-01 | cppcheck invoked on component dirs when on PATH | unit (mock binary or test with `which cppcheck`) | `cargo test --features internal -- cppcheck_invocation` | No — Wave 0 |
| CPPCHECK-02 | XML parsed → CWE IDs → SastFinding entries | unit (fixture XML in-process) | `cargo test --features internal -- parse_cppcheck_xml` | No — Wave 0 |
| CPPCHECK-03 | Binary not found → warning to stderr, Vec::new() | unit | `cargo test --features internal -- cppcheck_not_found` | No — Wave 0 |
| CPPCHECK-04 | `--cppcheck-path` uses custom binary | unit | `cargo test --features internal -- cppcheck_custom_path` | No — Wave 0 |
| CPPCHECK-05 | Deduplication by (file, line, cwe) | unit | `cargo test --features internal -- deduplicate_findings` | No — Wave 0 |
| Struct update | SastFinding + SastSource compile without error | compile test | `cargo build --features internal` | Existing tests break until struct updated |

### Sampling Rate

- **Per task commit:** `cargo test --features internal -- cppcheck`
- **Per wave merge:** `cargo test --features internal`
- **Phase gate:** Full suite green (`cargo test --features internal`) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/vulnerability_tests/cppcheck_tests.rs` — covers CPPCHECK-01 through CPPCHECK-05
- [ ] Update `tests/vulnerability_tests/cwe_scanner_tests.rs` — add `source: SastSource::Lexical` to all `SastFinding {}` literals (currently ~6 construction sites)
- [ ] Update `tests/cyclonedx_sast_tests.rs` — add `source: SastSource::Lexical` to `SastFinding {}` literals (line 33)
- [ ] Update `tests/vulnerability_tests/mod.rs` — add `pub mod cppcheck_tests`

---

## Security Domain

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V2 Authentication | No | — |
| V3 Session Management | No | — |
| V4 Access Control | No | — |
| V5 Input Validation | Yes — cppcheck XML from external process | quick-xml handles malformed XML gracefully; parse errors return empty findings |
| V6 Cryptography | No | — |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Malformed cppcheck XML output | Tampering | quick-xml event loop returns Err on malformed input; treat as skip-component |
| Path traversal via `file` attribute in XML | Tampering | `file_path` is stored as string only; no filesystem operation is performed on it |
| Binary injection via `--cppcheck-path` | Elevation of privilege | `--cppcheck-path` is behind `#[cfg(feature = "internal")]`; only available in internal builds |

---

## Sources

### Primary (HIGH confidence)

- `src/vulnerability/cwe_scanner.rs` — verified SastFinding struct (lines 23–31), run_lexical_scanner signature (line 215), scan_file construction (line 181)
- `src/main.rs` — verified sast_findings declaration (line 193), run_lexical_scanner call site (line 239), #[cfg(feature = "internal")] block
- `src/cli.rs` — verified #[arg(long)], #[cfg(feature = "internal")] pattern for all internal flags
- `src/parsers/ros.rs` — verified quick-xml 0.30 `Reader::from_str` + `read_event_into` + attribute iteration pattern
- `src/parsers/python.rs:162` — verified subprocess availability check pattern
- `src/parsers/c/so_scanner.rs:141` — verified `Command::new().output()` + status check pattern
- `src/scanner/mod.rs:473` — verified indicatif `ProgressBar::new_spinner()` + `ProgressStyle` pattern
- `Cargo.toml` — verified quick-xml = "0.30", indicatif = "0.17", no `which` crate
- Context7 `/tafia/quick-xml` — verified `read_event_into`, `try_get_attribute`, `Event::Empty`, attribute iteration API
- [cppcheck manual](https://cppcheck.sourceforge.io/manual.html) — verified XML v2 structure: `<results version="2">`, `<cppcheck>`, `<errors>`, `<error id cwe severity msg>`, `<location file line>`; cwe attribute optional, written to stderr

### Secondary (MEDIUM confidence)

- [cppcheck sourceforge discussion](https://sourceforge.net/p/cppcheck/discussion/general/thread/1b26222530/) — confirmed `<location>` is direct child of `<error>` without wrapper, can have multiple occurrences
- [SonarOpenCommunity sonar-cxx issue #794](https://github.com/SonarOpenCommunity/sonar-cxx/issues/794) — confirmed `cwe` attribute added in cppcheck 1.73, is optional

### Tertiary (LOW confidence)

None.

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all dependencies verified in Cargo.toml; cppcheck XML format verified against official docs
- Architecture: HIGH — all call sites read directly from source; decision constraints from CONTEXT.md are complete
- Pitfalls: HIGH — stderr vs stdout verified against manual; attribute optionality verified against manual + discussion

**Research date:** 2026-05-10
**Valid until:** 2026-08-10 (cppcheck XML format is stable; quick-xml 0.30 API is stable)
