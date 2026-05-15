---
title: "AUTOSAR version extraction (BUG-03)"
slug: autosar-version-extraction
date: 2026-05-11
status: complete
---

# AUTOSAR Version Extraction (BUG-03)

## Goal

Replace `unspecified` versions on AUTOSAR dependencies with real version strings by:
1. Parsing `.epd` files for `ECUC-MODULE-DEF SHORT-NAME` + `REVISION-LABEL`
2. Parsing Doxygen-style C/H file headers for `SW Version : X.Y.Z`

Both version maps feed into `parse_arxml` — if a dep name matches, its version is set.

## Context

- `.epd` files are standard AUTOSAR ECUC definition files, found at:
  - `<project>/autosar/*.epd` (project-level modules)
  - `<project>/plugins/<Name>_TS_T40D2M10I1R0/autosar/*.epd` (BSW plugin modules)
  - Multiple `.epd` files can share the same `SHORT-NAME` (different MCU variants) — use first found
- Doxygen header pattern: `* +SW Version +: +(\S+)` in `.c`/`.h` files
  - Group by immediate parent directory name as the component name
  - Use the version found in that directory (any file is representative)
- Version map is built per-project scan, then passed into `parse_arxml`

## Tasks

### Task 1: Add `collect_epd_versions` to `src/parsers/c/arxml.rs`

```rust
/// Walk `root` for *.epd files, extract ECUC-MODULE-DEF SHORT-NAME + first REVISION-LABEL.
/// Returns HashMap<module_name, version_string>.
pub fn collect_epd_versions(root: &Path) -> HashMap<String, String>
```

- Use `walkdir` to find all `*.epd` under `root`
- Parse with quick-xml: same Reader approach as `parse_arxml`
- Track state: `in_module_def: bool`, `in_short_name: bool`, `in_revision_label: bool`, `current_name: Option<String>`
- On `ECUC-MODULE-DEF` Start → set `in_module_def = true`, record depth
- On `SHORT-NAME` Start (depth == module_depth+1, in_module_def) → `in_short_name = true`
- On `REVISION-LABEL` Start (in_module_def) → `in_revision_label = true`
- On Text in_short_name → store as current_name
- On Text in_revision_label → if current_name is Some and not already in map, insert
- On ECUC-MODULE-DEF End → reset state
- Skip if name already in map (first wins across variant files)

### Task 2: Add `collect_doxygen_versions` to `src/parsers/c/arxml.rs`

```rust
/// Walk `root` for *.c and *.h files, extract "SW Version : X.Y.Z" from Doxygen headers.
/// Groups by immediate parent directory name → takes first version found per dir.
/// Returns HashMap<dir_name, version_string>.
pub fn collect_doxygen_versions(root: &Path) -> HashMap<String, String>
```

- Use `walkdir` to find `.c`/`.h` files under `root`
- Read only first 50 lines of each file (version comment is always in header)
- Regex: `^\s*\*\s+SW Version\s*:\s*(\S+)` (use the `regex` crate already in Cargo.toml)
- Key = file's parent directory name (last component), value = captured version string
- First match per directory wins; skip if already in map

### Task 3: Update `parse_arxml` signature to accept version maps

Change signature to:
```rust
pub fn parse_arxml(
    path: &Path,
    epd_versions: &HashMap<String, String>,
    doxygen_versions: &HashMap<String, String>,
) -> Result<Vec<Dependency>>
```

When building each `Dependency`, resolve version:
```
version = epd_versions.get(&name)
    .or_else(|| doxygen_versions.get(&name))
    .cloned()
    .unwrap_or_else(|| "unspecified".to_string())
```

### Task 4: Update call sites

- `src/scanner/mod.rs`: before calling `parse_arxml`, call `collect_epd_versions(project_root)` and `collect_doxygen_versions(project_root)` once per AUTOSAR project scan, pass results in
- `src/parsers/mod.rs`: re-export `collect_epd_versions`, `collect_doxygen_versions`
- All existing tests in `arxml.rs` must pass empty `HashMap`s for both params

### Task 5: Add tests

In `src/parsers/c/arxml.rs` tests:

1. `collect_epd_versions_extracts_revision_label` — write a temp `.epd` with `ECUC-MODULE-DEF/SHORT-NAME=Mcu` + `REVISION-LABEL=1.0.1`, assert map contains `Mcu → 1.0.1`
2. `collect_epd_versions_deduplicates_variants` — two temp `.epd` files both with `SHORT-NAME=Mcu`, different `REVISION-LABEL`; assert only first is kept
3. `collect_doxygen_versions_extracts_sw_version` — write a temp `.c` with `* SW Version : 2.3.0` in header; assert map contains dir_name → `2.3.0`
4. `parse_arxml_uses_epd_version` — arxml with `SW-COMPONENT-PROTOTYPE/SHORT-NAME=Mcu`; epd_versions `{Mcu: 1.0.1}`; assert dep version is `1.0.1`
5. `parse_arxml_falls_back_to_doxygen` — arxml with name=`Sensors`; no epd, doxygen_versions `{Sensors: 1.0.1}`; assert version `1.0.1`
6. `parse_arxml_unspecified_when_no_version` — no maps; assert version `unspecified`

### Task 6: Update ROADMAP.md

Add BUG-03 entry to Phase 17 Known Bugs section:
```
- **BUG-03:** AUTOSAR dependencies always show `@ unspecified`. `.epd` files contain `REVISION-LABEL` per `ECUC-MODULE-DEF`; Doxygen C/H headers contain `SW Version`. Both should be parsed and used to populate real versions. *(fixed 2026-05-11)*
```

## Success Criteria

- `cargo test -p radeis_sc2sbom parsers::c::arxml` — all tests pass
- `cargo build --features internal` — clean compile
- Running scan on AUTOSAR_SampleProject_S32K144 shows `IntegrationFramework @ 1.0.0` (from epd) and BSW modules with real versions instead of `unspecified`

## Files to Change

- `src/parsers/c/arxml.rs` — add two collection fns, update parse_arxml signature + tests
- `src/parsers/mod.rs` — re-export new fns
- `src/scanner/mod.rs` — collect version maps before calling parse_arxml
- `.planning/ROADMAP.md` — add BUG-03
