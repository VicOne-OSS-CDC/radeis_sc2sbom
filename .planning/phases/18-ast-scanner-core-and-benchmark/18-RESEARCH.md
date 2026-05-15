# Phase 18: ast-scanner-core-and-benchmark — Research

**Researched:** 2026-05-11
**Domain:** Rust / tree-sitter-c AST-based CWE scanner, integration wiring, static musl build
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Feature Flag & Integration**
- D-01: AST scanner merges into `feature = "internal"` — the separate `feature = "ast-scanner"` is dropped.
- D-02: AST scanner is the primary C/C++ scanner when `internal` is active. Lexical scanner is the per-file fallback when tree-sitter fails to parse.
- D-03: `SastSource` gets a new `Ast` variant. AST findings use `SastSource::Ast`.
- D-04: `ast_scanner.rs` emits `Vec<SastFinding>` directly. No intermediate type or conversion layer.

**Rule Expansion (AST argument inspection)**
- D-05: Rules use a data-driven table (analogous to `CWE_RULES`). Each rule declares: CWE ID, function name(s), and an `ArgCheck` enum variant.
- D-06: `ArgCheck` is a Rust enum with variants (e.g., `FixedSizeBuffer`, `NotStringLiteralAtIndex(u8)`, `ContainsTokens(&'static [&'static str])`, `AnyCall`). Enum, not closures.
- D-07: Per-CWE AST argument inspection for the 11 tractable CWEs: CWE-78, 119, 120, 122, 125, 134, 190, 242, 327, 369, 377, 732.
- D-08: CWE-362, CWE-416, CWE-476 deferred from Phase 18. They stay in the lexical fallback path.
- D-09: Existing CWE-120 PoC logic (fixed-array destination check) is ported as the `FixedSizeBuffer` ArgCheck variant.

**Benchmark**
- D-10: Benchmark is a Rust integration test at `tests/benchmark.rs`, behind `#[cfg(feature = "internal")]`. Not CI-gated — runs locally only.
- D-11: Test gracefully skips (with `eprintln!`) when fixture directories are not present.
- D-12: Two fixtures: AUTOSAR_SampleProject_S32K144 (primary) and a Juliet Test Suite C/C++ subset (secondary).
- D-13: Benchmark produces a committed `BENCHMARK.md` (at repo root or `docs/BENCHMARK.md`).
- D-14: `BENCHMARK.md` columns: CWE ID | AST TPs | AST FPs | AST FP% | cppcheck TPs | cppcheck FPs | cppcheck FP% | Lexical TPs | Lexical FPs | Lexical FP% — per fixture, per CWE.

**Grammar Embedding**
- D-15: Try `tree-sitter-c 0.24`'s built-in `build.rs` first. Only write a custom `build.rs` if the crate's internal build script fails to cross-compile cleanly.
- D-16: Grammar must be embedded in the binary (no runtime grammar file dependency).
- D-17: DIST-01 (license verification): confirm tree-sitter-c MIT license in `Cargo.toml` or a `LICENSE-NOTES.md`.

### Claude's Discretion

No explicit discretion areas defined — all architectural decisions are locked.

### Deferred Ideas (OUT OF SCOPE)

- CWE-416 (use-after-free) local heuristic
- CWE-476 (null deref) local heuristic
- CWE-362 (race condition) local heuristic
- Content-based fingerprinting
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| AST-01 | User can run sc2sbom with embedded tree-sitter-c AST scanner as the default C/C++ analysis path (no cppcheck required) | D-01/D-02 wiring in main.rs `#[cfg(feature = "internal")]` block; cppcheck graceful-degradation path already exists and can be kept as secondary |
| AST-02 | AST scanner detects all CWEs from the v1.0.17 rule set on AUTOSAR_SampleProject_S32K144 | 11 tractable CWEs get AST argument inspection (D-07); 3 deferred CWEs (362/416/476) remain via lexical fallback; note: requirements list 15 CWEs but document says "14" — see Open Questions |
| AST-03 | AST scanner produces `SastFinding` output compatible with existing SARIF writer, markdown report, and CycloneDX serializer — no downstream changes required | D-04: emit `Vec<SastFinding>` directly; downstream writers accept `&[SastFinding]` unchanged |
| AST-04 | Parse failure falls back to lexical scan for that file with a warning logged | Integration point in scanner loop: if `parser.parse()` returns `None`, call `scan_file()` (lexical) for that path |
| BENCH-01 | AST scanner benchmarked against cppcheck on AUTOSAR fixture and at least one additional fixture — FP rates documented | `tests/benchmark.rs` with `#[cfg(feature = "internal")]`; outputs `BENCHMARK.md` |
| DIST-01 | tree-sitter-c grammar license verified as MIT-compatible | VERIFIED: tree-sitter-c 0.24.2 license = "MIT" (Max Brunsfeld, 2014) in crate Cargo.toml and LICENSE file |
| DIST-02 | Binary compiled with tree-sitter-c grammar embedded — no runtime file system dependency; static musl build verified | tree-sitter-c's `build.rs` compiles `src/parser.c` via `cc::Build` into a static archive; grammar is statically linked |
</phase_requirements>

---

## Summary

Phase 18 wires the existing tree-sitter-c PoC (`ast_scanner.rs`) as the primary C/C++ scanner under the `internal` feature flag, expands it to the full 11-tractable-CWE rule set with per-CWE AST argument inspection, and produces a benchmark comparing AST vs. cppcheck vs. lexical results on reference fixtures.

The key integration change is in `main.rs`'s `#[cfg(feature = "internal")]` block: instead of calling `run_lexical_scanner()` unconditionally, the new primary path calls an AST scanner that iterates component files, attempts to parse each with tree-sitter, and falls back to `scan_file()` (lexical) for files where `parser.parse()` returns `None`. The `deduplicate_sast_findings()` and downstream writer calls are unchanged — they consume `&[SastFinding]` regardless of source.

Two build changes are required: (1) merge the `ast-scanner` feature into `internal` in `Cargo.toml`, and (2) verify the `tree-sitter-c 0.24.2` crate's built-in `build.rs` compiles cleanly for `x86_64-unknown-linux-musl`. The crate's `build.rs` uses `cc::Build` to compile a single `src/parser.c` into a static archive — this is the standard approach and musl-compatible. The grammar is embedded at compile time; no runtime file dependency exists.

**Primary recommendation:** Expand `ast_scanner.rs` in-place with the data-driven `AstCweRule`/`ArgCheck` table (mirroring `CWE_RULES` in `cwe_scanner.rs`), wire it as the primary scanner in `main.rs` with per-file lexical fallback, merge the feature flag, and write `tests/benchmark.rs` with graceful skip for missing fixtures.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| AST-based CWE detection | `src/vulnerability/ast_scanner.rs` | `src/vulnerability/cwe_scanner.rs` (fallback) | AST scanner is the new primary; lexical scanner is per-file fallback on parse failure |
| Scanner dispatch (primary vs. fallback) | `src/main.rs` (#[cfg(internal)] block) | — | The `run_lexical_scanner` call site in main.rs is where dispatch order changes |
| SastFinding deduplication | `src/vulnerability/cwe_scanner.rs` | — | `deduplicate_sast_findings()` is unchanged; AST findings and lexical fallback findings merge before dedup |
| SARIF / CDX / markdown output | `src/writers/*.rs` | — | Downstream writers accept `&[SastFinding]` — no changes required (AST-03) |
| Build-time grammar embedding | `tree-sitter-c 0.24.2` build.rs | `Cargo.toml` [features] | `cc::Build` compiles `parser.c` into static archive at build time |
| Benchmark reporting | `tests/benchmark.rs` | `BENCHMARK.md` | Integration test (not CI-gated) writes the artifact |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tree-sitter | 0.25.10 | Parser runtime, `Node`, `TreeCursor`, `Parser` types | Already in Cargo.lock; locks to this version |
| tree-sitter-c | 0.24.2 | C grammar for tree-sitter; compiles `parser.c` via `cc` | Already in Cargo.lock; MIT license verified |
| cc (build dep) | 1.2 | Compiles `parser.c` into static archive at build time | Pulled in by tree-sitter-c's build.rs; no action needed |

[VERIFIED: Cargo.lock and `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tree-sitter-c-0.24.2/`]

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| tempfile | 3.15 | Temporary test dirs for unit tests | Already in dev-dependencies; use in `ast_scanner.rs` `#[cfg(test)]` |
| walkdir | 2.5 | Per-component directory traversal | Reuse existing `WalkDir` loop from lexical scanner pattern |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `ArgCheck` enum | Closures / trait objects | D-06 explicitly prohibits closures — enum is exhaustive, no heap allocation, compiler-checked |
| Per-file fallback to lexical | Parse-error recovery only | Full-file lexical fallback is simpler and preserves recall; recovery would require tree-sitter-specific error handling |

**Installation:** No new dependencies. `tree-sitter` and `tree-sitter-c` are already in `Cargo.toml` as optional deps under `ast-scanner`. The only change is merging them into `internal`.

**Cargo.toml change:**
```toml
# Before:
internal = ["dep:reqwest"]
ast-scanner = ["dep:tree-sitter", "dep:tree-sitter-c"]

# After:
internal = ["dep:reqwest", "dep:tree-sitter", "dep:tree-sitter-c"]
# (drop the ast-scanner feature entirely)
```

[VERIFIED: `/Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/Cargo.toml`]

---

## Architecture Patterns

### System Architecture Diagram

```
scan_directory() → component_dirs HashMap
        │
        ▼ (main.rs, #[cfg(feature = "internal")], --check-vulnerabilities)
┌─────────────────────────────────────────────────────┐
│  run_ast_scanner(component_dirs)                    │
│    for each component_dir:                          │
│      for each .c/.h/.cpp/.hpp/.cc file:             │
│        try tree-sitter parse                        │
│          ┌── success → apply AstCweRule table ──┐   │
│          └── None     → run_lexical_scan_file() ┘   │
│                         + eprintln! warning          │
│    returns Vec<SastFinding> (source: Ast or Lexical) │
└─────────────────────────────────────────────────────┘
        │ ast_findings (primary)
        │
        ▼
┌────────────────────────────────────────────────────┐
│  run_cppcheck_scanner(component_dirs)              │
│    (secondary — graceful degradation if missing)   │
└────────────────────────────────────────────────────┘
        │ cppcheck_findings
        │
        ▼
deduplicate_sast_findings(ast_findings + lexical_fallback, cppcheck_findings)
        │
        ▼
suppress_lexical_false_positives()
        │
        ▼
SARIF writer / markdown report / CycloneDX serializer
```

### Recommended Project Structure

No new directories required. All changes are within existing structure:

```
src/vulnerability/
├── ast_scanner.rs    # EXPAND: replace AstFinding with SastFinding, add AstCweRule/ArgCheck table
├── cwe_scanner.rs    # MODIFY: add SastSource::Ast variant; expose scan_file() for fallback
├── mod.rs            # MODIFY: export run_ast_scanner, add #[cfg] gate for ast types
src/main.rs           # MODIFY: change scanner dispatch in #[cfg(feature = "internal")] block
Cargo.toml            # MODIFY: merge ast-scanner into internal feature
tests/
├── benchmark.rs      # NEW: integration test, #[cfg(feature = "internal")]
└── vulnerability_tests/
    └── ast_scanner_tests.rs  # NEW: unit tests for AstCweRule table
BENCHMARK.md          # NEW: committed artifact from benchmark run
```

### Pattern 1: AstCweRule + ArgCheck Data-Driven Table

**What:** Mirror the `CWE_RULES` static table pattern from `cwe_scanner.rs` but for AST-based rules. Each rule declares the CWE ID, function names, and an `ArgCheck` variant that specifies what AST condition to verify on the call arguments.

**When to use:** Any CWE that maps to a specific function name and whose dangerous-call condition can be checked by inspecting argument node kinds at the call site (no dataflow required).

**Example (based on existing PoC and cwe_scanner.rs patterns):**

```rust
// Source: ast_scanner.rs PoC + cwe_scanner.rs CWE_RULES pattern
// [VERIFIED: existing codebase]

enum ArgCheck {
    /// Destination arg (index 0) is an array_declarator in the same scope.
    FixedSizeBuffer,
    /// Format arg at given 0-based index is NOT a string_literal node.
    NotStringLiteralAtIndex(u8),
    /// ANY call — no argument inspection needed (name match sufficient).
    AnyCall,
}

struct AstCweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    arg_check: ArgCheck,
}

static AST_CWE_RULES: &[AstCweRule] = &[
    AstCweRule { cwe_id: 120, functions: &["strcpy", "strcat"], arg_check: ArgCheck::FixedSizeBuffer },
    AstCweRule { cwe_id: 134, functions: &["printf", "fprintf", "sprintf"], arg_check: ArgCheck::NotStringLiteralAtIndex(0) },
    AstCweRule { cwe_id: 78,  functions: &["system", "popen"], arg_check: ArgCheck::AnyCall },
    // ... (11 tractable CWEs total per D-07)
];
```

### Pattern 2: Per-File AST Parse with Lexical Fallback

**What:** Attempt tree-sitter parse on each C/C++ file. On `None` (parse failure), fall back to lexical `scan_file()` for that file and emit a warning.

**When to use:** Required by AST-04. Handles malformed/generated/partial translation units that tree-sitter cannot parse.

**Example:**

```rust
// Source: adapted from existing ast_scanner.rs PoC pattern
// [VERIFIED: existing codebase — parser.parse() returns Option<Tree>]

fn scan_file_ast_or_lexical(
    path: &Path,
    component_name: &str,
    component_ecosystem: &str,
    parser: &mut Parser,
) -> Vec<SastFinding> {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    match parser.parse(&code, None) {
        Some(tree) => apply_ast_rules(tree.root_node(), code.as_bytes(), path, component_name, component_ecosystem),
        None => {
            eprintln!("Warning: tree-sitter failed to parse {:?} — falling back to lexical scan", path);
            lexical_scan_file(path, component_name, component_ecosystem)
        }
    }
}
```

### Pattern 3: call_expression Field-Based Argument Access

**What:** Use `node.child_by_field_name("function")` and `node.child_by_field_name("arguments")` to access call_expression fields. This is the correct API for tree-sitter 0.25 field-based access — more robust than positional child indexing.

**Why this matters:** The existing PoC uses positional child access (`node.child(0)`, `arg_list.child(1)`). For the production rule table, field-based access via `child_by_field_name` is more reliable against grammar variations and clearer in intent.

**call_expression node structure (verified):**
```
call_expression
  function: (expression)     ← child_by_field_name("function")
  arguments: (argument_list) ← child_by_field_name("arguments")
    "("
    (expression)             ← named_child 0 = arg 0
    ","
    (expression)             ← named_child 1 = arg 1
    ")"
```

**argument_list named_children iteration:**
```rust
// Source: tree-sitter 0.25.10 API (named_children takes &mut TreeCursor)
// [VERIFIED: ~/.cargo/registry tree-sitter-0.25.10/binding_rust/lib.rs]

if let Some(arg_list) = call_node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    let args: Vec<Node> = arg_list.named_children(&mut cursor).collect();
    // args[0] is the first argument, args[1] is the second, etc.
}
```

### Pattern 4: FixedSizeBuffer ArgCheck — Scope-Level Array Collection

**What:** Port the existing PoC's `collect_fixed_arrays()` approach. Collect `array_declarator` identifiers from the same function scope (or file scope), then check if the first argument of a strcpy/strcat call matches one of those identifiers.

**Refinement for production:** The PoC's `collect_fixed_arrays_rec` performs a recursive walk with a shared `TreeCursor` which requires careful cursor reset. Prefer collecting all `array_declarator` nodes in a pre-pass over the function body, or pass a fresh cursor per recursion level.

### Pattern 5: Benchmark Integration Test with Graceful Skip

**What:** `tests/benchmark.rs` reads fixture directories from environment variables or hardcoded local paths. Uses `eprintln!` and early return (not `panic!`) when fixtures are missing.

**Example:**
```rust
// [ASSUMED — pattern based on D-10/D-11 decisions]
#[cfg(feature = "internal")]
#[test]
fn benchmark_ast_vs_cppcheck_autosar() {
    let fixture = match std::env::var("AUTOSAR_FIXTURE_PATH")
        .ok()
        .or_else(|| Some("../AUTOSAR_SampleProject_S32K144".to_string()))
        .filter(|p| std::path::Path::new(p).exists())
    {
        Some(p) => p,
        None => {
            eprintln!("SKIP: AUTOSAR fixture not found — set AUTOSAR_FIXTURE_PATH");
            return;
        }
    };
    // run ast scanner, run lexical scanner, compare...
    // write BENCHMARK.md
}
```

### Anti-Patterns to Avoid

- **Emitting `AstFinding` then converting:** The PoC has `AstFinding` → conversion to emit. D-04 eliminates this. `ast_scanner.rs` emits `SastFinding` directly.
- **`#![cfg(feature = "ast-scanner")]` at file top:** The PoC file has this. Phase 18 changes it to `#![cfg(feature = "internal")]`.
- **Positional child indexing for argument_list:** `arg_list.child(1)` works for simple cases but breaks on preprocessor tokens or comment nodes. Use `named_children()` for argument iteration.
- **Re-initializing the Parser per file:** `Parser::new()` + `set_language()` is expensive. Create the parser once and reuse it across files in the same component scan.
- **Hard-failing benchmark when fixtures absent:** Must use graceful skip per D-11, not `panic!` or `assert!`.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| C grammar static linking | Custom `build.rs` compiling `parser.c` | `tree-sitter-c 0.24.2`'s built-in `build.rs` | Already does exactly this via `cc::Build`; verified in `~/.cargo/registry` |
| AST node kind identification | String-matching on source text | `node.kind()` returns `"call_expression"`, `"array_declarator"`, etc. | tree-sitter grammar defines node kinds; use the node type API |
| Argument count inspection | Counting commas in source text | `arg_list.named_child_count()` | Handles nested commas in sub-expressions correctly |
| Grammar embedding | Separate grammar file, runtime load | `cc::Build` compilation in `build.rs` | Grammar is statically embedded; no `ts_parser_set_language_from_file` needed |

**Key insight:** The `tree-sitter-c` crate's `build.rs` already handles all build-time grammar compilation. Merging the feature flag is sufficient for DIST-02; no custom build script is needed unless musl CI reveals a cross-compilation failure.

---

## Common Pitfalls

### Pitfall 1: Parser Cursor Reset After Named-Children Iteration

**What goes wrong:** `named_children()` takes `&mut TreeCursor` and resets it to the node being iterated. After calling `named_children()` on an argument_list, the cursor is positioned at the last iterated child — subsequent calls to navigate the cursor will be in an unexpected position.

**Why it happens:** tree-sitter's `TreeCursor` is stateful. `named_children()` calls `cursor.reset(self)` at the start, but the cursor state after iteration is the last visited child.

**How to avoid:** Create a fresh cursor for each traversal level: `let mut cursor = node.walk();`. The PoC does this correctly for recursive descent; replicate the pattern.

**Warning signs:** Finding count is off by one or doubled; findings at wrong line numbers.

### Pitfall 2: `SastSource::Ast` Missing in Dedup Match Arms

**What goes wrong:** `deduplicate_sast_findings()` compares `SastSource` variants. Adding `SastSource::Ast` without updating all `match source` arms causes a compile error or silently drops the new variant from `Both` promotion logic.

**Why it happens:** Adding a new enum variant triggers exhaustiveness checking in Rust — this will be a compile error, not a silent bug. But any `_ => ...` catch-all in external code silently handles it.

**How to avoid:** Grep for all `match.*SastSource` and `SastSource::` usages before adding the variant. Update `deduplicate_sast_findings()` to treat `SastSource::Ast` the same as `SastSource::Lexical` for dedup/Both promotion purposes.

**Warning signs:** `cargo build` fails with non-exhaustive match; or `SastSource::Both` never appears in benchmark output.

### Pitfall 3: `collect_fixed_arrays` Scope — Global vs. Function-Level

**What goes wrong:** The PoC collects fixed-size array declarations from the entire file, not just the function scope enclosing the call site. A `char buf[64]` declared in `foo()` would be incorrectly treated as in-scope when checking a `strcpy(buf, ...)` in `bar()`.

**Why it happens:** The PoC walks the entire AST from root — correct for a quick demo, but produces false positives when names collide across functions.

**How to avoid:** Collect array declarations only from the `function_definition` ancestor of the call site. Find the enclosing `function_definition` node first, then collect `array_declarator` nodes within it. For file-scoped (global) arrays, collect those separately and always include them.

**Warning signs:** More CWE-120 findings than expected; findings in files with multiple functions that share a local variable name.

### Pitfall 4: `parser.parse()` Returns `Some` with `has_error() == true`

**What goes wrong:** tree-sitter returns `Some(tree)` even for files with syntax errors, but `tree.root_node().has_error()` returns `true`. The AST contains error nodes. If rules don't guard against error nodes, they may produce false positives or panics on `utf8_text()`.

**Why it happens:** tree-sitter performs error recovery — partial parse is returned rather than `None`. `parse()` returns `None` only on timeout (unbounded parse) or language not set.

**How to avoid:** Check `tree.root_node().has_error()` after a successful parse. On `true`, either fall back to lexical scan (conservative, aligns with AST-04 intent) or proceed with AST scan only on non-error subtrees. The fallback path is simpler and safer.

**Warning signs:** False positives in generated code or header files with non-standard syntax.

### Pitfall 5: `named_children` vs. All Children for Argument Enumeration

**What goes wrong:** `argument_list.child_count()` includes non-named nodes: `(`, `)`, `,` separators. Using `child(1)` to get the first argument works in the PoC because the first argument after `(` is at index 1 — but this is fragile and depends on the grammar placing `(` as child 0.

**Why it happens:** tree-sitter grammars distinguish named nodes (actual AST nodes) from anonymous nodes (punctuation). `named_child_count()` and `named_children()` filter to named nodes only.

**How to avoid:** Use `arg_list.named_children(&mut cursor)` to iterate arguments directly. `args[0]` = first argument, `args[1]` = second, etc. No off-by-one math needed.

**Warning signs:** ArgCheck logic misidentifies which argument to inspect; `NotStringLiteralAtIndex(0)` checks the wrong arg.

### Pitfall 6: Feature Flag in `#![cfg(...)]` vs. `#[cfg(...)]` — Module Visibility

**What goes wrong:** The PoC has `#![cfg(feature = "ast-scanner")]` at the top of `ast_scanner.rs`. After merging into `internal`, if the inner-attribute is updated to `#![cfg(feature = "internal")]` but `mod.rs` still has `pub mod ast_scanner` without a matching cfg gate, the module is only compiled under `internal` but always referenced.

**Why it happens:** Inner `#![cfg]` applies to the entire module file. Outer `#[cfg]` on the `mod` declaration gates the module import. Both must be consistent.

**How to avoid:** Change the file-top inner attribute AND the `mod.rs` declaration to use `#[cfg(feature = "internal")]`. Verify `vulnerability/mod.rs` `pub use` lines are also gated.

---

## Code Examples

Verified patterns from official sources and existing codebase:

### tree-sitter-c API: Call Expression Function Name

```rust
// Source: adapted from ast_scanner.rs PoC (existing codebase)
// [VERIFIED: existing codebase + tree-sitter-c node-types.json]

// call_expression has two fields: "function" and "arguments"
if node.kind() == "call_expression" {
    if let Some(func_node) = node.child_by_field_name("function") {
        if let Ok(func_name) = func_node.utf8_text(src) {
            // func_name is the function identifier text
        }
    }
}
```

### tree-sitter-c API: Named Children of argument_list

```rust
// Source: tree-sitter 0.25.10 Rust API
// [VERIFIED: ~/.cargo/registry tree-sitter-0.25.10/binding_rust/lib.rs]

if let Some(arg_list) = call_node.child_by_field_name("arguments") {
    let mut cursor = arg_list.walk();
    let args: Vec<_> = arg_list.named_children(&mut cursor).collect();
    // args.len() == number of actual arguments (no punctuation nodes)
    if let Some(first_arg) = args.first() {
        let kind = first_arg.kind();  // e.g., "identifier", "string_literal", "number_literal"
        let text = first_arg.utf8_text(src).unwrap_or("");
    }
}
```

### SastSource::Ast Variant Addition

```rust
// Source: existing cwe_scanner.rs pattern
// [VERIFIED: existing codebase src/vulnerability/cwe_scanner.rs]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SastSource {
    Lexical,
    Cppcheck,
    Both,
    Ast,  // NEW in Phase 18
}
```

### Benchmark Integration Test Skeleton

```rust
// Source: D-10/D-11 decisions, mirroring existing test patterns
// [ASSUMED — structure, not API]

#[cfg(feature = "internal")]
mod benchmark {
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn fixture_path(env_var: &str, default: &str) -> Option<PathBuf> {
        let path = std::env::var(env_var)
            .unwrap_or_else(|_| default.to_string());
        let p = PathBuf::from(&path);
        if p.exists() { Some(p) } else { None }
    }

    #[test]
    fn benchmark_autosar_fixture() {
        let fixture = match fixture_path("AUTOSAR_FIXTURE_PATH", "../AUTOSAR_SampleProject_S32K144") {
            Some(p) => p,
            None => {
                eprintln!("SKIP benchmark_autosar_fixture: fixture not present");
                return;
            }
        };
        // run ast, lexical, cppcheck scanners...
        // collect TPs/FPs per CWE...
        // write BENCHMARK.md
    }
}
```

---

## Runtime State Inventory

This phase is a greenfield expansion of existing scanner code — no rename/refactor/migration concerns apply.

**Nothing found in any category** — verified by review of the phase scope (new code + feature flag merge only).

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All compilation | ✓ | stable (cargo build succeeded) | — |
| tree-sitter crate | AST scanning | ✓ | 0.25.10 (in Cargo.lock) | — |
| tree-sitter-c crate | C grammar | ✓ | 0.24.2 (in Cargo.lock) | — |
| musl-gcc (Linux CI) | DIST-02 musl static build | ✓ (CI: apt musl-tools) | provided by `musl-tools` in build-release.yml | — |
| cppcheck | BENCH-01 comparison | unknown (local) | — | Benchmark test skips cppcheck column if not found |
| AUTOSAR_SampleProject_S32K144 | BENCH-01 primary fixture | unknown (local, not in repo) | — | Benchmark test skips with eprintln! if absent |
| Juliet Test Suite C/C++ subset | BENCH-01 secondary fixture | unknown (local, not in repo) | — | Benchmark test skips with eprintln! if absent (D-11) |

**Note on musl build:** macOS developer machines cannot run `cargo build --target x86_64-unknown-linux-musl` without `musl-gcc`. The CI Linux runner installs `musl-tools` via `apt-get`. DIST-02 verification happens in CI, not locally on macOS.

**Note on tree-sitter-c musl cross-compilation:** The crate's `build.rs` uses `cc::Build` which invokes the C compiler configured for the target (musl-gcc for `x86_64-unknown-linux-musl`). Only `src/parser.c` is compiled — no external C++ dependencies, no system headers beyond libc. This is expected to compile cleanly per D-15. A custom `build.rs` is only needed if CI reveals a failure.

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in `cargo test` |
| Config file | none (uses `Cargo.toml` `[dev-dependencies]`) |
| Quick run command | `cargo test --features internal 2>&1` |
| Full suite command | `cargo test --features internal --tests 2>&1` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| AST-01 | `run_ast_scanner()` returns findings without cppcheck installed | integration | `cargo test --features internal ast_scanner` | ❌ Wave 0 |
| AST-02 | All 11 tractable CWEs detected in synthetic C fixture | unit | `cargo test --features internal test_ast_all_cwes` | ❌ Wave 0 |
| AST-03 | `run_ast_scanner()` returns `Vec<SastFinding>` (not `Vec<AstFinding>`) | unit | `cargo test --features internal test_ast_emits_sast_finding` | ❌ Wave 0 |
| AST-04 | Parse failure → lexical fallback + warning, exit 0 | unit | `cargo test --features internal test_parse_failure_fallback` | ❌ Wave 0 |
| BENCH-01 | Benchmark test runs gracefully with or without fixtures | integration | `cargo test --features internal benchmark` | ❌ Wave 0 |
| DIST-01 | License documented in Cargo.toml or LICENSE-NOTES.md | manual/audit | n/a — verify once, document | n/a |
| DIST-02 | Musl binary verified to have no runtime grammar file dependency | manual/CI | CI build-release.yml (Linux musl build) | n/a |

### Sampling Rate

- **Per task commit:** `cargo test --features internal 2>&1`
- **Per wave merge:** `cargo test --features internal --tests 2>&1`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/vulnerability_tests/ast_scanner_tests.rs` — unit tests for AstCweRule table (AST-02, AST-03, AST-04)
- [ ] `tests/benchmark.rs` — integration test skeleton with graceful skip (BENCH-01)
- [ ] `tests/vulnerability_tests/mod.rs` — add `pub mod ast_scanner_tests;`

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `AstFinding` intermediate type in ast_scanner.rs | Emit `SastFinding` directly | Phase 18 (D-04) | Drops conversion layer; downstream writers unchanged |
| `feature = "ast-scanner"` separate flag | Merged into `feature = "internal"` | Phase 18 (D-01) | One build flag for all scanner code |
| Lexical scanner as primary C/C++ scanner | AST scanner as primary, lexical as per-file fallback | Phase 18 (D-02) | Higher precision; fewer FPs on safe wrappers |
| `#![cfg(feature = "ast-scanner")]` in ast_scanner.rs | `#![cfg(feature = "internal")]` | Phase 18 | Must update both file-top attr and mod.rs declaration |

**Deprecated/outdated:**
- `AstFinding` struct: removed in Phase 18; replaced by `SastFinding` with `source: SastSource::Ast`
- `scan_cwe120()` standalone function: replaced by the data-driven `AstCweRule`/`ArgCheck` table

---

## Open Questions

1. **CWE count discrepancy in AST-02**
   - What we know: REQUIREMENTS.md says "all 14 CWEs" but lists 15: CWE-78, 119, 120, 122, 125, 134, 190, 295, 319, 362, 367, 369, 416, 476, 732.
   - What's unclear: Whether "14" is a typo or whether one of the listed CWEs should be excluded.
   - Recommendation: Treat the explicit list as authoritative (15 CWEs). D-07 already clarifies which are tractable (11) vs. deferred (3: 362/416/476). The remaining CWE-295/319/732 (argument-value rules) are covered for Phase 18 as `AnyCall` or simple token-check ArgCheck variants; Phase 20 upgrades them to full AST argument inspection. This interpretation is consistent with CONTEXT.md.

2. **Where to write BENCHMARK.md (D-13)**
   - What we know: D-13 says "repo root or `docs/BENCHMARK.md`".
   - What's unclear: Which location is preferred.
   - Recommendation: `docs/BENCHMARK.md` — keeps the repo root clean and consistent with any future `docs/` additions. The planner should pick one location; the benchmark test must hard-code or configure the write path.

3. **Whether `has_error()` on the tree triggers the lexical fallback (AST-04)**
   - What we know: `parser.parse()` returns `Some(tree)` even for syntax-error files. `None` is only returned on parser timeout.
   - What's unclear: The requirement says "tree-sitter fails to parse" — does this mean `None` only, or also `has_error() == true`?
   - Recommendation: Treat `has_error() == true` as a fallback trigger as well as `None`. This gives a wider safety net for generated/partial files and is more conservative. Document in a comment.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Benchmark test writes BENCHMARK.md by running `cargo test --features internal` locally (not in CI) | Architecture, D-10 | Low: the decision is locked (D-10) — test is not CI-gated |
| A2 | The benchmark test determines TP/FP by comparing scanner findings against a manually curated ground truth for the AUTOSAR and Juliet fixtures | Benchmark test skeleton | Medium: if ground truth curation approach differs, the test structure changes. Planner should clarify how TPs and FPs are counted (ground truth file vs. manual inspection) |
| A3 | `SastSource::Ast` is treated like `SastSource::Lexical` in `deduplicate_sast_findings()` for the purpose of `SastSource::Both` promotion | Code Examples | Low risk: the dedup logic clearly uses source-equality checks; adding Ast as equivalent to Lexical is the natural extension |

**A2 is the highest-risk assumption.** If TP/FP counting requires a separate ground-truth oracle or annotation file, the benchmark test is more complex than a simple "run scanner + count findings". The planner should address this in the benchmark plan.

---

## Sources

### Primary (HIGH confidence)
- Existing codebase: `src/vulnerability/ast_scanner.rs`, `src/vulnerability/cwe_scanner.rs`, `src/scanner/mod.rs`, `src/main.rs`, `src/vulnerability/mod.rs` — [VERIFIED: direct file read]
- `Cargo.toml` and `Cargo.lock` — feature flag structure, library versions [VERIFIED: direct file read]
- `tree-sitter-c 0.24.2` source at `~/.cargo/registry/src/*/tree-sitter-c-0.24.2/` — `build.rs`, `bindings/rust/lib.rs`, `src/node-types.json` [VERIFIED: direct file read]
- `tree-sitter 0.25.10` Rust API at `~/.cargo/registry/src/*/tree-sitter-0.25.10/binding_rust/lib.rs` — `child_by_field_name`, `named_children` signatures [VERIFIED: direct file read]
- `.planning/phases/18-ast-scanner-core-and-benchmark/18-CONTEXT.md` — all decisions D-01 through D-17 [VERIFIED]
- `.planning/REQUIREMENTS.md` — AST-01 through DIST-02 [VERIFIED]
- `.github/workflows/build-release.yml` — musl cross-compile setup [VERIFIED]

### Secondary (MEDIUM confidence)
- `cargo metadata --features ast-scanner` output — tree-sitter, tree-sitter-c, tree-sitter-language versions and license fields [VERIFIED: tool output]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — versions verified from Cargo.lock and registry
- Architecture: HIGH — existing codebase read; integration points mapped
- Pitfalls: HIGH — derived from direct API inspection (node-types.json, existing PoC)
- Benchmark structure: MEDIUM — D-10..D-14 decisions are locked, but TP/FP counting approach assumed

**Research date:** 2026-05-11
**Valid until:** 2026-06-11 (stable ecosystem — tree-sitter API, Rust crate versions, codebase structure)
