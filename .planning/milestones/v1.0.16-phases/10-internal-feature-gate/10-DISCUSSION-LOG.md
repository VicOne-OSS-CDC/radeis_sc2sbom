# Phase 10: Internal Feature Gate - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-09
**Phase:** 10-internal-feature-gate
**Areas discussed:** Gate boundary, GATE-03 stub strategy, Test isolation, reqwest dependency, CI/Release workflow

---

## Gate Boundary — What Goes Behind the Flag

| Option | Description | Selected |
|--------|-------------|----------|
| Structs public, API logic gated | VulnerabilityInfo/CweInfo stay public; only scanning functions and call sites in main.rs go behind cfg | |
| Everything behind the feature | All vulnerability code — structs, formatters, CLI flags, API logic — is gated | ✓ |
| You decide | Claude picks the approach with minimal churn | |

**User's choice:** Everything behind the feature

| Option | Description | Selected |
|--------|-------------|----------|
| Keep .vulnerabilities field, default empty Vec | Field stays on Dependency unconditionally; VulnerabilityInfo defined inside cfg | |
| Gate the field with cfg_attr | `#[cfg_attr]` on the vulnerabilities field; absent in non-internal builds | ✓ |
| You decide | Claude picks least-churn approach | |

**User's choice:** Gate the field with cfg_attr

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — gate formatter branches too | Formatter code that emits vulnerability entries wrapped in cfg | ✓ |
| Leave formatters unconditional, use empty-vec guard | Formatters stay as-is; empty vec produces no output | |

**User's choice:** Yes — gate formatter branches too

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — gate CLI flags too | Public binary --help shows no vulnerability-related flags | ✓ |
| No — keep flags, just no-op without internal | Flags still appear in --help but produce no output | |
| You decide | Claude picks based on Clap setup | |

**User's choice:** Yes — gate CLI flags too

| Option | Description | Selected |
|--------|-------------|----------|
| Gate it — consistent with gating everything | VulnerabilityOutputMode and console rendering blocks gated | ✓ |
| Keep unconditional — display is separate concern | Only rendering code that references VulnerabilityInfo gets gated | |
| You decide | Claude resolves based on what compiles cleanly | |

**User's choice:** Gate it — consistent with gating everything

**Notes:** High-integrity approach — user wants the public binary's --help to show zero vulnerability-related flags or output. Everything related to vulnerability scanning, enrichment, or display goes behind the feature gate.

---

## GATE-03 Stub Strategy — Lexical Scanner Placeholder

| Option | Description | Selected |
|--------|-------------|----------|
| Just define the feature, no scanner stub | Add `internal = []` to Cargo.toml; GATE-03 satisfied structurally | |
| Add empty scanner_cwe module stub | Create src/vulnerability/cwe_scanner.rs stub wrapped in cfg | ✓ |
| You decide | Claude picks whichever satisfies GATE-03 with least noise | |

**User's choice:** Add empty scanner_cwe module stub

| Option | Description | Selected |
|--------|-------------|----------|
| src/vulnerability/cwe_scanner.rs | Sits alongside osv.rs and nvd.rs | ✓ |
| src/scanner/cwe_scanner.rs | Lives with existing scanner module | |
| src/cwe_scanner.rs (top level) | Flat layout | |

**User's choice:** src/vulnerability/cwe_scanner.rs

**Notes:** Empty stub with `#[cfg(feature = "internal")]` — Phase 11 fills it out.

---

## Test Isolation

| Option | Description | Selected |
|--------|-------------|----------|
| Gate entire vulnerability_tests/ module | `#[cfg(feature = "internal")]` at top of mod.rs | ✓ |
| Gate per-test-file with cfg at top of each file | Individual cfg per file | |
| Move to inline #[cfg(test)] blocks in src/ | Convert to inline unit tests | |

**User's choice:** Gate entire vulnerability_tests/ module

| Option | Description | Selected |
|--------|-------------|----------|
| Use cfg_attr on field, tests use ..Default::default() | Dependency derives Default; tests use struct update syntax | ✓ |
| Add a test helper Dependency::new_for_test() constructor | Single constructor with conditional logic | |
| You decide | Claude picks what compiles cleanest | |

**User's choice:** Use cfg_attr on field, tests use ..Default::default()

**Notes:** Dependency already derives Default — minimal change to external test files that construct Dependency structs.

---

## reqwest Dependency

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — make reqwest optional | `optional = true` in Cargo.toml; `internal = ["reqwest"]` | ✓ |
| No — keep reqwest unconditional | reqwest still compiles into public binary even though unused | |

**User's choice:** Yes — make reqwest optional

| Option | Description | Selected |
|--------|-------------|----------|
| Gate sha2 and dirs too — all internal-only deps optional | All three optional under internal feature | |
| Gate reqwest only — sha2/dirs are cheap enough | Only reqwest is worth gating; sha2/dirs are tiny | ✓ |
| You decide | Claude assesses which deps are internal-only | |

**User's choice:** Gate reqwest only — sha2/dirs are cheap enough

---

## CI / Release Workflow

**Context discovered:** `public-release.yml` already runs `scripts/strip_vulnerability.sh` which physically removes vulnerability code from the `pub_release/` branch. The user confirmed open-source distribution requires source-level removal, not just runtime gating.

| Option | Description | Selected |
|--------|-------------|----------|
| Replace strip script with feature flag in CI | Feature gate is enforcement; strip script deleted | |
| Keep strip script alongside the feature gate | Belt-and-suspenders: both mechanisms active | ✓ |
| You decide | Claude evaluates tradeoffs | |

**User's choice:** Keep strip script alongside the feature gate

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — source must also be absent from the public branch | Strip script stays; source-level privacy required | ✓ |
| Yes — source visibility is fine, just runtime gating needed | Feature flag alone is sufficient | |
| Undecided — defer strip script question | Phase 10 scoped to feature gate only | |

**User's choice:** Source must also be absent from the public branch

| Option | Description | Selected |
|--------|-------------|----------|
| Yes — update strip script in Phase 10 | Phase 10 adds cwe_scanner.rs; strip script updated to match | ✓ |
| No — strip script is Phase 11 or 12 concern | Defer until scanner has content | |

**User's choice:** Yes — update strip script in Phase 10

**Notes:** User raised the open-source question themselves after seeing the initial options. Key insight: the feature flag prevents execution but not source visibility. Both enforcement mechanisms are required for open-source distribution.

---

## Claude's Discretion

- Whether to gate `mod vulnerability;` at module declaration level vs. individual item-level `#[cfg]` guards — Claude picks what compiles cleanest
- Exact `#[cfg_attr]` syntax on the `Dependency.vulnerabilities` field — Claude picks the form with least churn to construction sites

## Deferred Ideas

None — discussion stayed within phase scope.
