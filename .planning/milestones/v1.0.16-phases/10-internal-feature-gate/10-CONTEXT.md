# Phase 10: Internal Feature Gate - Context

**Gathered:** 2026-05-09
**Status:** Ready for planning

<domain>
## Phase Boundary

Add `cargo feature = "internal"` so that `cargo build --release` (no flags) produces a public binary with **zero** vulnerability scanning code — no OSV API calls, no NVD enrichment, no lexical CWE scanner stub, no vulnerability CLI flags. Internally-distributed builds use `--features internal` to enable the full scanning stack. The public release workflow retains both the feature gate (runtime enforcement) and the strip script (source-level enforcement) as belt-and-suspenders.

Requirements in scope: GATE-01, GATE-02, GATE-03, GATE-04

</domain>

<decisions>
## Implementation Decisions

### Gate Boundary — What Goes Behind the Flag

- **D-01:** Everything vulnerability-related goes behind `#[cfg(feature = "internal")]` — structs, API logic, formatter branches, CLI flags, and console display code. No vulnerability code remains in the unconditional compilation path.
- **D-02:** `VulnerabilityInfo`, `CweInfo`, and all types in `src/vulnerability/` are gated.
- **D-03:** The `vulnerabilities` field on the `Dependency` struct is gated via `#[cfg_attr]` — it is absent in non-internal builds. Tests that construct `Dependency` use `..Default::default()` to handle the missing field.
- **D-04:** Formatter branches in `src/formats/cyclonedx.rs`, `src/formats/spdx.rs`, and `src/formats/console.rs` that emit vulnerability entries are wrapped in `#[cfg(feature = "internal")]`.
- **D-05:** `VulnerabilityOutputMode` enum and all console vulnerability rendering blocks are gated.
- **D-06:** CLI flags `--vulnerability-output`, `--cache-ttl`, `--vulnerability-timeout`, `--clear-vulnerability-cache` are gated — they do not appear in `--help` on the public binary.

### GATE-03 Stub — Lexical Scanner Placeholder

- **D-07:** Create `src/vulnerability/cwe_scanner.rs` as an empty stub, wrapped in `#[cfg(feature = "internal")]`. This file is the landing zone for Phase 11's scanner implementation.
- **D-08:** Location rationale: sits alongside `osv.rs` and `nvd.rs` in `src/vulnerability/` — all gated scanning code in one directory.

### Test Isolation

- **D-09:** Gate the entire `tests/vulnerability_tests/` module with `#[cfg(feature = "internal")]` at the top of `tests/vulnerability_tests/mod.rs`. Tests in that module only compile and run with `--features internal`.
- **D-10:** Integration/classifier/format tests that construct `Dependency` structs use `..Default::default()` for the cfg-gated `vulnerabilities` field. No per-test-file cfg annotations needed for those external test files.

### Cargo.toml — Optional Dependency

- **D-11:** `reqwest` becomes an optional dependency: `reqwest = { version = "0.11", optional = true, features = [...] }` with `internal = ["reqwest"]` in `[features]`. Public binary does not link reqwest.
- **D-12:** `sha2` and `dirs` remain unconditional (small crates, low overhead).

### CI / Release Workflow

- **D-13:** Both enforcement mechanisms stay active:
  1. **Feature gate** — `#[cfg(feature = "internal")]` prevents any gated code from compiling into the public binary.
  2. **Strip script** — `scripts/strip_vulnerability.sh` physically removes `src/vulnerability/` and related files from the `pub_release/` branch before it is pushed. Ensures source-level privacy for open-source distribution.
- **D-14:** Phase 10 updates `scripts/strip_vulnerability.sh` to also remove `src/vulnerability/cwe_scanner.rs` (the new stub). The script must stay in sync with the gate boundary after this phase.
- **D-15:** `build-release.yml` (internal CI) builds with `--features internal`. `public-release.yml` builds with `cargo build --release` (no feature flags) after the strip step.

### Claude's Discretion

- Whether to use a `#[cfg(feature = "internal")] mod vulnerability;` declaration in `lib.rs`/`main.rs` (gating the whole module at the mod declaration level) versus individual `#[cfg]` on each item — Claude picks the approach that compiles cleanest and requires fewest redundant annotations.
- Exact placement of `#[cfg_attr]` on the `Dependency.vulnerabilities` field (field-level vs. wrapper type) — Claude picks the form that compiles with the least churn to existing `Dependency` construction sites.

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements
- `.planning/REQUIREMENTS.md` §Feature Gate — GATE-01..GATE-04 definitions and acceptance criteria
- `.planning/ROADMAP.md` §Phase 10 — success criteria (4 items); the `--features internal` matrix is the acceptance test

### Source Files to Gate (read before planning file list)
- `src/main.rs` — imports `OsvProvider`, `query_vulnerabilities_batch`, `enrich_cwe_ids`, `clear_vulnerability_cache`; call sites in vulnerability scanning block need `#[cfg(feature = "internal")]`
- `src/vulnerability/mod.rs` — re-exports `osv`, `nvd`, `fix_recommendations`; entire module goes behind the gate
- `src/vulnerability/osv.rs` — `OsvProvider` struct + `query_vulnerabilities_batch` + `clear_vulnerability_cache`
- `src/vulnerability/nvd.rs` — `enrich_cwe_ids`
- `src/vulnerability/fix_recommendations.rs` — fix recommendation logic
- `src/formats/cyclonedx.rs` — vulnerability[] serialization branches
- `src/formats/spdx.rs` — CVE ExternalRef branches
- `src/formats/console.rs` — `VulnerabilityOutputMode` enum + all rendering branches
- `src/lib.rs` — `pub mod vulnerability;` declaration
- `Cargo.toml` — `[features]` section; reqwest becomes optional

### CI Workflows
- `.github/workflows/public-release.yml` — strip script invocation + audit grep; understand before updating
- `.github/workflows/build-release.yml` — internal build; verify `--features internal` is added
- `scripts/strip_vulnerability.sh` — must be updated to include `cwe_scanner.rs` in removal list

### Tests
- `tests/vulnerability_tests/mod.rs` — gate entire module with `#[cfg(feature = "internal")]`
- `tests/all_tests.rs` — check if vulnerability_tests is included here; may need cfg guard

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `[features]` section in `Cargo.toml` already exists (`cn-release = []`) — established pattern for adding `internal = []` and `internal = ["reqwest"]`
- `Dependency` struct already derives `Default` — non-internal builds can use `..Default::default()` at construction sites without structural changes

### Established Patterns
- No `#[cfg(feature = ...)]` guards exist anywhere in `src/` yet — Phase 10 establishes this pattern from scratch; researcher should verify the exact Rust idioms for gating a whole module (module-level `#[cfg]` on `mod` declaration vs. item-level guards)
- `cn-release` feature in `Cargo.toml` uses `[]` (no deps) as the pattern — `internal` follows same syntax but adds `["reqwest"]` for the optional dep

### Integration Points
- `src/main.rs` is the primary call site: the OSV/NVD scanning block (lines ~175–210) is the main `#[cfg(feature = "internal")]` gate in main logic
- `src/lib.rs:12` — `pub mod vulnerability;` becomes `#[cfg(feature = "internal")] pub mod vulnerability;`
- `Dependency` struct location: researcher should confirm exact file (`src/models/` or similar) and field definition to assess `cfg_attr` placement

</code_context>

<specifics>
## Specific Ideas

- The public binary should have **zero** vulnerability-related strings in `--help` output — this is not just about runtime behavior, it's about the public API surface
- Both strip script and feature gate are retained as belt-and-suspenders — this is a firm decision, not a "you decide" area
- `cwe_scanner.rs` stub should be skeletal (empty pub functions or just a module comment `// Phase 11: lexical CWE scanner`) — enough to prove the gate structure works, not placeholder implementation

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 10-internal-feature-gate*
*Context gathered: 2026-05-09*
