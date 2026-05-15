use crate::models::{Dependency, DependencyScope, Sbom, SubModelInfo};
use crate::supplier::SupplierResolver;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fmt::Write as FmtWrite;
use std::fs;
use uuid::Uuid;

/// Tool version, synchronized with Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Map DependencyScope to SPDX 2.3 primaryPackagePurpose (v1.0.6)
/// Valid SPDX 2.3 values: APPLICATION, FRAMEWORK, LIBRARY, CONTAINER, OPERATING-SYSTEM,
/// DEVICE, FIRMWARE, SOURCE, ARCHIVE, FILE, INSTALL, OTHER
/// Spec: https://spdx.github.io/spdx-spec/v2.3/package-information/#724-primary-package-purpose-field
fn map_scope_to_spdx_purpose(scope: &DependencyScope) -> String {
    match scope {
        DependencyScope::Runtime => "LIBRARY".to_string(),
        DependencyScope::Build => "OTHER".to_string(), // No direct SPDX equivalent for build-only deps
        DependencyScope::Test => "OTHER".to_string(),  // No direct SPDX equivalent for test deps
        DependencyScope::Development => "OTHER".to_string(), // No direct SPDX equivalent for dev tools
        DependencyScope::Optional => "LIBRARY".to_string(),
        DependencyScope::Provided => "LIBRARY".to_string(),
    }
}

// SPDX 2.3 Data Structures
#[derive(Debug, Serialize)]
pub struct SPDXDocument {
    #[serde(rename = "spdxVersion")]
    spdx_version: String,

    #[serde(rename = "dataLicense")]
    data_license: String,

    #[serde(rename = "SPDXID")]
    spdx_id: String,

    name: String,

    #[serde(rename = "documentNamespace")]
    document_namespace: String,

    #[serde(rename = "creationInfo")]
    creation_info: SPDXCreationInfo,

    #[serde(rename = "documentDescribes")]
    document_describes: Vec<String>,

    pub packages: Vec<SPDXPackage>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub relationships: Vec<SPDXRelationship>,
}

#[derive(Debug, Serialize)]
struct SPDXCreationInfo {
    created: String,
    creators: Vec<String>,
    #[serde(rename = "licenseListVersion")]
    license_list_version: String,
}

#[derive(Debug, Serialize)]
pub struct SPDXPackage {
    #[serde(rename = "SPDXID")]
    pub spdx_id: String,

    pub name: String,

    #[serde(rename = "versionInfo")]
    pub version_info: String,

    #[serde(rename = "downloadLocation")]
    pub download_location: String,

    #[serde(rename = "filesAnalyzed")]
    pub files_analyzed: bool,

    #[serde(rename = "licenseConcluded")]
    pub license_concluded: String,

    #[serde(rename = "licenseDeclared")]
    pub license_declared: String,

    #[serde(rename = "copyrightText")]
    pub copyright_text: String,

    #[serde(rename = "externalRefs")]
    pub external_refs: Vec<SPDXExternalRef>,

    // v0.8.0: NEW FIELDS for supplier, originator, and source tracking
    #[serde(skip_serializing_if = "Option::is_none", rename = "supplier")]
    pub supplier: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "originator")]
    pub originator: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", rename = "sourceInfo")]
    pub source_info: Option<String>,

    // v1.0.9: Package checksums (e.g. SHA-256 of GGUF model files)
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub checksums: Vec<SPDXChecksum>,

    // v1.0.6: Dependency scope classification
    #[serde(
        skip_serializing_if = "Option::is_none",
        rename = "primaryPackagePurpose"
    )]
    pub primary_package_purpose: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SPDXChecksum {
    pub algorithm: String,
    #[serde(rename = "checksumValue")]
    pub checksum_value: String,
}

#[derive(Debug, Serialize)]
pub struct SPDXExternalRef {
    #[serde(rename = "referenceCategory")]
    pub reference_category: String,

    #[serde(rename = "referenceType")]
    pub reference_type: String,

    #[serde(rename = "referenceLocator")]
    pub reference_locator: String,
}

#[derive(Debug, Serialize)]
pub struct SPDXRelationship {
    #[serde(rename = "spdxElementId")]
    pub spdx_element_id: String,

    #[serde(rename = "relationshipType")]
    pub relationship_type: String,

    #[serde(rename = "relatedSpdxElement")]
    pub related_spdx_element: String,
}

pub fn print_spdx_json(
    sbom: &Sbom,
    compact_spdx: bool,
    supplier_resolver: Option<&SupplierResolver>,
) -> Result<()> {
    let spdx_doc = convert_to_spdx(sbom, compact_spdx, supplier_resolver);
    let json = serde_json::to_string_pretty(&spdx_doc)?;
    println!("{}", json);
    Ok(())
}

pub fn print_spdx_tag_value(
    sbom: &Sbom,
    compact_spdx: bool,
    supplier_resolver: Option<&SupplierResolver>,
) {
    let spdx_doc = convert_to_spdx(sbom, compact_spdx, supplier_resolver);

    println!("SPDXVersion: {}", spdx_doc.spdx_version);
    println!("DataLicense: {}", spdx_doc.data_license);
    println!("SPDXID: {}", spdx_doc.spdx_id);
    println!("DocumentName: {}", spdx_doc.name);
    println!("DocumentNamespace: {}", spdx_doc.document_namespace);
    println!("Creator: {}", spdx_doc.creation_info.creators.join(", "));
    println!("Created: {}", spdx_doc.creation_info.created);
    println!(
        "LicenseListVersion: {}",
        spdx_doc.creation_info.license_list_version
    );
    println!();

    for package in &spdx_doc.packages {
        println!("##### Package: {}", package.name);
        println!();
        println!("PackageName: {}", package.name);
        println!("SPDXID: {}", package.spdx_id);
        if package.version_info != "NOASSERTION" {
            println!("PackageVersion: {}", package.version_info);
        }
        println!("PackageDownloadLocation: {}", package.download_location);
        println!("FilesAnalyzed: {}", package.files_analyzed);
        println!("PackageLicenseConcluded: {}", package.license_concluded);
        println!("PackageLicenseDeclared: {}", package.license_declared);
        println!("PackageCopyrightText: {}", package.copyright_text);

        for ext_ref in &package.external_refs {
            println!(
                "ExternalRef: {} {} {}",
                ext_ref.reference_category, ext_ref.reference_type, ext_ref.reference_locator
            );
        }
        println!();
    }

    // Print relationships
    if !spdx_doc.relationships.is_empty() {
        println!("##### Relationships");
        println!();
        for rel in &spdx_doc.relationships {
            println!(
                "Relationship: {} {} {}",
                rel.spdx_element_id, rel.relationship_type, rel.related_spdx_element
            );
        }
        println!();
    }
}

fn create_spdx_external_refs(dep: &Dependency, purl: String, supplier_resolver: Option<&SupplierResolver>) -> Vec<SPDXExternalRef> {
    let mut external_refs = vec![SPDXExternalRef {
        reference_category: "PACKAGE-MANAGER".to_string(),
        reference_type: "purl".to_string(),
        reference_locator: purl,
    }];

    // Add CPE identifier for security correlation
    if let Some(cpe) = generate_cpe_identifier(dep) {
        external_refs.push(SPDXExternalRef {
            reference_category: "SECURITY".to_string(),
            reference_type: "cpe23Type".to_string(),
            reference_locator: cpe,
        });
    }

    // v1.0.15 (Phase 7, OUT-02): emit autosar:layer + autosar:platform as
    // SPDX ExternalRef entries with referenceCategory: OTHER.
    // Pitfall 2 (07-RESEARCH.md): referenceType is a no-space idstring.
    if let Some(ref meta) = dep.autosar_metadata {
        external_refs.push(SPDXExternalRef {
            reference_category: "OTHER".to_string(),
            reference_type: "autosar-layer".to_string(),
            reference_locator: format!("autosar:layer={}", meta.layer),
        });
        external_refs.push(SPDXExternalRef {
            reference_category: "OTHER".to_string(),
            reference_type: "autosar-platform".to_string(),
            reference_locator: format!("autosar:platform={}", meta.platform),
        });
    }

    // Phase 8 (v1.0.15): Emit autosar:supplier ExternalRef for every AUTOSAR
    // component. Always emit — value is NOASSERTION when the resolver is
    // absent or has no entry for this component name (per D-09).
    if dep.autosar_metadata.is_some() {
        let supplier_value = supplier_resolver
            .and_then(|r| r.lookup(&dep.name))
            .unwrap_or("NOASSERTION");
        let supplier_encoded = supplier_value.replace(' ', "%20");
        external_refs.push(SPDXExternalRef {
            reference_category: "OTHER".to_string(),
            reference_type: "autosar-supplier".to_string(),
            reference_locator: format!("autosar:supplier={}", supplier_encoded),
        });
    }

    external_refs
}

/// Create supplier field based on ecosystem
fn create_supplier_field(dependency: &Dependency) -> Option<String> {
    match &dependency.ecosystem[..] {
        "npm" => Some("Organization: npmjs".to_string()),
        "pip" => Some("Organization: pypi".to_string()),
        "cargo" => Some("Organization: crates.io".to_string()),
        "go" => Some("Organization: go modules".to_string()),
        "ros" => Some("Organization: ROS".to_string()),
        "php" => Some("Organization: packagist".to_string()),
        "ruby" => Some("Organization: rubygems".to_string()),
        _ => None,
    }
}

/// Create originator field from author metadata
fn create_originator_field(dependency: &Dependency) -> Option<String> {
    dependency.author.as_ref().map(|author| {
        // S4 fix: don't double-prefix if author already contains a valid SPDX prefix
        if author.starts_with("Person:") || author.starts_with("Organization:") {
            return author.to_string();
        }
        // Check if author contains email or looks like an org
        if author.contains('@') || author.contains('<') {
            format!("Person: {}", author)
        } else if author.contains("Team") || author.contains("Foundation") || author.contains("Inc")
        {
            format!("Organization: {}", author)
        } else {
            format!("Person: {}", author)
        }
    })
}

/// Sanitize a string for use in an SPDXID.
/// SPDX 2.3 §2.2: only [a-zA-Z0-9.-] allowed after "SPDXRef-".
/// S2 fix: ecosystem values like "npm (dev)" contain spaces/parens that are illegal.
fn sanitize_for_spdx_id(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

/// Return version string or NOASSERTION for sentinel values
fn version_info_or_noassertion(version: &str) -> String {
    if version == "detected"
        || version == "unspecified"
        || version == "unknown"
        || version.contains("$(")
    {
        "NOASSERTION".to_string()
    } else {
        version.to_string()
    }
}

/// Get license string or fallback to NOASSERTION
fn get_license_or_noassertion(dependency: &Dependency) -> String {
    dependency
        .license
        .clone()
        .unwrap_or_else(|| "NOASSERTION".to_string())
}

/// Create UUID-based SPDX ID for a package
/// Format: SPDXRef-Package-{sanitized-name}-{uuid}
/// This provides better uniqueness than sequential IDs
/// v0.9.0: Uses short UUID (8 chars) for file size optimization
fn create_spdx_id_with_uuid(name: &str) -> String {
    // Sanitize package name using the shared helper (allowlist: [a-zA-Z0-9.-] only).
    // This covers names like "stdc++" where '+' is illegal in SPDX 2.3 §2.2 IDs.
    let sanitized = sanitize_for_spdx_id(name);

    let uuid = Uuid::new_v4();
    // Use only first 8 characters of UUID for file size optimization (v0.9.0)
    // Still provides good uniqueness: ~4 billion combinations
    let short_uuid = &uuid.to_string()[..8];
    format!("SPDXRef-Package-{}-{}", sanitized, short_uuid)
}

/// Generate CPE 2.3 identifier for a dependency
/// Format: cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*
fn generate_cpe_identifier(dependency: &Dependency) -> Option<String> {
    // Only generate CPE for known versions (not sentinels)
    if dependency.version == "unspecified"
        || dependency.version == "unknown"
        || dependency.version == "detected"
        || dependency.version.contains("$(") // guard against unexpanded Makefile refs
    {
        return None;
    }

    let vendor = extract_vendor_from_name(&dependency.name, &dependency.ecosystem);
    let product = extract_product_from_name(&dependency.name);
    let version = sanitize_version_for_cpe(&dependency.version);

    // CPE 2.3 format: cpe:2.3:a:vendor:product:version:*:*:*:*:*:*:*
    Some(format!(
        "cpe:2.3:a:{}:{}:{}:*:*:*:*:*:*:*",
        vendor, product, version
    ))
}

/// Extract vendor from package name based on ecosystem conventions
fn extract_vendor_from_name(name: &str, ecosystem: &str) -> String {
    match ecosystem {
        "npm" => {
            // Scoped packages: @scope/name -> scope
            if name.starts_with('@') {
                name.split('/')
                    .next()
                    .unwrap_or("npm")
                    .trim_start_matches('@')
                    .replace('-', "_")
            } else {
                "npm".to_string()
            }
        }
        "cargo" => "rust".to_string(),
        "pip" => "python".to_string(),
        "rubygems" => "ruby".to_string(),
        "composer" => {
            // Composer packages: vendor/package
            name.split('/').next().unwrap_or("php").replace('-', "_")
        }
        "go" => {
            // Go modules: github.com/org/repo -> org
            name.split('/').nth(1).unwrap_or("go").replace('-', "_")
        }
        _ => ecosystem.replace('-', "_"),
    }
}

/// Extract product name from package name
fn extract_product_from_name(name: &str) -> String {
    // Remove scope prefix for npm packages
    let product = if name.starts_with('@') {
        name.split('/').nth(1).unwrap_or(name)
    } else if name.contains('/') {
        // For Go/Composer, take last part
        name.split('/').last().unwrap_or(name)
    } else {
        name
    };

    product.replace('-', "_").replace('.', "_")
}

/// Sanitize version for CPE (remove operators like ^, ~, >=)
fn sanitize_version_for_cpe(version: &str) -> String {
    version
        .trim_start_matches('^')
        .trim_start_matches('~')
        .trim_start_matches(">=")
        .trim_start_matches("<=")
        .trim_start_matches('>')
        .trim_start_matches('<')
        .trim_start_matches('=')
        .replace('-', "_")
}

/// Create download location URL based on ecosystem
fn create_download_location(dependency: &Dependency) -> String {
    // Only generate URLs for packages with specific versions
    if dependency.version == "unknown"
        || dependency.version == "unspecified"
        || dependency.version == "detected"
        || dependency.version.contains("$(") // guard against unexpanded Makefile refs
    {
        return "NOASSERTION".to_string();
    }

    match &dependency.ecosystem[..] {
        "npm" => {
            // npm registry URL format: https://registry.npmjs.org/{package}/-/{filename}.tgz
            let package_name = if dependency.name.starts_with('@') {
                // Scoped package: @scope/name
                dependency.name.clone()
            } else {
                dependency.name.clone()
            };
            let filename = dependency
                .name
                .rsplit('/')
                .next()
                .unwrap_or(&dependency.name);
            format!(
                "https://registry.npmjs.org/{}/-/{}-{}.tgz",
                package_name, filename, dependency.version
            )
        }
        "pip" => {
            // PyPI URL format: https://pypi.org/project/{package}/{version}/
            format!(
                "https://pypi.org/project/{}/{}/",
                dependency.name, dependency.version
            )
        }
        "cargo" => {
            // crates.io download URL format
            format!(
                "https://crates.io/api/v1/crates/{}/{}/download",
                dependency.name, dependency.version
            )
        }
        "go" => {
            // Go modules use repository URLs
            dependency
                .repository_url
                .clone()
                .unwrap_or_else(|| "NOASSERTION".to_string())
        }
        "ros" => {
            // ROS packages use repository URLs from rosdistro (v0.9.1)
            dependency
                .repository_url
                .clone()
                .unwrap_or_else(|| "NOASSERTION".to_string())
        }
        "php" => {
            // Packagist URL format
            format!("https://packagist.org/packages/{}", dependency.name)
        }
        "ruby" => {
            // RubyGems URL format
            format!("https://rubygems.org/gems/{}", dependency.name)
        }
        _ => "NOASSERTION".to_string(),
    }
}

pub fn convert_to_spdx(
    sbom: &Sbom,
    _compact_spdx: bool,
    supplier_resolver: Option<&SupplierResolver>,
) -> SPDXDocument {
    let project_name = sbom
        .project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project");

    let namespace = format!(
        "https://sbom.example.com/{}/{}",
        project_name, sbom.generated_at
    );

    let mut packages: Vec<SPDXPackage> = Vec::new();
    let mut relationships: Vec<SPDXRelationship> = Vec::new();
    let mut document_describes: Vec<String> = Vec::new();

    // Filter dependencies based on mode
    #[cfg(feature = "internal")]
    let filtered_deps: Vec<&Dependency> = sbom.dependencies.iter().collect();
    #[cfg(not(feature = "internal"))]
    let filtered_deps: Vec<&Dependency> = sbom.dependencies.iter().collect();

    // Add ROS packages as primary packages if they exist
    if !sbom.ros_packages.is_empty() {
        // S3 fix: track seen deps by (name, version, ecosystem) to avoid duplicate package elements.
        // A shared library depended on by N ROS packages must appear exactly once.
        let mut seen_deps: std::collections::HashMap<(String, String, String), String> =
            std::collections::HashMap::new();

        for (pkg_idx, ros_pkg) in sbom.ros_packages.iter().enumerate() {
            let pkg_spdx_id = format!("SPDXRef-ROSPackage-{}", pkg_idx + 1);

            // Create SPDX package for ROS package itself
            packages.push(SPDXPackage {
                spdx_id: pkg_spdx_id.clone(),
                name: ros_pkg.metadata.name.clone(),
                version_info: ros_pkg.metadata.version.clone(),
                download_location: "NOASSERTION".to_string(),
                files_analyzed: false,
                license_concluded: "NOASSERTION".to_string(),
                license_declared: "NOASSERTION".to_string(),
                copyright_text: "NOASSERTION".to_string(),
                external_refs: vec![SPDXExternalRef {
                    reference_category: "PACKAGE-MANAGER".to_string(),
                    reference_type: "purl".to_string(),
                    reference_locator: format!(
                        "pkg:ros/{}@{}",
                        ros_pkg.metadata.name, ros_pkg.metadata.version
                    ),
                }],
                supplier: None,
                originator: None,
                source_info: None,
                checksums: vec![],
                primary_package_purpose: Some("LIBRARY".to_string()), // ROS packages are libraries
            });

            // Document describes this ROS package
            document_describes.push(pkg_spdx_id.clone());
            relationships.push(SPDXRelationship {
                spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
                relationship_type: "DESCRIBES".to_string(),
                related_spdx_element: pkg_spdx_id.clone(),
            });

            // Add dependencies and relationships, deduplicating shared libraries
            for dep in ros_pkg.dependencies.iter() {
                let dep_key = (dep.name.clone(), dep.version.clone(), dep.ecosystem.clone());

                let dep_spdx_id = if let Some(existing_id) = seen_deps.get(&dep_key) {
                    // Shared dep already has a package element — reuse its SPDXID
                    existing_id.clone()
                } else {
                    let new_id = format!(
                        "SPDXRef-Dep-{}-{}",
                        sanitize_for_spdx_id(&dep.ecosystem), // S2 fix: sanitize illegal chars
                        seen_deps.len() + 1
                    );
                    let purl = create_package_url(dep);
                    packages.push(SPDXPackage {
                        spdx_id: new_id.clone(),
                        name: dep.name.clone(),
                        version_info: version_info_or_noassertion(&dep.version),
                        download_location: create_download_location(dep),
                        files_analyzed: false,
                        license_concluded: get_license_or_noassertion(dep),
                        license_declared: get_license_or_noassertion(dep),
                        copyright_text: "NOASSERTION".to_string(),
                        external_refs: create_spdx_external_refs(dep, purl, supplier_resolver),
                        supplier: create_supplier_field(dep),
                        originator: create_originator_field(dep),
                        source_info: build_source_info(dep),
                        checksums: build_checksums(dep),
                        primary_package_purpose: if dep.ecosystem == "gguf"
                            || dep.ecosystem == "safetensors"
                        {
                            Some("OTHER".to_string())
                        } else {
                            Some(map_scope_to_spdx_purpose(&dep.scope))
                        },
                    });
                    seen_deps.insert(dep_key, new_id.clone());
                    new_id
                };

                // Always emit the relationship (both pkg_a and pkg_b DEPENDS_ON openssl)
                relationships.push(SPDXRelationship {
                    spdx_element_id: pkg_spdx_id.clone(),
                    relationship_type: "DEPENDS_ON".to_string(),
                    related_spdx_element: dep_spdx_id,
                });
            }
        }
    } else {
        // v0.8.0: Hierarchical dependency structure
        // Create synthetic root package using project name (v1.0.6)
        // Sanitize project name for use in SPDX ID (only [a-zA-Z0-9.-] allowed per SPDX 2.3 spec)
        // Use allowlist approach: replace any non-conforming char with '-', ensure non-empty
        let mut sanitized_project_name: String = project_name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                    c
                } else {
                    '-'
                }
            })
            .collect();
        if sanitized_project_name.is_empty() {
            sanitized_project_name.push_str("package");
        }
        let main_package_id = format!("SPDXRef-Package-{}", sanitized_project_name);
        packages.push(SPDXPackage {
            spdx_id: main_package_id.clone(),
            name: project_name.to_string(),
            version_info: "NOASSERTION".to_string(),
            download_location: "NOASSERTION".to_string(),
            files_analyzed: false,
            license_concluded: "NOASSERTION".to_string(),
            license_declared: "NOASSERTION".to_string(),
            copyright_text: "NOASSERTION".to_string(),
            external_refs: vec![],
            supplier: None,
            originator: None,
            source_info: None,
            checksums: vec![],
            primary_package_purpose: None, // Main package has no specific purpose
        });

        // 1. Document DESCRIBES main package
        document_describes.push(main_package_id.clone());
        relationships.push(SPDXRelationship {
            spdx_element_id: "SPDXRef-DOCUMENT".to_string(),
            relationship_type: "DESCRIBES".to_string(),
            related_spdx_element: main_package_id.clone(),
        });

        // 2. Main package CONTAINS each dependency
        for dep in filtered_deps.iter() {
            let package_id = create_spdx_id_with_uuid(&dep.name);
            let purl = create_package_url(dep);

            packages.push(SPDXPackage {
                spdx_id: package_id.clone(),
                name: dep.name.clone(),
                version_info: version_info_or_noassertion(&dep.version),
                download_location: create_download_location(dep),
                files_analyzed: false,
                license_concluded: get_license_or_noassertion(dep),
                license_declared: get_license_or_noassertion(dep),
                copyright_text: "NOASSERTION".to_string(),
                external_refs: create_spdx_external_refs(dep, purl, supplier_resolver),
                supplier: create_supplier_field(dep),
                originator: create_originator_field(dep),
                source_info: build_source_info(dep),
                checksums: build_checksums(dep),
                primary_package_purpose: if dep.ecosystem == "gguf"
                    || dep.ecosystem == "safetensors"
                {
                    Some("OTHER".to_string())
                } else {
                    Some(map_scope_to_spdx_purpose(&dep.scope))
                },
            });

            // Main package CONTAINS this dependency
            relationships.push(SPDXRelationship {
                spdx_element_id: main_package_id.clone(),
                relationship_type: "CONTAINS".to_string(),
                related_spdx_element: package_id.clone(),
            });

            // v1.0.13: Emit child packages for AI model sub-models
            if let Some(ref meta) = dep.ai_model_metadata {
                for (sm_idx, sm) in meta.sub_models.iter().enumerate() {
                    let sm_name = sm.model_type.as_deref().unwrap_or(&sm.modality);
                    let sm_spdx_id = format!("{}-sub-{}", package_id, sm_idx);
                    let source_info = build_sub_model_source_info(sm);
                    packages.push(SPDXPackage {
                        spdx_id: sm_spdx_id.clone(),
                        name: sm_name.to_string(),
                        version_info: "NOASSERTION".to_string(),
                        download_location: "NOASSERTION".to_string(),
                        files_analyzed: false,
                        license_concluded: "NOASSERTION".to_string(),
                        license_declared: "NOASSERTION".to_string(),
                        copyright_text: "NOASSERTION".to_string(),
                        external_refs: vec![],
                        supplier: None,
                        originator: None,
                        source_info: Some(source_info),
                        checksums: vec![],
                        primary_package_purpose: Some("LIBRARY".to_string()),
                    });
                    relationships.push(SPDXRelationship {
                        spdx_element_id: package_id.clone(),
                        relationship_type: "CONTAINS".to_string(),
                        related_spdx_element: sm_spdx_id,
                    });
                }
            }
        }
    }

    SPDXDocument {
        spdx_version: "SPDX-2.3".to_string(),
        data_license: "CC0-1.0".to_string(),
        spdx_id: "SPDXRef-DOCUMENT".to_string(),
        name: format!("{}-sbom", project_name),
        document_namespace: namespace,
        creation_info: SPDXCreationInfo {
            // SPDX 2.3 spec requires UTC timestamp as YYYY-MM-DDThh:mm:ssZ (no sub-seconds, Z suffix).
            created: DateTime::parse_from_rfc3339(&sbom.generated_at)
                .map(|dt| {
                    dt.with_timezone(&Utc)
                        .format("%Y-%m-%dT%H:%M:%SZ")
                        .to_string()
                })
                .unwrap_or_else(|_| sbom.generated_at.clone()),
            creators: vec![format!("Tool: radeis_sc2sbom-{}", VERSION)],
            license_list_version: "3.21".to_string(),
        },
        document_describes,
        packages,
        relationships,
    }
}

/// Build a human-readable sourceInfo string for a sub-model component.
fn build_sub_model_source_info(sm: &SubModelInfo) -> String {
    let mut parts = Vec::new();
    if let Some(ref mt) = sm.model_type {
        parts.push(format!("model_type={}", mt));
    }
    if let Some(layers) = sm.num_hidden_layers {
        parts.push(format!("layers={}", layers));
    }
    if let Some(hidden) = sm.hidden_size {
        parts.push(format!("hidden={}", hidden));
    }
    if let Some(heads) = sm.num_attention_heads {
        parts.push(format!("heads={}", heads));
    }
    if let Some(vocab) = sm.vocab_size {
        parts.push(format!("vocab={}", vocab));
    }
    if let Some(ctx) = sm.max_position_embeddings {
        parts.push(format!("context={}", ctx));
    }
    if let Some(kv_heads) = sm.num_key_value_heads {
        parts.push(format!("kv_heads={}", kv_heads));
    }
    if let Some(ref dtype) = sm.dtype {
        parts.push(format!("dtype={}", dtype));
    }
    if let Some(patch) = sm.patch_size {
        parts.push(format!("patch_size={}", patch));
    }
    if let Some(conv) = sm.conv_kernel_size {
        parts.push(format!("conv_kernel={}", conv));
    }
    if let Some(proj) = sm.output_proj_dims {
        parts.push(format!("output_proj={}", proj));
    }
    format!("Sub-model ({}): {}", sm.modality, parts.join(", "))
}

/// Build sourceInfo string for a dependency.
/// For GGUF/Safetensors AI model deps with metadata, produces a rich description.
/// For other deps, falls back to source_file.
fn build_source_info(dep: &Dependency) -> Option<String> {
    if dep.ecosystem == "gguf" {
        if let Some(ref meta) = dep.ai_model_metadata {
            let mut parts: Vec<String> = Vec::new();
            if let Some(ref arch) = meta.architecture {
                parts.push(format!("architecture={}", arch));
            }
            if let Some(count) = meta.parameter_count {
                parts.push(format!("parameters={}", count));
            } else if let Some(ref size) = meta.size_label {
                parts.push(format!("parameters={}", size));
            }
            if let Some(ref quant) = meta.quantization {
                parts.push(format!("quantization={}", quant));
            }
            if !meta.base_models.is_empty() {
                // Use the name of the first base model, or its organization/repo if name is absent
                if let Some(ref bm_name) = meta.base_models[0].name {
                    parts.push(format!("base_model={}", bm_name));
                } else if let Some(ref bm_org) = meta.base_models[0].organization {
                    if let Some(ref bm_repo) = meta.base_models[0].repo_url {
                        parts.push(format!("base_model={}/{}", bm_org, bm_repo));
                    } else {
                        parts.push(format!("base_model={}", bm_org));
                    }
                }
            }
            // Computed parameter count (tensor integrity verification)
            if let Some(computed) = meta.computed_parameter_count {
                parts.push(format!("computed_parameters={}", computed));
            }
            // Integrity warning if mismatch
            if let (Some(declared), Some(computed)) = (meta.parameter_count, meta.computed_parameter_count) {
                if declared != computed {
                    parts.push(format!(
                        "INTEGRITY_WARNING: declared({}) != computed({})",
                        declared, computed
                    ));
                }
            }
            // v1.0.12: Rich metadata summary
            if let Some(ref mt) = meta.model_type {
                parts.push(format!("model_type={}", mt));
            }
            if let Some(ctx) = meta.max_position_embeddings.or(meta.model_max_length) {
                parts.push(format!("context={}", ctx));
            }
            let mut modalities = vec!["text"];
            if meta.has_vision == Some(true) { modalities.push("vision"); }
            if meta.has_audio == Some(true) { modalities.push("audio"); }
            if meta.has_video == Some(true) { modalities.push("video"); }
            if modalities.len() > 1 {
                parts.push(format!("modalities={}", modalities.join("+")));
            }
            if !parts.is_empty() {
                return Some(format!("AI Model (GGUF): {}", parts.join(", ")));
            }
            return Some("AI Model (GGUF)".to_string());
        }
    }

    // v1.0.11: Safetensors sourceInfo
    if dep.ecosystem == "safetensors" {
        if let Some(ref meta) = dep.ai_model_metadata {
            let mut parts: Vec<String> = Vec::new();
            if let Some(ref arch) = meta.architecture {
                parts.push(format!("architecture={}", arch));
            }
            if let Some(shards) = meta.shard_count {
                parts.push(format!("shards={}", shards));
            }
            if let Some(total) = meta.total_size_bytes {
                parts.push(format!("total_size={:.1}GB", total as f64 / 1_073_741_824.0));
            }
            if let Some(ref dtype) = meta.torch_dtype {
                parts.push(format!("dtype={}", dtype));
            }
            if let Some(vocab) = meta.vocab_size {
                parts.push(format!("vocab_size={}", vocab));
            }
            // v1.0.12: Rich metadata summary
            if let Some(ref mt) = meta.model_type {
                parts.push(format!("model_type={}", mt));
            }
            if let Some(ctx) = meta.max_position_embeddings.or(meta.model_max_length) {
                parts.push(format!("context={}", ctx));
            }
            let mut modalities = vec!["text"];
            if meta.has_vision == Some(true) { modalities.push("vision"); }
            if meta.has_audio == Some(true) { modalities.push("audio"); }
            if meta.has_video == Some(true) { modalities.push("video"); }
            if modalities.len() > 1 {
                parts.push(format!("modalities={}", modalities.join("+")));
            }
            if !parts.is_empty() {
                return Some(format!("AI Model (Safetensors): {}", parts.join(", ")));
            }
            return Some("AI Model (Safetensors)".to_string());
        }
    }

    dep.source_file.clone()
}

/// Build SPDX checksums from dependency checksum fields.
fn build_checksums(dep: &Dependency) -> Vec<SPDXChecksum> {
    let mut checksums = Vec::new();
    if let Some(ref sha256) = dep.checksum_sha256 {
        checksums.push(SPDXChecksum {
            algorithm: "SHA256".to_string(),
            checksum_value: sha256.clone(),
        });
    }
    if let Some(ref sha512) = dep.checksum_sha512 {
        checksums.push(SPDXChecksum {
            algorithm: "SHA512".to_string(),
            checksum_value: sha512.clone(),
        });
    }
    checksums
}

pub fn create_package_url(dep: &Dependency) -> String {
    let version_is_unknown = dep.version == "detected"
        || dep.version == "unspecified"
        || dep.version == "unknown"
        || dep.version.contains("$("); // guard against unexpanded Makefile refs

    // v1.0.0: Handle git-submodule with proper purl based on Git URL
    if dep.ecosystem == "git-submodule" {
        if let Some(ref url) = dep.repository_url {
            if let Some(info) = crate::parsers::parse_git_url(url) {
                if version_is_unknown {
                    return format!(
                        "pkg:{}/{}/{}",
                        info.host_type.purl_type(),
                        info.owner,
                        info.repo
                    );
                }
                return info.to_purl(&dep.version);
            }
        }
        // Fallback to generic purl for git submodules without parseable URL
        if version_is_unknown {
            return format!("pkg:generic/{}", dep.name);
        }
        return format!("pkg:generic/{}@{}", dep.name, dep.version);
    }

    // v1.0.9: GGUF AI model files
    if dep.ecosystem == "gguf" {
        // Use official pkg:huggingface type when source URL is HuggingFace
        if let Some(ref url) = dep.repository_url {
            if url.contains("huggingface.co") {
                if let Some((namespace, name)) = parse_huggingface_url(url) {
                    if version_is_unknown {
                        return format!("pkg:huggingface/{}/{}", namespace, name);
                    }
                    return format!("pkg:huggingface/{}/{}@{}", namespace, name, dep.version);
                }
            }
        }
        // Fallback: pkg:generic with type=gguf
        if version_is_unknown {
            return format!("pkg:generic/{}?type=gguf", dep.name);
        }
        return format!("pkg:generic/{}@{}?type=gguf", dep.name, dep.version);
    }

    // v1.0.11: Safetensors AI model files
    if dep.ecosystem == "safetensors" {
        // Use official pkg:huggingface type when source URL is HuggingFace
        if let Some(ref url) = dep.repository_url {
            if url.contains("huggingface.co") {
                if let Some((namespace, name)) = parse_huggingface_url(url) {
                    if version_is_unknown {
                        return format!("pkg:huggingface/{}/{}", namespace, name);
                    }
                    return format!("pkg:huggingface/{}/{}@{}", namespace, name, dep.version);
                }
            }
        }
        // Model name might be "org/model" — try constructing HuggingFace URL
        if dep.name.contains('/') {
            let parts: Vec<&str> = dep.name.splitn(2, '/').collect();
            if parts.len() == 2 {
                let (ns, model) = (parts[0], parts[1]);
                if version_is_unknown {
                    return format!("pkg:huggingface/{}/{}", ns, model);
                }
                return format!("pkg:huggingface/{}/{}@{}", ns, model, dep.version);
            }
        }
        // Fallback: pkg:generic with type=safetensors
        if version_is_unknown {
            return format!("pkg:generic/{}?type=safetensors", dep.name);
        }
        return format!(
            "pkg:generic/{}@{}?type=safetensors",
            dep.name, dep.version
        );
    }

    // v1.0.1: Handle cmake with Git URL parsing
    if dep.ecosystem == "cmake" {
        if let Some(ref url) = dep.repository_url {
            // Only parse as Git URL if it actually looks like a Git repo
            // (not an archive download like releases/download/v1.0.0/file.tar.gz)
            if crate::parsers::is_git_repo_url(url) {
                if let Some(info) = crate::parsers::parse_git_url(url) {
                    if version_is_unknown {
                        return format!(
                            "pkg:{}/{}/{}",
                            info.host_type.purl_type(),
                            info.owner,
                            info.repo
                        );
                    }
                    return info.to_purl(&dep.version);
                }
            }
        }
        // Fallback to generic purl for non-Git sources
        if version_is_unknown {
            return format!("pkg:generic/{}", dep.name);
        }
        return format!("pkg:generic/{}@{}", dep.name, dep.version);
    }

    let ecosystem_type = match dep.ecosystem.as_str() {
        "npm" | "npm (dev)" => "npm",
        "cargo" => "cargo",
        "pip" => "pypi",
        "go" => "golang",
        "rubygems" => "gem",
        "composer" => "composer",
        "maven" => "maven",
        "ros" => "ros",
        "vcpkg" => "vcpkg",
        "conan" => "conan", // v1.0.2: Conan C++ package manager
        "pkg-config" => {
            // v1.0.3
            if version_is_unknown {
                return format!("pkg:generic/{}?type=pkg-config", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=pkg-config", dep.name, dep.version);
        }
        "autotools" => {
            // v1.0.3
            if version_is_unknown {
                return format!("pkg:generic/{}?type=autotools", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=autotools", dep.name, dep.version);
        }
        "system" => {
            // v1.0.3 - system deps from Makefiles and Meson
            if version_is_unknown {
                return format!("pkg:generic/{}?type=system", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=system", dep.name, dep.version);
        }
        "autosar" => {
            // AUTOSAR BSW components have no canonical registry;
            // use pkg:generic with type=autosar to preserve ecosystem context.
            if version_is_unknown {
                return format!("pkg:generic/{}?type=autosar", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=autosar", dep.name, dep.version);
        }
        "vendored" => {
            // v1.0.8 - vendored C/C++ libraries from library.json
            if version_is_unknown {
                return format!("pkg:generic/{}?type=vendored", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=vendored", dep.name, dep.version);
        }
        "micropython" => {
            if version_is_unknown {
                return format!("pkg:generic/{}?type=micropython", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=micropython", dep.name, dep.version);
        }
        // v1.0.4: Meson and Bazel build systems
        "meson" => {
            if version_is_unknown {
                return format!("pkg:generic/{}?type=meson", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=meson", dep.name, dep.version);
        }
        "meson-wrap" => {
            // For git-based wraps, try to create a proper purl
            if let Some(ref url) = dep.repository_url {
                // Only parse as git URL if it's actually a git repo (not an archive download)
                if crate::parsers::is_git_repo_url(url) {
                    if let Some(info) = crate::parsers::parse_git_url(url) {
                        let host = match info.host_type {
                            crate::parsers::GitHostType::GitHub => "github",
                            crate::parsers::GitHostType::GitLab => "gitlab",
                            crate::parsers::GitHostType::Bitbucket => "bitbucket",
                            _ => "generic",
                        };
                        if version_is_unknown {
                            return format!(
                                "pkg:{}/{}/{}?type=meson-wrap",
                                host, info.owner, info.repo
                            );
                        }
                        return format!(
                            "pkg:{}/{}/{}@{}?type=meson-wrap",
                            host, info.owner, info.repo, dep.version
                        );
                    }
                }
            }
            if version_is_unknown {
                return format!("pkg:generic/{}?type=meson-wrap", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=meson-wrap", dep.name, dep.version);
        }
        "meson-subproject" => {
            if version_is_unknown {
                return format!("pkg:generic/{}?type=meson-subproject", dep.name);
            }
            return format!(
                "pkg:generic/{}@{}?type=meson-subproject",
                dep.name, dep.version
            );
        }
        "bazel" | "bazel-bzlmod" => {
            // For git-based dependencies, try to create a proper purl
            if let Some(ref url) = dep.repository_url {
                // Only parse as git URL if it's actually a git repo (not an archive download)
                if crate::parsers::is_git_repo_url(url) {
                    if let Some(info) = crate::parsers::parse_git_url(url) {
                        let host = match info.host_type {
                            crate::parsers::GitHostType::GitHub => "github",
                            crate::parsers::GitHostType::GitLab => "gitlab",
                            crate::parsers::GitHostType::Bitbucket => "bitbucket",
                            _ => "generic",
                        };
                        if version_is_unknown {
                            return format!("pkg:{}/{}/{}?type=bazel", host, info.owner, info.repo);
                        }
                        return format!(
                            "pkg:{}/{}/{}@{}?type=bazel",
                            host, info.owner, info.repo, dep.version
                        );
                    }
                }
            }
            if version_is_unknown {
                return format!("pkg:generic/{}?type=bazel", dep.name);
            }
            return format!("pkg:generic/{}@{}?type=bazel", dep.name, dep.version);
        }
        _ => "generic",
    };

    let version_part = if version_is_unknown {
        String::new()
    } else {
        format!("@{}", dep.version)
    };
    format!("pkg:{}/{}{}", ecosystem_type, dep.name, version_part)
}

/// Parse HuggingFace URL to extract (namespace, model_name)
/// Handles: https://huggingface.co/org/model, https://huggingface.co/org/model/blob/main/file.gguf
fn parse_huggingface_url(url: &str) -> Option<(String, String)> {
    // Strip query parameters and fragments before processing
    let url = url.split('?').next().unwrap_or(url);
    let url = url.split('#').next().unwrap_or(url);
    let url = url.trim_end_matches('/');
    // Remove protocol
    let path = url
        .strip_prefix("https://huggingface.co/")
        .or_else(|| url.strip_prefix("http://huggingface.co/"))
        .or_else(|| url.strip_prefix("hf://"))?;
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        // Filter out non-model paths (datasets/, spaces/)
        if parts[0] == "datasets" || parts[0] == "spaces" {
            return None;
        }
        Some((parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

pub fn save_spdx_json(
    sbom: &Sbom,
    path: &str,
    compact_spdx: bool,
    supplier_resolver: Option<&SupplierResolver>,
) -> Result<()> {
    let spdx_doc = convert_to_spdx(sbom, compact_spdx, supplier_resolver);
    let json = serde_json::to_string_pretty(&spdx_doc)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn save_spdx_tag_value(
    sbom: &Sbom,
    path: &str,
    compact_spdx: bool,
    supplier_resolver: Option<&SupplierResolver>,
) -> Result<()> {
    let spdx_doc = convert_to_spdx(sbom, compact_spdx, supplier_resolver);
    let mut output = String::new();

    writeln!(output, "SPDXVersion: {}", spdx_doc.spdx_version)?;
    writeln!(output, "DataLicense: {}", spdx_doc.data_license)?;
    writeln!(output, "SPDXID: {}", spdx_doc.spdx_id)?;
    writeln!(output, "DocumentName: {}", spdx_doc.name)?;
    writeln!(output, "DocumentNamespace: {}", spdx_doc.document_namespace)?;
    writeln!(
        output,
        "Creator: {}",
        spdx_doc.creation_info.creators.join(", ")
    )?;
    writeln!(output, "Created: {}", spdx_doc.creation_info.created)?;
    writeln!(output)?;

    for package in &spdx_doc.packages {
        writeln!(output, "PackageName: {}", package.name)?;
        writeln!(output, "SPDXID: {}", package.spdx_id)?;
        // PackageVersion is optional per SPDX 2.3 spec — omit when version is unknown
        // (pyspdxtools rejects "PackageVersion: NOASSERTION" as invalid grammar)
        if package.version_info != "NOASSERTION" {
            writeln!(output, "PackageVersion: {}", package.version_info)?;
        }
        writeln!(
            output,
            "PackageDownloadLocation: {}",
            package.download_location
        )?;
        writeln!(output, "FilesAnalyzed: {}", package.files_analyzed)?;
        writeln!(
            output,
            "PackageLicenseConcluded: {}",
            package.license_concluded
        )?;
        writeln!(
            output,
            "PackageLicenseDeclared: {}",
            package.license_declared
        )?;
        writeln!(output, "PackageCopyrightText: {}", package.copyright_text)?;

        for ext_ref in &package.external_refs {
            writeln!(
                output,
                "ExternalRef: {} {} {}",
                ext_ref.reference_category, ext_ref.reference_type, ext_ref.reference_locator
            )?;
        }
        writeln!(output)?;
    }

    for rel in &spdx_doc.relationships {
        writeln!(
            output,
            "Relationship: {} {} {}",
            rel.spdx_element_id, rel.relationship_type, rel.related_spdx_element
        )?;
    }

    fs::write(path, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Dependency, DependencySource};

    fn create_test_dep(
        name: &str,
        version: &str,
        ecosystem: &str,
        repo_url: Option<String>,
    ) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: version.to_string(),
            ecosystem: ecosystem.to_string(),
            source: DependencySource::Manifest,
            source_file: Some("test".to_string()),
            is_dev: false,
            is_direct: true,
            repository_url: repo_url,
            ..Default::default()
        }
    }

    #[test]
    fn test_purl_meson_wrap_git() {
        let dep = create_test_dep(
            "zlib",
            "1.2.11",
            "meson-wrap",
            Some("https://github.com/madler/zlib.git".to_string()),
        );
        let purl = create_package_url(&dep);
        assert_eq!(purl, "pkg:github/madler/zlib@1.2.11?type=meson-wrap");
    }

    #[test]
    fn test_purl_meson_wrap_archive() {
        let dep = create_test_dep(
            "zlib",
            "1.2.11",
            "meson-wrap",
            Some("https://github.com/madler/zlib/archive/v1.2.11.tar.gz".to_string()),
        );
        let purl = create_package_url(&dep);
        // Archive URLs should not be parsed as git repos
        assert_eq!(purl, "pkg:generic/zlib@1.2.11?type=meson-wrap");
    }

    #[test]
    fn test_purl_meson_wrap_no_url() {
        let dep = create_test_dep("zlib", "1.2.11", "meson-wrap", None);
        let purl = create_package_url(&dep);
        assert_eq!(purl, "pkg:generic/zlib@1.2.11?type=meson-wrap");
    }

    #[test]
    fn test_purl_bazel_git_repository() {
        let dep = create_test_dep(
            "abseil-cpp",
            "20230802.1",
            "bazel",
            Some("https://github.com/abseil/abseil-cpp.git".to_string()),
        );
        let purl = create_package_url(&dep);
        assert_eq!(purl, "pkg:github/abseil/abseil-cpp@20230802.1?type=bazel");
    }

    #[test]
    fn test_purl_bazel_http_archive() {
        let dep = create_test_dep(
            "googletest",
            "1.12.1",
            "bazel",
            Some("https://github.com/google/googletest/archive/release-1.12.1.tar.gz".to_string()),
        );
        let purl = create_package_url(&dep);
        // Archive URLs should not be parsed as git repos
        assert_eq!(purl, "pkg:generic/googletest@1.12.1?type=bazel");
    }

    #[test]
    fn test_purl_bazel_bzlmod() {
        let dep = create_test_dep("abseil-cpp", "20230802.1", "bazel-bzlmod", None);
        let purl = create_package_url(&dep);
        assert_eq!(purl, "pkg:generic/abseil-cpp@20230802.1?type=bazel");
    }

    #[test]
    fn test_purl_meson_subproject() {
        let dep = create_test_dep("libfoo", "1.0.0", "meson-subproject", None);
        let purl = create_package_url(&dep);
        assert_eq!(purl, "pkg:generic/libfoo@1.0.0?type=meson-subproject");
    }
}
