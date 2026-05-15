---
phase: 260509-gwq
plan: 01
status: complete
started: 2026-05-09
completed: 2026-05-09
---

# Quick Task 260509-gwq: Fix --output Flag for Single Formats

## What Was Done

Honoured `--output` in all four single-format match arms (`spdx-json`, `cyclonedx-json`, `spdx-tag-value`, `console`). When `--output` is provided, each arm writes the expected filename (`{project}_{ext}`) to the specified directory using the appropriate `save_*` function. Omitting `--output` preserves the existing stdout behaviour.

A follow-up fix changed `--output` from a defaulted `String` to `Option<String>` so the guard correctly distinguishes "not supplied" from a user-supplied path of `"out"`.

## Key Commits

- `da82d26` — initial implementation: add `args.output != "out"` guards in four arms
- `4175237` — fix guard: `--output` matched literal `"out"`, not `"./out"` — always wrote to file
- `e3ab624` — refactor: `--output` changed to `Option<String>`; guard is now `if let Some(ref out) = args.output`

## Verification

All four single-format arms write files to the specified directory. Stdout fallback confirmed when `--output` is omitted. `cargo build` clean.
