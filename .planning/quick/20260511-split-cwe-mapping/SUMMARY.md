---
status: complete
---

Moved `cwe_name` from `src/formats/console.rs` into `src/vulnerability/cwe_map.rs`. Re-exported via `src/vulnerability/mod.rs` as `pub(crate)`. Updated imports in `console.rs` and `sarif.rs`. All tests pass.
