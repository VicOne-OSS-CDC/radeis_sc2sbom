---
phase: 260509-gwq
plan: 01
type: execute
wave: 1
depends_on: []
files_modified:
  - src/main.rs
autonomous: true
requirements:
  - GWQ-fix-output-flag
must_haves:
  truths:
    - "Running with --format spdx-json --output ./myout writes myout/{project}_spdx.json instead of printing to stdout"
    - "Running with --format cyclonedx-json --output ./myout writes myout/{project}_cyclonedx.json"
    - "Running with --format spdx-tag-value --output ./myout writes myout/{project}_spdx.spdx"
    - "Running with --format console --output ./myout writes myout/{project}_report.md"
    - "Omitting --output (default 'out') preserves existing stdout behavior for all single formats"
  artifacts:
    - path: src/main.rs
      provides: "Updated single-format match arms that check args.output != 'out'"
      contains: "save_spdx_json"
  key_links:
    - from: "OutputFormat::SpdxJson arm"
      to: "save_spdx_json"
      via: "args.output != \"out\" guard"
      pattern: "save_spdx_json"
---

<objective>
Fix the --output flag so it is honored by all single-format modes (spdx-json, cyclonedx-json, spdx-tag-value, console), not only by --format all.

Purpose: Users running a single-format CI job (e.g., only CycloneDX) expect --output ./artifacts to write the file to disk. Currently those arms always call print_* to stdout and ignore --output entirely.

Output: Modified src/main.rs where each single-format match arm detects a non-default --output value and calls the corresponding save_* function with the same filename convention used by the All arm.
</objective>

<execution_context>
@$HOME/.claude/get-shit-done/workflows/execute-plan.md
@$HOME/.claude/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/STATE.md
@src/main.rs
@src/cli.rs

<interfaces>
From src/formats/ (already imported in main.rs):

  print_sbom(sbom, vuln_output, tree_style, max_vulns, compact, relationships, check_vulns)
  save_console_report(sbom, path: &str, tree_style, vuln_output, max_vulns, relationships, summary_only, check_vulns) -> Result<()>

  print_spdx_json(sbom, mode, compact_spdx, supplier_resolver) -> Result<()>
  save_spdx_json(sbom, path: &str, mode, compact_spdx, supplier_resolver) -> Result<()>

  print_spdx_tag_value(sbom, mode, compact_spdx, supplier_resolver)   // no Result
  save_spdx_tag_value(sbom, path: &str, mode, compact_spdx, supplier_resolver) -> Result<()>

  print_cyclonedx_json(sbom, mode, supplier_resolver) -> Result<()>
  save_cyclonedx_json(sbom, path: &str, mode, supplier_resolver) -> Result<()>

Args fields used:
  args.output: String  (default "out")
  args.sbom_mode: SbomMode
  args.compact_spdx: bool
  args.compact: bool
  args.summary_only: bool
  args.tree_style: TreeStyle
  args.vulnerability_output: VulnerabilityOutputMode
  args.max_vulns_per_severity: usize
  args.check_vulnerabilities: bool
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Honor --output in single-format match arms</name>
  <files>src/main.rs</files>
  <action>
In the `match args.format` block (lines 222-299 of src/main.rs), update the four single-format arms. The guard condition is `args.output != "out"` — this preserves backward compatibility: omitting --output keeps existing stdout behavior unchanged.

When the guard is true, derive project_name and out_dir the same way the All arm does, create the directory, build the path, call the save_* function, and eprintln a confirmation. When false, fall through to the existing print_* call unchanged.

Apply this pattern to each arm:

Console arm — file suffix `{project_name}_report.md`, save_* call:
  save_console_report(&sbom, &path_str, &args.tree_style, &args.vulnerability_output, args.max_vulns_per_severity, &all_relationships, args.summary_only, args.check_vulnerabilities)?

SpdxJson arm — file suffix `{project_name}_spdx.json`, save_* call:
  save_spdx_json(&sbom, &path_str, &args.sbom_mode, args.compact_spdx, supplier_resolver.as_ref())?

SpdxTagValue arm — file suffix `{project_name}_spdx.spdx`, save_* call:
  save_spdx_tag_value(&sbom, &path_str, &args.sbom_mode, args.compact_spdx, supplier_resolver.as_ref())?

CyclonedxJson arm — file suffix `{project_name}_cyclonedx.json`, save_* call:
  save_cyclonedx_json(&sbom, &path_str, &args.sbom_mode, supplier_resolver.as_ref())?

Each arm should follow this structure (Console shown as example; repeat pattern for others):

```rust
OutputFormat::Console => {
    if args.output != "out" {
        let project_name = sbom.project_path.file_name()
            .and_then(|n| n.to_str()).unwrap_or("sbom");
        let out_dir = Path::new(&args.output);
        std::fs::create_dir_all(out_dir)?;
        let out_path = out_dir.join(format!("{}_report.md", project_name));
        let out_path_str = out_path.to_string_lossy();
        save_console_report(
            &sbom,
            &out_path_str,
            &args.tree_style,
            &args.vulnerability_output,
            args.max_vulns_per_severity,
            &all_relationships,
            args.summary_only,
            args.check_vulnerabilities,
        )?;
        eprintln!("✓ Console report saved to: {}", out_path.display());
    } else {
        print_sbom(
            &sbom,
            &args.vulnerability_output,
            &args.tree_style,
            args.max_vulns_per_severity,
            args.compact,
            &all_relationships,
            args.check_vulnerabilities,
        );
    }
}
```

Do not touch the All arm. Do not reformat surrounding code. Do not change any imports (all save_* functions are already imported at line 15-17).
  </action>
  <verify>
    <automated>cd /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom && cargo build 2>&1 | tail -8</automated>
  </verify>
  <done>
`cargo build` exits 0. The four single-format arms each contain an `if args.output != "out"` branch calling the appropriate save_* function.
  </done>
</task>

<task type="auto">
  <name>Task 2: Smoke-test file output for each single format</name>
  <files></files>
  <action>
Run the binary against this repository's own directory to verify each format writes a file. Use a temp directory. The project folder name is `radeis_sc2sbom` so expected filenames follow that prefix.

Run these checks in sequence and collect PASS/FAIL:

1. spdx-json: expect file `{tmpdir}/t1/radeis_sc2sbom_spdx.json`
2. cyclonedx-json: expect file `{tmpdir}/t2/radeis_sc2sbom_cyclonedx.json`
3. spdx-tag-value: expect file `{tmpdir}/t3/radeis_sc2sbom_spdx.spdx`
4. console: expect file `{tmpdir}/t4/radeis_sc2sbom_report.md`
5. stdout backward-compat: run spdx-json without --output, confirm stdout has `{` as first char of first line

All five must PASS. Clean up the temp directory afterward.
  </action>
  <verify>
    <automated>
cd /Users/amean_lin/Desktop/GitRepos/radeis_sc2sbom
TMPDIR=$(mktemp -d)
B=./target/debug/sc2sbom
PASS=0; FAIL=0
$B -p . -f spdx-json --output "$TMPDIR/t1" 2>/dev/null && test -f "$TMPDIR/t1/radeis_sc2sbom_spdx.json" && PASS=$((PASS+1)) && echo "PASS spdx-json" || { FAIL=$((FAIL+1)); echo "FAIL spdx-json"; }
$B -p . -f cyclonedx-json --output "$TMPDIR/t2" 2>/dev/null && test -f "$TMPDIR/t2/radeis_sc2sbom_cyclonedx.json" && PASS=$((PASS+1)) && echo "PASS cyclonedx-json" || { FAIL=$((FAIL+1)); echo "FAIL cyclonedx-json"; }
$B -p . -f spdx-tag-value --output "$TMPDIR/t3" 2>/dev/null && test -f "$TMPDIR/t3/radeis_sc2sbom_spdx.spdx" && PASS=$((PASS+1)) && echo "PASS spdx-tag-value" || { FAIL=$((FAIL+1)); echo "FAIL spdx-tag-value"; }
$B -p . -f console --output "$TMPDIR/t4" 2>/dev/null && test -f "$TMPDIR/t4/radeis_sc2sbom_report.md" && PASS=$((PASS+1)) && echo "PASS console" || { FAIL=$((FAIL+1)); echo "FAIL console"; }
$B -p . -f spdx-json 2>/dev/null | head -1 | grep -q '{' && PASS=$((PASS+1)) && echo "PASS stdout-fallback" || { FAIL=$((FAIL+1)); echo "FAIL stdout-fallback"; }
rm -rf "$TMPDIR"
echo "Results: $PASS PASS, $FAIL FAIL"
test $FAIL -eq 0
    </automated>
  </verify>
  <done>
All 5 checks print PASS. Results line shows "5 PASS, 0 FAIL". Command exits 0.
  </done>
</task>

</tasks>

<verification>
cargo build passes. All four single-format arms write files to the specified --output directory when --output differs from default. Stdout behavior is preserved when --output is omitted.
</verification>

<success_criteria>
- `cargo build` succeeds with no errors or new warnings
- Each of the four single-format modes writes the expected filename to the specified --output directory
- Omitting --output continues to print to stdout (backward compatible)
- No change to --format all behavior
</success_criteria>

<output>
After completion, create `.planning/quick/260509-gwq-fix-output-flag-single-formats/260509-gwq-SUMMARY.md`
</output>
