//! CMake ecosystem parsers
//!
//! This module provides parsers for CMake-based dependency declarations:
//! - FetchContent_Declare: Modern CMake 3.11+ dependency management
//! - ExternalProject_Add: Legacy CMake external project integration
//!
//! ## Features (v1.0.1)
//! - Parse FetchContent_Declare blocks from CMakeLists.txt
//! - Parse ExternalProject_Add blocks
//! - Extract GIT_REPOSITORY, GIT_TAG, URL, URL_HASH
//! - Static parsing (no CMake execution required)
//! - Skip dependencies with CMake variables (cannot be resolved statically)
//!
//! ## Limitations
//! - Cannot resolve CMake variables (${VAR}) - requires CMake execution
//! - Multi-line arguments with escaped newlines may not parse correctly
//! - Does not follow include() directives (v1.0.2+ feature)

pub mod external_project;
pub mod fetchcontent;
pub mod utils;

use crate::models::dependency::Dependency;
use std::fs;
use std::path::Path;

pub use external_project::parse_external_project;
pub use fetchcontent::parse_fetchcontent;

/// Parse all CMake dependency declarations from a file
///
/// Detects both FetchContent_Declare and ExternalProject_Add blocks.
/// Returns an empty vector if parsing fails (non-fatal errors).
pub fn parse_cmake_file(path: &Path) -> Result<Vec<Dependency>, Box<dyn std::error::Error>> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Failed to read {}: {}", path.display(), e);
            return Ok(Vec::new());
        }
    };

    let mut dependencies = Vec::new();

    // Parse FetchContent_Declare blocks
    match parse_fetchcontent(&content, path) {
        Ok(deps) => dependencies.extend(deps),
        Err(e) => eprintln!(
            "Warning: Failed to parse FetchContent in {}: {}",
            path.display(),
            e
        ),
    }

    // Parse ExternalProject_Add blocks
    match parse_external_project(&content, path) {
        Ok(deps) => dependencies.extend(deps),
        Err(e) => eprintln!(
            "Warning: Failed to parse ExternalProject in {}: {}",
            path.display(),
            e
        ),
    }

    Ok(dependencies)
}
