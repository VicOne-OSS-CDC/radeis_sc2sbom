---
quick_id: 260511-giz
slug: split-cwe-mapping
title: Split CWE ID→name mapping out of console.rs
date: 2026-05-11
status: in-progress
---

# Split CWE ID→name mapping out of console.rs

## Goal
Move `cwe_name()` from `src/formats/console.rs` into its own module (`src/vulnerability/cwe_map.rs`) since it is scanner/vulnerability domain logic, not console formatting. Update all callers.

## Tasks

- [ ] Create `src/vulnerability/cwe_map.rs` with `pub(crate) fn cwe_name`
- [ ] Re-export from `src/vulnerability/mod.rs`
- [ ] Remove `cwe_name` from `src/formats/console.rs`
- [ ] Update `src/formats/sarif.rs` import (`use super::console::cwe_name` → `use crate::vulnerability::cwe_name`)
- [ ] Update `src/formats/console.rs` internal usages to use the local import
- [ ] Verify: `cargo build --features internal` passes
- [ ] Verify: `cargo test --features internal` passes
- [ ] Atomic commit

## Constraints
- Keep function signature identical: `pub(crate) fn cwe_name(cwe_id: u32) -> &'static str`
- No behavior changes — pure refactor
