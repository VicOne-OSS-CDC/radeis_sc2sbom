pub mod arxml;
pub mod autotools;
pub mod known_licenses;
pub mod library_json;
pub mod makefile;
pub mod makefile_am;
pub mod mk_file;
pub mod pkgconfig;
pub mod pkgconfig_detector;
pub mod so_scanner;
pub mod vendored_3rdparty;

pub use arxml::{collect_doxygen_versions, collect_epd_versions, parse_arxml};
pub use autotools::parse_configure_ac;
pub use library_json::parse_library_json;
pub use makefile::parse_makefile;
pub use makefile_am::parse_makefile_am;
pub use mk_file::parse_mk_files_as_dependencies;
pub use pkgconfig::parse_pc_file;
#[allow(unused_imports)]
// re-exported for integration tests (tests/parser_tests/c_tests.rs:113)
pub use pkgconfig_detector::extract_pkgconfig_from_makefile;
pub use vendored_3rdparty::scan_vendored_3rdparty;
