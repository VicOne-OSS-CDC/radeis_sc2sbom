//! Meson build system parser
//!
//! Parses meson.build files and .wrap subproject files to extract dependencies.
//!
//! ## Supported formats:
//! - `meson.build`: dependency() calls, cc.find_library() calls, subproject() calls
//! - `*.wrap` files: WrapDB subproject definitions (wrap-file and wrap-git)
//!
//! ## Ecosystems:
//! - `"meson"`: For dependency() calls (pkg-config packages)
//! - `"meson-wrap"`: For .wrap subproject dependencies
//!
//! ## Limitations (v1.0.4):
//! - No variable tracking
//! - No conditional evaluation
//! - Regex-based extraction (no full AST parsing)

pub mod meson_build;
pub mod wrap;

pub use meson_build::parse_meson_build;
pub use wrap::parse_all_wraps;
