//! AUTOSAR BSW component classification (Phase 7, v1.0.15).
//!
//! Reads a curated BSW module list from a bundled YAML config and matches
//! dependency names case-insensitively. Matched components have their
//! `ecosystem` set to `"autosar"` and gain `AutosarMetadata`.

use crate::models::{AutosarMetadata, Dependency};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Bundled BSW module list. Compiled into the binary at build time;
/// `include_str!` resolves relative to this source file.
const DEFAULT_BSW_MODULES: &str = include_str!("bsw_modules.yaml");

/// One BSW module entry — layer + platform are read directly from the
/// YAML config (D-04, D-05, D-06). Forward-compatible: serde ignores
/// unknown fields by default, so future YAML additions (e.g. supplier
/// in Phase 8) will not break this struct.
#[derive(Debug, Deserialize)]
pub struct BswModuleEntry {
    pub layer: String,
    pub platform: String,
}

/// Parsed BSW configuration with O(1) case-insensitive lookup.
///
/// Internal layout: `HashMap<lowercase_name, (canonical_name, entry)>`.
/// Keys are normalized to lowercase once at parse time so each lookup
/// is a single `to_lowercase()` + HashMap probe (D-01).
pub struct BswConfig {
    modules: HashMap<String, (String, BswModuleEntry)>,
}

impl BswConfig {
    /// Load the bundled default config. Panics only if the bundled
    /// YAML is malformed — which would be a build-time defect, caught
    /// by tests in this plan.
    pub fn load_bundled() -> Self {
        Self::parse(DEFAULT_BSW_MODULES).expect("bundled BSW config is always valid")
    }

    /// Load a user-provided BSW config from disk. Returns Err if the
    /// file cannot be read or the YAML cannot be parsed (D-03 override).
    pub fn load_from_file(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::parse(&content)
    }

    fn parse(yaml: &str) -> anyhow::Result<Self> {
        let raw: HashMap<String, BswModuleEntry> = serde_yaml::from_str(yaml)?;
        let modules = raw
            .into_iter()
            .map(|(k, v)| (k.to_lowercase(), (k, v)))
            .collect();
        Ok(BswConfig { modules })
    }

    /// Case-insensitive exact-match lookup (D-01). Returns
    /// `Some((canonical_name, entry))` on hit, `None` otherwise.
    /// Substring lookups (e.g. `"NvM_Stub"`) return `None`.
    pub fn lookup(&self, name: &str) -> Option<(&str, &BswModuleEntry)> {
        self.modules
            .get(&name.to_lowercase())
            .map(|(canonical, entry)| (canonical.as_str(), entry))
    }
}

/// Classify dependencies against the BSW config. Mutates each matching
/// `Dependency` to set `ecosystem = "autosar"` (D-07) and populate
/// `autosar_metadata` (D-11). Non-matching dependencies are left
/// untouched (D-02).
///
/// Caller is responsible for gating this on `ScanContext.is_autosar`
/// (Phase 6) — see Plan 07-03 for the wiring.
pub fn classify_autosar_components(deps: &mut [Dependency], config: &BswConfig) {
    for dep in deps.iter_mut() {
        if let Some((canonical_name, entry)) = config.lookup(&dep.name) {
            dep.ecosystem = "autosar".to_string();
            dep.autosar_metadata = Some(AutosarMetadata {
                module_name: canonical_name.to_string(),
                layer: entry.layer.clone(),
                platform: entry.platform.clone(),
            });
        }
    }
}
