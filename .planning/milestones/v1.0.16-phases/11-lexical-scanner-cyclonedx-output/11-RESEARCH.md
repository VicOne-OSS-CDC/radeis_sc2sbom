# Phase 11: Lexical Scanner + CycloneDX Output - Research

**Researched:** 2026-05-09
**Domain:** Pure-Rust lexical CWE scanner + CycloneDX 1.5 vulnerability serialization
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Add `component_dirs: HashMap<(String, String), PathBuf>` to `ScanContext`. Key is `(name, ecosystem)`.
- **D-02:** Key choice is `(name, ecosystem)` — consistent with `dep_to_bom_ref` lookup in `cyclonedx.rs` (~line 261).
- **D-03:** Components with no recorded directory (e.g., so-scanner discoveries) are skipped — no path guessing.
- **D-04:** Lexical scanner runs **after** `enrich_cwe_ids`, immediately before formatters. Pipeline order: `scan_directory` → `query_vulnerabilities_batch` → `enrich_cwe_ids` → **`run_lexical_scanner`** → formatters.
- **D-05:** Scanner returns `Vec<SastFinding>`. Passed to CycloneDX formatter as trailing `&[SastFinding]` parameter.
- **D-06:** SPDX formatter signature is unchanged — no SAST findings.
- **D-07:** Rules are a `static` const array `&[CweRule]` — no external files, no config parsing.
- **D-08:** `CweRule` struct: `{ cwe_id: u32, functions: &'static [&'static str], requires_format_heuristic: bool }`. Flag is `true` only for CWE-134.
- **D-09:** One vulnerability entry per finding (file+line). Exact provenance per CDX-03.
- **D-10:** bom-ref format: `sast-{cwe_id}-{sanitized_path}-{line}` where `/` and `.` in path are replaced with `-`.
- **D-11:** `source.name = "radeis_sc2sbom static analysis"`. `analysis.state = "in_triage"`. `cwes` is integer array. All firm from REQUIREMENTS.md.

### Claude's Discretion

- Exact `SastFinding` struct field names/types (`file_path` as `PathBuf` or `String`, line as `u32` or `usize`).
- Whether `run_lexical_scanner` is a free function or method on a `LexicalScanner` struct.
- CWE-134 next-token heuristic implementation at byte/char level — firm it fires only when format arg is not a string literal; parsing approach is flexible.
- Whether to gate entire test module at `#[cfg(feature = "internal")]` or use `gated = true` annotations — consistent with Phase 10 D-09 pattern (module-level gate).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SCAN-01 | Scanner detects dangerous function calls in .c, .h, .cpp, .hpp, .cc files across component-mapped directories | WalkDir iteration + extension filter; `ScanContext.component_dirs` provides the directories |
| SCAN-02 | Scanner covers all 13 CWEs from the confirmed SEED-001 list | Static `CweRule` table — 13 rules, each with function list |
| SCAN-03 | CWE-134 format-arg heuristic — flags printf/fprintf/syslog family only when format arg is not a string literal | Next-token check: if first non-whitespace char after `(` is `"` → safe; else → finding |
| SCAN-04 | Scanner records file path and line number for each finding | Line counter during file iteration; `SastFinding` carries both |
| SCAN-05 | Scanner is scoped to component-mapped C/C++ directories only | Iterate only `component_dirs.values()`; skip missing/empty dirs |
| CDX-01 | Each finding emits CycloneDX 1.5 `vulnerabilities[]` entry with `cwes[]`, `source.name`, `analysis.state: "in_triage"` | New `CycloneDXVulnerabilityAnalysis` struct + `analysis` field on `CycloneDXVulnerability`; `source.url` made `Option<String>` |
| CDX-02 | Each entry includes `affects[].ref` linking to owning component bom-ref | `dep_to_bom_ref` pattern already in `cyclonedx.rs` — same lookup with `(name, ecosystem)` key |
| CDX-03 | File path and line number stored in `properties` as `sc2sbom:finding:file` / `sc2sbom:finding:line` | `CycloneDXProperty` already exists; add `properties` field to `CycloneDXVulnerability` |
| CDX-04 | SAST findings in CycloneDX only — SPDX 2.3 output is unchanged | SPDX formatter receives no `sast_findings` param; no changes to spdx.rs |
</phase_requirements>

---

## Summary

Phase 11 implements a pure-Rust lexical CWE scanner and integrates its findings into the CycloneDX 1.5 output. The scanner is a straightforward line-by-line token matcher — there are no external crates to add, no complex algorithms, and no new external dependencies. The entire implementation fits inside `src/vulnerability/cwe_scanner.rs` (the Phase 10 stub landing zone).

The primary technical work divides into four concerns: (1) populating `ScanContext.component_dirs` in each C/C++ manifest parser, (2) implementing the scanner loop with the CWE rule table and CWE-134 format-arg heuristic, (3) extending `CycloneDXVulnerability` with two missing fields (`analysis` and `properties`) and making `source.url` optional, and (4) threading the `&[SastFinding]` trailing parameter through the formatter call sites.

The main integration risk is that `CycloneDXVulnerabilitySource` currently requires a `url: String` field — SAST entries have no URL. This field must be made `Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]` before the SAST path can compile, which also touches how existing CVE entries set their source. That is a small, surgical change but must be planned as its own task to avoid breaking existing CycloneDX tests.

**Primary recommendation:** Implement in four sequential tasks — (1) extend `ScanContext` and manifest parsers, (2) implement the core scanner in `cwe_scanner.rs`, (3) extend the CycloneDX structs and formatter, (4) write gated tests.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| C/C++ file walking + token matching | Scanner (in-process Rust) | — | Pure filesystem + string ops; no network, no subprocess |
| Component-to-directory mapping | Scanner context (ScanContext) | Manifest parsers (population) | Each parser knows its file path at parse time; context carries the mapping forward |
| CWE rule table | Scanner (static const) | — | Static data, compile-time, zero runtime overhead (D-07) |
| CWE-134 format-arg heuristic | Scanner (per-rule flag) | — | Token lookahead is a scanner concern, not a formatter concern |
| SAST finding serialization | CycloneDX formatter | — | CycloneDX has a native vulnerability model; SPDX does not (CDX-04) |
| bom-ref resolution for SAST | CycloneDX formatter | — | Same `dep_to_bom_ref` HashMap already constructed at formatting time |

---

## Standard Stack

### Core

No new crates are needed. [VERIFIED: Cargo.toml]

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `walkdir` | 2.5 | C/C++ source file traversal | Already in Cargo.toml; established pattern throughout the scanner |
| `std::fs::File` + `std::io::BufReader` + `lines()` | stdlib | Line-by-line file reading | Zero-copy iteration; correct handling of partial-line files |
| `serde_json` + `serde` | 1.0 | CycloneDX JSON serialization | Already in Cargo.toml; all CDX structs already `#[derive(Serialize)]` |

### No New Dependencies

The lexical scanner requires only stdlib I/O. There is no case for adding `regex`, `nom`, `tree-sitter`, or any other parsing crate for Phase 11. Token matching is a `str::contains` or split-and-compare operation against a known function name list. [ASSUMED — training knowledge; consistent with the static-rule approach locked in D-07/D-08.]

---

## Architecture Patterns

### System Architecture Diagram

```
main.rs
  ├── scan_directory() ──────────────────────────────────────────────────────────────────┐
  │     └── manifest parsers (C/C++):                                                    │
  │           makefile.rs, cmake/, pkgconfig.rs, autotools.rs,                          │
  │           makefile_am.rs, vendored_3rdparty.rs                                      │
  │           └── populate ScanContext.component_dirs                                   │
  │                 (name, ecosystem) → PathBuf                               ──────────┤
  │                                                                                      │
  │   [#cfg(feature = "internal")]                                                       │
  ├── query_vulnerabilities_batch()  (OSV)                                              │
  ├── enrich_cwe_ids()               (NVD)                                              │
  ├── run_lexical_scanner(&scan_context.component_dirs)                                  │
  │     └── for each (name, ecosystem) → dir in component_dirs:                        │
  │           WalkDir(dir) → *.c, *.h, *.cpp, *.hpp, *.cc                             │
  │           for each file:                                                             │
  │             for each line:                                                           │
  │               for each CweRule:                                                      │
  │                 token match → optional format heuristic (CWE-134)                  │
  │                 → SastFinding { cwe_id, component_name, component_ecosystem,        │
  │                                 file_path, line }                                   │
  │           returns Vec<SastFinding>                            ──────────────────────┤
  │                                                                                      │
  └── formatters:                                                                        │
        save_cyclonedx_json(..., sast_findings: &[SastFinding])   ◄─────────────────────┘
          └── build_cyclonedx_vulnerabilities(deps, components)       (CVE path, unchanged)
          └── build_sast_vulnerabilities(sast_findings, components)   (new SAST path)
                  dep_to_bom_ref lookup → affects[].ref
                  → CycloneDXVulnerability per finding
        save_spdx_*()   (unchanged — no sast_findings param)
```

### Recommended Project Structure

No new directories are needed. All new files go in existing locations:

```
src/
├── vulnerability/
│   ├── cwe_scanner.rs   # Phase 10 stub → Phase 11 full implementation
│   └── mod.rs           # Add: pub mod cwe_scanner; pub use cwe_scanner::{SastFinding, run_lexical_scanner};
├── models/
│   └── dependency.rs    # Add component_dirs field to ScanContext
└── formats/
    └── cyclonedx.rs     # Extend CycloneDXVulnerability; add build_sast_vulnerabilities; update signatures
tests/
└── vulnerability_tests/
    └── cwe_scanner_tests.rs   # New — gated #[cfg(feature = "internal")]
```

### Pattern 1: Static CWE Rule Table

**What:** A `const` array of `CweRule` structs defined at the top of `cwe_scanner.rs`.
**When to use:** Compile-time data with no runtime construction overhead.

```rust
// Source: [VERIFIED: project decisions D-07/D-08 from 11-CONTEXT.md]
#[cfg(feature = "internal")]

struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
}

static CWE_RULES: &[CweRule] = &[
    CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat", "sprintf", "vsprintf"], requires_format_heuristic: false },
    CweRule { cwe_id: 78,  functions: &["system", "popen", "execl", "execlp", "execle", "execv", "execvp", "execvpe"], requires_format_heuristic: false },
    CweRule { cwe_id: 242, functions: &["gets", "mktemp"], requires_format_heuristic: false },
    CweRule { cwe_id: 327, functions: &["MD5", "MD5_Init", "SHA1", "SHA1_Init", "DES_ecb_encrypt", "EVP_md5", "EVP_sha1"], requires_format_heuristic: false },
    CweRule { cwe_id: 377, functions: &["tmpnam", "tempnam", "mktemp"], requires_format_heuristic: false },
    CweRule { cwe_id: 190, functions: &["malloc", "calloc", "realloc"], requires_format_heuristic: false },
    CweRule { cwe_id: 134, functions: &["printf", "fprintf", "sprintf", "snprintf", "vprintf", "vfprintf", "vsprintf", "vsnprintf", "syslog"], requires_format_heuristic: true },
    CweRule { cwe_id: 22,  functions: &["realpath", "getcwd", "chdir", "open", "fopen"], requires_format_heuristic: false },
    CweRule { cwe_id: 807, functions: &["getenv", "getlogin", "cuserid"], requires_format_heuristic: false },
    CweRule { cwe_id: 362, functions: &["access", "stat", "lstat"], requires_format_heuristic: false },
    CweRule { cwe_id: 367, functions: &["access", "stat"], requires_format_heuristic: false },
    CweRule { cwe_id: 20,  functions: &["atoi", "atol", "atof", "atoll", "strtol", "strtoul"], requires_format_heuristic: false },
    CweRule { cwe_id: 126, functions: &["strlen", "wcslen"], requires_format_heuristic: false },
    CweRule { cwe_id: 676, functions: &["gets", "scanf", "fscanf", "sscanf"], requires_format_heuristic: false },
];
```

**Important:** The exact function lists above are [ASSUMED] starters. The planner should treat these as a draft — the implementer must validate each list against MITRE's CWE descriptions and the project's SEED-001 list. CWE-190 (integer overflow in allocation), CWE-22 (path traversal), CWE-362/367 (race conditions), and CWE-807 (reliance on untrusted inputs) may need more representative function names than shown.

### Pattern 2: Line-by-Line Token Scanner with Format Heuristic

**What:** Read each file with `BufReader::lines()`, check each line for function token matches.
**When to use:** The only scan pattern for this phase — no AST required.

```rust
// Source: [ASSUMED — consistent with phase decisions and stdlib patterns]
#[cfg(feature = "internal")]
fn scan_file(path: &Path, component_name: &str, component_ecosystem: &str) -> Vec<SastFinding> {
    let file = match std::fs::File::open(path) { Ok(f) => f, Err(_) => return vec![] };
    let reader = std::io::BufReader::new(file);
    let mut findings = Vec::new();

    for (line_idx, line_result) in std::io::BufRead::lines(reader).enumerate() {
        let line = match line_result { Ok(l) => l, Err(_) => continue };
        let line_num = (line_idx + 1) as u32;

        for rule in CWE_RULES {
            for &func in rule.functions {
                if let Some(pos) = find_function_call(&line, func) {
                    if rule.requires_format_heuristic {
                        // CWE-134: only fire if format arg is NOT a string literal
                        if format_arg_is_literal(&line[pos + func.len()..]) {
                            continue; // safe — skip this occurrence
                        }
                    }
                    findings.push(SastFinding {
                        cwe_id: rule.cwe_id,
                        component_name: component_name.to_string(),
                        component_ecosystem: component_ecosystem.to_string(),
                        file_path: path.to_string_lossy().into_owned(),
                        line: line_num,
                    });
                }
            }
        }
    }
    findings
}

/// Return true if the format argument (everything after the opening paren)
/// starts with a string literal, i.e., the first non-whitespace char is `"`.
#[cfg(feature = "internal")]
fn format_arg_is_literal(after_func: &str) -> bool {
    // Skip past `(` and whitespace; check first content char
    let rest = after_func.trim_start_matches(|c: char| c == '(' || c.is_whitespace());
    rest.starts_with('"')
}
```

`find_function_call` should match `func` as a word boundary (preceded by non-alphanumeric/non-`_` or start of line, followed by `(`). A simple approach: check `line.contains(func)` and then verify the char before the match is not `[a-zA-Z0-9_]`. [ASSUMED — implementer chooses exact boundary logic per Claude's Discretion.]

### Pattern 3: CycloneDX SAST Vulnerability Construction

**What:** Convert each `SastFinding` to a `CycloneDXVulnerability` using existing structs.
**When to use:** Called in `build_sast_vulnerabilities` after the scanner runs.

Key struct gaps that must be added to `CycloneDXVulnerability` and its supporting types before this pattern works:

1. **`properties: Vec<CycloneDXProperty>`** — field does not exist on `CycloneDXVulnerability` today. [VERIFIED: read cyclonedx.rs lines 182-218]
2. **`analysis: Option<CycloneDXVulnerabilityAnalysis>`** — struct and field do not exist today. [VERIFIED: grepped cyclonedx.rs, no match]
3. **`source.url`** — currently `String` (non-optional). SAST entries have no URL. Must become `Option<String>`. [VERIFIED: read cyclonedx.rs lines 221-224]

```rust
// Source: [VERIFIED: struct layout from cyclonedx.rs + decisions D-09/D-10/D-11]
#[cfg(feature = "internal")]
fn sast_finding_to_vuln(
    finding: &SastFinding,
    dep_to_bom_ref: &HashMap<(String, String), String>,
) -> Option<CycloneDXVulnerability> {
    // Sanitize path: replace / and . with -
    let sanitized = finding.file_path.replace('/', "-").replace('.', "-");
    let bom_ref = format!("sast-{}-{}-{}", finding.cwe_id, sanitized, finding.line);

    let affects = dep_to_bom_ref
        .get(&(finding.component_name.clone(), finding.component_ecosystem.clone()))
        .map(|r| vec![CycloneDXVulnerabilityAffect { reference: r.clone() }])
        .unwrap_or_default();

    // Skip findings where we can't resolve a bom-ref (D-03 rationale)
    if affects.is_empty() {
        return None;
    }

    Some(CycloneDXVulnerability {
        bom_ref,
        id: format!("CWE-{}", finding.cwe_id),
        aliases: vec![],
        source: Some(CycloneDXVulnerabilitySource {
            name: "radeis_sc2sbom static analysis".to_string(),
            url: None,  // SAST entries have no advisory URL
        }),
        ratings: vec![],
        cwes: vec![finding.cwe_id],
        description: None,
        recommendation: None,
        published: None,
        updated: None,
        references: vec![],
        affects,
        analysis: Some(CycloneDXVulnerabilityAnalysis {
            state: "in_triage".to_string(),
        }),
        properties: vec![
            CycloneDXProperty { name: "sc2sbom:finding:file".to_string(), value: finding.file_path.clone() },
            CycloneDXProperty { name: "sc2sbom:finding:line".to_string(), value: finding.line.to_string() },
        ],
    })
}
```

### Anti-Patterns to Avoid

- **Over-matching function names:** `strlen` in a comment `/* strlen is O(n) */` must not fire. The token matcher must require `(` to follow the function name (with optional whitespace). A plain `str::contains` without the paren check will produce false positives in docstrings and comments.
- **Regex for what string ops can handle:** Adding `regex` for word-boundary matching adds compile time and crate weight. `str::find` + char boundary check is sufficient.
- **Cloning `dep_to_bom_ref`:** The same HashMap is already built inside `build_cyclonedx_vulnerabilities`. Reconstruct it once, pass a reference to both the CVE path and the SAST path. Do not rebuild it twice.
- **Panicking on non-UTF8 files:** C/C++ source files are almost always ASCII, but not guaranteed. Use `read_to_string` only when UTF-8 is assured, or use `BufRead::lines()` which handles line-level errors gracefully via `Result`.
- **Modifying `CycloneDXVulnerabilitySource.url` from `String` to `Option<String>` without updating all existing construction sites:** Two construction sites exist in `build_cyclonedx_vulnerabilities` that set `url: format!("https://osv.dev/...")`. These must be updated to `url: Some(format!(...))`. Failure to update them is a compile error.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| File traversal with vendor exclusion | Custom WalkDir wrapper | `walkdir::WalkDir` with `follow_links(true)` | Already used in `scan_directory`; consistent behavior |
| bom-ref deduplication | Custom HashMap | Use the `sast-{cwe_id}-{sanitized}-{line}` scheme (D-10) | Uniqueness is guaranteed by file path + line; no dedup needed |
| JSON serialization of new structs | Manual `impl Serialize` | `#[derive(Serialize)]` + `serde_json` | Established pattern for all CycloneDX structs |

**Key insight:** The lexical scanner has intentionally narrow scope (Phase 11 is a token-matcher, not an AST scanner — tree-sitter is deferred to v1.0.17). Every temptation to add sophistication — dataflow, taint tracking, comment exclusion — should be deferred.

---

## Common Pitfalls

### Pitfall 1: `CycloneDXVulnerabilitySource.url` Is Currently Non-Optional

**What goes wrong:** Adding SAST findings that set `url: None` fails to compile — the field is `String`, not `Option<String>`. [VERIFIED: cyclonedx.rs line 222]
**Why it happens:** The struct was built for CVE entries that always have an OSV advisory URL.
**How to avoid:** Change `url: String` to `url: Option<String>` with `#[serde(skip_serializing_if = "Option::is_none")]` as part of the struct-extension task. Update the two existing CVE construction sites: `url: Some(format!("https://osv.dev/..."))`.
**Warning signs:** Compiler error "expected `String`, found `Option<...>`" in the SAST formatter.

### Pitfall 2: `CycloneDXVulnerability` Has No `properties` or `analysis` Fields

**What goes wrong:** CDX-01 (`analysis.state`) and CDX-03 (`properties`) cannot be emitted until these fields are added.
**Why it happens:** The existing struct was built only for CVE data, which does not use these fields.
**How to avoid:** Add both fields in the same struct-extension task as Pitfall 1. Use `#[serde(skip_serializing_if = "Vec::is_empty", default)]` for `properties` and `#[serde(skip_serializing_if = "Option::is_none")]` for `analysis` so existing CVE serialization is byte-for-byte unchanged (CVE entries set neither field, so they serialize identically).
**Warning signs:** CDX-01/CDX-03 requirements cannot be satisfied; CONTEXT.md D-11 fields missing from output.

### Pitfall 3: `component_dirs` Population Scope

**What goes wrong:** If only some C/C++ parsers populate `component_dirs`, SAST findings will be missing for components detected by un-updated parsers.
**Why it happens:** There are six C/C++ parser files listed in CONTEXT.md canonical_refs. Each must be updated separately.
**How to avoid:** Update all six parsers in a single task (or verify each individually): `makefile.rs`, `cmake/`, `pkgconfig.rs`, `autotools.rs`, `makefile_am.rs`, `vendored_3rdparty.rs`.
**Warning signs:** Integration test against a project with mixed C build systems produces findings for some components but not others.

### Pitfall 4: `scan_directory` Signature Must Pass `component_dirs` Back

**What goes wrong:** `ScanContext` gains a new field but the callers and construction sites in `scanner/mod.rs` don't populate it, so it's always empty.
**Why it happens:** `ScanContext` is constructed at the end of `scan_directory`; parsers must write into a `component_dirs` HashMap during parsing and return it (or write into a mutable reference).
**How to avoid:** Add `component_dirs: HashMap<(String, String), PathBuf>` to `ScanContext::default()` as an empty map. Parser functions that currently return `Vec<Dependency>` need to also return (or accept `&mut`) a `component_dirs` entry. The simplest approach: pass a `&mut HashMap<(String, String), PathBuf>` into each C/C++ parser call site within `scan_directory`, so parsers can write into it directly. This avoids changing parser return types.
**Warning signs:** `scan_context.component_dirs` is always empty; `run_lexical_scanner` produces zero findings even against test projects with C sources.

### Pitfall 5: Feature Gate Scope of `component_dirs`

**What goes wrong:** `ScanContext.component_dirs` compiles fine in `internal` builds but the non-internal build fails because `HashMap<(String, String), PathBuf>` is always present in `ScanContext` regardless of feature flag.
**Why it happens:** `PathBuf` and `HashMap` are stdlib — not gated — so the field itself can compile unconditionally. The question is whether the parser-side population logic should be gated.
**How to avoid:** Keep `component_dirs: HashMap<(String, String), PathBuf>` unconditional on `ScanContext` (it's zero-cost when empty). Gate only the `run_lexical_scanner` call in `main.rs` and the scanner code itself behind `#[cfg(feature = "internal")]`. This minimizes cfg churn in parsers — they always populate `component_dirs`, and the data is only consumed when the feature is active. [ASSUMED — this is the approach CONTEXT.md code_context hints at: "the `component_dirs` field is always present but populated only under the feature flag — researcher should assess which approach minimizes cfg churn"; the unconditional-field approach is cleaner.]
**Warning signs:** Compile errors in non-internal builds referencing `component_dirs` population sites.

### Pitfall 6: Token Match Without Paren Check Fires in Comments

**What goes wrong:** The line `/* strlen returns length */` fires for CWE-126 because `strlen` is present.
**Why it happens:** `str::contains` has no context awareness.
**How to avoid:** After matching the function name, verify that the next non-whitespace character is `(`. Comment lines often have the function name followed by a space or other punctuation, not `(`. This single check eliminates comment false positives without needing a full comment-stripping pass.
**Warning signs:** Test fixture files with commented function names produce unexpected findings.

---

## Code Examples

### SastFinding Struct

```rust
// Source: [VERIFIED: decisions D-04/D-05 + discretion note on field types from 11-CONTEXT.md]
#[cfg(feature = "internal")]
#[derive(Debug, Clone)]
pub struct SastFinding {
    pub cwe_id: u32,
    pub component_name: String,
    pub component_ecosystem: String,
    pub file_path: String,   // relative or absolute string — planner decides
    pub line: u32,
}
```

### New CycloneDX Structs Required

```rust
// Source: [VERIFIED: missing from cyclonedx.rs + D-11 from CONTEXT.md]
// Add alongside existing CycloneDXVulnerability supporting structs

#[derive(Debug, Serialize)]
struct CycloneDXVulnerabilityAnalysis {
    state: String,  // "in_triage" for SAST findings
}
```

Add to `CycloneDXVulnerability` struct (both fields are skipped for CVE entries, preserving existing output):

```rust
#[serde(skip_serializing_if = "Option::is_none")]
analysis: Option<CycloneDXVulnerabilityAnalysis>,

#[serde(skip_serializing_if = "Vec::is_empty", default)]
properties: Vec<CycloneDXProperty>,
```

### `convert_to_cyclonedx` Signature Change

```rust
// Source: [VERIFIED: existing signature in cyclonedx.rs line 394; D-05 adds sast_findings param]
pub fn convert_to_cyclonedx(
    sbom: &Sbom,
    mode: &SbomMode,
    supplier_resolver: Option<&SupplierResolver>,
    sast_findings: &[SastFinding],   // new trailing param — empty slice when feature not active
) -> CycloneDXDocument
```

The `save_cyclonedx_json` and `print_cyclonedx_json` public functions also gain the trailing param and thread it through to `convert_to_cyclonedx`.

When `feature = "internal"` is inactive, `sast_findings` is not passed (the call sites in `main.rs` are gated). The non-internal build must still compile. Two options: (a) always add the parameter and pass `&[]` at call sites, or (b) use a conditional compilation block. Option (a) is simpler — `&[SastFinding]` compiles unconditionally because `SastFinding` can be an unconditional empty struct or the param can accept `&[]` at the type level. However, since `SastFinding` itself is `#[cfg(feature = "internal")]`, option (a) requires the param to also be gated. **Use option (b): wrap the SAST path in `cyclonedx.rs` with `#[cfg(feature = "internal")]` blocks, and add a non-internal no-op version of `convert_to_cyclonedx` without the param.** [ASSUMED — implementer should pick the cleanest approach that doesn't break non-internal builds.]

---

## Runtime State Inventory

> This is a pure new-feature phase (no rename/refactor). Omit full table.

Not applicable. Phase 11 adds new code; nothing is renamed or migrated.

---

## Environment Availability

No new external dependencies or services. All required tools are already in Cargo.toml. [VERIFIED: Cargo.toml]

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `walkdir` | SCAN-01 file traversal | ✓ | 2.5 | — |
| `serde` + `serde_json` | CDX-01..CDX-03 serialization | ✓ | 1.0 | — |
| `#[cfg(feature = "internal")]` gate | All scanner code | ✓ | set in Cargo.toml | — |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test runner |
| Config file | `Cargo.toml` `[dev-dependencies]` |
| Quick run command | `cargo test --features internal -p radeis_sc2sbom -- vulnerability_tests` |
| Full suite command | `cargo test --features internal` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SCAN-01 | Scanner detects dangerous calls in C/C++ files | unit | `cargo test --features internal -- cwe_scanner` | ❌ Wave 0 |
| SCAN-02 | All 13 CWEs fire on fixture with known calls | unit | `cargo test --features internal -- test_all_13_cwes` | ❌ Wave 0 |
| SCAN-03 | CWE-134 fires on variable format arg, skips literal | unit | `cargo test --features internal -- test_cwe134_heuristic` | ❌ Wave 0 |
| SCAN-04 | Findings record correct file path and line number | unit | `cargo test --features internal -- test_finding_location` | ❌ Wave 0 |
| SCAN-05 | Scanner only scans component-mapped dirs | unit | `cargo test --features internal -- test_scope_restriction` | ❌ Wave 0 |
| CDX-01 | CycloneDX output includes cwes[], source.name, analysis.state | unit | `cargo test --features internal -- cyclonedx_tests` | ✅ (needs extension) |
| CDX-02 | affects[].ref links to correct component bom-ref | unit | `cargo test --features internal -- cyclonedx_tests` | ✅ (needs extension) |
| CDX-03 | properties contain sc2sbom:finding:file and :line | unit | `cargo test --features internal -- cyclonedx_tests` | ✅ (needs extension) |
| CDX-04 | SPDX output byte-for-byte unchanged after adding sast_findings | unit | `cargo test -- spdx_tests` (no feature flag) | ✅ (existing; run baseline) |

### Sampling Rate

- **Per task commit:** `cargo test --features internal -- cwe_scanner`
- **Per wave merge:** `cargo test --features internal && cargo test` (both feature variants)
- **Phase gate:** Full suite green in both `--features internal` and default (no feature) before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/vulnerability_tests/cwe_scanner_tests.rs` — covers SCAN-01..SCAN-05; requires C/C++ fixture files in `tests/fixtures/c/`
- [ ] `tests/fixtures/c/dangerous_calls.c` — fixture with one call per CWE rule; also include a safe printf to verify CWE-134 non-fire
- [ ] Add `mod cwe_scanner_tests;` to `tests/vulnerability_tests/mod.rs` gated at `#[cfg(feature = "internal")]`

---

## Security Domain

> `security_enforcement` not configured — treating as enabled.

This phase implements the scanner itself, not an authenticated service.

| ASVS Category | Applies | Standard Control |
|---------------|---------|-----------------|
| V5 Input Validation | yes | `BufRead::lines()` handles malformed files gracefully — no panic on non-UTF8 lines |
| V6 Cryptography | no | No cryptographic operations |
| V2 Authentication | no | No user-facing auth |
| V3 Session Management | no | No sessions |
| V4 Access Control | no | Tool runs with invoker's filesystem permissions; no privilege escalation |

### Known Threat Patterns

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Path traversal via malformed `component_dirs` value | Spoofing | `PathBuf` and WalkDir stay within the provided dir; no `..` resolution needed for scanner |
| Denial-of-service via giant file | DoS | `BufRead::lines()` is streaming; no full-file load into memory |
| False positive injection via crafted C source | Tampering | Not a security concern for a SAST tool; false positives are a quality issue, not a security issue |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `component_dirs` field can be unconditional on `ScanContext` (stdlib types, zero-cost when empty) — only the population code and consumer are gated | Pitfall 5 | Non-internal builds may have compile errors if field access leaks through cfg guards; easy to fix |
| A2 | The function lists in the CWE rule table draft cover the SEED-001 intent — exact lists may need adjustment | Pattern 1 (CWE rule table) | False negatives (missed dangerous calls) or false positives (over-broad matching) |
| A3 | `convert_to_cyclonedx` uses conditional compilation for the SAST param (option b in Code Examples) rather than always-present `&[]` | Code Examples (signature change) | Non-internal build breaks if approach is wrong; compile error surfaced immediately |
| A4 | No new crates are needed for lexical token matching | Standard Stack | If stdlib string ops prove insufficient for accurate word-boundary detection, a regex may be needed (small impact) |
| A5 | xZETA treats `source.name != "OSV"` entries differently from CVE entries when `analysis.state = "in_triage"` | Open Questions | xZETA may surface SAST findings as CVE-like remediation tickets regardless of source.name (blocker noted in STATE.md) |

**If any A1-A4 are wrong:** Compile errors surface immediately during implementation — low risk of silent failure. A5 is an integration risk with xZETA that cannot be verified from source code alone.

---

## Open Questions (RESOLVED)

1. **xZETA SAST Ingestion Behavior (from STATE.md blocker)** — RESOLVED: Proceed per RESEARCH.md recommendation; validate with xZETA post-ship.
   - What we know: xZETA is the primary downstream consumer. All `vulnerabilities[]` entries in CycloneDX output are ingested by xZETA.
   - What's unclear: Does xZETA treat entries with `source.name = "radeis_sc2sbom static analysis"` + `analysis.state = "in_triage"` differently from CVE entries (i.e., no auto-remediation ticket), or does it create remediation tickets for all vulnerability entries regardless?
   - Recommendation: Validate with xZETA team before Phase 11 ships to production. This is a known open blocker per STATE.md. The implementation can proceed — if xZETA behavior is undesirable, the `source.name` or `analysis` fields can be adjusted post-implementation without structural changes.
   - **RESOLVED:** Implementation proceeds with `source.name = "radeis_sc2sbom static analysis"` and `analysis.state = "in_triage"` per D-11. Post-ship validation against xZETA ingestion pipeline is tracked as a manual verification in 11-VALIDATION.md.

2. **`SastFinding.file_path` — Relative or Absolute?** — RESOLVED: Use absolute path as returned by WalkDir (Claude's Discretion); executor may choose relative if preferred.
   - What we know: CDX-03 stores it in `sc2sbom:finding:file` as a string value.
   - What's unclear: Should the path be relative to the scan root (portability) or absolute (debuggability)? The bom-ref sanitization scheme (D-10) implies the path is used as an identifier — relative paths are shorter and more consistent across machines.
   - Recommendation: Use path relative to the scan root (i.e., relative to `args.path`). Consistent with how other file-path outputs work in the tool.
   - **RESOLVED:** Falls under Claude's Discretion (CONTEXT.md). Default: use absolute path as returned by WalkDir (path.to_string_lossy()); executor MAY choose to convert to scan-root-relative if it improves portability without altering bom-ref uniqueness.

---

## Sources

### Primary (HIGH confidence)
- `src/formats/cyclonedx.rs` — direct read; verified struct layout, missing fields, existing dep_to_bom_ref pattern
- `src/models/dependency.rs` lines 443-455 — direct read; verified current `ScanContext` fields
- `src/main.rs` lines 173-212 — direct read; verified `#[cfg(feature = "internal")]` block placement and post-`enrich_cwe_ids` insertion point
- `src/vulnerability/mod.rs` — direct read; verified module exports and missing `cwe_scanner` mod
- `.planning/phases/11-lexical-scanner-cyclonedx-output/11-CONTEXT.md` — user locked decisions D-01..D-11
- `.planning/REQUIREMENTS.md` SCAN-01..SCAN-05, CDX-01..CDX-04 — direct read

### Secondary (MEDIUM confidence)
- `.planning/phases/10-internal-feature-gate/10-CONTEXT.md` — D-07/D-08 stub location; D-09 test gating pattern
- `Cargo.toml` — verified no new dependencies are needed; `internal = ["dep:reqwest"]` feature confirmed

### Tertiary (LOW confidence)
- CWE function lists in Pattern 1 example — [ASSUMED] based on common SAST convention; needs validation against MITRE CWE descriptions and SEED-001

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — verified against Cargo.toml, no new deps
- Architecture: HIGH — verified against actual source files; all integration points confirmed
- Struct gaps (CycloneDXVulnerability): HIGH — verified by direct read of cyclonedx.rs lines 182-218
- CWE function lists: LOW — assumed from training data; must be validated against SEED-001 before implementation
- xZETA compatibility: LOW — cannot verify from source; open blocker

**Research date:** 2026-05-09
**Valid until:** 2026-06-09 (stable Rust project; no external API changes expected)
