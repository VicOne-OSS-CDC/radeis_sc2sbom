// Integration test file that imports all test modules
// Required for Rust's test discovery to find tests in subdirectories

#[path = "parser_tests/mod.rs"]
mod parser_tests;

#[path = "format_tests/mod.rs"]
mod format_tests;

#[path = "scanner_tests/mod.rs"]
mod scanner_tests;

#[path = "model_tests/mod.rs"]
mod model_tests;

#[path = "error_tests/mod.rs"]
mod error_tests;

#[path = "utility_tests/mod.rs"]
mod utility_tests;

#[path = "integration_tests/mod.rs"]
mod integration_tests;

#[cfg(feature = "internal")]

#[path = "classifier_tests/mod.rs"]
mod classifier_tests;

#[path = "common/mod.rs"]
mod common;
