---
phase: 10-internal-feature-gate
reviewed: 2026-05-09T00:00:00Z
depth: standard
files_reviewed: 57
files_reviewed_list:
  - .github/workflows/build-release.yml
  - Cargo.toml
  - scripts/strip_vulnerability.sh
  - src/classifier/ecosystem.rs
  - src/classifier/mod.rs
  - src/classifier/rules.rs
  - src/cli.rs
  - src/formats/console.rs
  - src/formats/cyclonedx.rs
  - src/formats/spdx.rs
  - src/lib.rs
  - src/main.rs
  - src/models/dependency.rs
  - src/models/mod.rs
  - src/parsers/bazel/module.rs
  - src/parsers/bazel/workspace.rs
  - src/parsers/c/autotools.rs
  - src/parsers/c/library_json.rs
  - src/parsers/c/makefile.rs
  - src/parsers/c/makefile_am.rs
  - src/parsers/c/mk_file.rs
  - src/parsers/c/pkgconfig.rs
  - src/parsers/c/pkgconfig_detector.rs
  - src/parsers/c/vendored_3rdparty.rs
  - src/parsers/cargo.rs
  - src/parsers/cmake/external_project.rs
  - src/parsers/cmake/fetchcontent.rs
  - src/parsers/cpp/conan.rs
  - src/parsers/cpp/conan_manifest.rs
  - src/parsers/cpp/vcpkg.rs
  - src/parsers/gguf.rs
  - src/parsers/go.rs
  - src/parsers/java.rs
  - src/parsers/meson/meson_build.rs
  - src/parsers/meson/wrap.rs
  - src/parsers/npm.rs
  - src/parsers/php.rs
  - src/parsers/python.rs
  - src/parsers/ros.rs
  - src/parsers/ruby.rs
  - src/parsers/safetensors.rs
  - src/parsers/source_scanner.rs
  - src/scanner/mod.rs
  - src/vulnerability/cwe_scanner.rs
  - src/vulnerability/mod.rs
  - tests/all_tests.rs
  - tests/classifier_tests/autosar_classification_tests.rs
  - tests/classifier_tests/scope_filter_tests.rs
  - tests/format_tests/cyclonedx_tests.rs
  - tests/format_tests/spdx_tests.rs
  - tests/integration_tests/autosar_e2e_tests.rs
  - tests/integration_tests/mcu_project_tests.rs
  - tests/integration_tests/production_mode_e2e_tests.rs
  - tests/integration_tests/scope_filtering_integration_tests.rs
  - tests/model_tests/dependency_tests.rs
  - tests/model_tests/sbom_tests.rs
  - tests/parser_tests/c_tests.rs
  - tests/parser_tests/conan_tests.rs
  - tests/parser_tests/ros_tests.rs
  - tests/parser_tests/safetensors_tests.rs
  - tests/scanner_tests/deduplication_tests.rs
findings:
  critical: 3
  warning: 5
  info: 2
  total: 10
status: issues_found
---

# Phase 10: Code Review Report

**Reviewed:** 2026-05-09T00:00:00Z
**Depth:** standard
**Files Reviewed:** 57
**Status:** issues_found

## Summary

This review covers the Phase 10 internal feature-gate migration. The primary goal was to
replace file-deletion–based public builds with a `#[cfg(feature = "internal")]` compile-time
gate, allowing a single source tree to produce both internal (CVE/CWE-enabled) and public
(stripped) binaries without running `strip_vulnerability.sh`.

The gate annotations on the Rust source side are structurally correct and the `Default`
migration is complete. Three blocking issues were found: (1) the format-test and several
integration-test files carry `#![cfg(feature = "internal")]` at the crate level, meaning
they are completely invisible to the public (no-feature) build — the public build therefore
has zero test coverage of the `convert_to_cyclonedx`, `convert_to_spdx`, and several
integration scenarios; (2) the `strip_vulnerability.sh` script's lib.rs/models/mod.rs patches
remove the bare `pub mod vulnerability;` lines but the file now contains
`#[cfg(feature = "internal")] pub mod vulnerability;` — the regex will silently fail to
match and leave a dangling cfg-guarded declaration in the stripped tree, causing a build
error on the public branch; (3) the CI workflow builds every artifact with
`--features internal`, meaning the public (no-feature) binary is never compiled or validated
in CI.

---

## Critical Issues

### CR-01: Format and integration tests entirely skipped in public (no-feature) build

**File:** `tests/format_tests/cyclonedx_tests.rs:1`, `tests/format_tests/spdx_tests.rs:1`,
`tests/integration_tests/production_mode_e2e_tests.rs:1`,
`tests/parser_tests/safetensors_tests.rs:1`

**Issue:** Every one of these test files begins with `#![cfg(feature = "internal")]` at the
crate level. This attribute disables the *entire* file when the `internal` feature is not
active. As a result, `cargo test` (the default, no-feature invocation) executes **zero tests**
for `cyclonedx_tests`, `spdx_tests`, `production_mode_e2e_tests`, and `safetensors_tests`.

The functions under test — `convert_to_cyclonedx`, `convert_to_spdx`, `print_spdx_json`,
`save_spdx_json`, `save_spdx_tag_value`, `print_cyclonedx_json`, `save_cyclonedx_json` — all
exist and are callable in the public build (they are not gated). Regressions in these paths
will not be caught before a public release.

**Fix:** Remove the file-level `#![cfg(feature = "internal")]` gate from each test file.
Restrict only the individual test functions (or their `SbomMode`/`Vulnerability` helper
calls) with item-level `#[cfg(feature = "internal")]` attributes. For the
`convert_to_cyclonedx` / `convert_to_spdx` call sites in these tests that accept a
`mode: &SbomMode` argument, wrap only that argument with `#[cfg]`:

```rust
// Before (entire file disabled):
#![cfg(feature = "internal")]

// After (only the internal-specific argument is gated):
#[test]
fn test_convert_to_cyclonedx_basic() {
    let cdx_doc = convert_to_cyclonedx(
        &sbom,
        #[cfg(feature = "internal")] &SbomMode::Complete,
        None,
    );
    // ...
}
```

Alternatively, provide a non-internal helper that calls the function without the
`SbomMode` argument, identical to how `main.rs` invokes it in the public build.

---

### CR-02: `strip_vulnerability.sh` regex fails silently on the now-gated `pub mod vulnerability;` lines

**File:** `scripts/strip_vulnerability.sh:89`, `scripts/strip_vulnerability.sh:109`

**Issue:** After Phase 10, `src/lib.rs` and `src/models/mod.rs` no longer contain bare
`pub mod vulnerability;` lines. Instead they contain:

```rust
// src/lib.rs line 12-13:
#[cfg(feature = "internal")]
pub mod vulnerability;

// src/models/mod.rs line 4-5:
#[cfg(feature = "internal")]
pub mod vulnerability;
```

The strip script uses these exact regex patterns:

```python
# lib.rs (line 89):
src = re.sub(r'^pub mod vulnerability;\n', '', src, flags=re.MULTILINE)

# models/mod.rs (line 109-110):
src = re.sub(r'^pub mod vulnerability;\n', '', src, flags=re.MULTILINE)
src = re.sub(r'^pub use vulnerability::\{[^}]+\};\n', '', src, flags=re.MULTILINE)
```

These patterns match only the bare `pub mod vulnerability;` line. The `#[cfg(feature = "internal")]` attribute line immediately preceding it prevents the regex from matching. The script will exit 0 (success — there is no `subn` assertion here), but the stripped tree will still contain the two-line cfg-guarded block, which then fails to compile because `src/vulnerability/` has been deleted in step 1.

**Fix:** Update the regexes to optionally match the cfg attribute line:

```python
# lib.rs
src = re.sub(
    r'#\[cfg\(feature = "internal"\)\]\npub mod vulnerability;\n',
    '',
    src,
    flags=re.MULTILINE,
)

# models/mod.rs — mod declaration
src = re.sub(
    r'#\[cfg\(feature = "internal"\)\]\npub mod vulnerability;\n',
    '',
    src,
    flags=re.MULTILINE,
)
# models/mod.rs — re-export block
src = re.sub(
    r'#\[cfg\(feature = "internal"\)\]\npub use vulnerability::\{[^}]+\};\n',
    '',
    src,
    flags=re.MULTILINE,
    flags=re.DOTALL,
)
```

Add `subn(count=1)` assertions (as done in sections 8b, 8b-8d) so any future mismatch fails
loudly instead of producing a silently broken stripped tree.

---

### CR-03: CI never compiles or tests the public (no-feature) build

**File:** `.github/workflows/build-release.yml:251`, `.github/workflows/build-release.yml:382`

**Issue:** Every `cargo build` call in CI uses `--features internal`:

```yaml
# build-macos job, line 251:
cargo build --release --features internal --target ${{ matrix.target }}

# build-linux job, line 382:
cargo build --release --features internal --target ${{ matrix.target }}
```

There is no job that builds or tests `cargo build --release` (no features) or
`cargo test` (no features). The public binary — the product actually shipped to customers
via `strip_vulnerability.sh` — is never compiled in CI. Compilation errors introduced in the
public code paths (e.g. mismatched signatures after stripping) will only surface after the
fact.

**Fix:** Add a `build-public` CI job that runs `cargo build --release` and
`cargo test` without `--features internal` on a Linux runner. This validates that the gated
code compiles cleanly in the public configuration:

```yaml
build-public:
  name: Build and Test Public (no-feature)
  runs-on:
    group: Default
  steps:
    - uses: actions/checkout@v4
      with:
        token: ${{ secrets.GIT_TOKEN }}
    - uses: dtolnay/rust-toolchain@stable
    - name: Build public binary
      run: cargo build --release
    - name: Run public tests
      run: cargo test
```

---

## Warnings

### WR-01: `Dependency` struct construction in test files uses explicit field syntax alongside `..Default::default()`, but still lists the `#[cfg(feature = "internal")]`-gated `vulnerabilities` field

**File:** `tests/format_tests/cyclonedx_tests.rs:15-37`, `tests/format_tests/spdx_tests.rs:12-35`,
`tests/model_tests/dependency_tests.rs:7-28`, `tests/scanner_tests/deduplication_tests.rs:7-50`

**Issue:** After the `..Default::default()` migration, test helper functions like `make_dep`
and `test_dependency_struct` no longer list `vulnerabilities:` explicitly — which is correct.
However, the explicit struct construction blocks still name many optional fields (e.g.,
`checksum_sha256: None, checksum_sha512: None, license: None, author: None, ...`) that are
already provided by `Default`. This redundancy means that if a new field is added to
`Dependency` without a `Default` impl, the tests will fail to compile with an opaque
"missing field" error rather than a clear message that `..Default::default()` should cover it.

The immediate concern is that these sites are inconsistent: some fields are explicitly set to
`None` while `..Default::default()` is also present. If a field is listed explicitly *and*
covered by Default, the explicit value wins — but if the explicit value differs from Default
(which it does not here), it silently overrides Default, which can mask bugs.

**Fix:** For test helper functions that are constructing a fully-default `Dependency` and then
overriding only `name`, `version`, `ecosystem`, use the provided `Dependency::new` constructor
or list only the non-default fields:

```rust
fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency::new(name.to_string(), version.to_string(), ecosystem.to_string())
        .with_source(DependencySource::LockFile)
}
```

---

### WR-02: `src/vulnerability/cwe_scanner.rs` is a stub with no implementation but is declared `pub` in `mod.rs`

**File:** `src/vulnerability/cwe_scanner.rs:1-2`, `src/vulnerability/mod.rs:1`

**Issue:** `cwe_scanner.rs` contains only a two-line comment:

```rust
// Phase 11: lexical CWE scanner — implementation pending
// This file is the landing zone for the C/C++ static analysis scanner (SCAN-01..SCAN-05)
```

`mod.rs` declares `pub mod cwe_scanner;` unconditionally. The entire `vulnerability` module
is gated with `#[cfg(feature = "internal")]`, so the stub compiles. However, `mod.rs` also
exports `pub use osv::{clear_vulnerability_cache, query_vulnerabilities_batch, OsvProvider};`
and `pub use nvd::enrich_cwe_ids;` — but it exports *nothing* from `cwe_scanner`. This means
the stub module is visible to callers of the `vulnerability` crate but provides no public API,
which is confusing for reviewers and tooling.

Additionally, the `strip_vulnerability.sh` comment at line 24 says "cwe_scanner.rs is covered
by the directory removal above — no separate rm command needed." This is correct for the
strip path, but the empty module declaration in `pub mod cwe_scanner;` is misleading: it
suggests public API that does not exist.

**Fix:** Either mark the module private (`mod cwe_scanner;`) until Phase 11 is implemented,
or add a `#[allow(dead_code)]` comment noting the intended Phase 11 integration point. Do not
export an empty public module.

---

### WR-03: `strip_vulnerability.sh` section [4] (lib.rs) removes `pub mod vulnerability;` but leaves the `#[cfg(feature = "internal")]` cfg annotations on `main.rs` imports intact — if the `internal` feature is not declared in the stripped `Cargo.toml`, those cfg annotations become permanently dead code

**File:** `scripts/strip_vulnerability.sh:43-76` (main.rs patch), `scripts/strip_vulnerability.sh:854-917` ([features] section removal)

**Issue:** For global builds, section 8b removes the entire `[features]` block from
`Cargo.toml`. This correctly eliminates the `internal` feature declaration. However, the
main.rs patch in section 3 (lines 43-76) removes the `use vulnerability::{...};` import and
the "Phase 3" block using regex, but it does **not** remove the `#[cfg(feature = "internal")]`
annotations on the function call arguments inside `match args.format { ... }`. After stripping,
`main.rs` will still contain:

```rust
save_spdx_json(&sbom, &out_path_str, #[cfg(feature = "internal")] &args.sbom_mode, ...)?;
```

With the `[features]` section removed, the `internal` feature is unknown. Rust treats an
unknown feature in `#[cfg(feature = "X")]` as always-false without error (it is not an error
in Rust — unknown features evaluate to false silently). This means the stripped public build
compiles, but the argument position is wrong: `#[cfg(feature = "internal")] &args.sbom_mode`
evaluates to nothing (the argument is omitted), which matches the no-feature signature. So it
*happens* to work for the current function signatures, but only because the stripped and
no-feature signatures are identical. If a future refactor changes parameter order, this
implicit no-op will become a hard-to-diagnose type error.

**Fix:** The main.rs section 8e patch (lines 785-808) should also remove the
`#[cfg(feature = "internal")]` attribute wrappers entirely, not just the argument values they
sometimes guard. After stripping, call sites should be clean call expressions with no cfg
attributes. Alternatively, the strip script should validate that the stripped `main.rs`
contains no remaining `#[cfg(feature =` occurrences.

---

### WR-04: The `validate-sbom` CI job validates the `internal`-feature binary against the rclcpp example, but does not validate the stripped (no-feature) binary's SPDX/CycloneDX output

**File:** `.github/workflows/build-release.yml:398-472`

**Issue:** The `validate-sbom` job downloads the Linux artifact (built `--features internal`)
and validates its SPDX and CycloneDX output. This confirms the internal binary produces valid
SBOM output, but the public binary — produced by running `strip_vulnerability.sh` followed by
`cargo build --release` — is never validated. If the strip script produces a broken tree or
if the no-feature code path produces invalid SBOM JSON, it will not be caught until a customer
reports it.

**Fix:** Add a second validation step (or a separate job) that:
1. Runs `scripts/strip_vulnerability.sh global`
2. Builds `cargo build --release` on the stripped tree
3. Runs the same SPDX/CycloneDX validation checks as the existing `validate-sbom` job

---

### WR-05: `enrich_cwe_ids` is called unconditionally after `query_vulnerabilities_batch` in `main.rs` without checking whether any vulnerabilities were found

**File:** `src/main.rs:206-211`

**Issue:** Inside the `#[cfg(feature = "internal")]` block, `enrich_cwe_ids` is called even
when `query_vulnerabilities_batch` returned an error or populated zero vulnerabilities:

```rust
// Line 206-211:
enrich_cwe_ids(
    &mut dependencies,
    std::time::Duration::from_secs(args.cache_ttl * 3600),
    std::time::Duration::from_secs(args.vulnerability_timeout),
);
```

The code comment says "Skips silently if none found," which is a claim about
`enrich_cwe_ids`'s internal behavior, not a guard in `main.rs`. If the OSV batch call failed
(the `Err` arm at line 196 prints a warning and continues), `enrich_cwe_ids` still makes NVD
API network calls for zero valid CWE IDs, incurring unnecessary latency. Worse, there is no
timeout on the NVD calls here; the duration is derived from `args.cache_ttl * 3600` which is
a cache TTL (not a per-request timeout), so the `Duration::from_secs(args.cache_ttl * 3600)`
argument is being used as a network timeout — which for the default `cache_ttl = 24` would be
86,400 seconds (24 hours), an effective no-timeout.

**Fix:** Guard the `enrich_cwe_ids` call:

```rust
let has_vulns = dependencies.iter().any(|d| !d.vulnerabilities.is_empty());
if has_vulns {
    enrich_cwe_ids(
        &mut dependencies,
        std::time::Duration::from_secs(args.cache_ttl * 3600),
        std::time::Duration::from_secs(args.vulnerability_timeout),
    );
}
```

Also audit the `enrich_cwe_ids` signature to confirm the first duration argument is the cache
TTL and the second is the network timeout, and that the network timeout is bounded (e.g., by
`args.vulnerability_timeout`, not `cache_ttl`).

---

## Info

### IN-01: `cwe_scanner.rs` is referenced in a strip-script comment as a "Phase 10 stub for Phase 11" but is not referenced from anywhere in the `vulnerability` module public API

**File:** `scripts/strip_vulnerability.sh:24-25`

**Issue:** The strip script comment says "src/vulnerability/cwe_scanner.rs (Phase 10 stub for
Phase 11 lexical scanner) is covered by the directory removal above." The module is declared in
`src/vulnerability/mod.rs` as `pub mod cwe_scanner;` but neither used nor re-exported. This
creates an orphaned module that will accumulate dead code warnings as Phase 11 items are added
to it. Low risk now, but worth tracking.

**Fix:** Add `#[allow(dead_code)]` and a TODO comment in `cwe_scanner.rs` referencing the
Phase 11 implementation ticket so the stub intent is machine-readable.

---

### IN-02: `Dependency::new()` builder method is marked `#[allow(dead_code)]` despite being used extensively in tests

**File:** `src/models/dependency.rs:386-394`

**Issue:** `Dependency::new`, `with_source`, `with_source_file`, `with_is_dev`,
`with_is_direct`, and `with_scope` are all marked `#[allow(dead_code)]`. These methods are
used in test files (`tests/integration_tests/production_mode_e2e_tests.rs`,
`tests/integration_tests/scope_filtering_integration_tests.rs`, etc.). In the `internal`
feature configuration, they are exercised; in the no-feature configuration, the tests that
call them are gated behind `#![cfg(feature = "internal")]` (see CR-01), so rustc correctly
sees them as dead in the public build. Once CR-01 is fixed (test files lose the file-level
cfg gate), these `#[allow(dead_code)]` annotations should be removed as they will no longer
be needed.

**Fix:** After fixing CR-01, remove the `#[allow(dead_code)]` from the builder methods to
restore normal dead-code warnings.

---

_Reviewed: 2026-05-09T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
