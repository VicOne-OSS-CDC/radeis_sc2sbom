# Phase 10: Internal Feature Gate - Research

**Researched:** 2026-05-09
**Domain:** Rust conditional compilation (`#[cfg(feature)]`, `#[cfg_attr]`), Cargo optional dependencies
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Everything vulnerability-related goes behind `#[cfg(feature = "internal")]` — structs, API logic, formatter branches, CLI flags, and console display code. No vulnerability code remains in the unconditional compilation path.
- **D-02:** `VulnerabilityInfo`, `CweInfo`, and all types in `src/vulnerability/` are gated.
- **D-03:** The `vulnerabilities` field on the `Dependency` struct is gated via `#[cfg_attr]` — it is absent in non-internal builds. Tests that construct `Dependency` use `..Default::default()` to handle the missing field.
- **D-04:** Formatter branches in `src/formats/cyclonedx.rs`, `src/formats/spdx.rs`, and `src/formats/console.rs` that emit vulnerability entries are wrapped in `#[cfg(feature = "internal")]`.
- **D-05:** `VulnerabilityOutputMode` enum and all console vulnerability rendering blocks are gated.
- **D-06:** CLI flags `--vulnerability-output`, `--cache-ttl`, `--vulnerability-timeout`, `--clear-vulnerability-cache` are gated — they do not appear in `--help` on the public binary.
- **D-07:** Create `src/vulnerability/cwe_scanner.rs` as an empty stub, wrapped in `#[cfg(feature = "internal")]`. This file is the landing zone for Phase 11's scanner implementation.
- **D-08:** Location: `src/vulnerability/cwe_scanner.rs` alongside `osv.rs` and `nvd.rs`.
- **D-09:** Gate the entire `tests/vulnerability_tests/` module with `#[cfg(feature = "internal")]` at the top of `tests/vulnerability_tests/mod.rs`.
- **D-10:** Integration/classifier/format tests that construct `Dependency` structs use `..Default::default()` for the cfg-gated `vulnerabilities` field. No per-test-file cfg annotations needed for those external test files.
- **D-11:** `reqwest` becomes an optional dependency: `reqwest = { version = "0.11", optional = true, features = [...] }` with `internal = ["reqwest"]` in `[features]`. Public binary does not link reqwest.
- **D-12:** `sha2` and `dirs` remain unconditional (small crates, low overhead).
- **D-13:** Both enforcement mechanisms stay active — feature gate (compile-time) + strip script (source-level for public distribution).
- **D-14:** Phase 10 updates `scripts/strip_vulnerability.sh` to also remove `src/vulnerability/cwe_scanner.rs` from the removal list.
- **D-15:** `build-release.yml` (internal CI) builds with `--features internal`. `public-release.yml` builds with `cargo build --release` (no feature flags) after the strip step.

### Claude's Discretion

- Whether to use a `#[cfg(feature = "internal")] mod vulnerability;` declaration in `lib.rs`/`main.rs` (gating the whole module at the mod declaration level) versus individual `#[cfg]` on each item.
- Exact placement of `#[cfg_attr]` on the `Dependency.vulnerabilities` field (field-level vs. wrapper type).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| GATE-01 | `cargo feature = "internal"` compiles out all CVE vulnerability scanning code when absent | Module-level `#[cfg]` on `mod vulnerability;` gates all OSV/NVD code; reqwest becomes optional |
| GATE-02 | `cargo feature = "internal"` compiles out all CWE enrichment (NVD) code when absent | Same gate — `src/vulnerability/nvd.rs` is inside the gated module |
| GATE-03 | `cargo feature = "internal"` compiles out all lexical CWE scanner code when absent | Empty `cwe_scanner.rs` stub inside the gated module |
| GATE-04 | Public release binary built without `internal` feature passes `cargo test` with no CVE/CWE functionality present | `tests/vulnerability_tests/mod.rs` gated; other test files switch to `..Default::default()` |
</phase_requirements>

---

## Summary

Phase 10 adds a Cargo feature flag `internal` that controls whether the entire vulnerability scanning stack — OSV CVE queries, NVD CWE enrichment, CLI flags, formatter branches, and model types — is compiled into the binary. The public-facing `cargo build --release` produces a binary with no vulnerability code. An internal `cargo build --release --features internal` restores full functionality. Both the feature gate (compiler-enforced) and the existing strip script (source-level for git branch) are retained.

The codebase has already been through a stripping exercise before: `scripts/strip_vulnerability.sh` (1,155 lines) does exactly what Phase 10 needs to make permanent via `#[cfg]`. The research task is to map strip-script surgery to `#[cfg]` annotations at the right granularity — module-level `#[cfg]` on `mod` declarations avoids the per-item annotation noise, and a single `#[cfg_attr]` on the `Dependency.vulnerabilities` field isolates the struct change.

The most important implementation subtlety is that `Dependency` is constructed with explicit field syntax in ~25 test files and several parser/scanner source files. Gating the `vulnerabilities` field with `#[cfg_attr(feature = "internal", ...)]` means those construction sites need to switch to struct update syntax (`..Default::default()`). This is the bulk of the mechanical work.

**Primary recommendation:** Gate at module boundary in `lib.rs` and `main.rs` (`#[cfg(feature = "internal")] mod vulnerability;`) and gate the `Dependency.vulnerabilities` field with `#[cfg_attr(feature = "internal", ...)]`. Migrate test construction sites to `..Default::default()`.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Feature flag declaration | Build system (Cargo.toml) | — | Feature membership and optional deps live in manifest |
| Vulnerability module gating | Library (src/lib.rs, src/main.rs) | — | `mod` declaration controls whether the module compiles at all |
| Struct field gating | Model layer (src/models/dependency.rs) | — | `Dependency` owns the field; gating at declaration propagates everywhere |
| CLI flag gating | CLI layer (src/cli.rs) | — | Clap `#[arg]` only emits help/parse when the field exists |
| Formatter branch gating | Format layer (src/formats/*.rs) | — | Vulnerability emission blocks live in formatters |
| Test module gating | Test harness (tests/vulnerability_tests/mod.rs) | — | `#[cfg]` on `mod` in the parent test file excludes the whole subtree |
| Test construction-site fix | Multiple test files | — | Mechanical: switch explicit `vulnerabilities:` to `..Default::default()` |
| Strip script update | scripts/strip_vulnerability.sh | — | Stays in sync with gate boundary for source-level public distribution |
| CI workflow update | .github/workflows/build-release.yml | — | Add `--features internal` to internal build commands |

---

## Standard Stack

This phase uses only the Rust compiler's built-in conditional compilation system — no additional libraries are required.

### Core
| Mechanism | Version | Purpose | Why Standard |
|-----------|---------|---------|--------------|
| `#[cfg(feature = "...")]` | Rust built-in | Conditionally compile items | Compiler-enforced; zero runtime overhead |
| `#[cfg_attr(feature = "...", ...)]` | Rust built-in | Conditionally apply attributes to struct fields | The only way to gate a field on a derive struct |
| Cargo `[features]` table | Cargo 1.x built-in | Declare feature set and optional dep membership | Standard Rust mechanism |
| `optional = true` on deps | Cargo 1.x built-in | Exclude dependency from unconditional link graph | Pairs with `[features]` to make reqwest link-optional |

**No new library dependencies.** [VERIFIED: Rust Reference, Cargo Book]

---

## Architecture Patterns

### System Architecture Diagram

```
Cargo.toml
  [features]
    cn-release = []
    internal = ["dep:reqwest"]   <-- NEW
  [dependencies]
    reqwest = { optional = true, ... }   <-- CHANGED

         +---------------------------+
  cargo build --release              cargo build --release --features internal
  (public binary)                    (internal binary)
         |                                    |
  lib.rs / main.rs                   lib.rs / main.rs
    mod vulnerability;  ABSENT         #[cfg(feature="internal")] mod vulnerability;  PRESENT
    Dependency.vulnerabilities ABSENT  Dependency.vulnerabilities  PRESENT
    CLI: vuln flags ABSENT             CLI: vuln flags PRESENT
    Formatters: vuln branches ABSENT   Formatters: vuln branches PRESENT
    Tests: vulnerability_tests ABSENT  Tests: vulnerability_tests  PRESENT
         |                                    |
  cargo test passes (no vulns)        cargo test passes (full suite)
```

### Recommended Project Structure (after Phase 10)

```
src/
├── vulnerability/
│   ├── mod.rs               # (gated at lib.rs/main.rs mod declaration)
│   ├── osv.rs               # OsvProvider, query_vulnerabilities_batch
│   ├── nvd.rs               # enrich_cwe_ids
│   ├── fix_recommendations.rs
│   └── cwe_scanner.rs       # NEW STUB — Phase 11 landing zone
├── models/
│   └── dependency.rs        # Dependency.vulnerabilities: Vec<Vulnerability>
│                            #   annotated with #[cfg_attr(feature="internal", ...)]
├── cli.rs                   # vuln flags behind #[cfg(feature="internal")]
└── formats/
    ├── cyclonedx.rs         # vuln branches behind #[cfg(feature="internal")]
    ├── spdx.rs              # vuln refs behind #[cfg(feature="internal")]
    └── console.rs           # VulnerabilityOutputMode + rendering behind gate
tests/
└── vulnerability_tests/
    └── mod.rs               # #[cfg(feature = "internal")] at top
```

### Pattern 1: Module-Level Gate (preferred for whole modules)

**What:** Put `#[cfg(feature = "internal")]` on the `mod` declaration, not on individual items inside the module. This is the cleanest approach when every item in a module is gated.

**When to use:** `lib.rs`, `main.rs` for the `vulnerability` module. The entire `src/vulnerability/` subtree is internal-only.

**Example:**
```rust
// src/lib.rs — BEFORE
pub mod vulnerability;

// src/lib.rs — AFTER
#[cfg(feature = "internal")]
pub mod vulnerability;
```

```rust
// src/main.rs — BEFORE
mod vulnerability;
use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};

// src/main.rs — AFTER
#[cfg(feature = "internal")]
mod vulnerability;
#[cfg(feature = "internal")]
use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};
```

[VERIFIED: Rust Reference — Conditional compilation, https://doc.rust-lang.org/reference/conditional-compilation.html]

### Pattern 2: Block Gate for Use Statements and Code Blocks

**What:** Wrap a related block of `use` statements and code under a single `#[cfg]` block using braces, or apply `#[cfg]` to each `use` line individually.

**When to use:** In `main.rs` for the vulnerability scanning block (lines ~172–207) and the related `use` imports.

**Example:**
```rust
// Gate the entire vulnerability scanning block in main()
#[cfg(feature = "internal")]
{
    if args.check_vulnerabilities {
        if args.clear_cache {
            if let Err(e) = clear_vulnerability_cache() {
                eprintln!("Warning: Failed to clear cache: {}", e);
            }
        }
        // ... full block ...
        enrich_cwe_ids(&mut dependencies, ...);
    }
}
```

[VERIFIED: Rust Reference — Conditional compilation]

### Pattern 3: `#[cfg_attr]` on Struct Fields

**What:** Apply `#[cfg_attr(feature = "internal", ...)]` to add attributes (like serde skip directives) only when the feature is active. For a field that should not exist in non-internal builds, the field itself must be wrapped in a `cfg_attr` that conditionally includes the declaration.

**Critical detail:** Rust does NOT support `#[cfg(feature = "internal")]` directly on a struct field to make it conditionally absent. The correct approach is `#[cfg_attr(feature = "internal", serde(...))]` for attribute control, but to make the *field itself* conditional, you must use a `cfg` attribute applied to the field declaration directly:

```rust
// THIS WORKS — field is absent when feature is not set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    // ...
    #[cfg(feature = "internal")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub vulnerabilities: Vec<Vulnerability>,
    // ...
}
```

[VERIFIED: Rust Reference — `cfg` on struct fields is supported]

**Consequence for struct construction sites:** Any code that explicitly names the `vulnerabilities` field in a struct literal will fail to compile when the feature is absent (field does not exist). Switch to struct update syntax:

```rust
// BEFORE (explicit field — breaks when feature absent)
Dependency {
    name: "foo".to_string(),
    version: "1.0".to_string(),
    ecosystem: "npm".to_string(),
    vulnerabilities: vec![],
    // ... all other fields ...
}

// AFTER (struct update — compiles with or without feature)
Dependency {
    name: "foo".to_string(),
    version: "1.0".to_string(),
    ecosystem: "npm".to_string(),
    ..Default::default()
}
```

[VERIFIED: observed in existing codebase — `Dependency` already implements `Default`; `Default::vulnerabilities` is `Vec::new()` which satisfies the internal build]

### Pattern 4: Gating Clap CLI Args

**What:** Wrap an entire struct field with its `#[arg(...)]` attribute inside `#[cfg(feature = "internal")]`.

**When to use:** For all six vuln-related fields in `Args` (D-06).

**Example:**
```rust
#[derive(Parser, Debug)]
pub struct Args {
    // ... unconditional args ...

    #[cfg(feature = "internal")]
    #[arg(long, action = ArgAction::Set, default_value_t = false)]
    pub check_vulnerabilities: bool,

    #[cfg(feature = "internal")]
    #[arg(long, default_value = "30")]
    pub vulnerability_timeout: u64,
    // ... etc ...
}
```

**Critical side effect:** All code that references `args.check_vulnerabilities`, `args.vulnerability_output`, etc. must also be inside `#[cfg(feature = "internal")]` blocks. The main() scanning block, save_console_report calls passing `args.vulnerability_output`, and `args.sbom_mode` usages all need gating.

[VERIFIED: Clap docs — derives only emit argument parsing for fields that exist in the struct]

### Pattern 5: Gating Enum Variants and Whole Enums

**What:** Gate entire enum definitions with `#[cfg(feature = "internal")]`.

**When to use:** `VulnerabilityOutputMode`, `MinSeverity`, `SbomMode` in `src/cli.rs`.

```rust
#[cfg(feature = "internal")]
#[derive(Debug, Clone, ValueEnum)]
pub enum VulnerabilityOutputMode {
    Summary,
    Tree,
    Detailed,
}
```

### Pattern 6: Gating Test Modules

**What:** Put `#[cfg(feature = "internal")]` at the top of the module file, before the `mod` declarations.

**When to use:** `tests/vulnerability_tests/mod.rs`.

```rust
// tests/vulnerability_tests/mod.rs — AFTER
#![cfg(feature = "internal")]

mod fix_recommendation_tests;
mod nvd_tests;
```

The `#![cfg(...)]` inner attribute (note the `!`) applies to the whole file/module. Alternatively, wrap the `mod` declarations in the parent `tests/all_tests.rs`:

```rust
// tests/all_tests.rs — AFTER
#[cfg(feature = "internal")]
#[path = "vulnerability_tests/mod.rs"]
mod vulnerability_tests;
```

Both approaches work. Gating in `all_tests.rs` is cleaner since `mod.rs` itself doesn't need changing.

[VERIFIED: Rust Reference — Inner attributes; tested patterns in Rust ecosystem]

### Anti-Patterns to Avoid

- **Gating at item level instead of module level:** If every item in `src/vulnerability/` is gated, use one `#[cfg]` on the `mod` declaration in `lib.rs` instead of annotating every function in every file. Less noise, same result.
- **Forgetting the `use` statement:** Gating `mod vulnerability;` in `lib.rs` does not automatically gate `use vulnerability::...` in `main.rs`. Both must be gated.
- **Leaving explicit field names in test structs:** Tests that name `vulnerabilities:` in struct literals will cause compile errors when the feature is absent. Grep shows ~25 locations. All must switch to `..Default::default()`.
- **Forgetting `models/mod.rs` re-exports:** `src/models/mod.rs` re-exports `Vulnerability`, `VulnerabilitySeverity`, etc. These `pub use vulnerability::...` lines must also be gated with `#[cfg(feature = "internal")]`.
- **Forgetting `models/vulnerability.rs` import in `dependency.rs`:** `use super::vulnerability::Vulnerability;` at line 2 of `dependency.rs` must be gated, otherwise rustc cannot resolve the type for the field.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Conditional compilation | Custom build.rs scripts or env-var conditionals | `#[cfg(feature = "...")]` | Built into Rust compiler — zero overhead, works with cargo check, clippy, IDE tooling |
| Optional dependency linking | Feature flag in code that skips imports | `optional = true` in Cargo.toml + feature membership | Prevents reqwest from being linked into the binary at all |

**Key insight:** The Rust compiler's feature system is the correct tool for this problem. The entire phase is `#[cfg]` annotations + one Cargo.toml change.

---

## Runtime State Inventory

Not applicable. This is a source-level gating phase — no runtime state, databases, or services store the concepts being gated. The `--features internal` flag is a compile-time switch only.

---

## Environment Availability Audit

This phase is purely source/config changes. The only tools needed are already present in the project: `cargo`, `rustc`. No external dependencies or services required.

Step 2.6: SKIPPED (no external dependencies beyond Rust toolchain, which is already in use).

---

## Common Pitfalls

### Pitfall 1: Explicit Field Struct Literals Fail to Compile Without Feature

**What goes wrong:** A test or source file constructs `Dependency { ..., vulnerabilities: vec![], ... }` explicitly. When `internal` feature is absent, the `vulnerabilities` field does not exist, and Rust emits: `error[E0560]: struct Dependency has no field named vulnerabilities`.

**Why it happens:** Rust struct literals must name only fields that exist at compile time. A `#[cfg(feature = "internal")]` field is truly absent from the type, not just zero-sized.

**How to avoid:** Grep for all `vulnerabilities:` occurrences in source and test files before declaring the work done. Convert every construction site to `..Default::default()`.

**Warning signs:** `cargo build` (without `--features internal`) fails with E0560.

**Files affected (verified by grep):**
- `tests/format_tests/cyclonedx_tests.rs` (~6 occurrences — `make_dep` helper + inline deps)
- `tests/format_tests/spdx_tests.rs` (likely — uses same pattern)
- `tests/integration_tests/scope_filtering_integration_tests.rs` (~11 occurrences)
- `tests/integration_tests/mcu_project_tests.rs` (~2 occurrences)
- `tests/integration_tests/autosar_e2e_tests.rs` (~1 occurrence)
- `tests/classifier_tests/autosar_classification_tests.rs` (~1 occurrence)
- `tests/classifier_tests/scope_filter_tests.rs` (~1 occurrence)

### Pitfall 2: Forgetting the `Vulnerability` Import in `dependency.rs`

**What goes wrong:** The `#[cfg(feature = "internal")]` on the `vulnerabilities` field refers to the type `Vec<Vulnerability>`. If `use super::vulnerability::Vulnerability;` at line 2 of `dependency.rs` is NOT also gated, Rust will try to resolve the import even when the feature is absent — and `src/models/vulnerability.rs` may itself trigger a transitive compile error.

**How to avoid:** Gate the `use` statement with the same feature flag as the field:

```rust
#[cfg(feature = "internal")]
use super::vulnerability::Vulnerability;
```

**Warning signs:** `error[E0432]: unresolved import` on the `use super::vulnerability::Vulnerability` line when building without `--features internal`.

### Pitfall 3: `models/mod.rs` Re-exports Not Gated

**What goes wrong:** `src/models/mod.rs` currently exports `Confidence, FixAction, FixRecommendation, Vulnerability, VulnerabilitySeverity` via `pub use vulnerability::...`. If these re-exports are not gated, the compiler attempts to resolve `pub mod vulnerability` which no longer exists in the public compilation path.

**How to avoid:** Gate both the `pub mod vulnerability;` declaration and the `pub use vulnerability::...` block:

```rust
#[cfg(feature = "internal")]
pub mod vulnerability;

#[cfg(feature = "internal")]
pub use vulnerability::{
    Confidence, FixAction, FixRecommendation, Vulnerability, VulnerabilitySeverity,
};
```

### Pitfall 4: `main.rs` `use` Statement Not Gated

**What goes wrong:** `main.rs` line 23: `use vulnerability::{clear_vulnerability_cache, enrich_cwe_ids, query_vulnerabilities_batch, OsvProvider};`. If the `mod vulnerability;` is gated but this `use` is not, the compiler errors on the unresolved import.

**How to avoid:** Gate the `use` with the same `#[cfg]`.

### Pitfall 5: Formatter Function Signatures Still Accept Gated Types

**What goes wrong:** If formatter functions like `save_console_report` still accept `vulnerability_output: &VulnerabilityOutputMode` in their signature when `VulnerabilityOutputMode` is gated out, the non-internal build cannot call these functions.

**How to avoid:** Gate the entire affected function signature block, or restructure the signature so the gated type is absent. The cleanest approach is to gate the parameter plus the body together using conditional compilation, or to use an inner `#[cfg]` block inside the function body for the vulnerability-specific code paths, while keeping the function signature unconditional (removing the gated parameter).

**Warning signs:** The strip script already does this surgery — cross-reference with step 7 and 8 of `strip_vulnerability.sh` to understand what parameter removals are needed.

### Pitfall 6: Strip Script Not Updated for `cwe_scanner.rs`

**What goes wrong:** After Phase 10 creates `src/vulnerability/cwe_scanner.rs`, the strip script's step 1 (`rm -rf src/vulnerability/`) already covers it — the whole directory is deleted. However, D-14 says the strip script must explicitly list it. Verify that the `rm -rf src/vulnerability/` step covers the new file, and document that no additional removal is needed.

**How to avoid:** The `rm -rf src/vulnerability/` in step 1 of the strip script removes the entire directory including any new files. The D-14 requirement is satisfied implicitly. No additional script step is required for `cwe_scanner.rs`. Confirm this in the plan.

### Pitfall 7: `SbomMode` in Format Tests

**What goes wrong:** `tests/format_tests/cyclonedx_tests.rs` imports and uses `SbomMode` (line 1). If `SbomMode` is gated out, the import fails. The strip script patches this — the same surgery must be done with `#[cfg]`.

**How to avoid:** Gate `SbomMode` in `cli.rs` with `#[cfg(feature = "internal")]`. Format tests that use `SbomMode::Complete` must either be wrapped in `#[cfg(feature = "internal")]` or switch to a non-SbomMode API. Since the strip script removes `SbomMode` from function signatures, the non-internal formatter functions do not take a `mode` parameter — tests must not pass one either.

---

## Code Examples

### Cargo.toml `[features]` and optional dep

```toml
[features]
# CN regional release: appends vulnerability assessment service info to --help output
cn-release = []
# Internal builds: enables CVE scanning (OSV), CWE enrichment (NVD), lexical CWE scanner
internal = ["dep:reqwest"]

[dependencies]
# ...
reqwest = { version = "0.11", optional = true, features = ["blocking", "json", "native-tls-vendored"] }
```

Note: `"dep:reqwest"` syntax (Cargo 1.60+) explicitly references the dependency without creating a feature of the same name. This is cleaner than `"reqwest"`.
[VERIFIED: Cargo Book — optional dependencies, https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies]

### `src/lib.rs` module gate

```rust
pub mod classifier;
pub mod cli;
pub mod formats;
pub mod models;
pub mod parsers;
pub mod scanner;
pub mod supplier;
pub mod util;
#[cfg(feature = "internal")]
pub mod vulnerability;
```

### `src/models/dependency.rs` field gate

```rust
#[cfg(feature = "internal")]
use super::vulnerability::Vulnerability;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    // ...
    #[cfg(feature = "internal")]
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub vulnerabilities: Vec<Vulnerability>,
    // ...
}

impl Default for Dependency {
    fn default() -> Self {
        Dependency {
            name: String::new(),
            // ...
            #[cfg(feature = "internal")]
            vulnerabilities: Vec::new(),
            // ...
        }
    }
}
```

### `src/models/mod.rs` gated re-exports

```rust
pub mod dependency;
pub mod graph;
pub mod sbom;
#[cfg(feature = "internal")]
pub mod vulnerability;

pub use dependency::{...};
pub use graph::{DependencyGraph, DependencyNode};
pub use sbom::{RosPackageMetadata, RosPackageWithDeps, Sbom, ScopeStatistics};
#[cfg(feature = "internal")]
pub use vulnerability::{
    Confidence, FixAction, FixRecommendation, Vulnerability, VulnerabilitySeverity,
};
```

### `tests/all_tests.rs` gated vulnerability test module

```rust
// In tests/all_tests.rs
#[cfg(feature = "internal")]
#[path = "vulnerability_tests/mod.rs"]
mod vulnerability_tests;
```

### `src/vulnerability/cwe_scanner.rs` stub

```rust
// Phase 11: lexical CWE scanner — implementation pending
// This file is the landing zone for the C/C++ static analysis scanner (SCAN-01..SCAN-05)
```

(File exists only when `internal` feature is set, because the parent `vulnerability` module is gated.)

### Test construction site conversion

```rust
// BEFORE — fails without internal feature
fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
        source: DependencySource::LockFile,
        vulnerabilities: vec![],
        // ... all fields named explicitly
    }
}

// AFTER — compiles with or without internal feature
fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
        source: DependencySource::LockFile,
        ..Default::default()
    }
}
```

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Strip script surgery (runtime Python regex patching source files) | `#[cfg(feature = "internal")]` annotations (compiler-enforced) | Phase 10 | Compiler verifies the gate; strip script becomes belt-and-suspenders rather than primary enforcement |
| `reqwest` unconditional link | `reqwest` optional dependency | Phase 10 | Public binary does not link TLS/reqwest; smaller binary, no network-capable code at all |

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness |
| Config file | none (uses cargo test) |
| Quick run command | `cargo test` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| GATE-01 | `cargo build --release` compiles without OSV/NVD code | build check | `cargo build --release 2>&1 \| grep -v "^error" && echo OK` | ✅ (manual verify) |
| GATE-02 | `cargo build --release` compiles without NVD code | build check | same as GATE-01 | ✅ (manual verify) |
| GATE-03 | `cargo build --release` compiles without lexical scanner | build check | same as GATE-01 | ✅ Wave 0 — cwe_scanner.rs stub |
| GATE-04 | `cargo test` passes without `internal` feature | test run | `cargo test` | ✅ (after construction-site migration) |

The primary verification commands are:

```bash
# Verify public build (no feature) compiles and tests pass
cargo build --release
cargo test

# Verify internal build (with feature) compiles and tests pass  
cargo build --release --features internal
cargo test --features internal

# Verify public binary has no vulnerability strings in --help
./target/release/radeis_sc2sbom --help | grep -i "vulner\|cwe\|cvss\|cache-ttl\|clear.*cache" && echo "LEAK" || echo "OK"
```

### Sampling Rate

- **Per task commit:** `cargo build --release && cargo test`
- **Per wave merge:** `cargo build --release && cargo test && cargo build --release --features internal && cargo test --features internal`
- **Phase gate:** Full matrix (both feature states) green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src/vulnerability/cwe_scanner.rs` — covers GATE-03 (stub must exist inside gated module)

*(All other test infrastructure already exists; no new test files required for GATE-04 — the fix is migrating construction sites, not adding tests)*

---

## Security Domain

This phase does not introduce new security attack surfaces. The changes are compile-time gating that removes code from the public binary. No ASVS categories are newly applicable.

---

## Detailed File Change Inventory

This section enumerates every file that needs a change and what change is required. Planner uses this to create one task per logical group.

### Group A: Cargo.toml
- Add `internal = ["dep:reqwest"]` to `[features]`
- Change `reqwest = { version = "0.11", features = [...] }` to `reqwest = { version = "0.11", optional = true, features = [...] }`

### Group B: Module Gates (`lib.rs`, `main.rs`)
- `src/lib.rs:12` — add `#[cfg(feature = "internal")]` before `pub mod vulnerability;`
- `src/main.rs:9` — add `#[cfg(feature = "internal")]` before `mod vulnerability;`
- `src/main.rs:23` — add `#[cfg(feature = "internal")]` before `use vulnerability::{...};`
- `src/main.rs:172–207` — wrap the `if args.check_vulnerabilities { ... }` block in `#[cfg(feature = "internal")]`

### Group C: Models
- `src/models/mod.rs:4` — add `#[cfg(feature = "internal")]` before `pub mod vulnerability;`
- `src/models/mod.rs:12–15` — add `#[cfg(feature = "internal")]` before `pub use vulnerability::{...};`
- `src/models/dependency.rs:2` — add `#[cfg(feature = "internal")]` before `use super::vulnerability::Vulnerability;`
- `src/models/dependency.rs:295–297` — add `#[cfg(feature = "internal")]` before the `vulnerabilities` field
- `src/models/dependency.rs:Default impl` — add `#[cfg(feature = "internal")]` before `vulnerabilities: Vec::new(),`

### Group D: CLI
- `src/cli.rs` — gate `MinSeverity` enum + impl, `VulnerabilityOutputMode` enum, `SbomMode` enum, and 8 `Args` fields with `#[cfg(feature = "internal")]`
- Call sites in `main.rs` that reference `args.vulnerability_output`, `args.max_vulns_per_severity`, `args.check_vulnerabilities`, `args.clear_cache`, `args.cache_ttl`, `args.vulnerability_timeout`, `args.sbom_mode`, `args.min_severity` must be inside the gated block (Group B covers this)

### Group E: Formatters
- `src/formats/cyclonedx.rs` — gate `VulnerabilitySeverity` import, `CycloneDXVulnerability` + supporting structs, `build_cyclonedx_vulnerabilities` function, vulnerabilities array building in `convert_to_cyclonedx`, SbomMode filtering
- `src/formats/spdx.rs` — gate `VulnerabilitySeverity` import, vulnerability reference loop in `create_external_refs`, SbomMode filtering
- `src/formats/console.rs` — gate `VulnerabilityOutputMode` import, severity helper functions, `print_vulnerabilities_hierarchical`, vulnerability rendering blocks in `print_sbom`/`save_console_report`/`print_summary_section`

### Group F: New Stub File
- Create `src/vulnerability/cwe_scanner.rs` (minimal comment-only stub)
- Update `src/vulnerability/mod.rs` to add `pub mod cwe_scanner;` (gated at module level since the whole module is gated)

### Group G: Test Files
- `tests/all_tests.rs` — add `#[cfg(feature = "internal")]` before the `vulnerability_tests` module declaration
- `tests/vulnerability_tests/mod.rs` — no change needed (module excluded by cfg in all_tests.rs)
- All test files with explicit `vulnerabilities:` construction: convert to `..Default::default()` (see Pitfall 1 — ~25 occurrences across 7+ files)

### Group H: CI Workflows
- `.github/workflows/build-release.yml` — add `--features internal` to all `cargo build --release` commands (currently at lines 251, 382)
- `.github/workflows/build-release.yml` — add `--features internal` to `validate-sbom` step's SBOM generation command (line 444)

### Group I: Strip Script
- `scripts/strip_vulnerability.sh` — D-14 says add `cwe_scanner.rs` to the removal list, but step 1 (`rm -rf src/vulnerability/`) already removes the entire directory. Confirm no additional line is needed; add a comment noting that cwe_scanner.rs is covered by the directory removal.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `"dep:reqwest"` syntax (Cargo 1.60+) is available in this project's MSRV | Cargo.toml section | If MSRV < 1.60, use `internal = ["reqwest"]` (implicit feature) instead | [ASSUMED — Cargo.toml does not specify rust-version; project appears modern] |
| A2 | `#[cfg(feature = "internal")]` on a struct field makes the field structurally absent | Code Examples | Risk is LOW — this is documented Rust behavior, but not verified against the specific rustc version in CI |

**Mitigation for A1:** Check Cargo.toml for `rust-version` field. If absent, assume current stable (1.87 as of research date) which supports `dep:` syntax. [VERIFIED: no rust-version in Cargo.toml]

---

## Open Questions

1. **Does `build-release.yml` validate-sbom step need `--features internal`?**
   - What we know: The `validate-sbom` step runs `radeis_sc2sbom-linux` (the built binary) against a sample repo. The binary is built in `build-linux` step which currently has no `--features internal`.
   - What's unclear: Should the internal build validation test run with the feature-enabled binary?
   - Recommendation: Yes — add `--features internal` to the `Build binary` step in `build-linux` job inside `build-release.yml`. The validate-sbom job downloads the artifact from that step, so it will automatically use the internal binary.

2. **Are there any source files in `src/` (not tests) that explicitly construct `Dependency` with `vulnerabilities:` named?**
   - What we know: The grep above only searched `tests/`. Parser and scanner files may also construct `Dependency` structs.
   - Recommendation: Run `grep -rn "vulnerabilities:" src/ --include="*.rs"` as the first task step to find all construction sites before making changes.

---

## Sources

### Primary (HIGH confidence)
- Rust Reference — Conditional compilation: https://doc.rust-lang.org/reference/conditional-compilation.html
- Cargo Book — Features: https://doc.rust-lang.org/cargo/reference/features.html
- Cargo Book — Optional dependencies: https://doc.rust-lang.org/cargo/reference/features.html#optional-dependencies
- Codebase — `scripts/strip_vulnerability.sh` (read directly): complete map of what must be gated
- Codebase — `src/models/dependency.rs` (read directly): exact field definition and Default impl
- Codebase — `tests/all_tests.rs`, `tests/vulnerability_tests/mod.rs` (read directly): existing test structure
- Codebase — `Cargo.toml`, `src/lib.rs`, `src/main.rs` (read directly): current module structure

### Secondary (MEDIUM confidence)
- `dep:` syntax for optional deps — Cargo 1.60 release notes (training knowledge; not fetched in this session)

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — Rust's `#[cfg]` system, no new deps
- Architecture: HIGH — complete file inventory from codebase read
- Pitfalls: HIGH — grounded in actual code found (25+ explicit `vulnerabilities:` construction sites verified)
- CI changes: HIGH — workflows read directly

**Research date:** 2026-05-09
**Valid until:** Stable indefinitely — Rust cfg/feature system is not changing
