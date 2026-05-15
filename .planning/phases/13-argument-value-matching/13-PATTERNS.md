# Phase 13: argument-value-matching - Pattern Map

**Mapped:** 2026-05-10
**Files analyzed:** 1 (single file, in-place extension)
**Analogs found:** 1 / 1 (the file being modified is its own analog)

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src/vulnerability/cwe_scanner.rs` | utility (lexical scanner) | transform (line → findings) | itself (Phase 11 baseline) | exact — extend in-place |

## Pattern Assignments

### `src/vulnerability/cwe_scanner.rs` (utility, transform)

**Analog:** `src/vulnerability/cwe_scanner.rs` (existing Phase 11 code, lines 1–302)

This is the only file changed in Phase 13. All new code follows patterns already established within
the file. Each section below maps a Phase 13 addition to the exact existing code it must mirror.

---

#### Feature gate (line 13)

All code in this module is already gated. No per-function `cfg` needed.

```rust
// src/vulnerability/cwe_scanner.rs line 13
#![cfg(feature = "internal")]
```

---

#### CweRule struct — existing shape (lines 34–46)

Extend this struct by adding `arg_value_contains` as the last field. Every existing struct literal
in `CWE_RULES` (lines 52–69) must receive `, arg_value_contains: None`.

```rust
// src/vulnerability/cwe_scanner.rs lines 34–46 (current)
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    // ADD after format_arg_index:
    // arg_value_contains: Option<&'static [&'static str]>,
}
```

**After extension:**
```rust
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
    arg_value_contains: Option<&'static [&'static str]>,
}
```

---

#### Existing CWE_RULES entries — migration pattern (lines 52–69)

Every existing entry gains `, arg_value_contains: None`. Representative example:

```rust
// src/vulnerability/cwe_scanner.rs line 52 (before)
CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat"], requires_format_heuristic: false, format_arg_index: 0 },

// After adding the new field:
CweRule { cwe_id: 120, functions: &["gets", "strcpy", "strcat"], requires_format_heuristic: false, format_arg_index: 0, arg_value_contains: None },
```

Apply this mechanical change to all 15 existing entries (lines 52–69).

---

#### New CWE_RULES entries — pattern to copy (lines 52–69)

New rules follow the exact same struct literal layout. Format:

```rust
// Copy the layout of any existing entry; set arg_value_contains to Some(&[...]).
// CWE-295 (D-06):
CweRule {
    cwe_id: 295,
    functions: &["SSL_CTX_set_verify", "SSL_set_verify", "wolfSSL_CTX_set_verify"],
    requires_format_heuristic: false,
    format_arg_index: 0,
    arg_value_contains: Some(&["SSL_VERIFY_NONE"]),
},
// CWE-319 entries (D-07) — one per curl option:
CweRule {
    cwe_id: 319,
    functions: &["curl_easy_setopt"],
    requires_format_heuristic: false,
    format_arg_index: 0,
    arg_value_contains: Some(&["CURLOPT_USE_SSL", "CURLUSESSL_NONE"]),
},
// ... (three more CWE-319 entries following identical shape)
// CWE-732 entries (D-08):
CweRule {
    cwe_id: 732,
    functions: &["umask"],
    requires_format_heuristic: false,
    format_arg_index: 0,
    arg_value_contains: Some(&["0"]),
},
CweRule {
    cwe_id: 732,
    functions: &["SetSecurityDescriptorDacl"],
    requires_format_heuristic: false,
    format_arg_index: 0,
    arg_value_contains: Some(&["NULL"]),
},
```

---

#### `find_function_call` — word-boundary pattern to replicate (lines 124–151)

`paren_args_contain_all` must apply the same left/right word-boundary logic that `find_function_call`
uses for each token. The key pattern to copy:

```rust
// src/vulnerability/cwe_scanner.rs lines 124–151
fn find_function_call(line: &str, func: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let func_bytes = func.as_bytes();
    let mut search_from = 0usize;
    while let Some(rel) = line[search_from..].find(func) {
        let pos = search_from + rel;
        let left_ok = if pos == 0 {
            true
        } else {
            let prev = bytes[pos - 1];
            !(prev.is_ascii_alphanumeric() || prev == b'_')
        };
        if left_ok {
            let after_idx = pos + func_bytes.len();
            if after_idx <= bytes.len() {
                let after = &line[after_idx..];
                let trimmed = after.trim_start();
                if trimmed.starts_with('(') {
                    return Some(pos);
                }
            }
        }
        search_from = pos + 1;
        if search_from >= line.len() { break; }
    }
    None
}
```

**New `paren_args_contain_all` follows this loop structure** but operates on `after_func` (the
slice already starting at `(`), extracts the inner arg string up to matching `)` tracking paren
depth, then applies per-token word-boundary checks. The right-boundary check for numeric tokens
also excludes `.` (D-05).

---

#### `format_arg_is_literal` — paren-slice input convention (lines 78–118)

`paren_args_contain_all` receives the same `after_func: &str` input (slice starting at `(`). Copy
the `strip_prefix('(')` pattern and the paren-depth tracking loop:

```rust
// src/vulnerability/cwe_scanner.rs lines 78–89
fn format_arg_is_literal(after_func: &str, arg_index: u8) -> bool {
    let rest = after_func.trim_start();
    let rest = match rest.strip_prefix('(') {
        Some(r) => r,
        None => return false,
    };
    // ...
}

// Paren-depth loop pattern (lines 93–116):
let mut depth: i32 = 0;
// ...
match ch {
    '(' | '[' | '{' => depth += 1,
    ')' | ']' | '}' => {
        if depth == 0 { /* close outer paren */ }
        depth -= 1;
    }
    // ...
}
```

`paren_args_contain_all` uses the same `strip_prefix('(')` entry and depth tracking, but stops
only on `)` at depth 0 (not `]`/`}`) to extract the arg slice.

---

#### `scan_file` — rule loop and finding push (lines 154–193)

The CWE-369 path and `arg_value_contains` check are added inside this function. Copy the existing
loop structure and finding-push pattern:

```rust
// src/vulnerability/cwe_scanner.rs lines 172–193
for rule in CWE_RULES {
    for &func in rule.functions {
        if let Some(pos) = find_function_call(&line, func) {
            if rule.requires_format_heuristic {
                let after = &line[pos + func.len()..];
                if format_arg_is_literal(after, rule.format_arg_index) {
                    continue;
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
```

**Phase 13 modification:** After the `requires_format_heuristic` block and before the `findings.push`,
add an `arg_value_contains` check:

```rust
// Insert after format-heuristic block, before findings.push:
if let Some(tokens) = rule.arg_value_contains {
    let after = &line[pos + func.len()..];
    if !paren_args_contain_all(after, tokens) {
        continue;
    }
}
```

**CWE-369 path:** After the entire `for rule in CWE_RULES` loop (after line 190), add:

```rust
// After CWE_RULES loop, same SastFinding push shape:
if contains_div_by_zero(&line) {
    findings.push(SastFinding {
        cwe_id: 369,
        component_name: component_name.to_string(),
        component_ecosystem: component_ecosystem.to_string(),
        file_path: path.to_string_lossy().into_owned(),
        line: line_num,
    });
}
```

---

#### `run_lexical_scanner` — dedup pattern (lines 215–235)

The existing `HashMap` import on line 16 is extended to include `HashSet`. The dedup goes at the
end of `run_lexical_scanner` before returning `all_findings`:

```rust
// src/vulnerability/cwe_scanner.rs line 16 (extend):
use std::collections::HashMap;
// becomes:
use std::collections::{HashMap, HashSet};

// src/vulnerability/cwe_scanner.rs lines 215–235 (current return):
all_findings
// becomes:
let mut seen: HashSet<(String, u32, u32)> = HashSet::new();
all_findings.retain(|f| seen.insert((f.file_path.clone(), f.line, f.cwe_id)));
all_findings
```

---

#### Test pattern — tempfile + inline C + scan_file (lines 243–301)

All Phase 13 tests go in the existing `mod tests` block (line 238). Copy the established pattern:

```rust
// src/vulnerability/cwe_scanner.rs lines 243–247 (representative test skeleton)
#[test]
fn fallback_true_for_c_file() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("foo.c"), "int main() {}").unwrap();
    assert!(has_c_cpp_files(dir.path(), 3));
}
```

New tests replace `has_c_cpp_files` with `scan_file` and assert on `findings.iter().any(|f| f.cwe_id == N)`.
`use std::fs` is already imported (line 241). One test function per ARGVAL requirement (D-12).

---

#### Rule-count test to update (lines 292–301)

```rust
// src/vulnerability/cwe_scanner.rs lines 292–301 (current)
#[test]
fn test_rule_table_has_fourteen_cwes() {
    let mut ids: Vec<u32> = CWE_RULES.iter().map(|r| r.cwe_id).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 14, "SCAN-02 requires 14 distinct CWE IDs in CWE_RULES");
}
```

Rename to `test_rule_table_has_eighteen_cwes`. Update the assertion count from 14 to 18.
Update the doc comment to reference ARGVAL-01 through ARGVAL-04 as the four additions.
CWE-369 is detected via separate code path but counts as a distinct CWE in the test because
the test iterates `CWE_RULES` — CWE-369 must also appear as a rule-table entry OR the test
must be updated to account for CWE-369 being outside the table. Per D-09, CWE-369 is NOT a
`CweRule` entry. The test must be updated to count 17 from `CWE_RULES` plus 1 (369) hardcoded,
or the assertion logic must be adjusted to reflect this. **Recommendation:** assert 17 distinct
CWE IDs in `CWE_RULES` (CWE-295, CWE-319, CWE-732 added = 17) and add a separate assertion
that CWE-369 is detected by `scan_file`. Alternatively, if the decision is that the test counts
"all CWEs the scanner can detect" (including the separate code path), rename the test and assert
18 total, with CWE-369 verified separately via the scan_file path.

> **Clarification for planner:** D-11 says "assert 18 distinct CWE IDs" but CWE-369 is explicitly
> NOT a CweRule entry (D-09). The planner/implementer should assert 17 in CWE_RULES and verify
> CWE-369 detection via the ARGVAL-04 scan_file test. If the test counts all detectable CWEs
> (not just table entries), add a comment clarifying that 369 is counted separately.

---

## Shared Patterns

### Feature gate
**Source:** `src/vulnerability/cwe_scanner.rs` line 13
**Apply to:** All new code in this file — no action needed, the module-level gate already covers it.

### Graceful I/O error handling
**Source:** `src/vulnerability/cwe_scanner.rs` lines 154–169
```rust
let file = match std::fs::File::open(path) {
    Ok(f) => f,
    Err(_) => return Vec::new(),
};
// ...
Err(e) => {
    eprintln!("Warning: read error in {:?} at line {}: {}", path, line_idx + 1, e);
    continue;
}
```
**Apply to:** Any new I/O in Phase 13 (none expected — `scan_file` signature is unchanged).

### SastFinding construction
**Source:** `src/vulnerability/cwe_scanner.rs` lines 181–188
```rust
findings.push(SastFinding {
    cwe_id: rule.cwe_id,
    component_name: component_name.to_string(),
    component_ecosystem: component_ecosystem.to_string(),
    file_path: path.to_string_lossy().into_owned(),
    line: line_num,
});
```
**Apply to:** CWE-369 separate code path — copy this push verbatim, substituting `cwe_id: 369`.

### Manual byte scan loop
**Source:** `src/vulnerability/cwe_scanner.rs` lines 124–151 (`find_function_call`)
**Apply to:** `token_present_with_boundary` inner function and `contains_div_by_zero` — same
`while let Some(rel) = haystack[i..].find(token)` sliding-window pattern.

## No Analog Found

All Phase 13 code has a direct analog within `cwe_scanner.rs` itself. No files are without
codebase precedent.

## Metadata

**Analog search scope:** `src/vulnerability/cwe_scanner.rs` (only file modified)
**Files scanned:** 1
**Pattern extraction date:** 2026-05-10
