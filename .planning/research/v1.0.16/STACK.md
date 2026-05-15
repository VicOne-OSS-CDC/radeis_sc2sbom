# Static Analysis CWE Detection — Stack Research

**Milestone:** v1.0.16 — Source Code Static Analysis for CWE Detection
**Researched:** 2026-05-08
**Overall Confidence:** HIGH (primary tools), MEDIUM (native Rust path)

---

## Context

radeis_sc2sbom is a Rust CLI SBOM tool targeting C/C++ automotive Linux projects. It produces a static musl binary with no runtime dependencies. Any static analysis integration must account for:

- Subprocess invocation from Rust via `std::process::Command`
- Output parsing in Rust (XML, JSON, CSV, or SARIF)
- Distribution impact: does the user need an external tool installed?
- CWE ID extraction to enrich SBOM output
- License compatibility with commercial VicOne shipping

---

## Tool Assessments

### 1. cppcheck

**Current version:** 2.20 (as of 2026-05)
**License:** GPL-3.0

**CWE Coverage (HIGH confidence)**
Maps to ~48 distinct CWE IDs. Key ones:
- CWE-119 Buffer Boundaries Violation
- CWE-131 Incorrect Calculation of Buffer Size
- CWE-170 Improper Null Termination
- CWE-190 Integer Overflow or Wraparound
- CWE-195 Signed to Unsigned Conversion Error
- CWE-252 Unchecked Return Value
- CWE-362 Race Condition (Concurrent Execution via Shared Resource)
- CWE-369 Divide By Zero
- CWE-401 Memory Leak
- CWE-415 Double Free
- CWE-416 Use After Free
- CWE-467 sizeof on Pointer Type
- CWE-476 Null Pointer Dereference
- CWE-562 Return of Stack Variable Address
- CWE-665 Improper Initialization
- CWE-682 Incorrect Calculation
- CWE-704 Incorrect Type Conversion or Cast
- CWE-758 Reliance on Undefined Behavior
- CWE-762 Mismatched new/delete or malloc/free
- CWE-772 Missing Release of Resource
- CWE-786 / CWE-788 Out-of-bounds memory access
- CWE-908 Use of Uninitialized Resource
...and ~26 more

**Invocation (HIGH confidence)**
```
cppcheck --xml --xml-version=2 --enable=all <dir>
# CWE appears as cwe="NNN" attribute in each <error> element

# SARIF output (v2.16+, released Oct 2024):
cppcheck --output-format=sarif --enable=all <dir>
```

From Rust:
```rust
let output = std::process::Command::new("cppcheck")
    .args(["--xml", "--xml-version=2", "--enable=all", source_dir])
    .output()?;
// Parse output.stderr (cppcheck writes XML to stderr)
```

Note: cppcheck writes XML to stderr, not stdout. Parse `output.stderr`.

**Output parsing effort:** LOW — XML with `cwe="NNN"` attribute is straightforward to parse with `quick-xml` or `roxmltree`. SARIF is JSON parseable with `serde_json`.

**Distribution impact:** HIGH IMPACT — cppcheck must be installed on the user's system. Not bundleable in a static Rust binary. Package is available on Ubuntu 22.04 via `apt install cppcheck` and Alpine Linux packages. Binary is ~10–15 MB installed. No pre-compiled single-file static download available.

**Automotive context fit:** EXCELLENT — cppcheck supports MISRA C/C++ add-on (separate commercial product). Designed for embedded/automotive use. Native understanding of non-standard embedded syntax. Widely used in automotive Linux projects (matching the xcar-linux target repo).

**Recommendation: STRONG YES — primary tool.** Best CWE breadth for C/C++ of any free tool, automotive-grade pedigree, XML+SARIF output, straightforward subprocess invocation. The only downside is the external install requirement, which is acceptable if documented as a prerequisite.

---

### 2. Flawfinder

**Current version:** 2.0.19
**License:** GPL-2.0+

**CWE Coverage (HIGH confidence)**
Lexical scanner — pattern matches ~219 dangerous C/C++ function signatures. Maps to CWE IDs including:
- CWE-120 Buffer Copy without Checking Size of Input (strcpy, gets, sprintf, etc.)
- CWE-119 Improper Restriction of Operations within Bounds of a Memory Buffer
- CWE-134 Uncontrolled Format String (printf family)
- CWE-78 OS Command Injection (system, popen)
- CWE-362 Race Conditions (temp file operations, access/open TOCTOU)
- CWE-327 Use of Broken Cryptographic Algorithm (weak RNG functions)
- CWE-732 Incorrect Permission Assignment
- Risk level 0–5 per finding; CWE IDs included in CSV and SARIF output

**Invocation (HIGH confidence)**
```
flawfinder --csv <dir>          # CSV output with CWE column
flawfinder --sarif <dir>        # SARIF JSON output
flawfinder --minlevel 3 --csv . # Filter by risk level
```

From Rust:
```rust
let output = std::process::Command::new("flawfinder")
    .args(["--csv", source_dir])
    .output()?;
// Parse output.stdout as CSV; columns include CWEs field
```

CSV columns: File, Line, Column, Level, Category, Name, Warning, Suggestion, Note, CWEs, Context, Fingerprint. Very easy to parse without a library; split by comma or use `csv` crate.

**Output parsing effort:** VERY LOW — CSV is the simplest output to parse. SARIF is an alternative if JSON is preferred.

**Distribution impact:** HIGH IMPACT — requires Python 3 installed (`pip install flawfinder` or distro package). Adds a Python runtime dependency on top of the tool itself. This is a heavier prerequisite than cppcheck on a musl-based system.

**CWE depth limitation:** Lexical analysis only — no data flow, no control flow. Will miss CWEs that require understanding variable state (e.g., CWE-416 use-after-free only detectable by pattern proximity, not actual ownership analysis). High false positive rate for experienced developers.

**Recommendation: SECONDARY / COMPLEMENTARY.** Useful as a fast pre-filter or fallback if cppcheck is unavailable. Simpler to invoke and parse than cppcheck. However, requires Python runtime, has shallower CWE coverage (lexical only), and produces more false positives. Not the primary choice but worth considering as an optional fast-path.

---

### 3. Semgrep / OpenGrep

**Current version:** Semgrep CE ~1.x; OpenGrep fork active as of early 2025
**License:**
- Semgrep engine: LGPL 2.1
- Semgrep rules (semgrep-rules repo): Semgrep Rules License v1.0 — permits internal use only, prohibits use in competing products or SaaS offerings. This is a commercial restriction.
- OpenGrep engine: LGPL 2.1 (fork committed to staying open)
- Community rule collections (e.g., 0xdea/semgrep-rules): MIT

**C/C++ Support Status (HIGH confidence)**
- Semgrep CE: C/C++ is "community supported" — experimental-grade, single-function analysis only, no cross-file dataflow
- Semgrep Code (commercial): C/C++ is GA with cross-file dataflow and 150+ professional rules — not free
- OpenGrep: supports C and C++ (inherited from Semgrep CE), same limitations
- C/C++ support was announced as GA in Semgrep Code on Feb 27, 2024, but specifically NOT in the OSS engine

**CWE Coverage (MEDIUM confidence)**
Community C/C++ rules exist (e.g., 0xdea/semgrep-rules with 40+ rules under MIT license) covering:
- Buffer overflow patterns (strcpy, gets, scanf family) — CWE-119/120
- Integer issues — CWE-190
- Memory management (use-after-free, double-free) — CWE-416, CWE-415
- Command injection — CWE-78
- Race conditions / TOCTOU — CWE-362
- Format string — CWE-134

CWE Top 25 ruleset at semgrep.dev/p/cwe-top-25 exists but licensing is unclear for commercial redistribution contexts.

**Invocation**
```
semgrep --config=p/cwe-top-25 --json <dir>
# or
opengrep scan -f rules/ --sarif-output=sarif.json <dir>
```

**Output parsing effort:** LOW — JSON and SARIF outputs both well-structured and easily parsed with `serde_json`.

**Distribution impact:** HIGH IMPACT — semgrep/opengrep binary is a ~50–100 MB download (written in OCaml, ships as a compiled binary). Requires separate installation. Larger footprint than cppcheck.

**Licensing risk for radeis_sc2sbom:** MEDIUM — Semgrep's official rules have commercial-use restrictions. Using community MIT-licensed rules (0xdea) avoids this. OpenGrep is LGPL which is acceptable, but C/C++ analysis quality in OSS engine is limited compared to cppcheck.

**Recommendation: NOT RECOMMENDED as primary.** C/C++ support in free tier is limited. Rule licensing is complex. Binary footprint is large. cppcheck offers better C/C++ CWE coverage with less complexity and cleaner licensing. Semgrep Code (commercial) is competitive but requires paid subscription — out of scope for a shipping tool.

---

### 4. clang-tidy

**Current version:** LLVM 21.x (2025)
**License:** Apache 2.0 with LLVM exceptions

**CWE Coverage (MEDIUM confidence)**
clang-tidy does not natively map checks to CWE IDs. Check categories that are security-relevant:
- `bugprone-*`: unsafe functions (maps to CERT MSC24-C, MSC33-C), null pointer patterns
- `cert-*`: CERT C Coding Standard rules (overlap with CWE-78, CWE-120, CWE-134, CWE-190)
- `clang-analyzer-security.*`: Clang Static Analyzer security checks (CWE-119, CWE-476, CWE-134 etc.)
- No first-class CWE ID output — would require post-processing to map check names to CWE IDs via a custom mapping table

**Invocation model (HIGH confidence)**
clang-tidy requires `compile_commands.json` (compilation database) to work correctly on real projects. Without it, it falls back to basic analysis on individual files. For automotive Linux projects (make-based, complex), generating `compile_commands.json` via Bear or cmake is a prerequisite — and often non-trivial.

```
clang-tidy -checks='bugprone-*,cert-*,clang-analyzer-security.*' file.cpp \
    -p compile_commands.json
```

Output: plain text or YAML (for fixes). No native JSON/SARIF output for findings without third-party wrappers (CodeChecker, etc.).

**Output parsing effort:** HIGH — no structured machine-readable output without a wrapper tool. Finding-to-CWE mapping requires custom maintenance of a check-name → CWE table.

**Distribution impact:** VERY HIGH IMPACT — clang-tidy is part of LLVM/clang toolchain. Ubuntu 22.04 `apt install clang-tidy` installs ~200–400 MB of LLVM dependencies. Far heavier than cppcheck. Also requires compile_commands.json generation, which is a separate tooling step.

**Analysis quality:** HIGHEST of any tool listed — actual AST-based analysis, genuine dataflow understanding, very low false positive rate. But this quality is only realized with proper compilation database setup.

**Recommendation: NOT RECOMMENDED for v1.0.16.** Dependency weight and compile_commands.json requirement make it impractical for an SBOM tool that scans arbitrary source trees without building them. No native CWE output. Best reserved for CI/CD integration by developers building the project, not for a scanning tool.

---

### 5. Native Rust (tree-sitter based)

**Available crates:**
- `tree-sitter` (0.22.x) — Rust bindings for the tree-sitter parsing library
- `tree-sitter-c` — C grammar for tree-sitter
- No production-ready security/CWE detection crate exists as of research date

**CWE Coverage (LOW confidence)**
No pre-built CWE detection ruleset exists in Rust using tree-sitter. Building one would require:
1. Parsing C/C++ source with tree-sitter-c
2. Writing tree-sitter query patterns for each vulnerability class
3. Mapping each query to a CWE ID
4. Maintaining the rule set

This is essentially building a mini-SAST engine from scratch. Academic/research projects exist (Microsoft RIFT for Rust malware analysis uses similar techniques) but nothing production-ready for C/C++ CWE detection exists as a Rust crate.

**Distribution impact:** NONE — pure Rust, ships inside the binary. No external dependency required.

**Invocation:** N/A — embedded library, not subprocess.

**Implementation effort:** VERY HIGH — writing even 10 meaningful CWE patterns (e.g., CWE-120, CWE-476, CWE-134, CWE-416) with tree-sitter queries is a multi-week effort with significant ongoing maintenance cost.

**Recommendation: NOT RECOMMENDED for v1.0.16, but worth flagging as a long-term option.** The zero-distribution-overhead advantage is real and strategically attractive (no user prerequisites). However, the implementation cost to reach even 20% of cppcheck's CWE breadth is prohibitive for a milestone scope. Revisit in a future milestone if "zero external dependency" becomes a hard requirement.

---

## Comparison Matrix

| Tool | CWE Count | Analysis Depth | Output Format | Parse Effort | External Dep | License | Recommendation |
|------|-----------|---------------|---------------|-------------|--------------|---------|---------------|
| cppcheck 2.20 | ~48 CWEs | Dataflow (bi-directional) | XML (stderr), SARIF (v2.16+) | LOW | cppcheck binary | GPL-3.0 | **PRIMARY** |
| Flawfinder 2.0.19 | ~15 CWE classes | Lexical only | CSV, SARIF, text | VERY LOW | Python 3 + pip | GPL-2.0+ | SECONDARY |
| Semgrep CE / OpenGrep | ~15 CWEs (community rules) | Single-function | JSON, SARIF | LOW | Large binary (~100 MB) | LGPL-2.1 / rules MIT | NOT RECOMMENDED |
| clang-tidy 21.x | High (no CWE IDs natively) | Full AST + dataflow | Text/YAML only | HIGH | LLVM ~200–400 MB | Apache 2.0 | NOT RECOMMENDED |
| tree-sitter (native Rust) | 0 (must build) | AST pattern match | N/A (embedded) | — | None | MIT | NOT NOW |

---

## Recommended Approach for v1.0.16

**Primary:** cppcheck as an optional subprocess.

Gate the feature behind a check: if `cppcheck` is not on PATH, emit a warning and skip CWE detection from source analysis. Document `cppcheck` as a runtime prerequisite for this feature, consistent with how the tool already requires OSV/NVD network access for vulnerability scanning.

**Invocation design:**
```rust
// Detect availability
let available = std::process::Command::new("cppcheck")
    .arg("--version")
    .output()
    .is_ok();

// Run if available
let output = std::process::Command::new("cppcheck")
    .args(["--xml", "--xml-version=2", "--enable=all", "--quiet", source_dir])
    .output()?;

// Parse stderr as XML; extract <error cwe="NNN"> elements
```

**Output format choice:** XML v2 with `--xml --xml-version=2` is the most widely supported and stable format. SARIF (`--output-format=sarif`, requires cppcheck 2.16+) is an alternative if a minimum version can be enforced.

**CWE deduplication:** Multiple findings with the same CWE ID against the same component should be deduplicated to a single CWE entry in the SBOM component record.

**Feature gating:** CWE source detection should follow the existing `cargo feature = "internal"` pattern used for CVE/CWE enrichment.

---

## Open Questions for Phase Design

1. **Minimum cppcheck version to require**: 2.16+ enables SARIF output; 2.20 is current. Recommend requiring 2.16+ so SARIF can be used as the parsing format (cleaner than XML stderr parsing).

2. **Scope of source scanning**: Scan the entire project source tree, or only vendored/third-party directories? Scanning all source code generates many findings against the project's own code; the feature is likely most valuable scoped to vendored C/C++ libraries.

3. **CWE-to-SBOM mapping**: CWE findings from source analysis should be attached to the SBOM component (the vendored library), not the project itself. This requires correlating cppcheck output file paths back to the component that owns them.

4. **False positive volume**: cppcheck on a large automotive repo (e.g., xcar-linux) may produce thousands of findings. A minimum severity threshold or specific check subset (e.g., `--enable=warning,portability` not `--enable=all`) may be needed.

5. **Flawfinder as fallback**: If Python availability can be detected, Flawfinder could serve as a fallback when cppcheck is absent, providing reduced CWE coverage but zero additional binary dependencies beyond Python.

---

## Sources

- [Cppcheck official site — version 2.20](https://cppcheck.sourceforge.io/)
- [Cppcheck 2.16 release notes — SARIF support](https://sourceforge.net/p/cppcheck/news/2024/10/cppcheck-2160/)
- [Cppcheck CWE list — Static Analysis Rules summary](https://github.com/wcventure/Static-Analysis-Rules/blob/master/Summary%20of%20static%20analysis%20in%20C%20%26%20C%2B%2B/README.md)
- [Cppcheck XML format with CWE attribute](https://cppcheck.sourceforge.io/manual.html)
- [Flawfinder home page — v2.0.19, GPL-2.0+](https://dwheeler.com/flawfinder/)
- [Flawfinder man page — --csv, --sarif flags](https://manpages.debian.org/testing/flawfinder/flawfinder.1.en.html)
- [Flawfinder CWE compatibility — MITRE](https://cwe.mitre.org/compatible/questionnaires/28.html)
- [Semgrep C/C++ support — GA in Code, community in CE](https://semgrep.dev/products/product-updates/c-support/)
- [Semgrep CE language support page](https://semgrep.dev/docs/semgrep-ce-languages)
- [Semgrep licensing page](https://semgrep.dev/docs/licensing)
- [OpenGrep — LGPL fork, C/C++ support confirmed](https://github.com/opengrep/opengrep)
- [0xdea semgrep-rules — MIT licensed C/C++ rules](https://github.com/0xdea/semgrep-rules)
- [Clang-tidy documentation](https://clang.llvm.org/extra/clang-tidy/)
- [tree-sitter crate on crates.io](https://crates.io/crates/tree-sitter)
- [Reachability analysis with tree-sitter in Rust](https://kwekmh.com/posts/reachability-analysis-with-tree-sitter-in-rust-part-1/)
