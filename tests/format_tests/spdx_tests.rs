#[cfg(feature = "internal")]
use radeis_sc2sbom::formats::spdx::{convert_to_spdx, save_spdx_tag_value};
use radeis_sc2sbom::models::{
    AIModelMetadata, AutosarMetadata, Dependency, DependencyScope, DependencySource,
    RosPackageMetadata, RosPackageWithDeps, Sbom, SubModelInfo,
};
#[cfg(feature = "internal")]
use std::path::PathBuf;

// ---- Test helpers ----

fn make_dep(name: &str, version: &str, ecosystem: &str) -> Dependency {
    Dependency {
        name: name.to_string(),
        version: version.to_string(),
        ecosystem: ecosystem.to_string(),
        source: DependencySource::LockFile,
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
        scope: DependencyScope::default(),
        scope_confidence: 0.0,
        scope_reason: "Not classified".to_string(),
        ai_model_metadata: None,
        autosar_metadata: None,
        ..Default::default()
    }
}

fn make_sbom(deps: Vec<Dependency>) -> Sbom {
    Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: deps,
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    }
}

// ---- End test helpers ----

// v0.8.0 Hierarchical Relationships Tests

#[test]
fn test_hierarchical_relationships_basic() {
    // Create SBOM with 3 dependencies
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![
            Dependency {
                name: "axios".to_string(),
                version: "1.0.0".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                checksum_sha256: None,
                checksum_sha512: None,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: Some("Identified by javascript/packagelockjson".to_string()),
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
            Dependency {
                name: "serde".to_string(),
                version: "1.0.0".to_string(),
                ecosystem: "cargo".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                checksum_sha256: None,
                checksum_sha512: None,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: Some("Identified by rust/cargolock".to_string()),
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
            Dependency {
                name: "requests".to_string(),
                version: "2.28.0".to_string(),
                ecosystem: "pip".to_string(),
                source: DependencySource::Manifest,
                is_dev: false,
                is_direct: true,
                checksum_sha256: None,
                checksum_sha512: None,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: Some("Identified by python/requirements".to_string()),
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
        ],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    // Convert to SPDX with hierarchical mode
    let spdx_doc = convert_to_spdx(&sbom, false, None);

    // Verify hierarchical structure:
    // 1 DESCRIBES (Document -> main) + 3 CONTAINS (main -> deps) = 4
    assert_eq!(
        spdx_doc.relationships.len(),
        4,
        "Expected 4 hierarchical relationships"
    );

    // Verify Document DESCRIBES main package
    let describes_rel = spdx_doc
        .relationships
        .iter()
        .find(|r| r.relationship_type == "DESCRIBES")
        .expect("No DESCRIBES relationship found");
    assert_eq!(describes_rel.spdx_element_id, "SPDXRef-DOCUMENT");
    assert_eq!(describes_rel.related_spdx_element, "SPDXRef-Package-test");

    // Verify main CONTAINS all 3 dependencies
    let main_contains_count = spdx_doc
        .relationships
        .iter()
        .filter(|r| {
            r.spdx_element_id == "SPDXRef-Package-test" && r.relationship_type == "CONTAINS"
        })
        .count();
    assert_eq!(main_contains_count, 3, "main package should CONTAIN 3 deps");

    // Verify no CONTAINS NOASSERTION relationships (removed in v1.0.8)
    let noassertion_count = spdx_doc
        .relationships
        .iter()
        .filter(|r| r.related_spdx_element == "NOASSERTION")
        .count();
    assert_eq!(
        noassertion_count, 0,
        "CONTAINS NOASSERTION relationships must not be emitted"
    );
}

#[test]
fn test_spdx_sourceinfo_field() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true,
            checksum_sha256: None,
            checksum_sha512: None,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: Some("Identified by the javascript/packagelockjson extractor from /path/to/package-lock.json".to_string()),
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);

    // Find the axios package (skip main package)
    let axios_pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .expect("axios package not found");

    // Verify sourceInfo field is populated
    assert!(
        axios_pkg.source_info.is_some(),
        "sourceInfo should be populated"
    );
    let source_info = axios_pkg.source_info.as_ref().unwrap();
    assert!(
        source_info.starts_with("Identified by the javascript/packagelockjson extractor"),
        "sourceInfo format incorrect: {}",
        source_info
    );
    assert!(
        source_info.contains("package-lock.json"),
        "sourceInfo should contain filename: {}",
        source_info
    );
}

#[test]
fn test_main_package_creation() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);

    // Verify main package exists
    let main_pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.spdx_id == "SPDXRef-Package-test")
        .expect("main package not found");

    assert_eq!(main_pkg.name, "test");
    assert_eq!(main_pkg.version_info, "NOASSERTION");
    assert_eq!(main_pkg.download_location, "NOASSERTION");
}

// v0.8.0 UUID-based SPDXID Tests
// Note: These tests verify the UUID pattern after it's implemented in Phase 9

#[test]
fn test_uuid_based_spdx_ids() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![
            Dependency {
                name: "axios".to_string(),
                version: "1.0.0".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: None,
                checksum_sha256: None,
                checksum_sha512: None,
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
            Dependency {
                name: "@types/node".to_string(),
                version: "18.0.0".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: None,
                checksum_sha256: None,
                checksum_sha512: None,
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
        ],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);

    for pkg in &spdx_doc.packages {
        if pkg.spdx_id != "SPDXRef-Package-test" {
            // Verify UUID pattern: SPDXRef-Package-{name}-{uuid}
            assert!(
                pkg.spdx_id.starts_with("SPDXRef-Package-"),
                "SPDX ID should start with SPDXRef-Package-: {}",
                pkg.spdx_id
            );

            // Sanitized name should be in the ID
            let sanitized_name = pkg
                .name
                .replace('@', "")
                .replace('/', "-")
                .replace('.', "-");
            assert!(
                pkg.spdx_id.contains(&sanitized_name),
                "SPDX ID should contain sanitized package name: {}",
                pkg.spdx_id
            );

            // v0.9.0: Short UUID pattern verification (8 character hex string)
            // Format: SPDXRef-Package-{name}-{8charUUID}
            let parts: Vec<&str> = pkg.spdx_id.split('-').collect();
            assert!(
                parts.len() >= 4,
                "SPDX ID should have at least 4 parts (SPDXRef-Package-name-uuid): {}",
                pkg.spdx_id
            );

            // Verify last part is an 8-character hex string (short UUID)
            let uuid_part = parts.last().unwrap();
            assert_eq!(
                uuid_part.len(),
                8,
                "UUID part should be 8 characters: {}",
                uuid_part
            );
            assert!(
                uuid_part.chars().all(|c: char| c.is_ascii_hexdigit()),
                "UUID part should be hexadecimal: {}",
                uuid_part
            );
        }
    }
}

#[test]

fn test_spdx_id_uniqueness() {
    // Create SBOM with duplicate package names
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![
            Dependency {
                name: "axios".to_string(),
                version: "1.0.0".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: None,
                checksum_sha256: None,
                checksum_sha512: None,
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
            Dependency {
                name: "axios".to_string(),
                version: "1.0.1".to_string(),
                ecosystem: "npm".to_string(),
                source: DependencySource::LockFile,
                is_dev: false,
                is_direct: true,
                license: None,
                author: None,
                maintainers: None,
                repository_url: None,
                homepage_url: None,
                source_file: None,
                checksum_sha256: None,
                checksum_sha512: None,
                scope: DependencyScope::default(),
                scope_confidence: 0.0,
                scope_reason: "Not classified".to_string(),
                ai_model_metadata: None,
                autosar_metadata: None,
                ..Default::default()
            },
        ],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);

    let mut seen_ids = std::collections::HashSet::new();
    for pkg in &spdx_doc.packages {
        assert!(
            seen_ids.insert(pkg.spdx_id.clone()),
            "Duplicate SPDX ID found: {}",
            pkg.spdx_id
        );
    }
}

// v0.8.0 Download Location URL Tests
// Note: These tests will pass after Phase 9 download_location implementation

#[test]

fn test_download_location_npm() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let axios_pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .unwrap();

    assert!(axios_pkg.download_location.contains("registry.npmjs.org"));
    assert!(axios_pkg.download_location.contains("axios"));
    assert!(axios_pkg.download_location.contains("1.0.0"));
}

#[test]

fn test_download_location_unspecified_version() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "unspecified".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let axios_pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .unwrap();

    assert_eq!(axios_pkg.download_location, "NOASSERTION");
}

// CPE Identifier Tests will be added after Phase 9 implementation

// v0.8.0 CPE Identifier Tests

#[test]
fn test_cpe_generation_npm() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let axios_pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .unwrap();

    // Should have CPE external reference
    let cpe_ref = axios_pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(cpe_ref.is_some(), "Should have CPE external reference");

    let cpe = &cpe_ref.unwrap().reference_locator;
    assert!(
        cpe.starts_with("cpe:2.3:a:"),
        "CPE should start with cpe:2.3:a:"
    );
    assert!(cpe.contains("axios"), "CPE should contain package name");
    assert!(
        cpe.contains("1.0.0") || cpe.contains("1_0_0"),
        "CPE should contain version"
    );
}

#[test]
fn test_cpe_generation_scoped_npm() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "@types/node".to_string(),
            version: "18.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "@types/node")
        .unwrap();

    let cpe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(cpe_ref.is_some());

    let cpe = &cpe_ref.unwrap().reference_locator;
    assert!(cpe.starts_with("cpe:2.3:a:types:node:"));
}

#[test]
fn test_cpe_generation_cargo() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "serde".to_string(),
            version: "1.0.195".to_string(),
            ecosystem: "cargo".to_string(),
            source: DependencySource::LockFile,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "serde")
        .unwrap();

    let cpe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(cpe_ref.is_some());

    let cpe = &cpe_ref.unwrap().reference_locator;
    assert_eq!(cpe, "cpe:2.3:a:rust:serde:1.0.195:*:*:*:*:*:*:*");
}

#[test]
fn test_cpe_not_generated_for_unspecified_version() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "unspecified".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .unwrap();

    let cpe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(
        cpe_ref.is_none(),
        "Should not generate CPE for unspecified version"
    );
}

#[test]
fn test_cpe_version_sanitization() {
    let sbom = Sbom {
        project_path: PathBuf::from("/test"),
        generated_at: "2026-01-27T00:00:00Z".to_string(),
        dependencies: vec![Dependency {
            name: "axios".to_string(),
            version: "^1.0.0".to_string(),
            ecosystem: "npm".to_string(),
            source: DependencySource::Manifest,
            is_dev: false,
            is_direct: true,
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: "Not classified".to_string(),
            ai_model_metadata: None,
            autosar_metadata: None,
            ..Default::default()
        }],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let spdx_doc = convert_to_spdx(&sbom, false, None);
    let pkg = spdx_doc
        .packages
        .iter()
        .find(|p| p.name == "axios")
        .unwrap();

    let cpe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(cpe_ref.is_some());

    let cpe = &cpe_ref.unwrap().reference_locator;
    // Should have sanitized version (removed ^)
    assert!(cpe.contains("1.0.0") || cpe.contains("1_0_0"));
    assert!(
        !cpe.contains("^"),
        "CPE should not contain version operators"
    );
}

// v1.0.8 SBOM spec compliance: downloadLocation must be NOASSERTION for "detected" version

#[test]
fn test_spdx_download_location_detected_is_noassertion() {
    let dep = {
        let mut d = make_dep("utime", "detected", "pip");
        d.source = DependencySource::ImportScan;
        d
    };
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    // packages[0] = root, packages[1] = utime
    assert_eq!(
        doc.packages[1].download_location, "NOASSERTION",
        "detected version must not produce fake URL"
    );
}

#[test]
fn test_purl_detected_version_omits_version_component() {
    let dep = make_dep("utime", "detected", "pip");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let purl = &doc.packages[1].external_refs[0].reference_locator;
    assert!(
        !purl.contains("@detected"),
        "PURL must not contain @detected: {}",
        purl
    );
    assert_eq!(purl, "pkg:pypi/utime");
}

#[test]
fn test_purl_unspecified_version_omits_version_component() {
    let dep = make_dep("decoder", "unspecified", "system");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let purl = &doc.packages[1].external_refs[0].reference_locator;
    assert!(
        !purl.contains("@unspecified"),
        "PURL must not contain @unspecified: {}",
        purl
    );
    assert_eq!(purl, "pkg:generic/decoder?type=system");
}

#[test]
fn test_purl_unknown_version_omits_version_component() {
    let dep = make_dep("some-lib", "unknown", "cargo");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let purl = &doc.packages[1].external_refs[0].reference_locator;
    assert!(
        !purl.contains("@unknown"),
        "PURL must not contain @unknown: {}",
        purl
    );
    assert_eq!(purl, "pkg:cargo/some-lib");
}

// v1.0.8 SBOM spec compliance: root package versionInfo must be NOASSERTION

#[test]
fn test_spdx_root_package_version_is_noassertion() {
    let doc = convert_to_spdx(&make_sbom(vec![]), false, None);
    assert_eq!(
        doc.packages[0].version_info, "NOASSERTION",
        "Root package versionInfo must be NOASSERTION, got: {}",
        doc.packages[0].version_info
    );
}

// v1.0.8 SBOM spec compliance: no CONTAINS NOASSERTION relationships

#[test]
fn test_spdx_no_contains_noassertion_relationships() {
    let dep = make_dep("axios", "1.0.0", "npm");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let noassertion_rels: Vec<_> = doc
        .relationships
        .iter()
        .filter(|r| r.related_spdx_element == "NOASSERTION")
        .collect();
    assert!(
        noassertion_rels.is_empty(),
        "Found {} CONTAINS NOASSERTION relationships, expected 0",
        noassertion_rels.len()
    );
}

// v1.0.8 SBOM spec compliance: DependencyScope::Provided must map to LIBRARY, not SOURCE

#[test]
fn test_spdx_system_lib_purpose_is_library() {
    let mut dep = make_dep("dl", "unspecified", "system");
    dep.scope = DependencyScope::Provided;
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    assert_eq!(
        doc.packages[1].primary_package_purpose,
        Some("LIBRARY".to_string()),
        "Provided scope must map to LIBRARY, not SOURCE"
    );
}

// v1.0.8 SBOM spec compliance: CPE must not be generated for "detected" sentinel version

#[test]
fn test_cpe_not_generated_for_detected_version() {
    let dep = make_dep("utime", "detected", "pip");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let pkg = doc.packages.iter().find(|p| p.name == "utime").unwrap();
    let cpe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "cpe23Type");
    assert!(
        cpe_ref.is_none(),
        "Should not generate CPE for detected version"
    );
}

// v1.0.8 SBOM spec compliance: versionInfo must be NOASSERTION for sentinel versions

#[test]
fn test_spdx_dep_version_detected_is_noassertion() {
    let dep = make_dep("utime", "detected", "pip");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let pkg = doc.packages.iter().find(|p| p.name == "utime").unwrap();
    assert_eq!(
        pkg.version_info, "NOASSERTION",
        "detected version must produce NOASSERTION versionInfo, got: {}",
        pkg.version_info
    );
}

// Post-scan review: S1 — CVE/CWE refs must use "advisory" type with NVD/MITRE URLs

#[cfg(feature = "internal")]
#[test]
fn test_spdx_cve_uses_advisory_reference_type() {
    let mut dep = make_dep("openssl", "3.0.2", "pkg-config");

    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let pkg = doc.packages.iter().find(|p| p.name == "openssl").unwrap();

    // CVE ref: there must be an NVD URL advisory entry for this CVE
    let nvd_ref = pkg.external_refs.iter().find(|r| {
        r.reference_locator
            .starts_with("https://nvd.nist.gov/vuln/detail/CVE-2023-1234")
    });
    assert!(
        nvd_ref.is_some(),
        "NVD URL advisory ref must exist for CVE-2023-1234"
    );
    assert_eq!(
        nvd_ref.unwrap().reference_type,
        "advisory",
        "CVE NVD refs must use reference_type 'advisory'. Got: {}",
        nvd_ref.unwrap().reference_type
    );

    // There must be NO external ref with reference_type "cve" or "cwe"
    let bad_cve = pkg.external_refs.iter().find(|r| r.reference_type == "cve");
    assert!(
        bad_cve.is_none(),
        "'cve' is not a valid SPDX 2.3 referenceType — found: {:?}",
        bad_cve
    );
    let bad_cwe = pkg.external_refs.iter().find(|r| r.reference_type == "cwe");
    assert!(
        bad_cwe.is_none(),
        "'cwe' is not a valid SPDX 2.3 referenceType — found: {:?}",
        bad_cwe
    );

    // CWE ref must use "advisory" with MITRE URL
    let cwe_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_locator.contains("cwe.mitre.org"));
    assert!(cwe_ref.is_some(), "CWE MITRE URL advisory ref must exist");
    assert_eq!(
        cwe_ref.unwrap().reference_type,
        "advisory",
        "CWE refs must use reference_type 'advisory'. Got: {}",
        cwe_ref.unwrap().reference_type
    );
}

// Post-scan review: S2 — SPDXID must contain only [a-zA-Z0-9.-]
// The bug is in the ROS path where dep.ecosystem is embedded raw in the SPDXID.

#[test]
fn test_spdx_id_no_illegal_characters() {
    // "npm (dev)" contains space and parens — both illegal in SPDXID per SPDX 2.3 §2.2.
    // Use a ROS package scan to exercise the path where ecosystem is embedded in the ID.
    let dep = make_dep("react", "18.0.0", "npm (dev)");
    let sbom = Sbom {
        project_path: PathBuf::from("/test/ros-project"),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![RosPackageWithDeps {
            metadata: RosPackageMetadata {
                name: "my_pkg".to_string(),
                version: "1.0.0".to_string(),
                source_file: PathBuf::from("/test/my_pkg/package.xml"),
                license: None,
                maintainers: vec![],
                authors: vec![],
                description: None,
            },
            dependencies: vec![dep],
        }],
        scope_statistics: None,
    };
    let doc = convert_to_spdx(&sbom, false, None);
    for pkg in &doc.packages {
        let id = &pkg.spdx_id;
        assert!(
            id.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
            "Illegal chars in SPDXID '{}' — only [a-zA-Z0-9.-] allowed",
            id
        );
    }
    for rel in &doc.relationships {
        for id in [&rel.spdx_element_id, &rel.related_spdx_element] {
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
                "Illegal chars in relationship ID '{}' — only [a-zA-Z0-9.-] allowed",
                id
            );
        }
    }
}

// Post-scan review: S3 — shared ROS deps must not be duplicated in SPDX packages

#[test]
fn test_ros_shared_dep_not_duplicated_in_spdx() {
    let shared_dep = make_dep("openssl", "3.0.2", "pkg-config");

    let sbom = Sbom {
        project_path: PathBuf::from("/test/ros-project"),
        generated_at: "2026-01-01T00:00:00Z".to_string(),
        dependencies: vec![],
        ros_package: None,
        ros_packages: vec![
            RosPackageWithDeps {
                metadata: RosPackageMetadata {
                    name: "pkg_a".to_string(),
                    version: "1.0.0".to_string(),
                    source_file: PathBuf::from("/test/pkg_a/package.xml"),
                    license: None,
                    maintainers: vec![],
                    authors: vec![],
                    description: None,
                },
                dependencies: vec![shared_dep.clone()],
            },
            RosPackageWithDeps {
                metadata: RosPackageMetadata {
                    name: "pkg_b".to_string(),
                    version: "1.0.0".to_string(),
                    source_file: PathBuf::from("/test/pkg_b/package.xml"),
                    license: None,
                    maintainers: vec![],
                    authors: vec![],
                    description: None,
                },
                dependencies: vec![shared_dep.clone()],
            },
        ],
        scope_statistics: None,
    };

    let doc = convert_to_spdx(&sbom, false, None);

    let openssl_count = doc.packages.iter().filter(|p| p.name == "openssl").count();
    assert_eq!(
        openssl_count, 1,
        "openssl shared by 2 ROS packages must appear once in SPDX packages, got {}",
        openssl_count
    );
}

// Post-scan review: S4 — originator must not be double-prefixed

#[test]
fn test_originator_not_double_prefixed() {
    let mut dep = make_dep("mylib", "1.0.0", "pip");
    dep.author = Some("Person: John Doe".to_string());
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    let pkg = doc.packages.iter().find(|p| p.name == "mylib").unwrap();
    if let Some(originator) = &pkg.originator {
        assert!(
            !originator.starts_with("Person: Person:"),
            "Double-prefixed originator: '{}'",
            originator
        );
        assert!(
            !originator.starts_with("Organization: Organization:"),
            "Double-prefixed originator: '{}'",
            originator
        );
    }
}

// Tag-Value: PackageVersion must be omitted (not "NOASSERTION") for unknown versions

#[test]
fn test_spdx_tag_value_no_package_version_noassertion() {
    // pyspdxtools rejects "PackageVersion: NOASSERTION" — the field must be omitted
    // when the version is unknown (it is optional per SPDX 2.3 spec §3.3)
    use tempfile::NamedTempFile;
    let dep = make_dep("utime", "detected", "pip");
    let sbom = make_sbom(vec![dep]);
    let file = NamedTempFile::new().unwrap();
    save_spdx_tag_value(
        &sbom,
        file.path().to_str().unwrap(),
        false,
        None,
    )
    .unwrap();
    let content = std::fs::read_to_string(file.path()).unwrap();
    assert!(
        !content.contains("PackageVersion: NOASSERTION"),
        "Tag-Value must not emit 'PackageVersion: NOASSERTION' — omit the field instead"
    );
}

// S2b: package names with '+' (e.g. stdc++) must produce legal SPDXIDs

#[test]
fn test_spdx_id_package_name_with_plus() {
    let dep = make_dep("stdc++", "unspecified", "system");
    let doc = convert_to_spdx(&make_sbom(vec![dep]), false, None);
    for pkg in &doc.packages {
        let id = &pkg.spdx_id;
        assert!(
            id.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-'),
            "Illegal chars in SPDXID '{}' — '+' and other chars are not allowed",
            id
        );
    }
}

// ── v1.0.13: Sub-model CONTAINS relationships ───────────────────────────────

#[test]
fn test_spdx_sub_model_contains_relationships() {
    let mut dep = make_dep("gemma4-test", "unknown", "safetensors");
    dep.ai_model_metadata = Some(AIModelMetadata {
        architecture: Some("Gemma4ForConditionalGeneration".to_string()),
        sub_models: vec![
            SubModelInfo {
                model_type: Some("gemma4_text".to_string()),
                modality: "text".to_string(),
                num_hidden_layers: Some(35),
                hidden_size: Some(1536),
                ..Default::default()
            },
            SubModelInfo {
                model_type: Some("gemma4_vision".to_string()),
                modality: "vision".to_string(),
                num_hidden_layers: Some(16),
                hidden_size: Some(768),
                patch_size: Some(16),
                ..Default::default()
            },
        ],
        ..Default::default()
    });

    let sbom = make_sbom(vec![dep]);
    let doc = convert_to_spdx(&sbom, false, None);
    let json_str = serde_json::to_string_pretty(&doc).unwrap();
    let json: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    // Find sub-model packages
    let packages = json["packages"].as_array().unwrap();
    let sub_model_pkgs: Vec<&serde_json::Value> = packages
        .iter()
        .filter(|p| {
            p["sourceInfo"]
                .as_str()
                .map(|s| s.starts_with("Sub-model"))
                .unwrap_or(false)
        })
        .collect();
    assert_eq!(sub_model_pkgs.len(), 2, "should have 2 sub-model child packages");

    // Verify text sub-model package
    let text_pkg = sub_model_pkgs.iter().find(|p| p["name"] == "gemma4_text").unwrap();
    let text_si = text_pkg["sourceInfo"].as_str().unwrap();
    assert!(text_si.contains("layers=35"), "text sourceInfo should contain layers");
    assert!(text_si.contains("hidden=1536"), "text sourceInfo should contain hidden size");

    // Verify vision sub-model package
    let vision_pkg = sub_model_pkgs.iter().find(|p| p["name"] == "gemma4_vision").unwrap();
    let vision_si = vision_pkg["sourceInfo"].as_str().unwrap();
    assert!(vision_si.contains("patch_size=16"), "vision sourceInfo should contain patch_size");

    // Verify CONTAINS relationships exist
    let relationships = json["relationships"].as_array().unwrap();
    let contains_rels: Vec<&serde_json::Value> = relationships
        .iter()
        .filter(|r| r["relationshipType"] == "CONTAINS")
        .collect();

    // At least 2 CONTAINS relationships for sub-models (parent may also have a CONTAINS from root)
    let sub_model_contains: Vec<&serde_json::Value> = contains_rels
        .iter()
        .filter(|r| {
            r["relatedSpdxElement"]
                .as_str()
                .map(|s| s.contains("-sub-"))
                .unwrap_or(false)
        })
        .copied()
        .collect();
    assert_eq!(
        sub_model_contains.len(),
        2,
        "should have 2 CONTAINS relationships for sub-models"
    );
}

#[test]
fn test_autosar_spdx_external_refs() {
    use radeis_sc2sbom::models::AutosarMetadata;

    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(AutosarMetadata {
        module_name: "NvM".to_string(),
        layer: "BSW-Memory".to_string(),
        platform: "Classic".to_string(),
    });

    let sbom = make_sbom(vec![dep]);
    let spdx = convert_to_spdx(&sbom, false, None);

    let pkg = spdx
        .packages
        .iter()
        .find(|p| p.name == "NvM")
        .expect("NvM package must be present in SPDX output");

    let layer_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "autosar-layer")
        .expect("autosar-layer ExternalRef must be present");
    assert_eq!(layer_ref.reference_category, "OTHER");
    assert_eq!(layer_ref.reference_locator, "autosar:layer=BSW-Memory");

    let platform_ref = pkg
        .external_refs
        .iter()
        .find(|r| r.reference_type == "autosar-platform")
        .expect("autosar-platform ExternalRef must be present");
    assert_eq!(platform_ref.reference_category, "OTHER");
    assert_eq!(platform_ref.reference_locator, "autosar:platform=Classic");
}

// ---- Phase 8 (v1.0.15): autosar:supplier SPDX ExternalRef tests ----

fn make_autosar_metadata_spdx() -> AutosarMetadata {
    AutosarMetadata {
        module_name: "NvM".to_string(),
        layer: "BSW-Memory".to_string(),
        platform: "Classic".to_string(),
    }
}

#[test]
fn test_autosar_supplier_mapped_spdx() {
    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(make_autosar_metadata_spdx());
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let spdx = convert_to_spdx(&sbom, false, Some(&resolver));
    let pkg = spdx.packages.iter().find(|p| p.name == "NvM").unwrap();
    let xref = pkg
        .external_refs
        .iter()
        .find(|x| x.reference_type == "autosar-supplier")
        .expect("autosar-supplier ExternalRef must be emitted");
    assert_eq!(xref.reference_category, "OTHER");
    assert_eq!(xref.reference_locator, "autosar:supplier=Vector%20Informatik");
}

#[test]
fn test_autosar_supplier_noassertion_no_config_spdx() {
    let mut dep = make_dep("NvM", "unspecified", "autosar");
    dep.autosar_metadata = Some(make_autosar_metadata_spdx());
    let sbom = make_sbom(vec![dep]);

    let spdx = convert_to_spdx(&sbom, false, None);
    let xref = spdx
        .packages
        .iter()
        .find(|p| p.name == "NvM")
        .unwrap()
        .external_refs
        .iter()
        .find(|x| x.reference_type == "autosar-supplier")
        .unwrap();
    assert_eq!(xref.reference_locator, "autosar:supplier=NOASSERTION");
}

#[test]
fn test_autosar_supplier_noassertion_no_entry_spdx() {
    let mut dep = make_dep("Com", "unspecified", "autosar");
    dep.autosar_metadata = Some(AutosarMetadata {
        module_name: "Com".to_string(),
        layer: "BSW-Communication".to_string(),
        platform: "Classic".to_string(),
    });
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("NvM".to_string(), "Vector Informatik".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let spdx = convert_to_spdx(&sbom, false, Some(&resolver));
    let xref = spdx
        .packages
        .iter()
        .find(|p| p.name == "Com")
        .unwrap()
        .external_refs
        .iter()
        .find(|x| x.reference_type == "autosar-supplier")
        .unwrap();
    assert_eq!(xref.reference_locator, "autosar:supplier=NOASSERTION");
}

#[test]
fn test_non_autosar_no_supplier_spdx() {
    let dep = make_dep("openssl", "3.0.0", "cargo"); // autosar_metadata: None
    let sbom = make_sbom(vec![dep]);

    let mut mappings = std::collections::HashMap::new();
    mappings.insert("openssl".to_string(), "Should Not Appear".to_string());
    let resolver = radeis_sc2sbom::supplier::SupplierResolver::from_map(mappings);

    let spdx = convert_to_spdx(&sbom, false, Some(&resolver));
    let pkg = spdx.packages.iter().find(|p| p.name == "openssl").unwrap();
    assert!(
        pkg.external_refs
            .iter()
            .all(|x| x.reference_type != "autosar-supplier"),
        "non-AUTOSAR package must not carry autosar-supplier ExternalRef"
    );
}
