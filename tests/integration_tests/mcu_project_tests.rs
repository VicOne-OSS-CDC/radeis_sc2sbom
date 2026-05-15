#[test]
fn test_op_kraken_mcu_project_detection() {
    use radeis_sc2sbom::parsers::deduplicate_dependencies;
    use std::path::Path;

    let project_path = Path::new("customer_report/quanta/MCUTest/op_kraken_04a_uart_360x360");
    // Skip if customer project not present (CI)
    if !project_path.exists() {
        eprintln!("Skipping: customer project not present");
        return;
    }

    // 1. library.json
    let lib_json_path = project_path.join("lvgl-simulator/lv_drivers/library.json");
    assert!(lib_json_path.exists(), "library.json fixture must exist");
    let deps = radeis_sc2sbom::parsers::parse_library_json(&lib_json_path).unwrap();
    assert_eq!(deps.len(), 1);
    assert_eq!(deps[0].name, "lv_drivers");
    assert_eq!(deps[0].version, "7.11.0");
    assert_eq!(deps[0].ecosystem, "vendored");

    // 2. MicroPython detection in gui_guider.py
    let gui_py = project_path.join("generated/gui_guider.py");
    if gui_py.exists() {
        let py_deps = radeis_sc2sbom::parsers::scan_python_imports(&gui_py).unwrap();
        let pip_deps: Vec<_> = py_deps.iter().filter(|d| d.ecosystem == "pip").collect();
        assert!(
            pip_deps.is_empty(),
            "gui_guider.py is MicroPython — no imports should be pip, found: {:?}",
            pip_deps.iter().map(|d| &d.name).collect::<Vec<_>>()
        );
    }

    // 3. SDL2 deduplication
    let sdl2_system = radeis_sc2sbom::models::Dependency {
        name: "SDL2".to_string(),
        version: "unspecified".to_string(),
        ecosystem: "system".to_string(),
        source: radeis_sc2sbom::models::DependencySource::Manifest,
        is_dev: false,
        is_direct: true,
        checksum_sha256: None,
        checksum_sha512: None,
        license: None,
        author: None,
        maintainers: None,
        repository_url: None,
        homepage_url: None,
        source_file: None,
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: String::new(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    };
    let sdl2_pkgconfig = radeis_sc2sbom::models::Dependency {
        name: "sdl2".to_string(),
        version: "2.0.12".to_string(),
        ecosystem: "pkg-config".to_string(),
        source: radeis_sc2sbom::models::DependencySource::Manifest,
        is_dev: false,
        is_direct: true,
        checksum_sha256: None,
        checksum_sha512: None,
        license: None,
        author: None,
        maintainers: None,
        repository_url: None,
        homepage_url: None,
        source_file: None,
        scope: radeis_sc2sbom::models::DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: String::new(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    };
    let deduped = deduplicate_dependencies(vec![sdl2_system, sdl2_pkgconfig]);
    assert_eq!(deduped.len(), 1);
    assert_eq!(deduped[0].version, "2.0.12");
    assert_eq!(deduped[0].ecosystem, "pkg-config");
}
