//! C++ ecosystem parsers
//!
//! This module provides parsers for C++ package managers:
//! - vcpkg: Microsoft's C++ package manager (vcpkg.json manifest)
//! - Conan: C/C++ package manager (conan.lock, conanfile.txt, conanfile.py)
//!
//! v1.0.0: vcpkg.json manifest parsing
//! v1.0.1+: vcpkg.lock, metadata fetching (deferred)
//! v1.0.2: Conan package manager support

pub mod conan;
pub mod conan_manifest;
pub mod vcpkg;

// Re-export parser functions
pub use conan::parse_conan_lock;
pub use conan_manifest::{parse_conanfile_py, parse_conanfile_txt};
pub use vcpkg::parse_vcpkg_json;
