# Phase 14: cppcheck-integration - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 7 (6 primary + 1 mod.rs re-export update)
**Analogs found:** 7 / 7

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/cwe_scanner.rs` | service | batch + event-driven (subprocess) | `src/vulnerability/cwe_scanner.rs` itself (extend) + `src/parsers/python.rs` (subprocess) | exact (struct) + role-match (subprocess) |
| `src/vulnerability/mod.rs` | config (re-export) | — | `src/vulnerability/mod.rs` itself (extend) | exact |
| `src/cli.rs` | config | — | `src/cli.rs` itself (extend) | exact |
| `src/main.rs` | controller | request-response | `src/main.rs` itself (extend, lines 191–241) | exact |
| `src/formats/cyclonedx.rs` | service | transform | `src/formats/cyclonedx.rs` itself (struct update) | exact |
| `src/formats/console.rs` | service | transform | `src/formats/console.rs` itself (struct update) | exact |
| `tests/vulnerability_tests/cppcheck_scanner_tests.rs` | test | — | `tests/vulnerability_tests/cwe_scanner_tests.rs` | exact |

---

## Pattern Assignments

### `src/vulnerability/cwe_scanner.rs` — SastSource enum + source field + run_cppcheck_scanner

**Analogs:**
- Struct pattern: self (lines 22–31)
- Static override table: self (lines 50–69, `CWE_RULES`)
- Subprocess preflight: `src/parsers/python.rs` (lines 162–169)
- Subprocess invocation with error handling: `src/parsers/python.rs` (lines 180–213)
- quick-xml event loop: `src/parsers/ros.rs` (lines 47–82)
- Progress bar: `src/scanner/mod.rs` (lines 473–480)

---

**Pattern A: SastSource enum declaration** — new, add immediately before `SastFinding` struct

```rust
// Add above SastFinding (cwe_scanner.rs:22)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SastSource {
    Lexical,
    Cppcheck,
    Both,
}
```

---

**Pattern B: SastFinding struct extension** (lines 22–31 currently; add `source` field)

```rust
// src/vulnerability/cwe_scanner.rs:22–31 (current)
/// Single dangerous-function call site detected by the lexical scanner.
#[derive(Debug, Clone)]
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
}

// Extended version (D-13): add source field as last field
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,
    pub line: u32,
    pub source: SastSource,  // new — add this field
}
```

---

**Pattern C: Existing SastFinding construction site** (line 181 — add `source: SastSource::Lexical`)

```rust
// src/vulnerability/cwe_scanner.rs:181–187 (current)
findings.push(SastFinding {
    cwe_id: rule.cwe_id,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: line_num,
    // ADD: source: SastSource::Lexical,
});
```

---

**Pattern D: Static override table** — mirror `CWE_RULES` table style (lines 50–69)

```rust
// src/vulnerability/cwe_scanner.rs:50 — mirror this pattern for CPPCHECK_CWE_OVERRIDES
static CWE_RULES: &[CweRule] = &[
    CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat"], ... },
    // ...
];

// New table for cppcheck IDs without cwe attribute:
static CPPCHECK_CWE_OVERRIDES: &[(&str, u32)] = &[
    // ("cppcheck_error_id", cwe_id)
    // Populated during Wave 1 audit of cppcheck output
];
```

---

**Pattern E: Binary preflight check** — from `src/parsers/python.rs:162–169`

```rust
// src/parsers/python.rs:162–169
let pip_check = Command::new("pip").args(["--version"]).output();

if pip_check.is_err() || !pip_check.unwrap().status.success() {
    eprintln!("Warning: pip not available, skipping transitive dependency resolution");
    return Ok(LockFileData { ... });
}

// Adapted for cppcheck (D-09):
let bin = cppcheck_bin.unwrap_or_else(|| std::ffi::OsStr::new("cppcheck"));
let check = Command::new(bin).args(["--version"]).output();
if check.is_err() || !check.as_ref().map(|o| o.status.success()).unwrap_or(false) {
    eprintln!("⚠ cppcheck not found — lexical-only results. Install cppcheck or use --cppcheck-path.");
    return Vec::new();
}
```

---

**Pattern F: Subprocess invocation with stderr capture + error handling** — from `src/parsers/python.rs:180–213`, adapted for stderr piping (D-10)

```rust
// src/parsers/python.rs:180–213 — base pattern
let output = Command::new("pip")
    .args(["install", "--dry-run", "--report", "-", "--quiet", "-r", requirements_path])
    .output();

let output = match output {
    Ok(o) => o,
    Err(e) => {
        eprintln!("Warning: Failed to run pip for transitive resolution: {}", e);
        return Ok(LockFileData { ... });
    }
};

if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("Warning: pip resolution failed: {}", stderr);
    return Ok(LockFileData { ... });
}

// Adapted for cppcheck (cppcheck writes XML to stderr, not stdout — D-10):
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
        // parse out.stderr as XML — VERIFIED: cppcheck writes XML to stderr
        let findings = parse_cppcheck_xml(&out.stderr, name, ecosystem);
        all_findings.extend(findings);
    }
}
```

---

**Pattern G: quick-xml event loop with attribute reading** — from `src/parsers/ros.rs:47–82`

```rust
// src/parsers/ros.rs:47–82
let mut reader = Reader::from_str(&content);
reader.trim_text(true);
let mut buf = Vec::new();

loop {
    match reader.read_event_into(&mut buf) {
        Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
            for attr in e.attributes() {
                if let Ok(attr) = attr {
                    if attr.key.as_ref() == b"email" {
                        current_email = attr.unescape_value().ok().map(|s| s.to_string());
                    }
                }
            }
        }
        Ok(Event::Eof) => break,
        _ => {}
    }
    buf.clear();
}

// Adapted for cppcheck XML v2 — parse_cppcheck_xml():
// NOTE: quick-xml 0.30 uses read_event_into (not read_event)
// NOTE: <location> is Event::Empty (self-closing), not Event::Start
// NOTE: cwe attribute is absent (not "0") when cppcheck has no mapping
fn parse_cppcheck_xml(xml_bytes: &[u8], component_name: &str, component_ecosystem: &str) -> Vec<SastFinding> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    let mut findings = Vec::new();

    let mut current_error_id = String::new();
    let mut current_cwe: u32 = 0;
    let mut location_taken = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e))
                if e.name().as_ref() == b"error" =>
            {
                current_error_id.clear();
                current_cwe = 0;
                location_taken = false;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"id"  => current_error_id = String::from_utf8_lossy(&attr.value).into_owned(),
                        b"cwe" => current_cwe = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0),
                        _ => {}
                    }
                }
                // If cwe still 0, check override table
                if current_cwe == 0 {
                    for &(id, cwe) in CPPCHECK_CWE_OVERRIDES {
                        if id == current_error_id.as_str() {
                            current_cwe = cwe;
                            break;
                        }
                    }
                }
            }
            Ok(Event::Empty(ref e)) if e.name().as_ref() == b"location" && !location_taken && current_cwe != 0 => {
                // Use FIRST <location> only (D-02 — "use the first location")
                let mut file = String::new();
                let mut line: u32 = 0;
                for attr in e.attributes().flatten() {
                    match attr.key.as_ref() {
                        b"file" => file = String::from_utf8_lossy(&attr.value).into_owned(),
                        b"line" => line = String::from_utf8_lossy(&attr.value).parse().unwrap_or(0),
                        _ => {}
                    }
                }
                if !file.is_empty() && line > 0 {
                    findings.push(SastFinding {
                        cwe_id: current_cwe,
                        component_name: component_name.to_string(),
                        component_ecosystem: component_ecosystem.to_string(),
                        file_path: file,
                        line,
                        source: SastSource::Cppcheck,
                    });
                    location_taken = true;
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

---

**Pattern H: run_cppcheck_scanner function signature** — mirror `run_lexical_scanner` (lines 215–235)

```rust
// src/vulnerability/cwe_scanner.rs:215–235 — mirror signature
pub fn run_lexical_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
) -> Vec<SastFinding> {
    let mut all_findings = Vec::new();
    for ((name, ecosystem), dir) in component_dirs.iter() {
        if !dir.exists() { continue; }
        // ...
    }
    all_findings
}

// New function — same signature shape plus cppcheck_bin:
pub fn run_cppcheck_scanner(
    component_dirs: &HashMap<(String, String), PathBuf>,
    cppcheck_bin: Option<&std::ffi::OsStr>,
) -> Vec<SastFinding> {
    // preflight → loop per component → parse XML → return findings
}
```

---

**Pattern I: indicatif progress bar** — from `src/scanner/mod.rs:473–480`

```rust
// src/scanner/mod.rs:473–480
let spinner = ProgressBar::new_spinner();
let spinner_style = ProgressStyle::default_spinner()
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .template("[1/5] Walking directory tree... {msg} {spinner}")
    .map_err(|e| anyhow::anyhow!("Failed to create spinner template: {}", e))?;
spinner.set_style(spinner_style);
spinner.enable_steady_tick(std::time::Duration::from_millis(100));

// Adapted for run_cppcheck_scanner (non-Result return — use unwrap_or, not ?):
use indicatif::{ProgressBar, ProgressStyle};
let pb = ProgressBar::new_spinner();
let style = ProgressStyle::default_spinner()
    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    .template("[cppcheck] Scanning {msg} {spinner}")
    .unwrap_or_else(|_| ProgressStyle::default_spinner());
pb.set_style(style);
pb.enable_steady_tick(std::time::Duration::from_millis(100));
// in loop:
pb.set_message(format!("{} ({}/{})", name, i + 1, total));
// after loop:
pb.finish_and_clear();
eprintln!("cppcheck: {} findings from {} components", total_findings, total_components);
```

---

### `src/vulnerability/mod.rs` — add SastSource and run_cppcheck_scanner exports

**Analog:** self (lines 1–13)

**Current re-export pattern** (lines 11–13):

```rust
// src/vulnerability/mod.rs:11–13
#[cfg(feature = "internal")]
pub use cwe_scanner::{has_c_cpp_files, run_lexical_scanner, SastFinding};
```

**Extended re-export** — add `SastSource` and `run_cppcheck_scanner` to the existing line:

```rust
#[cfg(feature = "internal")]
pub use cwe_scanner::{has_c_cpp_files, run_cppcheck_scanner, run_lexical_scanner, SastFinding, SastSource};
```

---

### `src/cli.rs` — add --cppcheck-path flag

**Analog:** `src/cli.rs` existing internal flags pattern (lines 121–149)

**Existing internal flag pattern** (lines 121–124):

```rust
// src/cli.rs:121–124 — representative internal flag
/// Enable vulnerability checking (requires network connection)
#[cfg(feature = "internal")]
#[arg(long, action = ArgAction::Set, default_value_t = false)]
pub check_vulnerabilities: bool,

// Pattern with Option<PathBuf> — follow supplier_config (lines 238–240):
/// Path to custom BSW module config (YAML). Overrides bundled default. (v1.0.15)
#[arg(long)]
pub bsw_config: Option<PathBuf>,

// New flag to add (D-15) — combine both patterns:
/// Path to cppcheck binary. When provided, uses this binary instead of PATH lookup. (v1.0.17)
#[cfg(feature = "internal")]
#[arg(long)]
pub cppcheck_path: Option<PathBuf>,
```

Insertion point: after `supplier_config` field (line 271), before the closing `}` of `Args`.

---

### `src/main.rs` — add run_cppcheck_scanner call + deduplication block

**Analog:** self, lines 191–241 (the existing lexical scanner block)

**Current lexical scanner call site** (lines 237–241):

```rust
// src/main.rs:237–241
// Phase 11 (D-04): lexical CWE scanner runs after enrichment, before formatters.
// SCAN-05 scope: only scans component-mapped C/C++ directories.
sast_findings = crate::vulnerability::run_lexical_scanner(&component_dirs);
```

**Extension — add immediately after line 239** (still inside `#[cfg(feature = "internal")]` block):

```rust
// After run_lexical_scanner call — inside #[cfg(feature = "internal")] block
let lexical_findings = crate::vulnerability::run_lexical_scanner(&component_dirs);

let cppcheck_findings = crate::vulnerability::run_cppcheck_scanner(
    &component_dirs,
    args.cppcheck_path.as_deref().map(|p| p.as_os_str()),
);

// Deduplication (D-11, D-12) — HashMap<key, index> to allow source=Both mutation
use std::collections::HashMap;
use std::path::PathBuf;
let mut deduped: Vec<crate::vulnerability::SastFinding> = Vec::new();
let mut seen: HashMap<(String, u32, u32), usize> = HashMap::new();

for f in lexical_findings {
    let canon = std::path::Path::new(&f.file_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&f.file_path));
    let key = (canon.to_string_lossy().into_owned(), f.line, f.cwe_id);
    seen.insert(key, deduped.len());
    deduped.push(f);
}

for f in cppcheck_findings {
    let canon = std::path::Path::new(&f.file_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&f.file_path));
    let key = (canon.to_string_lossy().into_owned(), f.line, f.cwe_id);
    if let Some(&idx) = seen.get(&key) {
        deduped[idx].source = crate::vulnerability::SastSource::Both;
    } else {
        seen.insert(key, deduped.len());
        deduped.push(f);
    }
}
sast_findings = deduped;
```

---

### `src/formats/cyclonedx.rs` — handle new SastFinding.source field

**Analog:** self (lines 1–16 for import pattern, then wherever SastFinding is constructed or consumed)

**Import pattern** (lines 7–10):

```rust
// src/formats/cyclonedx.rs:7–10
#[cfg(feature = "internal")]
use crate::vulnerability::SastFinding;
```

**Update needed:** Add `SastSource` to the import if/when it is used in this file. Based on D-14, `source` is metadata only — no behavioral change needed in cyclonedx.rs for Phase 14. The struct field addition is purely additive; the file accesses fields by name so it will compile without changes as long as no `SastFinding { ... }` literals exist in this file. Verify with `cargo build --features internal`.

---

### `src/formats/console.rs` — handle new SastFinding.source field

**Analog:** self (lines 11–14 for import pattern)

**Import pattern** (lines 11–14):

```rust
// src/formats/console.rs:11–14
#[cfg(feature = "internal")]
use crate::vulnerability::cwe_scanner::SastFinding;
```

Same situation as cyclonedx.rs: no `SastFinding { ... }` construction in this file; field access is by name. No behavioral change needed for Phase 14. Verify with `cargo build --features internal`.

---

### `tests/vulnerability_tests/cppcheck_scanner_tests.rs` — new test file

**Analog:** `tests/vulnerability_tests/cwe_scanner_tests.rs` (full file — exact match)

**Test file structure** (cwe_scanner_tests.rs lines 1–12):

```rust
// tests/vulnerability_tests/cwe_scanner_tests.rs:1–12
use radeis_sc2sbom::vulnerability::{run_lexical_scanner, SastFinding};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_one_file(name: &str, contents: &[u8]) -> (TempDir, HashMap<(String, String), PathBuf>) {
    let tmp = TempDir::new().unwrap();
    std::fs::write(tmp.path().join(name), contents).unwrap();
    let mut m = HashMap::new();
    m.insert(("testlib".to_string(), "makefile".to_string()), tmp.path().to_path_buf());
    (tmp, m)
}
```

**New cppcheck test file header** — mirror this pattern:

```rust
// tests/vulnerability_tests/cppcheck_scanner_tests.rs
#![cfg(feature = "internal")]

use radeis_sc2sbom::vulnerability::{run_cppcheck_scanner, SastFinding, SastSource};
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;

// Helper: create fixture XML bytes (bypasses actual cppcheck binary)
fn cppcheck_xml_fixture(cwe: u32, error_id: &str, file: &str, line: u32) -> Vec<u8> {
    format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<results version="2">
  <cppcheck version="2.x"/>
  <errors>
    <error id="{error_id}" severity="error" cwe="{cwe}" msg="test">
      <location file="{file}" line="{line}"/>
    </error>
  </errors>
</results>"#).into_bytes()
}
```

**Test function pattern** — mirror cwe_scanner_tests.rs assertion style:

```rust
// Follow this assertion style from cwe_scanner_tests.rs:14–23
#[test]
fn test_cwe120_strcpy() {
    let (_t, dirs) = setup_one_file("a.c", b"void f(char *d, char *s){ strcpy(d, s); }\n");
    let findings = run_lexical_scanner(&dirs);
    assert!(findings.iter().any(|f| f.cwe_id == 120));
    let f = findings.iter().find(|f| f.cwe_id == 120).unwrap();
    assert_eq!(f.component_name, "testlib");
    assert_eq!(f.line, 1);
}
```

**Required test cases** (per RESEARCH.md validation map):
- `parse_cppcheck_xml_with_cwe_attr` — fixture XML with cwe attribute → SastFinding with correct cwe_id, file, line, source=Cppcheck
- `parse_cppcheck_xml_cwe_from_override_table` — fixture XML with no cwe attr + id in override table → correct CWE
- `parse_cppcheck_xml_no_cwe_dropped` — fixture XML with no cwe attr + id NOT in override table → empty Vec
- `cppcheck_not_found_returns_empty_vec` — invoke run_cppcheck_scanner with a nonexistent binary path → Vec::new() + no panic
- `deduplicate_findings_sets_both_source` — construct lexical + cppcheck findings with same (file, line, cwe) → merged entry with source=Both
- `deduplicate_findings_unique_entries_kept` — non-overlapping findings → all kept, source fields unchanged

---

### Existing test files that need `source` field update

**`tests/vulnerability_tests/cwe_scanner_tests.rs`** — no `SastFinding { }` literals (tests call `run_lexical_scanner` and check fields by name, not by constructing structs directly). No changes needed to this file.

**`tests/cyclonedx_sast_tests.rs`** — two `SastFinding { }` literals (lines 33–39 and 94–100) and one in helper function. **Each needs `source: SastSource::Lexical` added.**

```rust
// tests/cyclonedx_sast_tests.rs:33–39 — add source field
let finding = SastFinding {
    cwe_id: 120,
    component_name: "zlib".to_string(),
    component_ecosystem: "pkg-config".to_string(),
    file_path: "src/zlib.c".to_string(),
    line: 42,
    source: SastSource::Lexical,  // ADD THIS
};
```

**`tests/format_tests/sast_report_tests.rs`** — `make_finding` helper constructs `SastFinding { }` (lines 17–25). **Needs `source: SastSource::Lexical` added.**

```rust
// tests/format_tests/sast_report_tests.rs:17–25 — add source field
fn make_finding(component: &str, file: &str, line: u32, cwe_id: u32) -> SastFinding {
    SastFinding {
        cwe_id,
        component_name: component.to_string(),
        component_ecosystem: "vendored".to_string(),
        file_path: file.to_string(),
        line,
        source: SastSource::Lexical,  // ADD THIS
    }
}
```

Import to add in both test files:

```rust
use radeis_sc2sbom::vulnerability::SastSource;
// or for cwe_scanner path:
use radeis_sc2sbom::vulnerability::cwe_scanner::SastSource;
```

**`tests/vulnerability_tests/mod.rs`** — add new test module:

```rust
// tests/vulnerability_tests/mod.rs — add line:
#[cfg(feature = "internal")]
mod cppcheck_scanner_tests;
```

---

## Shared Patterns

### Feature Gate
**Source:** `src/vulnerability/cwe_scanner.rs:13`
**Apply to:** All new code in `cwe_scanner.rs` for cppcheck functionality; entire new scanner function

```rust
#![cfg(feature = "internal")]
// Module-level gate — entire file is already gated. No per-item cfg needed inside cwe_scanner.rs.
```

**Source:** `src/cli.rs:122`
**Apply to:** `--cppcheck-path` flag in `Args` struct

```rust
#[cfg(feature = "internal")]
#[arg(long)]
pub cppcheck_path: Option<PathBuf>,
```

### Warn-and-Continue Error Handling
**Source:** `src/parsers/python.rs:192–213`
**Apply to:** Per-component cppcheck subprocess failure (D-10)

```rust
// Pattern: match on output result, eprintln! component name + error, continue loop
match output {
    Err(e) => { eprintln!("cppcheck: failed to spawn for {}: {}", component_name, e); continue; }
    Ok(out) if !out.status.success() => {
        eprintln!("cppcheck: non-zero exit for {}: {}", component_name, String::from_utf8_lossy(&out.stderr));
        continue;
    }
    Ok(out) => { /* parse out.stderr */ }
}
```

### Canonicalize with Fallback
**Source:** `src/parsers/ros.rs:39–41`
**Apply to:** Deduplication key construction in `main.rs` (D-11)

```rust
// src/parsers/ros.rs:39–41
let absolute_path = package_xml_path
    .canonicalize()
    .unwrap_or_else(|_| package_xml_path.to_path_buf());

// In dedup block (main.rs): apply to both lexical and cppcheck file_path strings
let canon = std::path::Path::new(&f.file_path)
    .canonicalize()
    .unwrap_or_else(|_| PathBuf::from(&f.file_path));
```

### quick-xml Import Block
**Source:** `src/parsers/ros.rs:4–5`
**Apply to:** `parse_cppcheck_xml` function imports in `cwe_scanner.rs`

```rust
// src/parsers/ros.rs:4–5
use quick_xml::events::Event;
use quick_xml::Reader;
```

---

## No Analog Found

All files have close analogs in the codebase. No files require RESEARCH.md patterns as primary reference.

| File | Note |
|------|------|
| `CPPCHECK_CWE_OVERRIDES` table contents | Initial entries must be determined by auditing cppcheck on a C fixture. An empty `&[]` is valid for Wave 0 (D-04: silently drop when no CWE). |

---

## Metadata

**Analog search scope:** `src/vulnerability/`, `src/parsers/`, `src/scanner/`, `src/cli.rs`, `src/main.rs`, `src/formats/`, `tests/vulnerability_tests/`, `tests/cyclonedx_sast_tests.rs`, `tests/format_tests/`
**Files scanned:** 14 source files read directly
**Pattern extraction date:** 2026-05-10
