---
phase: quick/260513-fsg
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/vulnerability/cwe_scanner.rs
  - tests/vulnerability_tests/cwe_scanner_tests.rs
  - tests/fixtures/c/dangerous_calls.c
  - src/vulnerability/ast_scanner.rs
  - benchmark/juliet/ANALYSIS-phase24-preview.md
autonomous: true
requirements: [SCAN-02]

must_haves:
  truths:
    - "CWE-20 and CWE-22 produce no findings (rules deleted)"
    - "cargo test passes with no failures"
    - "ANALYSIS doc records the ad-hoc drops with rationale"
  artifacts:
    - path: "src/vulnerability/cwe_scanner.rs"
      provides: "Lexical CWE rule table without CWE-20 and CWE-22"
      contains: "12 distinct CWE IDs"
    - path: "benchmark/juliet/ANALYSIS-phase24-preview.md"
      provides: "Ad-hoc Drops section"
      contains: "Ad-hoc Drops"
  key_links:
    - from: "src/vulnerability/cwe_scanner.rs"
      to: "tests/vulnerability_tests/cwe_scanner_tests.rs"
      via: "test_all_thirteen_cwes expected array and test_rule_table_has_fourteen_cwes count"
      pattern: "ids.len\\(\\), 12"
---

<objective>
Drop CWE-20 (Improper Input Validation — atoi/strtol family) and CWE-22 (Path
Traversal — realpath/open/fopen family) from the lexical scanner rule table.
Both have 0 TPs and 100% FP on the Juliet corpus and require taint/dataflow
analysis to fix correctly; no tractable local fix exists.

Purpose: Reduce false-positive noise from the lexical scanner; keep the rule
table honest about what pure function-name matching can detect reliably.

Output: Updated cwe_scanner.rs (12 CWEs), updated tests, updated fixture,
updated ast_scanner.rs doc comment, updated ANALYSIS doc with Ad-hoc Drops
section, git commit, passing cargo test.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md

Key facts discovered during planning:
- CWE-20 and CWE-22 live in `src/vulnerability/cwe_scanner.rs` (the lexical scanner),
  NOT in `src/vulnerability/ast_scanner.rs`. They were never in AST_CWE_RULES.
- Current rule table: 15 entries, 14 distinct CWE IDs (two CWE-134 entries).
- After removal: 13 entries, 12 distinct CWE IDs.
- Three tests must be updated to reflect the new count:
    1. `cwe_scanner.rs` line ~511: `test_rule_table_has_fourteen_cwes` — asserts `ids.len() == 14`
    2. `cwe_scanner_tests.rs` line ~35: `test_all_thirteen_cwes` — `expected` array includes 20u32 and 22
    3. The fixture `dangerous_calls.c` has `realpath(s, d);` (CWE-22) at line 12
       and `atoi(s);` (CWE-20) at line 15 — these lines must be removed.
- `ast_scanner.rs` module doc says "Phase 23 Plan 01 expansion → 49 CWEs total" but
  Phase 24 dropped CWE-256 making it 48. The doc needs a note that CWE-20/22 are also
  dropped from the lexical scanner (they were never in the AST scanner).

<interfaces>
<!-- From src/vulnerability/cwe_scanner.rs (relevant entries) -->
struct CweRule {
    cwe_id: u32,
    functions: &'static [&'static str],
    requires_format_heuristic: bool,
    format_arg_index: u8,
}

// Line 78 — DELETE this entry:
CweRule { cwe_id: 22, functions: &["realpath", "getcwd", "chdir", "open", "fopen"], requires_format_heuristic: false, format_arg_index: 0 },
// Line 82 — DELETE this entry:
CweRule { cwe_id: 20, functions: &["atoi", "atol", "atof", "atoll", "strtol", "strtoul"], requires_format_heuristic: false, format_arg_index: 0 },
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Remove CWE-20 and CWE-22 rules, update counts and fixture</name>
  <files>
    src/vulnerability/cwe_scanner.rs,
    tests/vulnerability_tests/cwe_scanner_tests.rs,
    tests/fixtures/c/dangerous_calls.c
  </files>
  <action>
Four surgical edits — no other lines touched:

**1. src/vulnerability/cwe_scanner.rs**

a) Delete the CWE-22 entry (line 78):
   Remove: `CweRule { cwe_id: 22, functions: &["realpath", "getcwd", "chdir", "open", "fopen"], requires_format_heuristic: false, format_arg_index: 0 },`

b) Delete the CWE-20 entry (line 82):
   Remove: `CweRule { cwe_id: 20, functions: &["atoi", "atol", "atof", "atoll", "strtol", "strtoul"], requires_format_heuristic: false, format_arg_index: 0 },`

c) Update the header comment at line ~63 from:
   `/// All CWEs: 14 from SEED-001 (SCAN-02, updated to include CWE-126). CWE-295/319/732 are now`
   to:
   `/// All CWEs: 12 from SEED-001 (SCAN-02, updated to include CWE-126). CWE-295/319/732 are now`
   (keep the rest of the sentence unchanged)

d) Update `test_rule_table_has_fourteen_cwes`:
   - Rename the function to `test_rule_table_has_twelve_cwes`
   - Change the assert message and the count:
     `assert_eq!(ids.len(), 12, "CWE-20/22 dropped (100% FP, no tractable local fix); CWE-295/319/732 moved to AST scanner; 12 distinct CWE IDs remain in lexical table (CWE-369 detected via separate code path)");`
   - Update the doc comment above the function to match (remove "14" references, add drop rationale).

**2. tests/vulnerability_tests/cwe_scanner_tests.rs**

Update `test_all_thirteen_cwes`:
- Rename to `test_all_twelve_cwes` (matches the new count)
- Remove `20u32` and `22` from the `expected` array:
  Change: `let expected = [20u32, 22, 78, 120, 126, 134, 190, 242, 327, 362, 367, 377, 676, 807];`
  To:     `let expected = [78u32, 120, 126, 134, 190, 242, 327, 362, 367, 377, 676, 807];`
- Update the comment above the assert from "SCAN-02: all 14 distinct CWE IDs" to
  "SCAN-02: all 12 distinct CWE IDs"

**3. tests/fixtures/c/dangerous_calls.c**

Remove the two call lines (and their comments):
- Line 12: `    realpath(s, d);                   /* CWE-22 */`
- Line 15: `    atoi(s);                          /* CWE-20 */`
  </action>
  <verify>
    <automated>cd /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom && cargo test --features internal 2>&1 | tail -30</automated>
  </verify>
  <done>
    cargo test passes with no failures. `grep "cwe_id: 20\b\|cwe_id: 22\b" src/vulnerability/cwe_scanner.rs` returns empty.
    `grep "ids.len(), 12" src/vulnerability/cwe_scanner.rs` matches. `grep "20u32\|, 22," tests/vulnerability_tests/cwe_scanner_tests.rs` returns empty.
  </done>
</task>

<task type="auto">
  <name>Task 2: Update ast_scanner.rs doc and add Ad-hoc Drops to ANALYSIS</name>
  <files>
    src/vulnerability/ast_scanner.rs,
    benchmark/juliet/ANALYSIS-phase24-preview.md
  </files>
  <action>
**1. src/vulnerability/ast_scanner.rs** — module-level doc comment (lines 1-22)

Replace the opening line:
  `//! Phase 18 (v1.0.18): Production AST-based CWE scanner using tree-sitter-c.`
with:
  `//! Phase 18 (v1.0.18): Production AST-based CWE scanner using tree-sitter-c.`
  (unchanged — the AST scanner never had CWE-20/22)

Change line 3:
  `//! AST-detected (Phase 23 Plan 01 expansion → 49 CWEs total):`
to:
  `//! AST-detected (Phase 24 post-drop → 48 CWEs total; CWE-256 removed Ph24):`

Add after line 16 (`//! Deferred to lexical fallback only: 362, 367, 416, 476`):
```
//! Lexical-only drops (ad-hoc, 2026-05-13): CWE-20, CWE-22 — both 100% FP on
//! Juliet corpus, 0 TPs; require taint/dataflow analysis to reduce FP rate; no
//! tractable local fix. Removed from CWE_RULES in cwe_scanner.rs.
```

**2. benchmark/juliet/ANALYSIS-phase24-preview.md**

Append a new section at the end of the file (after the last `---` separator or after the Phase 24 Notes block, whichever is last):

```markdown
---

## Ad-hoc Drops (2026-05-13)

Post-Phase-24 rule removals applied outside the normal phase cadence.
Both CWEs are also 100% FP in the AST scanner (when AST scanner covers the
same functions) and 100% FP in the lexical scanner; removed from
`CWE_RULES` in `cwe_scanner.rs`.

| CWE | Scanner | TPs | FPs | FP% | Rationale | Fix path |
|-----|---------|-----|-----|-----|-----------|----------|
| CWE-20 | Lexical | 0 | 2,766 | 100% | `atoi`/`strtol` family fires on ALL numeric conversion calls regardless of input source; input-validation detection requires taint/dataflow | Taint analysis (deferred) |
| CWE-22 | Lexical | 0 | 12,390 | 100% | `realpath`/`open`/`fopen` fires in CWE-23/36 files and across all path-handling code; path-traversal detection requires dataflow tracing from user input to file call | Dataflow analysis (deferred) |

**Effect on scanner totals:**

| | Before drop | After drop |
|---|---|---|
| Lexical CWE coverage | 14 | 12 |
| Lexical FP count (Juliet) | 180,954 | ~165,798 |
| AST CWE coverage | 48 | 48 (unchanged — CWE-20/22 were never in AST_CWE_RULES) |
```
  </action>
  <verify>
    <automated>grep -n "Ad-hoc Drops" /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/benchmark/juliet/ANALYSIS-phase24-preview.md && grep -n "48 CWEs total" /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom/src/vulnerability/ast_scanner.rs</automated>
  </verify>
  <done>
    ANALYSIS-phase24-preview.md contains "Ad-hoc Drops" section. ast_scanner.rs doc says "48 CWEs total". cargo test still passes.
  </done>
</task>

<task type="auto">
  <name>Task 3: Commit</name>
  <files></files>
  <action>
Stage and commit the five changed files:

```
git add src/vulnerability/cwe_scanner.rs \
        tests/vulnerability_tests/cwe_scanner_tests.rs \
        tests/fixtures/c/dangerous_calls.c \
        src/vulnerability/ast_scanner.rs \
        benchmark/juliet/ANALYSIS-phase24-preview.md
git commit -m "feat(lexical): drop CWE-20 and CWE-22 (100% FP, no tractable local fix)

Both rules produce 0 TPs and 100% FP on the Juliet corpus.
Fixing either requires taint/dataflow analysis beyond lexical token matching.

- Remove CweRule entries for CWE-20 (atoi family) and CWE-22 (path family)
- Update internal count assertion: 14 → 12 distinct CWE IDs
- Update integration test fixture and expected-CWE array
- Document removal in ANALYSIS-phase24-preview.md under Ad-hoc Drops
- Update ast_scanner.rs doc (CWE-20/22 were never in AST_CWE_RULES;
  clarify 48-CWE count post-Phase-24 CWE-256 removal)"
```
  </action>
  <verify>
    <automated>git log --oneline -1</automated>
  </verify>
  <done>Most recent commit message contains "drop CWE-20 and CWE-22".</done>
</task>

</tasks>

<verification>
After all tasks:

```bash
cd /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom
cargo test --features internal 2>&1 | grep -E "^test result|FAILED|error"
grep -c "cwe_id: 2[02]\b" src/vulnerability/cwe_scanner.rs   # must be 0
grep "ids.len(), 12" src/vulnerability/cwe_scanner.rs         # must match
grep "Ad-hoc Drops" benchmark/juliet/ANALYSIS-phase24-preview.md
```
</verification>

<success_criteria>
- `cargo test --features internal` passes with 0 failures
- No `cwe_id: 20` or `cwe_id: 22` entries remain in `cwe_scanner.rs`
- `test_rule_table_has_twelve_cwes` asserts count of 12
- `test_all_twelve_cwes` expected array has 12 elements (no 20 or 22)
- `dangerous_calls.c` has no `atoi` or `realpath` lines
- `ast_scanner.rs` doc says "48 CWEs total"
- `ANALYSIS-phase24-preview.md` has an "Ad-hoc Drops" section with CWE-20 and CWE-22
- Git commit exists with all 5 files
</success_criteria>

<output>
After completion, create `.planning/quick/260513-fsg-drop-cwe-20-and-cwe-22-from-ast-scanner-/260513-fsg-SUMMARY.md`
</output>
