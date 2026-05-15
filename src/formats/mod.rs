pub mod console;
pub mod cyclonedx;
pub mod spdx;

// Re-export commonly used functions
pub use console::{print_sbom, save_console_report};
#[cfg(feature = "internal")]
pub use console::save_static_analysis_report;

pub use spdx::{
    create_package_url, print_spdx_json, print_spdx_tag_value, save_spdx_json, save_spdx_tag_value,
};

pub use cyclonedx::{print_cyclonedx_json, save_cyclonedx_json};
