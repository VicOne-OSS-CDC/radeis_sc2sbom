use crate::models::{Dependency, DependencySource};
use crate::parsers::format_source_info;
use anyhow::Result;
use quick_xml::events::Event;
use quick_xml::Reader;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;
use walkdir::WalkDir;

/// Walk `root` for *.epd files, extract ECUC-MODULE-DEF SHORT-NAME + first REVISION-LABEL.
/// Returns HashMap<module_name, version_string>.
pub fn collect_epd_versions(root: &Path) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    let epd_files: Vec<_> = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "epd")
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();

    for epd_path in epd_files {
        let content = match fs::read_to_string(&epd_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let mut reader = Reader::from_str(&content);
        reader.trim_text(true);
        let mut buf = Vec::new();

        let mut depth: usize = 0;
        let mut in_module_def = false;
        let mut module_depth: usize = 0;
        let mut in_short_name = false;
        let mut in_revision_label = false;
        let mut current_name: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    depth += 1;
                    let tag = std::str::from_utf8(e.name().as_ref())
                        .unwrap_or("")
                        .to_uppercase();
                    match tag.as_str() {
                        "ECUC-MODULE-DEF" => {
                            in_module_def = true;
                            module_depth = depth;
                            current_name = None;
                        }
                        "SHORT-NAME" if in_module_def && depth == module_depth + 1 => {
                            in_short_name = true;
                        }
                        "REVISION-LABEL" if in_module_def => {
                            in_revision_label = true;
                        }
                        _ => {}
                    }
                }
                Ok(Event::End(_)) => {
                    if in_module_def && depth == module_depth {
                        in_module_def = false;
                        current_name = None;
                        in_short_name = false;
                        in_revision_label = false;
                    } else {
                        if in_short_name {
                            in_short_name = false;
                        }
                        if in_revision_label {
                            in_revision_label = false;
                        }
                    }
                    depth = depth.saturating_sub(1);
                }
                Ok(Event::Text(ref e)) if in_short_name => {
                    let name = e.unescape().unwrap_or_default().trim().to_string();
                    if !name.is_empty() {
                        current_name = Some(name);
                    }
                    in_short_name = false;
                }
                Ok(Event::Text(ref e)) if in_revision_label => {
                    let version = e.unescape().unwrap_or_default().trim().to_string();
                    if !version.is_empty() {
                        if let Some(ref name) = current_name {
                            // First found wins — skip if already in map
                            map.entry(name.clone()).or_insert(version);
                        }
                    }
                    in_revision_label = false;
                }
                Ok(Event::Eof) => break,
                Err(_) => break,
                _ => {}
            }
            buf.clear();
        }
    }

    map
}

/// Walk `root` for *.c and *.h files, extract "SW Version : X.Y.Z" from Doxygen headers.
/// Groups by immediate parent directory name → takes first version found per dir.
/// Returns HashMap<dir_name, version_string>.
pub fn collect_doxygen_versions(root: &Path) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();

    let re = match Regex::new(r"^\s*\*\s+SW Version\s*:\s*(\S+)") {
        Ok(r) => r,
        Err(_) => return map,
    };

    let source_files: Vec<_> = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let ext = e
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("");
            ext == "c" || ext == "h"
        })
        .map(|e| e.into_path())
        .collect();

    for src_path in source_files {
        // Key = immediate parent directory name
        let dir_name = match src_path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
        {
            Some(n) => n.to_string(),
            None => continue,
        };

        // Skip if we already have a version for this directory
        if map.contains_key(&dir_name) {
            continue;
        }

        let file = match fs::File::open(&src_path) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            if i >= 50 {
                break;
            }
            let line = match line {
                Ok(l) => l,
                Err(_) => break,
            };
            if let Some(caps) = re.captures(&line) {
                let version = caps[1].to_string();
                map.entry(dir_name.clone()).or_insert(version);
                break;
            }
        }
    }

    map
}

/// Parse an AUTOSAR `.arxml` file and extract software component names as
/// AUTOSAR-ecosystem dependencies.
///
/// Handles three arxml patterns:
/// - Composition instances: `<SW-COMPONENT-PROTOTYPE><SHORT-NAME>` — SWC instances
/// - SWC type definitions: `<APPLICATION-SW-COMPONENT-TYPE>`, `<ECU-ABSTRACTION-SW-COMPONENT-TYPE>`,
///   `<SERVICE-SW-COMPONENT-TYPE>`, `<COMPOSITION-SW-COMPONENT-TYPE>`,
///   `<COMPLEX-DEVICE-DRIVER-SW-COMPONENT-TYPE>` — component type names
/// - BSW modules: `<BSW-MODULE-DESCRIPTION><SHORT-NAME>` — BSW module names
///
/// Deduplicates by name. Version is resolved from epd_versions (ECUC REVISION-LABEL),
/// falling back to doxygen_versions (SW Version header), then "unspecified".
pub fn parse_arxml(
    path: &Path,
    epd_versions: &HashMap<String, String>,
    doxygen_versions: &HashMap<String, String>,
) -> Result<Vec<Dependency>> {
    let content = fs::read_to_string(path)?;
    let source_info = format_source_info(
        "autosar/arxml",
        path,
        None,
        false,
    );

    let mut reader = Reader::from_str(&content);
    reader.trim_text(true);

    let mut deps: Vec<Dependency> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut buf = Vec::new();

    // Stack tracks which container element we are inside.
    // We extract SHORT-NAME only when inside a known component element.
    let mut in_component = false;
    let mut in_short_name = false;
    let mut depth: usize = 0;
    let mut component_depth: usize = 0;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                depth += 1;
                let tag = std::str::from_utf8(e.name().as_ref())
                    .unwrap_or("")
                    .to_uppercase();
                match tag.as_str() {
                    "SW-COMPONENT-PROTOTYPE"
                    | "BSW-MODULE-DESCRIPTION"
                    | "APPLICATION-SW-COMPONENT-TYPE"
                    | "ECU-ABSTRACTION-SW-COMPONENT-TYPE"
                    | "SERVICE-SW-COMPONENT-TYPE"
                    | "COMPOSITION-SW-COMPONENT-TYPE"
                    | "COMPLEX-DEVICE-DRIVER-SW-COMPONENT-TYPE" => {
                        in_component = true;
                        component_depth = depth;
                    }
                    "SHORT-NAME" if in_component && depth == component_depth + 1 => {
                        in_short_name = true;
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                if in_component && depth == component_depth {
                    in_component = false;
                }
                if in_short_name {
                    in_short_name = false;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Text(ref e)) if in_short_name => {
                let name = e.unescape().unwrap_or_default().trim().to_string();
                if !name.is_empty() && seen.insert(name.clone()) {
                    let version = epd_versions
                        .get(&name)
                        .or_else(|| doxygen_versions.get(&name))
                        .cloned()
                        .unwrap_or_else(|| "unspecified".to_string());
                    deps.push(Dependency {
                        name,
                        version,
                        ecosystem: "autosar".to_string(),
                        source: DependencySource::Manifest,
                        source_file: Some(source_info.clone()),
                        is_dev: false,
                        is_direct: true,
                        ..Default::default()
                    });
                }
                in_short_name = false;
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(deps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::{NamedTempFile, TempDir};

    fn write_arxml(content: &str) -> NamedTempFile {
        let mut f = NamedTempFile::with_suffix(".arxml").unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parses_swc_composition() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>Composition</SHORT-NAME>
    <ELEMENTS><COMPOSITION-SW-COMPONENT-TYPE>
      <SHORT-NAME>TempControlComposition</SHORT-NAME>
      <COMPONENTS>
        <SW-COMPONENT-PROTOTYPE>
          <SHORT-NAME>EcuAbstComp</SHORT-NAME>
        </SW-COMPONENT-PROTOTYPE>
        <SW-COMPONENT-PROTOTYPE>
          <SHORT-NAME>ApplicationSWC</SHORT-NAME>
        </SW-COMPONENT-PROTOTYPE>
      </COMPONENTS>
    </COMPOSITION-SW-COMPONENT-TYPE></ELEMENTS>
  </AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"EcuAbstComp"), "missing EcuAbstComp: {:?}", names);
        assert!(names.contains(&"ApplicationSWC"), "missing ApplicationSWC: {:?}", names);
        assert!(deps.iter().all(|d| d.ecosystem == "autosar"));
        assert!(deps.iter().all(|d| d.is_direct));
    }

    #[test]
    fn parses_bsw_module_description() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>AUTOSAR_Mcu</SHORT-NAME>
    <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>BswModuleDescriptions</SHORT-NAME>
      <ELEMENTS>
        <BSW-MODULE-DESCRIPTION>
          <SHORT-NAME>Mcu</SHORT-NAME>
        </BSW-MODULE-DESCRIPTION>
      </ELEMENTS>
    </AR-PACKAGE></AR-PACKAGES>
  </AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"Mcu"), "missing Mcu: {:?}", names);
    }

    #[test]
    fn parses_swc_type_definitions() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>SWComponent</SHORT-NAME><ELEMENTS>
    <APPLICATION-SW-COMPONENT-TYPE>
      <SHORT-NAME>TemControlSWC</SHORT-NAME>
    </APPLICATION-SW-COMPONENT-TYPE>
    <ECU-ABSTRACTION-SW-COMPONENT-TYPE>
      <SHORT-NAME>TempHBridgeSWC</SHORT-NAME>
    </ECU-ABSTRACTION-SW-COMPONENT-TYPE>
  </ELEMENTS></AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert!(names.contains(&"TemControlSWC"), "missing TemControlSWC: {:?}", names);
        assert!(names.contains(&"TempHBridgeSWC"), "missing TempHBridgeSWC: {:?}", names);
    }

    #[test]
    fn deduplicates_repeated_names() {
        // Comp (COMPOSITION-SW-COMPONENT-TYPE) + EngineCtrl (SW-COMPONENT-PROTOTYPE, deduplicated) = 2
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>Root</SHORT-NAME><ELEMENTS>
    <COMPOSITION-SW-COMPONENT-TYPE><SHORT-NAME>Comp</SHORT-NAME><COMPONENTS>
      <SW-COMPONENT-PROTOTYPE><SHORT-NAME>EngineCtrl</SHORT-NAME></SW-COMPONENT-PROTOTYPE>
      <SW-COMPONENT-PROTOTYPE><SHORT-NAME>EngineCtrl</SHORT-NAME></SW-COMPONENT-PROTOTYPE>
    </COMPONENTS></COMPOSITION-SW-COMPONENT-TYPE>
  </ELEMENTS></AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        let names: Vec<&str> = deps.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(deps.len(), 2, "expected Comp + deduplicated EngineCtrl: {:?}", deps);
        assert!(names.contains(&"Comp"));
        assert!(names.contains(&"EngineCtrl"));
    }

    #[test]
    fn empty_arxml_returns_empty() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        assert!(deps.is_empty());
    }

    // --- New tests for BUG-03 ---

    #[test]
    fn collect_epd_versions_extracts_revision_label() {
        let dir = TempDir::new().unwrap();
        let epd_content = r#"<?xml version="1.0"?>
<AUTOSAR>
  <AR-PACKAGES>
    <AR-PACKAGE>
      <ELEMENTS>
        <ECUC-MODULE-DEF>
          <SHORT-NAME>Mcu</SHORT-NAME>
          <ADMIN-DATA>
            <DOC-REVISIONS>
              <DOC-REVISION>
                <REVISION-LABEL>1.0.1</REVISION-LABEL>
              </DOC-REVISION>
            </DOC-REVISIONS>
          </ADMIN-DATA>
        </ECUC-MODULE-DEF>
      </ELEMENTS>
    </AR-PACKAGE>
  </AR-PACKAGES>
</AUTOSAR>"#;
        let epd_path = dir.path().join("Mcu.epd");
        fs::write(&epd_path, epd_content).unwrap();

        let map = collect_epd_versions(dir.path());
        assert_eq!(map.get("Mcu"), Some(&"1.0.1".to_string()), "expected Mcu -> 1.0.1, got {:?}", map);
    }

    #[test]
    fn collect_epd_versions_deduplicates_variants() {
        let dir = TempDir::new().unwrap();
        let make_epd = |version: &str| {
            format!(r#"<?xml version="1.0"?>
<AUTOSAR>
  <AR-PACKAGES><AR-PACKAGE><ELEMENTS>
    <ECUC-MODULE-DEF>
      <SHORT-NAME>Mcu</SHORT-NAME>
      <ADMIN-DATA><DOC-REVISIONS><DOC-REVISION>
        <REVISION-LABEL>{}</REVISION-LABEL>
      </DOC-REVISION></DOC-REVISIONS></ADMIN-DATA>
    </ECUC-MODULE-DEF>
  </ELEMENTS></AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#, version)
        };
        fs::write(dir.path().join("Mcu_S32K144.epd"), make_epd("1.0.1")).unwrap();
        fs::write(dir.path().join("Mcu_S32K148.epd"), make_epd("2.0.0")).unwrap();

        let map = collect_epd_versions(dir.path());
        // Only one entry for Mcu — first found wins
        assert_eq!(map.len(), 1, "expected exactly 1 entry, got {:?}", map);
        assert!(map.contains_key("Mcu"), "expected Mcu key: {:?}", map);
        // The value is one of the two versions (whichever file was iterated first)
        let v = map.get("Mcu").unwrap();
        assert!(v == "1.0.1" || v == "2.0.0", "unexpected version: {}", v);
    }

    #[test]
    fn collect_doxygen_versions_extracts_sw_version() {
        let dir = TempDir::new().unwrap();
        let comp_dir = dir.path().join("Sensors");
        fs::create_dir(&comp_dir).unwrap();
        let c_content = r#"/**
*   @file       SwcSensors.c
*
*   SW Version : 2.3.0
*   Build Version : something
*/
int main() { return 0; }
"#;
        fs::write(comp_dir.join("SwcSensors.c"), c_content).unwrap();

        let map = collect_doxygen_versions(dir.path());
        assert_eq!(
            map.get("Sensors"),
            Some(&"2.3.0".to_string()),
            "expected Sensors -> 2.3.0, got {:?}",
            map
        );
    }

    #[test]
    fn parse_arxml_uses_epd_version() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>Root</SHORT-NAME>
    <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>BswModuleDescriptions</SHORT-NAME>
      <ELEMENTS>
        <BSW-MODULE-DESCRIPTION>
          <SHORT-NAME>Mcu</SHORT-NAME>
        </BSW-MODULE-DESCRIPTION>
      </ELEMENTS>
    </AR-PACKAGE></AR-PACKAGES>
  </AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let mut epd = HashMap::new();
        epd.insert("Mcu".to_string(), "1.0.1".to_string());
        let deps = parse_arxml(f.path(), &epd, &HashMap::new()).unwrap();
        let mcu = deps.iter().find(|d| d.name == "Mcu").expect("Mcu not found");
        assert_eq!(mcu.version, "1.0.1");
    }

    #[test]
    fn parse_arxml_falls_back_to_doxygen() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>Root</SHORT-NAME><ELEMENTS>
    <SW-COMPONENT-PROTOTYPE>
      <SHORT-NAME>Sensors</SHORT-NAME>
    </SW-COMPONENT-PROTOTYPE>
  </ELEMENTS></AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let mut dox = HashMap::new();
        dox.insert("Sensors".to_string(), "1.0.1".to_string());
        let deps = parse_arxml(f.path(), &HashMap::new(), &dox).unwrap();
        let sensors = deps.iter().find(|d| d.name == "Sensors").expect("Sensors not found");
        assert_eq!(sensors.version, "1.0.1");
    }

    #[test]
    fn parse_arxml_unspecified_when_no_version() {
        let f = write_arxml(r#"<?xml version="1.0"?>
<AUTOSAR xmlns="http://autosar.org/schema/r4.0">
  <AR-PACKAGES><AR-PACKAGE><SHORT-NAME>Root</SHORT-NAME><ELEMENTS>
    <SW-COMPONENT-PROTOTYPE>
      <SHORT-NAME>Unknown</SHORT-NAME>
    </SW-COMPONENT-PROTOTYPE>
  </ELEMENTS></AR-PACKAGE></AR-PACKAGES>
</AUTOSAR>"#);
        let deps = parse_arxml(f.path(), &HashMap::new(), &HashMap::new()).unwrap();
        let dep = deps.iter().find(|d| d.name == "Unknown").expect("Unknown not found");
        assert_eq!(dep.version, "unspecified");
    }
}
