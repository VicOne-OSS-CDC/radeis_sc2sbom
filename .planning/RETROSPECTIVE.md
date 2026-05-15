# Retrospective: radeis_sc2sbom

---

## Milestone: v1.0.14 — Reliability & Compatibility

**Shipped:** 2026-04-24
**Phases:** 5 | **Plans:** 6
**Timeline:** ~2 days (2026-04-23 → 2026-04-24)
**Commits:** ~58 in milestone branch + 4 hotfix commits (PRs #42–43)

### What Was Built

1. WalkDir broken-symlink errors converted to stderr warnings — all four WalkDir sites warn and continue instead of aborting or silently dropping
2. Makefile variable refs `$(VAR)` filtered at parser level + formatter-level NOASSERTION guards (defense-in-depth)
3. `known_licenses.rs` with 24 SPDX-mapped entries for common C/C++ system libraries
4. `License:` field parsing added to pkgconfig.rs
5. `known_licenses::lookup()` fallback wired into makefile.rs and mk_file.rs
6. Linux binary switched to `x86_64-unknown-linux-musl` — static, no glibc version dependency
7. Follow-up hotfixes (PRs #42–43) to resolve reqwest TLS compilation failures on musl target

### What Worked

- **Subagent-driven execution** — each phase planned and executed in isolated context, reducing cross-phase contamination
- **Verification.md per phase** — green scores (7/7, 5/5, 4/4) gave clear confidence before advancing
- **Defense-in-depth for Makefile vars** — parser + formatter guards caught edge cases that a single-layer fix would have missed
- **Phase 5 identified by verification** — running the integration scan against xcar-linux after Phase 1 revealed the three additional WalkDir abort paths; the verification step caught what code review didn't

### What Was Inefficient

- **GSD state not updated during execution** — SUMMARY.md files not written until post-hoc cleanup; state showed phases as "in-progress" when they had already shipped
- **Musl TLS required two hotfix PRs** — reqwest's OpenSSL dependency wasn't caught at plan time; could have been addressed in Phase 4 planning by auditing transitive dependencies
- **Phase 5 was unplanned** — discovered during Phase 1 verification; added reactively. Better pre-phase verification planning could catch this earlier

### Patterns Established

- `warn_on_err` WalkDir helper extracted to reduce duplication across all four WalkDir sites (`refactor: extract walkdir warn-on-err helper`)
- Formatter-level version guards as a second layer behind parser filters for `$(VAR)` values
- SUMMARY.md should be written immediately after execution completes, not deferred

### Key Lessons

- **Run integration scan early** — verification against the actual target repo (xcar-linux) revealed Phase 5 scope that static analysis missed
- **Check transitive deps before musl plan** — `reqwest` pulling in `openssl` should be caught at plan time via `cargo tree`
- **Keep GSD state current** — stale SUMMARY.md files create confusion about milestone completion; write them as part of the execution step, not cleanup

### Cost Observations

- Model mix: sonnet executor, opus planner
- Sessions: multiple short sessions across 2 days
- Notable: phase-level isolation kept each session focused; no context blowout

---

## Milestone: v1.0.15 — AUTOSAR Support & CVE Enrichment

**Shipped:** 2026-05-09
**Phases:** 4 (6–9) | **Plans:** 10
**Timeline:** 15 days (2026-04-24 → 2026-05-09) | 119 commits | +17,603 / -2,291 lines

### What Was Built

1. AUTOSAR project detection pre-pass via .arxml, BSW/MCAL/RTE dirs, and build-file variable heuristics — `ScanContext.is_autosar`
2. BSW module classifier from bundled `bsw_modules.yaml` (80+ modules) with `autosar:layer` and `autosar:platform` properties in both SPDX 2.3 and CycloneDX 1.5
3. `--supplier-config` YAML flag for component→supplier mapping; NOASSERTION for unmatched components
4. `Option<&SupplierResolver>` threaded as trailing parameter through all formatter signatures (6 call sites in main.rs)
5. NVD 2.0 CWE enrichment with TTL disk cache, 6-second rate-limit sleep, and sha256-keyed cache entries; CWE IDs appear in all three output formats
6. Quick task: `--output` flag fixed for single-format mode (prints to stdout by default)
7. Code review cycle on Phase 8 caught 4 issues (percent-encoding in SPDX, safetensors guard, colon vs hyphen in ExternalRef, variable naming)

### What Worked

- **Subagent-driven TDD** — test-first execution caught integration issues before they reached review
- **RESEARCH.md before planning** — Phase 8 research surfaced the 6th formatter call site (`save_spdx_tag_value`) before plan execution, preventing a missed update
- **Code review per phase** — Phase 8 review found the percent-encoding and safetensors guard issues that unit tests didn't cover
- **Worktree isolation** — quick task executed in parallel worktree without touching the main branch mid-milestone
- **NVD cache design** — TTL + sha256 key + `SC2SBOM_NVD_CACHE_DIR_OVERRIDE` for test isolation made the NVD module deterministically testable

### What Was Inefficient

- **Phase 10 deferred** — the internal feature gate was scoped into v1.0.15 but never executed; was always a prerequisite for v1.0.16 and should have been planned there from the start
- **ROADMAP.md progress table not updated during execution** — table still showed stale status at milestone close; GSD state and SUMMARY.md were accurate but ROADMAP table lagged
- **SDK accomplishment extraction** — `gsd-sdk query milestone.complete` failed to extract one-liners from SUMMARY.md files (format mismatch); required manual correction of MILESTONES.md

### Patterns Established

- `Option<&SupplierResolver>` as trailing parameter pattern for optional context threading through formatters — avoids struct proliferation for single-consumer options
- `autosar_metadata.is_some()` guard before AUTOSAR-scoped property emission — prevents AUTOSAR properties from bleeding into non-AUTOSAR components
- `SC2SBOM_*_OVERRIDE` env var pattern for test isolation of filesystem-dependent modules (NVD cache dir)
- sha256-keyed disk cache for external API responses — stable key, TTL-checkable without parsing the payload

### Key Lessons

- **Scope phase 10 (feature gates) as a prerequisite of the milestone that needs them** — don't scope it into the milestone that precedes the consumer; it'll always slip
- **Write ROADMAP progress table updates as part of phase execution commit** — not at milestone close
- **Verify SDK summary extraction format before milestone close** — run `gsd-sdk query summary-extract` during phase wrap-up to confirm output is parseable

### Cost Observations

- Model mix: sonnet executor, opus planner
- Sessions: ~10 sessions over 15 days
- Notable: worktree isolation for the quick task worked cleanly; no main-branch disruption

---

## Milestone: v1.0.16 — C/C++ Lexical CWE Scanner

**Shipped:** 2026-05-10
**Phases:** 3 (10–12) | **Plans:** 11
**Timeline:** 2 days (2026-05-09 → 2026-05-10) | ~124 commits | +2,407 / -1,150 lines (src/tests)

### What Was Built

1. `cargo feature = "internal"` gate — all CVE/CWE/scanner code absent from public binary at compile time; 58 test construction sites migrated to `..Default::default()`; 4 test files gated with `#![cfg(feature = "internal")]`
2. Pure-Rust lexical CWE scanner — 14-CWE static rule table, paren-bound word-boundary matcher, CWE-134 format-arg heuristic; detects dangerous C/C++ function calls by file and line
3. `ScanContext.component_dirs` populated at all six C/C++ parser call sites in `scan_directory` — scanner is scoped to component-mapped directories only (SCAN-05)
4. CycloneDX 1.5 `vulnerabilities[]` entries for SAST findings — `cwes[]`, `affects[].ref`, `sc2sbom:finding:file/line` properties, `analysis.state: "in_triage"`
5. `_static_analysis.md` report with per-component CWE summary table and file:line findings grouped by component then CWE
6. SAST findings section injected into main `_report.md` alongside CVE findings
7. CLI disclaimer: "Pattern-based — complex data-flow vulnerabilities not covered" emitted on stderr
8. Quick tasks: fix component-mapping inflation (260510-2a4), fallback for standalone C repos with no manifest (260510-326), strip script updated for all v1.0.16 cfg-gated code patterns (3 fix iterations for PR #46 CI)

### What Worked

- **SEED-based planning** — SEED-001 captured the full feature spec before a line of code was written; the REQUIREMENTS.md translated directly to success criteria per plan
- **Inline `#[cfg(feature = "internal")]` parameter gating** — consistent with existing SbomMode pattern; no code duplication, no cfg-split functions needed
- **TDD discipline held** — RED tests written first in Phase 12 Plan 01 caught the `SastFinding` field name discrepancy (no `function_name`) before GREEN implementation, saving a refactor
- **Quick tasks for production bugs** — component-mapping inflation and the Juliet standalone repo gap were caught and fixed within the same milestone via worktree-isolated quick tasks
- **Fallback synthetic entry pattern** — solving the "no manifest → no component_dirs → scanner skips everything" case with a single synthetic entry in `main.rs` was minimal and correct

### What Was Inefficient

- **Strip script required 3 CI fix rounds** — the public-release CI audit checks were not run locally before pushing; each round was a separate commit+push+wait cycle. Should dry-run CI checks locally before first push.
- **Phase 12 VERIFICATION.md marked `human_needed`** — the stderr disclaimer test was intentionally `#[ignore]`'d, leaving RPT-03 unverified until milestone close. The live smoke test is a 30-second command; it should be part of the verification step, not a pre-close blocker.
- **Audit SDK slug-match bug** — `gsd-sdk query audit-open` reported 3 quick tasks as `missing` despite SUMMARY files existing on disk. Required manual investigation to confirm they were false positives. The SDK bug means audit results cannot be trusted blindly.

### Patterns Established

- `component_dirs` field unconditional on `ScanContext` — cfg-gating a struct field causes a cascade of `#[cfg]` on every site that reads it; prefer unconditional fields that carry no runtime cost when unused
- Synthetic fallback entry for manifest-free repos: detect C/C++ files under scan root → insert `(project_name, "C/C++") → scan_root` into `component_dirs` in `main.rs` before calling the scanner
- Strip script dry-run protocol: after each change to `strip_vulnerability.sh`, run `git restore src/ tests/ && bash scripts/strip_vulnerability.sh && cargo build --release` locally, then run each of the three CI audit greps before committing

### Key Lessons

1. **Run CI audit checks locally before pushing strip script changes** — `grep -rn "VulnerabilityOutputMode\|SbomMode" $(git diff --name-only HEAD) | grep -v "^#"` takes 5 seconds; waiting for CI takes 10 minutes
2. **Close human-verification items during phase execution, not at milestone close** — any test marked `#[ignore]` for "requires integration harness" should be accompanied by a manual smoke test in the VERIFICATION.md checklist, completed before the phase wraps
3. **Unconditional struct fields for cfg-gated consumers** — `component_dirs` on `ScanContext` is the right call; a zero-cost field is better than a cfg cascade through 6+ call sites
4. **SEED spec → REQUIREMENTS.md translation works** — SEED-001 had 13 CWEs; REQUIREMENTS.md had 14 (one was split); the translation was clean and requirements → plan → verify was traceable throughout

### Cost Observations

- Model mix: sonnet executor, opus planner
- Sessions: ~6 sessions over 2 days
- Notable: 3 strip-script fix rounds added ~30 min overhead that local dry-run would have eliminated

---

## Milestone: v1.0.17 — Advanced C/C++ SAST Scanner

**Shipped:** 2026-05-11
**Phases:** 5 (13–17) | **Plans:** 12
**Timeline:** 2 days (2026-05-10 → 2026-05-11) | 134 commits | +16,421 / -364 lines

### What Was Built

1. Argument-value matching (Phase 13): AND-all paren-arg token matching for CWE-295 (SSL_VERIFY_NONE), CWE-319 (CURLOPT_USE_SSL), CWE-732 (umask/DACL), CWE-369 (literal division by zero) — zero external deps
2. cppcheck integration (Phase 14): `SastSource` enum, `parse_cppcheck_xml` with 15-entry CWE override table, `run_cppcheck_scanner` subprocess driver with indicatif spinner and graceful degradation, `deduplicate_sast_findings` by `(file, line, cwe)`
3. SARIF 2.1 writer (Phase 15): hand-rolled serde structs, BTreeSet rule deduplication, `--sarif-output` CLI flag, per-CWE `helpUri` in `rules[]`
4. SARIF as authoritative store (Phase 16): SHA-256 `partialFingerprints`, `suppress_lexical_false_positives` drops Lexical findings where cppcheck ran and didn't confirm, `--sarif-baseline` CI gate (exits 1 on new findings, writes diff SARIF only on regression)
5. AUTOSAR arxml parser + version extraction (Phase 17): `parse_arxml` extracts SW-COMPONENT-PROTOTYPE, BSW-MODULE-DESCRIPTION, and 5 SWC type definition element types; `collect_epd_versions` + `collect_doxygen_versions` supply real version strings; post-walk pass upgrades system linker deps to autosar ecosystem — AUTOSAR_SampleProject_S32K144 goes from 0 to 17/18 components with real versions

### What Worked

- **Phase 16 inserted mid-milestone cleanly** — SARIF-as-authoritative-store was not in the original v1.0.17 plan; inserted as Phase 16 after Phases 13–15 completed without disrupting the roadmap or commit history
- **Bug triage via verified scan output** — running the scanner against AUTOSAR_SampleProject_S32K144 after Phase 15 revealed all four AUTOSAR bugs in a single pass; each bug was scoped as a discrete fix and committed atomically
- **quick task for BUG-03 planning** — the version extraction task was complex enough to warrant its own PLAN.md; the quick task format gave it a spec without the overhead of a full phase
- **SDK milestone.complete** — automated the archive creation; saved ~30 minutes of manual file prep (though accomplishment extraction still failed due to format mismatch — see below)

### What Was Inefficient

- **SUMMARY.md one-liner format mismatch (again)** — `gsd-sdk query milestone.complete` extracted "One-liner:" placeholders for 8 of 12 plans because those SUMMARY files use `**One-liner:**` in markdown body, not a `one_liner:` YAML frontmatter field. MILESTONES.md required manual correction for the third consecutive milestone. The fix is to standardize SUMMARY.md format across all phases.
- **Phase 17 directory not created before planning** — the phase directory didn't exist when `gsd-progress` ran, showing Phase 17 as "not started" even though bugs were already fixed. Should create the directory + PLAN.md at bug-discovery time, not after.
- **Planning docs lagged commits** — BUG-01 through BUG-04 were all fixed before Phase 17's PLAN.md and SUMMARY.md were written. The commits were clean but the planning state was stale for hours.

### Patterns Established

- **Atomic bug-fix commits with phase/BUG reference** — `fix(17/BUG-01): ...` commit convention links each commit to its planning artifact; easy to bisect and trace
- **Post-walk ecosystem upgrade pass** — `scanner/mod.rs` post-walk loop matches system linker deps against epd/doxygen version maps and upgrades them to autosar ecosystem; cleaner than patching during the walk
- **Version map built once per project scan** — `collect_epd_versions` + `collect_doxygen_versions` called once at scan start, passed as `&HashMap` into `parse_arxml`; avoids per-file re-walks

### Key Lessons

1. **Standardize SUMMARY.md one-liner format** — use `one_liner:` in YAML frontmatter, not `**One-liner:**` in markdown body. SDK extraction breaks every milestone until this is consistent.
2. **Create phase directory at bug-discovery time** — when a bug phase is added to ROADMAP.md, create the directory and PLAN.md immediately so planning state reflects reality
3. **Insert PLAN.md before the first commit, SUMMARY.md immediately after the last** — planning docs should bracket the work, not be written retroactively

### Cost Observations

- Model mix: sonnet executor, opus planner
- Sessions: ~4 sessions over 2 days
- Notable: Phase 16 insertion was smooth — mid-milestone scope addition without planning debt

---

## Cross-Milestone Trends

| Milestone | Phases | Plans | Unplanned Work | Hotfixes |
|-----------|--------|-------|----------------|----------|
| v1.0.14 | 5 | 6 | Phase 5 (symlink gap) | PRs #42–43 (musl TLS) |
| v1.0.15 | 4 | 10 | Quick task: --output fix | None |
| v1.0.16 | 3 | 11 | Quick tasks: mapping inflation + Juliet fallback | 3 strip-script CI rounds (PR #46) |
| v1.0.17 | 5 | 12 | Phase 16 inserted (SARIF authoritative store) + Phase 17 (AUTOSAR bugs) | None |

**Recurring theme:** Integration testing against the real target repo consistently reveals scope that static code analysis misses. Make it a standard step.

**v1.0.15 observation:** Feature gates (internal feature, CI workflow changes) slip when scoped into the milestone before their consumers. Scope them into the milestone that actually needs them.

**v1.0.16 observation:** Manual steps with external feedback loops (CI, live binary runs) should have local dry-run protocols established in the plan. When those protocols exist, the feedback loop shortens from minutes to seconds.

**v1.0.17 observation:** SUMMARY.md one-liner extraction has failed for three consecutive milestones due to format mismatch (`**One-liner:**` markdown vs `one_liner:` YAML frontmatter). This is the highest-priority process fix before v1.0.18.
