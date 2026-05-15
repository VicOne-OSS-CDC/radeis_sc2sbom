# v1.0.16 Research Summary: Source-Code CWE Detection

**Synthesized:** 2026-05-08
**Scope:** C/C++ static analysis CWE detection for SBOM enrichment

## Recommended Stack

**v1.0.16:** Built-in Rust lexical scanner — dangerous-function name matching in `.c .h .cpp .hpp .cc` files. Zero external dependency. Covers Tier 1 CWEs only.

**v1.0.17 (deferred):** cppcheck 2.16+ as optional subprocess (~48 CWEs, XML/SARIF output, automotive pedigree). Must be optional with graceful degradation.

**Ruled out:** Flawfinder (Python dep), Semgrep CE (weak C/C++ OSS + license risk), clang-tidy (requires compile_commands.json + 200-400 MB LLVM), native tree-sitter (full SAST engine, out of scope).

## Feature Scope — Tier 1 CWEs (C/C++ only)

Table stakes for v1.0.16:
- **CWE-120**: `strcpy`, `strcat`, `gets`, `memcpy` without bounds guard
- **CWE-476**: unchecked return from `malloc`/`calloc` used as pointer
- **CWE-134**: `printf`/`syslog` with non-literal first argument
- **CWE-676**: presence of `gets()` (deprecated unsafe function)
- **CWE-787**: write to fixed-size buffer without bounds check

Output: CycloneDX 1.5 `vulnerabilities[]` entries with `source.name = "radeis_sc2sbom static analysis"`, `affects[].ref` to component, `analysis.state: "in_triage"`, file/line in `properties`. SPDX 2.3 has no native model — CycloneDX only.

Deferred to v1.0.17+: CWE-416 (use-after-free), CWE-190 (integer overflow), SARIF output, suppress-list, per-component CWE summary.

## Top 3 Pitfalls

1. **Scope drift** — users will ask for file/line, severity scoring, false positive suppression (all SAST scope). Hard boundary: CWE IDs attached to components in SBOM metadata only.
2. **Compliance theater** — never emit "0 CWE findings" without stating the rule set checked. Every finding carries `analysis.state: "in_triage"`. CLI must note: "Pattern-based — complex data-flow vulnerabilities not covered."
3. **False positive volume** — CWE-787 via `strcpy`/`memcpy` will hit wrapper functions. Benchmark against xcar-linux corpus before finalizing rule set.

## Open Questions

1. **Scan scope**: component-mapped C/C++ dirs only (recommended) vs. full source tree?
2. **xZETA behavior**: does xZETA treat all `vulnerabilities[]` entries as CVEs requiring remediation tracking, regardless of `source.name`? Needs validation before shipping.
3. **Feature gate**: same `internal` cargo feature as CVE/CWE enrichment, or separate `--static-analysis` flag?

## Roadmap for v1.0.16

- **Phase 11**: Lexical scanner + CycloneDX output — Rust-native dangerous-function scanner, Tier 1 CWEs, wired into existing vulnerability serializer
- **Phase 12**: Report section + scope boundary — "Static Analysis Findings" section, component-scoped C/C++ files, `[WARN]` messaging

---
*Research completed: 2026-05-08*
