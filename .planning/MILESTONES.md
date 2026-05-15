# Milestones: radeis_sc2sbom

## v1.0.17 — Advanced C/C++ SAST Scanner

**Shipped:** 2026-05-11
**Phases:** 5 (13–17) | **Plans:** 12
**Timeline:** 2026-05-10 → 2026-05-11 (2 days) | 134 commits | +16,421 / -364 lines

### Delivered

Extended the internal SAST scanner with argument-value CWE matching (zero-dep TLS/crypto detection), cppcheck subprocess integration (dataflow-backed findings merged via dedup pipeline), SARIF 2.1 output as authoritative finding store with stable SHA-256 fingerprints and `--sarif-baseline` CI gate, and AUTOSAR arxml/epd/Doxygen version extraction shipping 17/18 BSW components with real version strings.

### Key Accomplishments

1. Argument-value matching (Phase 13): AND-all paren-arg token matching adds CWE-295 (TLS verify disabled), CWE-319 (insecure curl options), CWE-732 (permissive umask/DACL), CWE-369 (literal divide-by-zero) — zero external dependencies
2. cppcheck pipeline (Phase 14): `SastSource` enum + `parse_cppcheck_xml` + `run_cppcheck_scanner` subprocess driver + `deduplicate_sast_findings` by `(file, line, cwe)` — dual-detected findings tagged `SastSource::Both`
3. SARIF 2.1 writer (Phase 15): hand-rolled serde structs with `BTreeSet` rule deduplication; `--sarif-output` CLI flag; `rules[]` entries with CWE `helpUri`
4. SARIF as authoritative store (Phase 16): SHA-256 `partialFingerprints` on every result; `suppress_lexical_false_positives` drops Lexical findings covered by cppcheck; `--sarif-baseline` exits 1 on new findings, writes diff SARIF only on regressions
5. AUTOSAR arxml parser (Phase 17/BUG-01–02): `src/parsers/c/arxml.rs` extracts `SW-COMPONENT-PROTOTYPE`, `BSW-MODULE-DESCRIPTION`, and all SWC type definition elements
6. AUTOSAR version extraction (Phase 17/BUG-03–04): `.epd` `REVISION-LABEL` + Doxygen `SW Version` headers populate real versions; post-walk pass upgrades system linker deps to autosar ecosystem — AUTOSAR_SampleProject_S32K144 shows 17/18 components with real versions

### Known Deferred Items at Close: 7

- 4 quick tasks (3 SDK slug-match false positives + autosar-version-extraction now complete with SUMMARY.md)
- SEED-001/002: superseded by v1.0.16/v1.0.17 respectively
- SEED-003: pending for v1.0.18 tree-sitter AST scanner
- See STATE.md Deferred Items for full list

### Archive

- [v1.0.17-ROADMAP.md](milestones/v1.0.17-ROADMAP.md)
- [v1.0.17-REQUIREMENTS.md](milestones/v1.0.17-REQUIREMENTS.md)

---

## v1.0.16 — C/C++ Lexical CWE Scanner

**Shipped:** 2026-05-10
**Phases:** 3 (10–12) | **Plans:** 11
**Timeline:** 2026-05-09 → 2026-05-10 (2 days) | ~124 commits | +2,407 / -1,150 lines

### Delivered

Pure-Rust lexical SAST scanner detecting 14 CWE categories in C/C++ source; findings surfaced in CycloneDX 1.5 output and a dedicated `_static_analysis.md` report. All CVE/CWE/scanner code gated behind `cargo feature = "internal"` so the public binary compiles with zero vulnerability symbols.

### Key Accomplishments

1. `cargo feature = "internal"` gate — public binary contains zero CVE/CWE/scanner symbols; all four mount points gated (Cargo.toml, lib.rs, main.rs, models/mod.rs); reqwest made optional via `dep:` syntax (GATE-01..04)
2. Pure-Rust lexical scanner with 14-CWE static rule table, paren-bound word-boundary matcher, CWE-134 format-arg heuristic; `ScanContext.component_dirs` scopes scanning to component-mapped C/C++ dirs only (SCAN-01..05)
3. CycloneDX 1.5 `vulnerabilities[]` entries for each finding — `cwes[]` integer array, `affects[].ref` to owning bom-ref, `sc2sbom:finding:file` and `sc2sbom:finding:line` properties, `analysis.state: "in_triage"` (CDX-01..04)
4. `_static_analysis.md` report: per-component CWE summary table + file:line findings grouped by component then CWE; zero-findings case handled gracefully (RPT-01)
5. SAST findings section injected into main `_report.md` alongside CVE block; ordering verified by line-number check (RPT-02)
6. CLI stderr disclaimer: "Pattern-based — complex data-flow vulnerabilities not covered"; runtime-verified against real C project (RPT-03)
7. Quick task 260510-2a4: component-mapping inflation fixed via `resolve_component_dir` helper at 6 insertion sites
8. Quick task 260510-326: fallback synthetic `(project_name, "C/C++") → scan_root` entry enables scanning standalone C repos (Juliet, bare projects) with no manifest-derived component dirs

### Known Deferred Items at Close: 5

- 3 quick tasks reported missing by audit SDK (false positives — SUMMARY files exist on disk; audit slug-match bug)
- SEED-001: superseded by this milestone (shipped as v1.0.16)
- SEED-002: intentional future milestone seed for v1.0.17 cppcheck integration

### Archive

- [v1.0.16-ROADMAP.md](milestones/v1.0.16-ROADMAP.md)
- [v1.0.16-REQUIREMENTS.md](milestones/v1.0.16-REQUIREMENTS.md)

---

## v1.0.15 — AUTOSAR Support & CVE Enrichment

**Shipped:** 2026-05-09
**Phases:** 4 (6–9) | **Plans:** 10
**Timeline:** 2026-04-24 → 2026-05-09 (15 days) | 119 commits | +17,603 / -2,291 lines

### Delivered

AUTOSAR project detection, BSW module classification with layer/platform properties in SPDX and CycloneDX output, supplier mapping via YAML config, and NVD CWE enrichment for discovered CVEs.

### Key Accomplishments

1. AUTOSAR pre-pass detects projects via .arxml files, BSW/MCAL/RTE directory names, and AUTOSAR_VERSION build-file variables (DET-01/02/03)
2. BSW module classifier matches 80+ modules from bundled `bsw_modules.yaml`, assigns `autosar:layer` and `autosar:platform` properties in SPDX 2.3 and CycloneDX 1.5 output (CLS-01/02/03, OUT-01/02)
3. `--supplier-config` YAML flag maps component names to supplier strings; unmatched components emit NOASSERTION (OUT-03)
4. `Option<&SupplierResolver>` threaded as trailing parameter through all SPDX and CycloneDX formatter signatures, wired at all 6 main.rs call sites
5. NVD 2.0 CWE enrichment module with TTL disk cache and 6-second rate-limit sleep; enriched CWE IDs appear in SPDX ExternalRef, CycloneDX cwes[], and markdown report (CWE-01/02/03/04)
6. `--output` flag fixed for single formats (spdx-json, cyclonedx-json, spdx-tag-value, console) — prints to stdout by default, writes to file when --output specified

### Known Deferred Items at Close: 4

- GATE-01..GATE-04 (Phase 10 — internal feature gate) deferred to v1.0.16 as prerequisite for lexical scanner
- See STATE.md Deferred Items for full list

### Archive

- [v1.0.15-ROADMAP.md](milestones/v1.0.15-ROADMAP.md)
- [v1.0.15-REQUIREMENTS.md](milestones/v1.0.15-REQUIREMENTS.md)

---

## v1.0.14 — Reliability & Compatibility

**Shipped:** 2026-04-24
**Phases:** 5 | **Plans:** 6
**PRs:** #41 (feat/v1.0.14-milestone), #42–43 (hotfix/musl-openssl-fix)

### Delivered

Fixed 5 user-reported bugs from xcar-linux scan and shipped a stable, xZETA-compatible static Linux binary running on Ubuntu 22.04+.

### Key Accomplishments

1. WalkDir broken-symlink errors converted to stderr warnings — all four WalkDir sites (scanner/mod.rs, main.rs, vendored_3rdparty.rs, so_scanner.rs) warn and continue
2. Parser-level and formatter-level guards block Makefile variable refs like `$(OPENSSL_VERSION)` from appearing as versionInfo in SPDX/CycloneDX output
3. `known_licenses.rs` (24 SPDX-mapped entries) + pkgconfig `License:` field parsing — C/C++ license detection foundation
4. `known_licenses::lookup()` wired into makefile.rs and mk_file.rs — xcar-linux license count improved measurably above 2/288
5. Linux binary switched to `x86_64-unknown-linux-musl` — fully static, no glibc dependency, runs on Ubuntu 22.04+
6. Three additional WalkDir abort paths closed — xcar-linux scan completes exit 0 with full SPDX/CycloneDX output

### Archive

- [v1.0.14-ROADMAP.md](milestones/v1.0.14-ROADMAP.md)
- [v1.0.14-REQUIREMENTS.md](milestones/v1.0.14-REQUIREMENTS.md)

---
*Archived: 2026-05-08*
