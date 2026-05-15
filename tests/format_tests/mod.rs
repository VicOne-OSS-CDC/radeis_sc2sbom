// Format test modules
pub mod cyclonedx_tests;
pub mod spdx_tests;
pub mod spdx_validation_tests;
#[cfg(feature = "internal")]
pub mod sast_report_tests;
#[cfg(feature = "internal")]
pub mod sarif_tests;
