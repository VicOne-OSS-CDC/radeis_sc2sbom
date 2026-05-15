---
title: "AUTOSAR version extraction (BUG-03)"
slug: autosar-version-extraction
date: 2026-05-11
status: complete
---

# AUTOSAR Version Extraction (BUG-03) — Summary

## What Was Done

Replaced `unspecified` versions on AUTOSAR dependencies with real version strings.

- Added `collect_epd_versions` to `src/parsers/c/arxml.rs` — walks `*.epd` files under project root, extracts `ECUC-MODULE-DEF SHORT-NAME` + `REVISION-LABEL` via quick-xml; first-found wins across variant files
- Added `collect_doxygen_versions` — walks `.c`/`.h` files, reads first 50 lines, extracts `SW Version : X.Y.Z` from Doxygen headers; keyed by immediate parent directory name
- Updated `parse_arxml` signature to accept both version maps; resolution order: epd → doxygen → `"unspecified"`
- Updated call site in `src/scanner/mod.rs` to collect both maps once per AUTOSAR project scan
- Added 6 unit tests covering: epd extraction, variant dedup, doxygen extraction, epd-wins, doxygen-fallback, unspecified-fallback — all pass
- Also included follow-on fixes: post-walk pass in `src/scanner/mod.rs` upgrades system linker deps (`-lAdc`, `-lGpt`, SWC dirs) to autosar ecosystem using the same epd/doxygen version maps; shadowed system entries deduped

## Result

AUTOSAR_SampleProject_S32K144 scan: 17 of 18 components now show real version strings (was 0). `Det` remains `unspecified` — no `.epd` or Doxygen header exists in that project, which is the correct SBOM behavior.

## Commits

- 14e60a5 fix(autosar): extract real versions from .epd and Doxygen headers (BUG-03)
- b15067b fix(dedup): drop system linker deps shadowed by autosar ecosystem entries
- 9ba0c2b fix(autosar): upgrade system linker deps to autosar ecosystem using epd versions
- d8ca713 fix(autosar): also upgrade system deps with doxygen versions to autosar ecosystem
