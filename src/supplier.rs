//! Supplier configuration loader for AUTOSAR components (Phase 8, v1.0.15).
//!
//! Loads a flat YAML map of component-name → supplier-string and answers
//! per-component lookup queries. Used by the SPDX and CycloneDX formatters
//! to emit the `autosar:supplier` SBOM property. Components without an
//! entry resolve to `NOASSERTION` at the call site (this module returns
//! `None` and the caller decides the fallback string).

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::Path;

/// Resolves AUTOSAR component names to supplier strings via a user-provided
/// YAML config file. Constructed once in `main.rs` after `Args::parse()`.
#[derive(Debug)]
pub struct SupplierResolver {
    mappings: HashMap<String, String>,
}

impl SupplierResolver {
    /// Load and parse the supplier config YAML file at `path`.
    ///
    /// Hard-errors (per D-02) if the file cannot be read or if the YAML
    /// fails to deserialize as `HashMap<String, String>`. The error chain
    /// always includes the file path for diagnostics.
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path).with_context(|| {
            format!("Failed to read supplier config: {}", path.display())
        })?;
        let mappings: HashMap<String, String> = serde_yaml::from_str(&content)
            .with_context(|| {
                format!("Failed to parse supplier config YAML: {}", path.display())
            })?;
        Ok(Self { mappings })
    }

    /// Look up a component name. Returns `Some(&str)` if the name appears
    /// in the config, `None` otherwise. Matching is exact and
    /// case-sensitive (per D-05).
    pub fn lookup(&self, component_name: &str) -> Option<&str> {
        self.mappings.get(component_name).map(String::as_str)
    }

    /// Construct a `SupplierResolver` from an in-memory map. Bypasses the
    /// YAML file load. Callable from integration tests under `tests/` (not
    /// `#[cfg(test)]` because those tests cannot see cfg(test) items in src/).
    pub fn from_map(mappings: std::collections::HashMap<String, String>) -> Self {
        Self { mappings }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_temp_yaml(contents: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().expect("create tempfile");
        f.write_all(contents.as_bytes()).expect("write tempfile");
        f
    }

    #[test]
    fn test_load_valid_yaml() {
        let f = write_temp_yaml("NvM: \"Vector Informatik\"\nCom: \"EB\"\n");
        let resolver = SupplierResolver::load(f.path()).expect("load valid yaml");
        assert_eq!(resolver.lookup("NvM"), Some("Vector Informatik"));
        assert_eq!(resolver.lookup("Com"), Some("EB"));
    }

    #[test]
    fn test_load_missing_file() {
        let path = Path::new("/nonexistent/path/to/supplier-config-does-not-exist.yaml");
        let err = SupplierResolver::load(path).expect_err("missing file must error");
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("Failed to read supplier config"),
            "error chain should mention read failure, got: {}",
            msg
        );
        assert!(
            msg.contains("supplier-config-does-not-exist"),
            "error chain should include path, got: {}",
            msg
        );
    }

    #[test]
    fn test_load_invalid_yaml() {
        let f = write_temp_yaml(": : :\n  - not a map\n");
        let err = SupplierResolver::load(f.path()).expect_err("invalid yaml must error");
        let msg = format!("{:#}", err);
        assert!(
            msg.contains("Failed to parse supplier config YAML"),
            "error chain should mention parse failure, got: {}",
            msg
        );
    }

    #[test]
    fn test_lookup_found() {
        let mut mappings = HashMap::new();
        mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
        let resolver = SupplierResolver { mappings };
        assert_eq!(resolver.lookup("NvM"), Some("Vector Informatik"));
    }

    #[test]
    fn test_lookup_not_found() {
        let mut mappings = HashMap::new();
        mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
        let resolver = SupplierResolver { mappings };
        assert_eq!(resolver.lookup("Com"), None);
    }

    #[test]
    fn test_lookup_case_sensitive() {
        let mut mappings = HashMap::new();
        mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
        let resolver = SupplierResolver { mappings };
        assert_eq!(resolver.lookup("nvm"), None);
        assert_eq!(resolver.lookup("NVM"), None);
        assert_eq!(resolver.lookup("NvM"), Some("Vector Informatik"));
    }
}
