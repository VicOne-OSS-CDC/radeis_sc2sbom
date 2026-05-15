---
phase: 10-internal-feature-gate
plan: 02
subsystem: models
tags: [rust, cfg-feature, dependency-model, struct-migration, feature-gate]

# Dependency graph
requires: [10-01]
provides:
  - Dependency.vulnerabilities field gated with #[cfg(feature = "internal")]
  - Vulnerability import in dependency.rs gated
  - All 33 source files outside src/vulnerability/ migrated to ..Default::default()
  - Zero explicit vulnerabilities: construction sites in non-gated code
affects:
  - 10-03 (cli.rs/formatter vulnerability symbol gating — remaining public-build errors)
  - 10-04 (integration verification)

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "#[cfg(feature = \"internal\")] on individual struct field declaration"
    - "#[cfg(feature = \"internal\")] on struct-level field initializer inside Default impl"
    - "..Default::default() struct update syntax to fill gated fields transparently"

key-files:
  created: []
  modified:
    - src/models/dependency.rs
    - src/parsers/ruby.rs
    - src/parsers/npm.rs (8 construction sites)
    - src/parsers/cmake/external_project.rs
    - src/parsers/cmake/fetchcontent.rs
    - src/parsers/cargo.rs (2 sites)
    - src/parsers/source_scanner.rs (5 sites)
    - src/parsers/java.rs
    - src/parsers/gguf.rs
    - src/parsers/meson/wrap.rs (2 sites)
    - src/parsers/meson/meson_build.rs (3 sites)
    - src/parsers/safetensors.rs
    - src/parsers/cpp/vcpkg.rs
    - src/parsers/cpp/conan.rs
    - src/parsers/cpp/conan_manifest.rs
    - src/parsers/bazel/workspace.rs (3 sites)
    - src/parsers/bazel/module.rs
    - src/parsers/php.rs
    - src/parsers/ros.rs
    - src/parsers/go.rs
    - src/parsers/python.rs (7 sites)
    - src/parsers/c/makefile_am.rs
    - src/parsers/c/pkgconfig.rs
    - src/parsers/c/autotools.rs (2 sites)
    - src/parsers/c/vendored_3rdparty.rs
    - src/parsers/c/mk_file.rs
    - src/parsers/c/library_json.rs
    - src/parsers/c/pkgconfig_detector.rs (2 sites)
    - src/parsers/c/makefile.rs
    - src/classifier/rules.rs
    - src/classifier/mod.rs
    - src/classifier/ecosystem.rs
    - src/scanner/mod.rs (2 sites)
    - src/formats/spdx.rs (test helper)

key-decisions:
  - "Leave src/vulnerability/nvd.rs and src/vulnerability/osv.rs as-is — their construction sites are inside the gated module subtree and compile only with --features internal"
  - "console.rs check_vulnerabilities parameter names, cyclonedx.rs CycloneDXDocument.vulnerabilities field, and cli.rs check_vulnerabilities field are NOT Dependency construction sites — left for plan 03"

# Metrics
duration: 15min
completed: 2026-05-09
---

# Phase 10 Plan 02: Dependency Field Gate and Construction Site Migration

**`Dependency.vulnerabilities` field gated with `#[cfg(feature = "internal")]`; all 34 construction sites in `src/` (excluding `src/vulnerability/`) migrated from explicit `vulnerabilities: Vec::new()` to `..Default::default()`**

## Performance

- **Duration:** ~15 min
- **Completed:** 2026-05-09
- **Tasks:** 2
- **Files modified:** 34

## Accomplishments

- Applied three `#[cfg(feature = "internal")]` annotations in `src/models/dependency.rs`:
  - Import: `use super::vulnerability::Vulnerability`
  - Field declaration: `pub vulnerabilities: Vec<Vulnerability>`
  - Default impl initializer: `vulnerabilities: Vec::new()`
- Ran discovery grep and identified 53 `vulnerabilities:` occurrences outside the gated module
- Migrated all `Dependency` struct construction sites (53 occurrences across 33 files) to `..Default::default()` — removing the explicit `vulnerabilities: Vec::new()` / `vec![]` lines
- Excluded `src/vulnerability/nvd.rs` and `src/vulnerability/osv.rs` from migration (they are gated subtree, correct to keep explicit field)
- Excluded `console.rs` function parameter names, `cyclonedx.rs` CycloneDXDocument field, and `cli.rs` Args field — these are not `Dependency` construction sites

## Construction Sites Migrated Per File

| File | Sites |
|------|-------|
| src/parsers/npm.rs | 8 |
| src/parsers/python.rs | 7 |
| src/parsers/source_scanner.rs | 5 |
| src/parsers/bazel/workspace.rs | 3 |
| src/parsers/meson/meson_build.rs | 3 |
| src/parsers/cargo.rs | 2 |
| src/parsers/meson/wrap.rs | 2 |
| src/parsers/cpp/vcpkg.rs | 2 |
| src/parsers/c/autotools.rs | 2 |
| src/parsers/c/pkgconfig_detector.rs | 2 |
| src/scanner/mod.rs | 2 |
| src/parsers/ruby.rs | 1 |
| src/parsers/cmake/external_project.rs | 1 |
| src/parsers/cmake/fetchcontent.rs | 1 |
| src/parsers/java.rs | 1 |
| src/parsers/gguf.rs | 1 |
| src/parsers/safetensors.rs | 1 |
| src/parsers/cpp/conan.rs | 1 |
| src/parsers/cpp/conan_manifest.rs | 1 |
| src/parsers/bazel/module.rs | 1 |
| src/parsers/php.rs | 1 |
| src/parsers/ros.rs | 1 |
| src/parsers/go.rs | 1 |
| src/parsers/c/makefile_am.rs | 1 |
| src/parsers/c/pkgconfig.rs | 1 |
| src/parsers/c/vendored_3rdparty.rs | 1 |
| src/parsers/c/mk_file.rs | 1 |
| src/parsers/c/library_json.rs | 1 |
| src/parsers/c/makefile.rs | 1 |
| src/classifier/rules.rs | 1 |
| src/classifier/mod.rs | 1 |
| src/classifier/ecosystem.rs | 1 |
| src/formats/spdx.rs (test helper) | 1 |
| **Total** | **53** |

Files with zero `vulnerabilities:` construction sites (already absent): none — all listed files were migrated.

## Task Commits

1. **Task 1: Gate Dependency.vulnerabilities field, import, and Default initializer** - `31a2231` (feat)
2. **Task 2: Migrate all construction sites** - `f0abf41` (feat)

## Verification Results

- `grep -rn "vulnerabilities: vec!" src/ --include="*.rs" | grep -v "src/vulnerability/"` → 0 lines
- `grep -rn "vulnerabilities: Vec::new" src/ | grep -v "src/vulnerability/" | grep -v "src/models/dependency.rs"` → 0 lines
- `cargo build --release --features internal` → exits 0 (2 unrelated dead-code warnings only)
- `cargo build --release 2>&1 | grep "no field named .vulnerabilities" | grep -v "src/vulnerability/" | grep -v "src/cli.rs" | grep -v "src/formats"` → 0 lines
- Public-build remaining errors confined to: `src/cli.rs` (Vulnerability/VulnerabilitySeverity/FixAction unresolved), `src/formats/console.rs` (dep.vulnerabilities reads, VulnerabilityOutputMode, SbomMode), `src/formats/cyclonedx.rs` (reqwest, Vulnerability symbols), `src/parsers/*.rs` reqwest references (inside gated scanner code) — all plan 03 scope

## Decisions Made

- Left `src/vulnerability/nvd.rs` and `src/vulnerability/osv.rs` construction sites unchanged — these are inside the `cfg(feature = "internal")` module gate from plan 01 and only compile when the feature is on, where the `vulnerabilities` field exists
- Did not migrate `check_vulnerabilities: bool` parameter names in `console.rs` — these are function signatures, not `Dependency` struct fields

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None.

## Threat Flags

None — no new network endpoints, auth paths, or file access patterns introduced.

---
*Phase: 10-internal-feature-gate*
*Completed: 2026-05-09*

## Self-Check: PASSED

Files verified:
- src/models/dependency.rs: FOUND, has 3 `#[cfg(feature = "internal")]` annotations
- Zero construction sites outside gated module (verified by grep)
- Commit 31a2231: FOUND
- Commit f0abf41: FOUND
- Internal build: exits 0
- Public build construction errors: 0 outside cli.rs/formats
