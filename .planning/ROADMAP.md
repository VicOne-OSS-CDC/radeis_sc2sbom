# Roadmap: radeis_sc2sbom

## Milestones

- ✅ **v1.0.14 Reliability & Compatibility** — Phases 1–5 (shipped 2026-04-24)
- ✅ **v1.0.15 AUTOSAR Support & CVE Enrichment** — Phases 6–9 (shipped 2026-05-09)
- ✅ **v1.0.16 C/C++ Lexical CWE Scanner** — Phases 10–12 (shipped 2026-05-10)
- ✅ **v1.0.17 Advanced C/C++ SAST Scanner** — Phases 13–17 (shipped 2026-05-11)
- 🔄 **v1.0.18 Tree-sitter AST Scanner** — Phases 18–25 (in progress)

## Phases

<details>
<summary>✅ v1.0.14 Reliability & Compatibility (Phases 1–5) — SHIPPED 2026-04-24</summary>

- [x] Phase 1: broken-symlink-fix (1/1 plans) — WalkDir errors → warnings in scanner/mod.rs
- [x] Phase 2: makefile-var-version (1/1 plans) — $(VAR) refs emit NOASSERTION
- [x] Phase 3: c-license-detection (2/2 plans) — known_licenses.rs + pkgconfig License: field
- [x] Phase 4: musl-build (1/1 plans) — static Linux binary, no glibc dependency
- [x] Phase 5: symlink-gap-fix (1/1 plans) — three additional WalkDir abort paths closed

Full archive: [milestones/v1.0.14-ROADMAP.md](milestones/v1.0.14-ROADMAP.md)

</details>

<details>
<summary>✅ v1.0.15 AUTOSAR Support & CVE Enrichment (Phases 6–9) — SHIPPED 2026-05-09</summary>

- [x] Phase 6: autosar-detection (1/1 plans) — .arxml + dir + build-var heuristics; ScanContext.is_autosar
- [x] Phase 7: autosar-classification-output (4/4 plans) — BSW module classifier; autosar:layer + autosar:platform in SPDX/CycloneDX
- [x] Phase 8: supplier-config (3/3 plans) — --supplier-config YAML; autosar:supplier or NOASSERTION
- [x] Phase 9: cve-cwe-enrichment (2/2 plans) — NVD CWE enrichment with TTL cache; CWE IDs in all output formats

Full archive: [milestones/v1.0.15-ROADMAP.md](milestones/v1.0.15-ROADMAP.md)

</details>

<details>
<summary>✅ v1.0.16 C/C++ Lexical CWE Scanner (Phases 10–12) — SHIPPED 2026-05-10</summary>

- [x] Phase 10: internal-feature-gate (4/4 plans) — CVE/CWE/scanner gated behind `cargo feature = "internal"`; public binary compiles without vulnerability code
- [x] Phase 11: lexical-scanner-cyclonedx-output (4/4 plans) — Pure-Rust dangerous-function scanner detects 14 CWEs; findings in CycloneDX 1.5 vulnerabilities[] entries
- [x] Phase 12: static-analysis-report (3/3 plans) — `_static_analysis.md` report + SAST section in main report + CLI disclaimer

Full archive: [milestones/v1.0.16-ROADMAP.md](milestones/v1.0.16-ROADMAP.md)

</details>

<details>
<summary>✅ v1.0.17 Advanced C/C++ SAST Scanner (Phases 13–17) — SHIPPED 2026-05-11</summary>

- [x] Phase 13: argument-value-matching (1/1 plans) — CWE-295/319/732/369 via AND-all paren-arg token matching
- [x] Phase 14: cppcheck-integration (5/5 plans) — SastSource enum, XML parser, subprocess driver, dedup pipeline
- [x] Phase 15: sarif-output (2/2 plans) — SARIF 2.1 writer + --sarif-output CLI flag
- [x] Phase 16: sarif-as-authoritative-store (3/3 plans) — SHA-256 fingerprints, cppcheck suppression, --sarif-baseline CI gate
- [x] Phase 17: v1.0.17-bug-fixes (1/1 plans) — arxml parser, epd/Doxygen version extraction, ecosystem dedup

Full archive: [milestones/v1.0.17-ROADMAP.md](milestones/v1.0.17-ROADMAP.md)

</details>

### v1.0.18 Tree-sitter AST Scanner (Phases 18–25)

- [x] **Phase 18: ast-scanner-core-and-benchmark** — AST scanner wired as default, full 14-CWE rule set, benchmarked against cppcheck with documented false-positive rates (completed 2026-05-11)
- [x] **Phase 19: cppcheck-removal** — cppcheck removed or demoted based on Phase 18 benchmark data; graceful-degradation messaging updated (completed 2026-05-12)
- [x] **Phase 20: argument-value-ast-migration** — CWE-295/319/732 argument-value rules migrated from paren-bound string scanning to AST argument node inspection; verified against v1.0.17 baseline (completed 2026-05-12)
- [x] **Phase 21: ast-cwes-anyCall-argPattern-expansion** — 12 new CWEs added via AnyCall/ArgPattern/FixedSizeBuffer/OperatorPattern rules: CWE-121, 126, 328, 338, 369, 426, 467, 526, 535, 676, 680, 780 (CWE-605 deferred per D-12); total coverage 13→25 CWEs (completed 2026-05-12)
- [x] **Phase 22: ast-cwes-structuralPattern-expansion** — 15 new CWEs added via pure AST structural-shape rules: CWE-256, 398, 478, 480, 481, 482, 483, 484, 562, 570, 571, 587, 617, 674, 835; total coverage 26→41 CWEs (completed 2026-05-12)
- [x] **Phase 23: ast-cwes-domainSpecific-expansion** — 8 new CWEs added via narrow domain/API rules: CWE-114, 272, 284, 427, 479, 591, 762, 785; total coverage 41→49 CWEs (completed 2026-05-12)
- [x] **Phase 24: tune-high-fp-cwe-rules-from-phases-19-23** — Audit and tighten 17 CWE rules exceeding the 35% FP gate; add context guards, restrict function lists, or promote to ArgAtIndex (completed 2026-05-13)
- [x] **Phase 25: experiment-scan-mode** — Gate the 17 high-FP CWEs (Marginal + Poor + No-signal-unconfirmed) behind `--experiment-scan`; default scan retains only the 22 Clean/Good/AUTOSAR-confirmed CWEs (completed 2026-05-13)

## Phase Details

### Phase 18: ast-scanner-core-and-benchmark
**Goal**: Users can run sc2sbom with tree-sitter-c as the default C/C++ scanner — no cppcheck install required — and benchmark output documents how AST-detected findings compare to cppcheck on the reference fixture
**Depends on**: Phase 17 (PoC ast_scanner.rs committed in v1.0.17)
**Requirements**: AST-01, AST-02, AST-03, AST-04, BENCH-01, DIST-01, DIST-02
**Success Criteria** (what must be TRUE):
  1. Running sc2sbom on a C/C++ project with no cppcheck installed produces SAST findings in SARIF, CycloneDX, and markdown report — identical downstream formats to v1.0.17
  2. The AST scanner detects all 14 CWEs (CWE-78, 119, 120, 122, 125, 134, 190, 295, 319, 362, 367, 369, 416, 476, 732) on the AUTOSAR_SampleProject_S32K144 fixture
  3. A C file that fails to parse triggers a logged warning and falls back to lexical scan — the overall run still completes exit 0
  4. A documented comparison (markdown or SARIF diff) shows AST vs cppcheck false-positive rates on AUTOSAR_SampleProject_S32K144 and at least one additional fixture
  5. The tree-sitter-c grammar license is confirmed MIT-compatible and the static musl binary embeds the grammar with no runtime file dependency
**Plans**: 3 plans
- [x] 18-01-PLAN.md — Foundation: feature-flag merge, SastSource::Ast variant, test scaffolds (AST-03, DIST-01)
- [x] 18-02-PLAN.md — AST scanner core: 11-CWE rule table, parse-failure fallback, main.rs dispatch (AST-01..04, DIST-02)
- [x] 18-03-PLAN.md — Benchmark + docs/BENCHMARK.md + fixture guide (BENCH-01, DIST-01)
**UI hint**: no

### Phase 19: cppcheck-removal
**Goal**: cppcheck is removed from the default execution path (or demoted to an opt-in flag) based on the Phase 18 benchmark data, and the tool's messaging accurately reflects what scanner is running
**Depends on**: Phase 18
**Requirements**: CPP-01
**Success Criteria** (what must be TRUE):
  1. Running sc2sbom without any cppcheck-related flags produces SAST findings using only the AST scanner — no cppcheck subprocess is spawned
  2. If cppcheck is demoted rather than removed, running `--features cppcheck` (or equivalent escape hatch) still invokes the cppcheck subprocess as before
  3. CLI output and any graceful-degradation messaging correctly names the active scanner — no stale references to cppcheck in the default path
**Plans**: 1 plan
- [x] 19-01-PLAN.md — Hard-remove cppcheck: delete runner/parser/suppress fn/CLI arg/Cppcheck variant, revise deduplicate_sast_findings(ast, lexical), delete benchmark and cppcheck/suppression test files (CPP-01)
**UI hint**: no

### Phase 20: argument-value-ast-migration
**Goal**: The CWE-295, CWE-319, and CWE-732 argument-value detection rules use AST argument node inspection rather than paren-bound string scanning, with no new false positives introduced against the v1.0.17 baseline
**Depends on**: Phase 19
**Requirements**: ARGVAL-01, ARGVAL-02
**Success Criteria** (what must be TRUE):
  1. CWE-295 (SSL_VERIFY_NONE), CWE-319 (CURLOPT_USE_SSL), and CWE-732 (umask/DACL) findings on the AUTOSAR_SampleProject_S32K144 fixture match or improve on the v1.0.17 finding count
  2. Running the migrated rules on AUTOSAR_SampleProject_S32K144 introduces zero new false positives compared to the v1.0.17 SARIF baseline
  3. A call-site with the dangerous argument buried inside a nested expression (not a direct paren-bound match) is correctly detected by the AST argument node rule
**Plans**: 2 plans
- [x] 20-01-PLAN.md — AST scanner: add ArgAtIndex variant, migrate CWE-295/319/732 rules, add wolfSSL gap fix, add 10 AST tests (ARGVAL-01, ARGVAL-02)
- [x] 20-02-PLAN.md — Lexical scanner cleanup: remove arg_value_contains field/rules, delete paren_args_contain_all, remove lexical CWE-295/319/732 tests, update rule-count assertion (ARGVAL-01, ARGVAL-02)
**UI hint**: no

## Progress Table

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 18. ast-scanner-core-and-benchmark | 3/3 | Complete | 2026-05-11 |
| 19. cppcheck-removal | 1/1 | Complete   | 2026-05-12 |
| 20. argument-value-ast-migration | 2/2 | Complete    | 2026-05-12 |
| 21. ast-cwes-anyCall-argPattern-expansion | 3/3 | Complete    | 2026-05-12 |
| 22. ast-cwes-structuralPattern-expansion | 4/4 | Complete    | 2026-05-12 |
| 23. ast-cwes-domainSpecific-expansion | 3/3 | Complete    | 2026-05-12 |
| 24. tune-high-fp-cwe-rules-from-phases-19-23 | 4/4 | Complete   | 2026-05-13 |
| 25. experiment-scan-mode | 2/2 | Complete    | 2026-05-13 |

## Backlog

### Phase 999.1: Auto-generate supplier config from AUTOSAR project structure (BACKLOG)

**Goal:** Add a `--generate-supplier-config <OUTPUT>` flag that performs a Pass 1 scan — walking `plugins/` directory names, `Modules.mak`, and `.arxml` files to infer component→vendor mappings — and writes a `supplier.yaml` the user can review before running Pass 2 with `--supplier-config`. Optionally collapsed into `--auto-supplier` for CI pipelines.
**Requirements:** TBD
**Plans:** 2/2 plans complete

**Context from discussion (2026-05-09):**
- Pass 1: infer vendors from `plugins/<Name>_TS_T40D2M10I1R0/` suffix table (T40→NXP, T17→EB Tresos, etc.) + `Modules.mak` MCAL vs app-layer bucketing
- Pass 2: existing `--supplier-config` flow unchanged
- Two-pass is preferred over one-pass: `supplier.yaml` becomes a committed artifact, making SBOMs reproducible and auditable
- `--auto-supplier` shortcut for CI where inference is trusted
- Risk: suffix→vendor table needs maintenance; only reliable for standard MCAL plugin distributions

### Phase 21: ast-cwes-anyCall-argPattern-expansion

**Goal**: Expand AST scanner from 13 to 26 CWEs by adding rules that require only call-site detection — AnyCall (fire on any invocation of a dangerous function), ArgPattern (fire when a specific argument matches a token/structure pattern), FixedSizeBuffer (existing mechanism, new functions), and OperatorPattern (fire on a specific operator shape). All rules validated against Juliet ground truth with FP% tracked in benchmark/juliet/ANALYSIS.md.
**Depends on**: Phase 20
**Requirements**: CWEXP-01
**Success Criteria** (what must be TRUE):
  1. All 12 new CWEs (121, 126, 328, 338, 369, 426, 467, 526, 535, 676, 680, 780) produce ≥1 TP on the Juliet corpus (or, for CWEs whose Juliet test shape does not match the rule's AST pattern — confirmed corpus gaps: CWE-121 array-subscript shape vs `alloca`, CWE-369 runtime-variable divisor vs literal `/0`, CWE-676 `cin >>` operator vs function-call rule — the TP may be supplied via the synthetic-fixture unit test in `tests/vulnerability_tests/ast_scanner_tests.rs`; ANALYSIS.md must document each synthetic-TP CWE and cite the covering unit test)
  2. FP% for each new CWE is ≤35% using the file-level oracle (scanner CWE matches Juliet directory CWE family)
  3. No regression on existing 13 CWEs — AUTOSAR fixture finding counts unchanged
  4. benchmark/juliet/ANALYSIS.md updated with new per-CWE TP/FP rows
**Plans**: 3 plans
- [x] 21-01-PLAN.md — Infrastructure: ArgCheck::SizeofPointer variant + arm + pointer-scope collectors; apply_division_rules() helper + scan-loop dispatch; 2 unit tests (CWE-467, CWE-369)
- [x] 21-02-PLAN.md — Rule table expansion: 11 new AstCweRule entries (CWE-121, 126, 328 ×3, 338, 426, 526, 535, 676, 680, 780 ×2) + 10 unit tests
- [x] 21-03-PLAN.md — Validation: regenerate benchmark/juliet/ast.json + update benchmark/juliet/ANALYSIS.md with 12 new per-CWE TP/FP rows + human-review checkpoint
**UI hint**: no

### Phase 22: ast-cwes-structuralPattern-expansion

**Goal**: Expand AST scanner from 26 to 41 CWEs by adding rules that detect structural code patterns — pure AST shape queries requiring no dataflow or type analysis. Patterns include: missing switch default/break, assignment-in-condition, return of stack address, always-true/false expressions, infinite loops, and direct unguarded recursion. Validated against Juliet ground truth.
**Depends on**: Phase 21
**Requirements**: CWEXP-02
**Success Criteria** (what must be TRUE):
  1. All 15 new CWEs (256, 398, 478, 480, 481, 482, 483, 484, 562, 570, 571, 587, 617, 674, 835) produce ≥1 TP on the Juliet corpus
  2. FP% for each new CWE is ≤40% using file-level oracle
  3. No regression on existing 26 CWEs
  4. benchmark/juliet/ANALYSIS.md updated
**Plans**: 3 plans
- [x] 23-01-PLAN.md — Table rules: 5 AstCweRule entries for CWE-114, 272, 284, 427, 785 + module doc comment + 6 unit tests
- [x] 23-02-PLAN.md — Structural helpers: apply_signal_handler_rules (CWE-479), apply_paired_lock_rules (CWE-591), apply_delete_rules (CWE-762) + scan_file_ast_or_lexical wiring + synthetic fixture + 5 unit tests
- [x] 23-03-PLAN.md — Juliet benchmark re-run + ANALYSIS.md 8-row update + regression check (49-CWE final table)
**UI hint**: no

### Phase 23: ast-cwes-domainSpecific-expansion

**Goal**: Expand AST scanner from 41 to 49 CWEs by adding rules for narrow domain/API patterns — Windows privilege APIs, crypto mismatches, signal-handler non-reentrant calls, and path-manipulation functions. These rules fire on specific named APIs with optional argument token matching. Validated against Juliet ground truth where test cases exist; AUTOSAR fixture used as secondary validation for Windows API rules.
**Depends on**: Phase 22
**Requirements**: CWEXP-03
**Success Criteria** (what must be TRUE):
  1. All 8 new CWEs (114, 272, 284, 427, 479, 591, 762, 785) have rules implemented and documented
  2. CWEs with Juliet test cases produce ≥1 TP; CWEs without Juliet coverage validated on synthetic fixtures
  3. FP% for each new CWE is ≤40% using file-level oracle where Juliet coverage exists
  4. No regression on existing 41 CWEs
  5. benchmark/juliet/ANALYSIS.md updated with final 49-CWE coverage table
**Plans**: 3 plans
- [x] 23-01-PLAN.md — Table rules: 5 AstCweRule entries for CWE-114, 272, 284, 427, 785 + module doc comment + 6 unit tests
- [x] 23-02-PLAN.md — Structural helpers: apply_signal_handler_rules (CWE-479), apply_paired_lock_rules (CWE-591), apply_delete_rules (CWE-762) + scan_file_ast_or_lexical wiring + synthetic fixture + 5 unit tests
- [x] 23-03-PLAN.md — Juliet benchmark re-run + ANALYSIS.md 8-row update + regression check (49-CWE final table)
**UI hint**: no

### Phase 24: tune-high-fp-cwe-rules-from-phases-19-23

**Goal:** Audit and tighten the AnyCall/FixedSizeBuffer/structural CWE rules added in phases 19–23 that exceed the 35% FP gate on the Juliet corpus. Add context guards, restrict function lists, or promote to ArgAtIndex where applicable.
**Depends on:** Phase 23
**Requirements:** TBD

**Target CWEs (phases 19-21):** CWE-126 (94.8%), CWE-338 (99.9%), CWE-426 (100%), CWE-467 (65.4%), CWE-535 (50%), CWE-676 (100%), CWE-680 (97.5%), CWE-780 (95.3%).

**Target CWEs (phase 22, structural):** CWE-256 (100% — corpus mismatch, no Juliet files match pattern), CWE-478 (73.9%), CWE-480 (99.9% — overly broad func-ptr null compare), CWE-483 (93.2% — missing-braces rule fires on valid single-statement bodies), CWE-562 (99.8% — local-var return check fires on all non-static identifiers), CWE-570 (99.9% — literal-only by design per D-06), CWE-571 (100% — literal-only by design per D-06), CWE-587 (73.9% — fixed-address threshold too low).

**Target CWEs (phase 23, domain-specific):** CWE-762 (58.5% — text-level `delete` token scan fires across all `.cpp` files; needs co-occurrence guard + namespace exclusion per 23-REVIEW.md IN-03).

**Plans:** 3/4 plans executed
- [x] 24-01-PLAN.md — Function-list tightening + CWE-256 removal + updated/new unit tests (D-03/04/05/06/07/08/22)
- [x] 24-02-PLAN.md — New ArgCheck variants for CWE-126/680 + CWE-467 struct-ptr exclusion + CWE-535 dedicated visitor (D-09/10/11/17)
- [x] 24-03-PLAN.md — Structural visitor guards for CWE-478/480/483/562/570/571/587/762 + CWE-587 investigation (D-12/13/14/15/16/18/19/20)
- [x] 24-04-PLAN.md — Juliet oracle re-run + ANALYSIS.md regeneration + human review + AUTOSAR regression (D-21/23/25/26/27)
**UI hint**: no

### Phase 25: experiment-scan-mode

**Goal**: Users get a clean, high-precision default scan (22 CWEs — Clean/Good/AUTOSAR-confirmed) and can opt into 17 high-FP experimental rules via `--experiment-scan`. Default scan output has no FP noise from Poor/Marginal CWEs; experimental findings are emitted alongside default findings when the flag is passed.
**Depends on**: Phase 24
**Requirements**: TBD
**Milestone**: v1.0.18

**Default scan (22 CWEs):** Clean 16 (FP% ≤10%) + Good 4 (FP% 11–35%) + CWE-362 + CWE-367 (0 Juliet hits but confirmed TP on AUTOSAR)

**Experiment scan additions (17 CWEs, additive):** Marginal 1 (CWE-467) + Poor 12 (CWE-120, 122, 126, 190, 480, 483, 562, 570, 571, 676, 680, 780) + No-signal-unconfirmed 4 (CWE-338, 426, 478, 535)

**Plans:** 2/2 plans complete
- [x] 25-01-PLAN.md — Add experimental: bool to AstCweRule; thread experiment_scan: bool through call chain; gate 17 experimental CWEs; --experiment-scan CLI flag; 3 unit tests (D-11) + AUTOSAR regression update
- [x] 25-02-PLAN.md — Juliet oracle validation (default 22 CWEs unchanged, experimental CWEs fire with flag); annotate ANALYSIS.md tier table; update STATE.md and ROADMAP.md
**UI hint**: no
