# Feature Landscape: Source-Code CWE Detection

**Domain:** Static analysis CWE detection in an SBOM CLI tool
**Milestone:** v1.0.16
**Researched:** 2026-05-08
**Overall confidence:** HIGH (CWE list, CycloneDX schema); MEDIUM (workflow/compliance patterns)

---

## 1. Which CWEs Are Most Relevant for C/C++ Automotive/Embedded Code

Source: MITRE CWE Top 25 (2025), Code Intelligence C/C++ analysis, Parasoft ISO 26262 guidance, Flawfinder detection table.

### Tier 1 — Memory Safety (highest impact, detectable from source patterns)

| CWE ID | Name | 2025 Top 25 Rank | C/C++ Specific? | Detection Signal |
|--------|------|-----------------|-----------------|-----------------|
| CWE-787 | Out-of-bounds Write | 5 | YES | strcpy/memcpy into fixed-size buffer; array write without bounds check |
| CWE-125 | Out-of-bounds Read | 8 | YES | Array read without bounds check; off-by-one index |
| CWE-119 | Improper Buffer Bounds Operation | (parent) | YES | General unchecked copy/move operations |
| CWE-120 | Buffer Copy w/o Size Check | 11 | YES | calls to strcpy, strcat, gets, memcpy without guard |
| CWE-416 | Use After Free | 7 | YES | free() followed by pointer dereference |
| CWE-476 | NULL Pointer Dereference | 13 | YES | pointer used before null check |
| CWE-121 | Stack-based Buffer Overflow | 14 | YES | fixed-size stack arrays + unchecked writes |
| CWE-122 | Heap-based Buffer Overflow | 16 | YES | malloc'd buffer + unchecked writes |

### Tier 2 — Integer and Numeric (medium impact, detectable heuristically)

| CWE ID | Name | 2025 Top 25 Rank | Detection Signal |
|--------|------|-----------------|-----------------|
| CWE-190 | Integer Overflow or Wraparound | — | arithmetic on size_t/int before allocation or array index |
| CWE-191 | Integer Underflow | — | subtraction on unsigned types used as size/index |
| CWE-197 | Numeric Truncation Error | — | cast from larger to smaller integer type before use as size |

### Tier 3 — Input Handling (detectable from specific dangerous-function calls)

| CWE ID | Name | 2025 Top 25 Rank | Detection Signal |
|--------|------|-----------------|-----------------|
| CWE-134 | Use of Externally-Controlled Format String | — | printf/syslog with non-literal first argument |
| CWE-676 | Use of Potentially Dangerous Function | — | presence of gets(), strcpy(), sprintf() without safe alternative |
| CWE-78 | OS Command Injection | 9 | system(), popen(), execv() with non-literal argument |

### Automotive/Embedded Priority Note

For ISO 26262 / UN R155 compliance, memory safety CWEs (Tier 1) dominate. Undefined behaviors account for 37% of all CVEs industry-wide; in embedded C the ratio is higher. CWE-787 + CWE-120 + CWE-476 together cover the majority of exploitable findings in automotive C/C++ codebases. CWE-134 (format string) is lower-incidence but high-severity in firmware.

---

## 2. How SBOM Tools Surface CWE Findings

### CycloneDX 1.5 Modeling (VERIFIED against spec and Python library)

CycloneDX 1.5 uses the `vulnerabilities` top-level array to surface both CVE-backed and non-CVE findings. Key fields:

**Vulnerability object fields relevant to source-code CWE detection:**

```json
{
  "bom-ref": "sa-CWE-120-libfoo-main.c-42",
  "id": "SA-CWE-120-2026-001",
  "source": {
    "name": "radeis_sc2sbom static analysis",
    "url": "https://cwe.mitre.org/data/definitions/120.html"
  },
  "cwes": [120],
  "description": "Buffer copy without size check: strcpy() call at main.c:42",
  "affects": [
    {
      "ref": "<component bom-ref>",
      "versions": [{ "version": "<component version>", "status": "affected" }]
    }
  ],
  "analysis": {
    "state": "in_triage",
    "detail": "Detected by source pattern scan"
  },
  "properties": [
    { "name": "sc2sbom:finding:file", "value": "src/main.c" },
    { "name": "sc2sbom:finding:line", "value": "42" },
    { "name": "sc2sbom:finding:function", "value": "strcpy" }
  ]
}
```

Key points:
- `cwes` is `array<integer>` — just the numeric CWE ID, no prefix
- `id` can be any internal string (not required to be a CVE)
- `source.name` identifies the tool, not a CVE database
- `affects[].ref` links to a component's `bom-ref` — attaches the finding to a component, not a raw file path
- File/line location goes in `properties` as custom key-value pairs
- This is IDENTICAL to how CVE findings are modeled — no schema distinction between SAST and CVE

**There is no separate "findings" or "defects" object in CycloneDX 1.5.** Static analysis findings are modeled as vulnerabilities with no external CVE ID. The `source.name` field distinguishes them.

Note: CycloneDX 1.6 added `declarations` for compliance attestations — that is a different concept (regulatory conformance claims, not individual findings). CycloneDX 1.5 does not have declarations.

### SPDX 2.3 Modeling

SPDX 2.3 has no native vulnerability or finding object. The only mechanism for attaching CWE information is:
- `ExternalRef` on a package with `SECURITY` category — e.g., `ExternalRef: SECURITY url https://cwe.mitre.org/data/definitions/120.html`
- `Snippet` elements can reference byte ranges in files, but there is no CWE field on snippets

SPDX 2.3 is not the right format for surfacing per-finding CWE data. The correct pattern is: emit CWE findings only in CycloneDX output, and optionally include a `static-analysis-report` ExternalRef on the package in SPDX pointing to an out-of-band SARIF file.

### Attachment Pattern: Component vs File vs Standalone

Findings in the ecosystem attach at three levels:

| Level | Approach | Consumer Use |
|-------|----------|-------------|
| **Component** | `affects[].ref` points to component bom-ref | Correct for SBOM — tool sees which library/package has findings |
| **File** | `properties` with `finding:file` + `finding:line` | Supplemental; needed for developer fix workflow |
| **Standalone** | No `affects`, top-level vulnerability | Anti-pattern — compliance consumers can't map to inventory |

Recommendation: attach to component (required) + file path + line in properties (supplemental). Do not emit findings without an `affects` ref.

---

## 3. Table Stakes vs Differentiators

### Table Stakes (minimum viable — missing = product feels incomplete)

| Feature | Why Expected | Complexity |
|---------|--------------|------------|
| Detect Tier 1 CWEs (CWE-120, CWE-787, CWE-416, CWE-476, CWE-134) via dangerous-function pattern scan | All compliance tools (Parasoft, cppcheck) cover these; auditors ask for them by name | Low — regex/grep over .c/.h files |
| Attach CWE findings to component in CycloneDX `vulnerabilities[].cwes` | xZETA already ingests this field for CVE-based CWEs; same path for SAST findings | Low — reuse existing vulnerability serializer |
| Include file path + line number in `properties` | Without file/line, developers cannot act on findings | Low — store during scan |
| Report section: "Static Analysis Findings" separate from "Vulnerabilities (CVE)" | Compliance engineers need to distinguish sourced CVEs from tool-detected patterns | Low — separate report section |
| Scope to C/C++ files only (.c, .h, .cpp, .hpp, .cc) | Other languages (Rust, Python) have different patterns; overapplication creates noise | Low |
| Configurable: allow disabling static analysis scan | Some users run dedicated SAST already; forced scan creates friction | Low |

### Differentiators (valued but not expected)

| Feature | Value Proposition | Complexity |
|---------|-------------------|------------|
| CWE severity/confidence score per finding | Lets compliance engineers prioritize (CWE-120 via gets() is near-certain; via memcpy is possible) | Medium |
| Deduplication: same dangerous function in multiple files → grouped by CWE category per component | Reduces noise from large codebases | Medium |
| ISO 26262 / ASIL tag on findings | Maps CWE to safety integrity level for TARA (threat analysis) workflows | Medium-High — needs ASIL-CWE mapping table |
| Suppress-list support (`sc2sbom-nocheck` comment annotation or config file) | Lets teams mark false positives; required for mature integration | Medium |
| SARIF output for IDE/PR integration | Standard format (GitHub, VS Code) for developer workflow | Medium |

### Anti-Features — Do Not Build

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| Full dataflow / taint analysis | This is SAST scope, not SBOM scope; requires AST parsing, interprocedural analysis — 10x implementation complexity | Invoke cppcheck/semgrep as subprocess, parse their output |
| Inter-file analysis (tracking pointer across translation units) | Same: SAST domain; requires a full build graph | Stay within per-file pattern matching |
| Attempt to determine exploitability | CVSSv3 vector scoring for custom findings is speculative without runtime context | Emit state: "in_triage" in analysis field; let xZETA score |
| Scan non-C files for C-style patterns | False positive rate is high for vendored headers or generated code | Gate on file extension |
| Replace or compete with cppcheck/Flawfinder | Tool market is saturated; this is a compliance metadata enrichment tool | Optionally wrap cppcheck output, don't reimpliment it |
| Emit findings without component `affects` ref | Orphaned findings break xZETA ingestion and SBOM-level traceability | Always link to component |

---

## 4. User Workflow — How Compliance Engineers Consume CWE Findings

Based on: Parasoft ISO 26262 guidance, UN R155 / xZETA patterns, xZETA product documentation.

**Typical workflow:**

1. **SBOM generation** — run radeis_sc2sbom on automotive Linux source tree; output CycloneDX 1.5 JSON
2. **SBOM import into xZETA** — xZETA ingests the CycloneDX file; CWEs in `vulnerabilities[].cwes` appear in component view
3. **Triage** — compliance engineer filters by CWE category (e.g., all CWE-120 findings in AUTOSAR BSW layer)
4. **TARA documentation** — findings feed into ISO/SAE 21434 Threat Analysis and Risk Assessment; CWE ID maps to attack vector in TARA template
5. **Audit report** — export as PDF / compliance matrix with CWE findings per component; submitted to OEM or type-approval body under UN R155

**What format compliance engineers need:**

- CycloneDX 1.5 JSON (machine-readable, xZETA-ingestible) — primary deliverable
- Markdown/console report section listing: Component → CWE ID → CWE name → file:line → function — secondary (developer + auditor readable)
- Per-component CWE summary count ("libfoo: 3 CWE-120 findings") in report header — facilitates risk prioritization

**What they do NOT need from the SBOM tool:**
- Full remediation guidance (that's the IDE/SAST tool's job)
- Confirmed exploitability scores (requires DAST / runtime context)
- Git blame or author attribution

---

## 5. CWE vs SAST Overlap — Where Is the Line

**SBOM tool scope (radeis_sc2sbom stays here):**
- Dangerous-function presence detection: `strcpy`, `gets`, `sprintf`, `printf` (non-literal format arg), `system`, `free` + subsequent dereference — detectable by lexical/regex scan
- CWE ID assignment based on detected function → known CWE mapping table
- Attachment of findings to components in SBOM output
- Inventory-level aggregation: "this component contains N potential CWE-120 instances"

**SAST tool scope (out of scope for radeis_sc2sbom):**
- Interprocedural dataflow: does tainted input actually reach the dangerous call?
- Reachability: is the vulnerable code path ever executed?
- Fix suggestions: what safe function to use instead
- Branch/path-sensitive analysis

**The key distinction (HIGH confidence):**
SBOM tools detect *what is present in the inventory*; SAST tools determine *whether a weakness is exploitable*. An SBOM finding of CWE-120 says "this component calls strcpy without checking bounds." A SAST finding says "this strcpy call is reachable from user input and the buffer is insufficient." These are complementary, not duplicative. xZETA explicitly surfaces both CVE and CWE — the expectation is that an SBOM tool contributes CWE IDs to the inventory, not that it replaces Parasoft or cppcheck.

**Practical boundary for implementation:**
Use **lexical pattern matching** (function name recognition in .c/.h/.cpp files) as the detection method. Do not attempt to build even a simple AST — that slides toward SAST territory and multiplies implementation complexity without a corresponding increase in compliance value at the SBOM layer. Flawfinder and similar tools have proven that dangerous-function name detection alone is sufficient for compliance reporting purposes.

**Integration option (differentiator, not table stakes):**
If cppcheck is available on the host, optionally invoke it as a subprocess and parse its XML output (`--xml` flag emits CWE IDs per finding). This gives higher-confidence results for teams that want them without reimplementing analysis. Gate this behind a `--with-cppcheck` flag.

---

## Feature Dependencies

```
Dangerous-function scan (per file)
  → CWE-to-function mapping table
    → CycloneDX vulnerability entries (cwes[] + affects[])
      → Report section "Static Analysis Findings"

Optional: cppcheck subprocess integration
  → Parse cppcheck XML
    → Same CycloneDX vulnerability entries path
```

---

## MVP Recommendation

**Build for v1.0.16:**

1. Lexical dangerous-function scanner over .c/.h/.cpp/.hpp files
2. Function → CWE mapping table covering Tier 1 CWEs: CWE-120 (strcpy, strcat, gets, memcpy), CWE-134 (printf/syslog with non-literal arg), CWE-676 (gets specifically), CWE-476 (NULL dereference heuristic: unchecked return from malloc/calloc used immediately)
3. CycloneDX 1.5 output: findings as vulnerability entries with `cwes[]`, `affects[].ref`, `source.name = "radeis_sc2sbom"`, and file/line in `properties`
4. Report section: per-component CWE finding list
5. Gate behind `--static-analysis` flag (or combine with existing `internal` feature gate)

**Defer:**
- CWE-416 (use-after-free): requires tracking free() calls + subsequent dereference across multiple lines — slides toward dataflow; too many false negatives from pure lexical scan
- CWE-190 (integer overflow): arithmetic context required; high false-positive rate from lexical scan
- cppcheck integration: differentiator for v1.0.17+
- SARIF output: differentiator, not required for xZETA ingestion

---

## Sources

- MITRE CWE Top 25 (2025): https://cwe.mitre.org/top25/archive/2025/2025_cwe_top25.html
- Code Intelligence C/C++ CWE analysis: https://www.code-intelligence.com/blog/most-dangerous-vulnerabilities-cwes-in-c-2025
- Parasoft ISO 26262 CWE guidance: https://www.parasoft.com/learning-center/iso-26262/cwe/
- CycloneDX Python library vulnerability model: https://cyclonedx-python-library.readthedocs.io/en/v9.1.0/autoapi/cyclonedx/model/vulnerability/
- CycloneDX vulnerability disclosure use case: https://cyclonedx.org/use-cases/vulnerability-disclosure/
- CycloneDX 1.5 JSON reference: https://cyclonedx.org/docs/1.5/json/
- Flawfinder dangerous-function table: https://dwheeler.com/flawfinder/correct-results.html
- CWE-676 (dangerous functions): https://cwe.mitre.org/data/definitions/676.html
- fkie-cad/cwe_checker (binary analysis reference): https://github.com/fkie-cad/cwe_checker
- SBOM vs SAST boundary: https://www.mayhem.security/blog/sca-sbom-vulnerability-management-sast-or-dast-tools-which-is-best-for-your-team
- xZETA CWE support: https://vicone.com/products/xzeta
- Embedded appsec dangerous functions: https://scriptingxss.gitbook.io/embedded-appsec-best-practices/1_buffer_and_stack_overflow_protection
