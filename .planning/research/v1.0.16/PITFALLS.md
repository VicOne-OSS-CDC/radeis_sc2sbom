# Domain Pitfalls: Adding Static Analysis CWE Detection to radeis_sc2sbom

**Domain:** SBOM CLI tool + CWE static analysis integration
**Researched:** 2026-05-08
**Overall confidence:** MEDIUM — most findings verified across multiple sources; tool-specific performance numbers are empirical estimates

---

## Critical Pitfalls

Mistakes that cause rewrites, broken distribution, or compliance credibility loss.

---

### Pitfall 1: Static Binary Identity Destroyed by External Tool Dependency

**What goes wrong:** radeis_sc2sbom is a zero-dependency static musl binary. Its core value proposition — "drop one binary, run anywhere on Ubuntu 22.04+" — is destroyed the moment the CWE scan path invokes a subprocess (`cppcheck`, `semgrep`, etc.) that may not be installed on the target machine.

**Why it happens:** The natural implementation impulse is `Command::new("cppcheck").args(...)`, which works locally and passes CI (where cppcheck is installed), then silently fails or panics in customer environments.

**Consequences:**
- Customers on embedded build hosts (common in automotive Linux) have no internet access to install cppcheck; they get unhelpful "No such file or directory" panics rather than a clear capability message.
- The binary stops being self-contained; the README grows an installation prerequisites section; this erodes trust.
- If you ship a musl binary and the subprocess is a glibc-linked cppcheck, it still needs glibc present — defeating the entire musl strategy.

**Prevention:**
- Treat external tool presence as an optional capability, not a hard dependency. CWE scanning must degrade gracefully: if the tool is absent, emit a structured warning and continue (same pattern used for broken symlinks in v1.0.14).
- Use `which::which("cppcheck")` (or equivalent `std::process::Command` probe at startup) to detect presence before attempting scan. Surface this as `--check-cwe` being silently skipped with a `[WARN] cppcheck not found — CWE scan skipped` message.
- Gate the entire CWE scan path behind both the `internal` cargo feature AND a runtime tool-present check. The feature gates the code from the public binary; the runtime check gates execution.
- Document clearly in `--help` which external tools are required and where to install them. This converts a confusing runtime crash into an actionable user message.
- **Do not bundle cppcheck** inside the binary. Distributing a compiled C++ binary inside a Rust musl binary is impractical, and cppcheck is already 30MB+ on its own.

**Detection:** CI matrix with no cppcheck installed. The test should verify the tool exits with code 0 and emits a warning, not a panic.

---

### Pitfall 2: CWE Findings Treated as Authoritative in a Compliance Context

**What goes wrong:** Static analysis tools detect 46–55% of real vulnerabilities in comparative studies, with cppcheck reporting the highest false positive rate among common open-source tools. Outputting CWE IDs in SPDX/CycloneDX fields without communicating this limitation leads compliance reviewers to either over-trust or completely distrust the findings.

**Why it happens:** SBOM formats (especially CycloneDX 1.5) have structured `vulnerabilities.weaknesses` fields that look authoritative when populated. Once CWE IDs appear in machine-readable output, downstream consumers (xZETA, audit tools) treat them as ground-truth findings.

**Consequences:**
- False positives in compliance contexts (ISO 26262, UN R155) create audit findings that require formal investigation and closure. One false CWE-119 (buffer overflow) finding can trigger a multi-week triage cycle.
- False negatives give a false sense of safety: customers believe they have CWE coverage because the SBOM contains CWE IDs, but pattern-based tools miss complex control-flow vulnerabilities entirely.
- VicOne's credibility with automotive OEMs is at stake if the tool produces a compliance artifact with unreliable CWE data.

**Prevention:**
- Emit CWE findings under a dedicated source-analysis section distinct from NVD-enriched CVE→CWE mappings (which are already gated behind `internal`). Never mix the two provenance types in the same output field.
- Add a mandatory metadata annotation to every CWE finding from static analysis, e.g., `analysisMethod: "pattern-based-static-analysis"` and `confidence: "low"` or `confidence: "medium"` depending on the rule. CycloneDX 1.5 supports `analysis.state` and `analysis.justification` for exactly this purpose.
- Include a machine-readable disclaimer in the SBOM metadata: `"staticAnalysisCoverage": "pattern-based, not exhaustive. Rule-dependent false positive and false negative rates apply."` This gives downstream tools a flag to show warnings in their UI.
- Scope to a curated, high-confidence CWE subset rather than running all checks. CWE-119, CWE-120, CWE-134 (buffer/format string) have reliable detection; CWE-362 (race conditions) and CWE-416 (use-after-free) require data-flow analysis that cppcheck handles poorly.

**Detection:** Review cppcheck output against a known-good test case. Count false positives on the xcar-linux sample corpus before shipping.

---

### Pitfall 3: Scan Time Makes the Tool Unusable on Real Targets

**What goes wrong:** cppcheck on a large automotive Linux C/C++ codebase (xcar-linux likely has hundreds of thousands of lines) can take hours without configuration. A tool that takes 3+ hours to complete a scan stops being used.

**Why it happens:** Default cppcheck settings enable cross-translation-unit (CTU) analysis (`--max-ctu-depth=2`) and full check level. With template-heavy or macro-heavy C code (common in automotive BSW), analysis time is superlinear in codebase size.

**Consequences:**
- Users disable the CWE scan flag permanently after the first slow run.
- The feature becomes a one-time-use artifact rather than a CI gate.
- Feedback loop with false positives: a slow scan that also produces noise is doubly punishing.

**Prevention:**
- Default to `--check-level=reduced` and `--max-ctu-depth=0` (no CTU) for the first release. Document how to enable deeper analysis. This keeps runtime under 10 minutes for typical automotive codebases.
- Add a `--cwe-timeout <seconds>` flag that kills the subprocess and emits a partial result with a timeout warning. Never let a subprocess block the main SBOM output indefinitely.
- Parallelize with `-j $(nproc)` by default, configurable via `--cwe-jobs N`.
- Scope analysis to files that belong to identified SBOM components rather than scanning all C/C++ files. xcar-linux has a lot of generated code that is not worth scanning.
- Report scan time in the output summary so users can make informed decisions about configuration.

**Detection:** Benchmark on the xcar-linux corpus before release. Set a pass/fail threshold of 10 minutes for the reduced check profile.

---

## Moderate Pitfalls

---

### Pitfall 4: "SBOM Tool with CWE Detection" Becomes a SAST Tool

**What goes wrong:** The scope boundary between an SBOM tool and a SAST tool is the difference between "what components exist and what known weaknesses are associated with them" vs. "what bugs exist in this codebase." Once you add static analysis, every feature request that follows is a SAST request: "Can you show me the file and line?", "Can you auto-fix it?", "Can you filter by severity?", "Can you integrate with our defect tracker?"

**Why it happens:** The initial CWE feature is small and well-bounded. But once users see CWE IDs in their SBOM, their natural next question is "where in the code?" This is reasonable but leads the tool into SAST territory.

**The line:**
- **SBOM tool scope:** CWE IDs associated with components, sourced from static analysis, with provenance metadata. The output is in the SBOM artifact.
- **SAST tool scope:** Findings with file/line references, remediation guidance, false positive triage workflow, IDE integration, CI policy enforcement.

**Consequences:**
- Scope creep consumes v1.0.17, v1.0.18, v1.0.19 building SAST features that duplicate existing tools (cppcheck, semgrep, SonarQube).
- The tool loses its identity as an SBOM generator.
- Competitive positioning vs. dedicated SAST tools is unfavorable.

**Prevention:**
- Explicitly state in the feature spec what is NOT included: no file/line findings in output, no remediation guidance, no false positive triage workflow.
- The CWE output belongs in the SBOM metadata (`vulnerabilities.weaknesses` in CycloneDX or `externalRef` in SPDX), not in a separate findings report.
- If users request SAST features, redirect them to cppcheck's own HTML report output (`cppcheck --output-format=html`) rather than replicating it.
- Phase gate: v1.0.16 delivers CWE IDs in SBOM metadata only. File-level findings are explicitly out of scope with a logged rationale.

**Detection:** Review the feature spec for any mention of "file," "line number," "remediation," or "fix" — these are SAST scope signals.

---

### Pitfall 5: cargo feature = "internal" Interaction — Feature Leaks and Test Coverage Gaps

**What goes wrong:** The `internal` cargo feature gates CVE scanning today and will gate CWE scanning in v1.0.16. The risk is that feature-gated code is tested only with the feature enabled (in internal CI), while the public binary path (feature disabled) is never tested for regressions introduced by the feature-gated code path sharing state with non-gated code.

**Why it happens:** `cargo test` runs with default features. If the `internal` feature is in default features for development, all tests pass. But CI for the public binary build (`cargo build --release --no-default-features --features cn-release`) may not run the full test suite.

**Specific risks for CWE integration:**
- A `#[cfg(feature = "internal")]` block that modifies a `SbomComponent` struct (adding CWE fields) can silently cause serialization differences in the public build if the struct derives are not carefully scoped.
- The subprocess invocation code, even gated behind `internal`, may introduce a dependency (e.g., `which` crate) that gets compiled into all builds, adding binary size without corresponding functionality for public users.
- Typos in feature names (e.g., `#[cfg(feature = "internals")]`) compile silently without the `unexpected_cfgs` lint, causing the feature to be permanently disabled.

**Prevention:**
- Enable `unexpected_cfgs` lint explicitly: add `check-cfg = ['cfg(feature, values("internal", "cn-release"))']` to Cargo.toml (RFC 3013, stable as of Rust 1.80).
- Run the full test suite with `--no-default-features` in CI as a separate job. This catches public binary regressions.
- Do not add crates to `[dependencies]` solely to support `internal` feature code; use `[target.'cfg(feature = "internal")'.dependencies]` scoping where possible, though Cargo's support for this is limited — prefer keeping feature-only deps in a workspace crate if they become large.
- Review struct definitions: CWE fields on `SbomComponent` or output models should be `Option<Vec<CweId>>` so they serialize cleanly as `null`/absent in the public build.

**Detection:** Add a CI job: `cargo build --release --target x86_64-unknown-linux-musl --no-default-features --features cn-release` followed by binary size assertion (should not grow vs. baseline).

---

### Pitfall 6: CWE False Negatives Not Communicated — Compliance Theater Risk

**What goes wrong:** A compliance team uses the SBOM CWE section as evidence of security analysis and gets a "clean" report (few or no CWE findings). Pattern-based tools like cppcheck miss complex vulnerabilities (CWE-362 race conditions, CWE-416 use-after-free, CWE-787 out-of-bounds write in complex pointer arithmetic). The team files the SBOM as evidence of due diligence. A real vulnerability in one of those missed categories later becomes a CVE.

**Why it happens:** The SBOM artifact looks authoritative. A machine-readable CWE section in a CycloneDX document implies "these weaknesses were checked for." There is no standard way in CycloneDX 1.5 to declare which CWE IDs were checked and found absent vs. which were not checked at all.

**Consequences:**
- UN R155 compliance requires "reasonable efforts to detect and avoid common weaknesses." A tool that produces zero CWE findings from a real automotive codebase will raise questions in a TÜV audit.
- VicOne's liability exposure if the SBOM artifact is used as a security attestation.

**Prevention:**
- Include a `checkedCweIds` metadata field in the SBOM output listing every CWE rule that was actually run. This distinguishes "checked, not found" from "not checked."
- Include the cppcheck version and rule set in the SBOM metadata. This gives auditors the ability to assess coverage independently.
- Add a CLI output summary: `CWE scan complete: 47 rules checked across 312 files. 3 potential issues flagged. Pattern-based analysis — complex data-flow vulnerabilities not covered.`
- Never emit a "0 CWE issues found" summary without also stating the rule set used.

**Detection:** Show the output to a security engineer unfamiliar with the tool. If they interpret "0 CWE findings" as "no vulnerabilities," the output is misleading.

---

## Minor Pitfalls

---

### Pitfall 7: Subprocess Output Encoding and JSON Parsing Brittleness

**What goes wrong:** cppcheck's `--output-format=xml` output changes between major versions. Parsing its output with a hand-written XML parser tied to version 2.x will break on version 2.17 → 2.20 transitions, which have already broken third-party integrations.

**Prevention:** Pin the cppcheck version in CI. Detect the installed version at runtime (`cppcheck --version`) and emit a warning if it is outside the tested range. Prefer `--output-format=xml2` (the newer format) and test against cppcheck 2.14+ only.

---

### Pitfall 8: CWE IDs in Wrong SBOM Field

**What goes wrong:** Putting static analysis CWE IDs in the same `vulnerabilities` section as CVE findings from OSV makes them indistinguishable in downstream tools. xZETA and similar platforms may treat all entries in `vulnerabilities` as confirmed CVEs requiring remediation tracking.

**Prevention:** Use a separate metadata property (`properties` array in CycloneDX or `externalRef` with a custom category in SPDX) for static analysis CWE findings. Never use the `vulnerabilities` section for findings that are not confirmed CVE-mapped vulnerabilities. The existing CVE→CWE enrichment (NVD-sourced) is correct in `vulnerabilities.weaknesses`; source-analysis CWEs belong in component properties.

---

### Pitfall 9: Scan Invocation Fails Silently on Read-Only Paths

**What goes wrong:** cppcheck writes temp files and cache to the scanned directory by default. Automotive build systems often mount source trees read-only. The scan fails silently or with a confusing write-permission error.

**Prevention:** Always pass `--cppcheck-build-dir=$(mktemp -d)` and clean up afterwards. Never assume the scan target is writable. Use `tempfile` crate (already in Cargo.toml) to manage the temp directory.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| External tool integration | Static binary identity destroyed | Runtime tool-presence check; graceful skip with warning |
| CWE output format | CWE IDs in wrong SBOM section | Properties array for source-analysis CWEs; never mix with CVE vulnerabilities |
| Performance on xcar-linux | Hours-long scan blocks CI | Default reduced check level; mandatory timeout flag |
| cargo feature gating | Public binary regressions | CI job: build + test with `--no-default-features` |
| Compliance presentation | False negative = compliance theater | Emit checked CWE rule list in metadata; never claim "0 issues = safe" |
| Scope definition | SAST feature requests flood in | Hard scope boundary: CWE IDs in SBOM only, no file/line findings |
| cppcheck version | XML parsing breaks on upgrade | Version detection at runtime; pin tested range in CI |

---

## Sources

- Cppcheck manual and performance discussion: https://cppcheck.sourceforge.io/manual.pdf, https://sourceforge.net/p/cppcheck/discussion/development/thread/fde0271a77/
- False positive rate comparative study: https://www.mdpi.com/2079-9292/12/16/3518
- Cargo features pitfalls: https://effective-rust.com/features.html, https://rust-lang.github.io/rfcs/3013-conditional-compilation-checking.html
- CycloneDX vulnerability fields: https://cyclonedx.org/use-cases
- SAST vs SBOM scope distinction: https://www.mayhem.security/blog/sca-sbom-vulnerability-management-sast-or-dast-tools-which-is-best-for-your-team
- cppcheck `--check-level` and `--max-ctu-depth`: https://manpages.debian.org/testing/cppcheck/cppcheck.1.en.html
- Semgrep C/C++ analysis: https://semgrep.dev/blog/2024/modernizing-static-analysis-for-c/
- CWE compliance for C/C++: https://www.parasoft.com/solutions/cwe/
