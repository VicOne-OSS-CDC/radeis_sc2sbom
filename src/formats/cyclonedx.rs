use crate::formats::spdx::create_package_url;
use crate::models::{
    AIModelMetadata, AutosarMetadata, Dependency, DependencySource, RosPackageWithDeps, Sbom,
};
use crate::supplier::SupplierResolver;
use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;

/// Tool version, synchronized with Cargo.toml
const VERSION: &str = env!("CARGO_PKG_VERSION");

// CycloneDX 1.5 Data Structures
/// CycloneDX 1.5 document structure
/// Spec: https://cyclonedx.org/docs/1.5/json/
#[derive(Debug, Serialize)]
pub struct CycloneDXDocument {
    #[serde(rename = "bomFormat")]
    pub bom_format: String,

    #[serde(rename = "specVersion")]
    pub spec_version: String,

    #[serde(rename = "serialNumber")]
    pub serial_number: String,

    pub version: u32,

    pub metadata: CycloneDXMetadata,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<CycloneDXComponent>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub dependencies: Vec<CycloneDXDependency>,

}

#[derive(Debug, Serialize)]
pub struct CycloneDXMetadata {
    pub timestamp: String,
    pub tools: CycloneDXToolsContainer,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<CycloneDXComponent>,
}

#[derive(Debug, Serialize)]
pub struct CycloneDXToolsContainer {
    pub components: Vec<CycloneDXToolComponent>,
}

#[derive(Debug, Serialize)]
pub struct CycloneDXToolComponent {
    #[serde(rename = "type")]
    pub component_type: String,
    pub name: String,
    pub version: String,
}

/// CycloneDX component representing a software package
/// See: https://cyclonedx.org/docs/1.5/json/#components
#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXComponent {
    #[serde(rename = "type")]
    pub component_type: String,

    #[serde(rename = "bom-ref")]
    pub bom_ref: String,

    pub name: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub purl: Option<String>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub hashes: Vec<CycloneDXHash>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub licenses: Vec<CycloneDXLicenseChoice>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub properties: Vec<CycloneDXProperty>,

    #[serde(rename = "modelCard", skip_serializing_if = "Option::is_none")]
    pub model_card: Option<CycloneDXModelCard>,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub components: Vec<CycloneDXComponent>,
}

/// CycloneDX hash entry
#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXHash {
    pub alg: String,
    pub content: String,
}

/// CycloneDX license entry
/// See: https://cyclonedx.org/docs/1.5/json/#components_items_licenses
#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXLicenseChoice {
    pub license: CycloneDXLicense,
}

#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXLicense {
    /// SPDX license identifier (e.g. "Apache-2.0", "MIT") — mutually exclusive with `name`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Free-form license name for non-SPDX licenses — mutually exclusive with `id`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Check whether a license string is a known SPDX identifier.
/// If true, it should be placed in the `id` field; otherwise use `name`.
fn is_known_spdx_id(license: &str) -> bool {
    matches!(license,
        "Apache-2.0" | "MIT" | "GPL-2.0-only" | "GPL-3.0-only" |
        "LGPL-2.1-only" | "LGPL-3.0-only" | "BSD-2-Clause" | "BSD-3-Clause" |
        "MPL-2.0" | "ISC" | "Unlicense" | "0BSD" |
        "CC-BY-4.0" | "CC-BY-SA-4.0" | "CC-BY-NC-4.0" | "CC-BY-NC-SA-4.0" | "CC-BY-NC-ND-4.0" |
        "AGPL-3.0-only" | "Artistic-2.0" | "WTFPL" | "Zlib" | "BSL-1.0"
    )
}

#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXProperty {
    pub name: String,
    pub value: String,
}

/// CycloneDX 1.5 Model Card for machine-learning-model components
#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXModelCard {
    #[serde(rename = "modelParameters", skip_serializing_if = "Option::is_none")]
    pub model_parameters: Option<CycloneDXModelParameters>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXModelParameters {
    #[serde(rename = "approach", skip_serializing_if = "Option::is_none")]
    pub approach: Option<CycloneDXModelApproach>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub datasets: Vec<CycloneDXModelDataset>,
}

#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXModelApproach {
    #[serde(rename = "type")]
    pub approach_type: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct CycloneDXModelDataset {
    #[serde(rename = "type")]
    pub dataset_type: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CycloneDXDependency {
    #[serde(rename = "ref")]
    pub reference: String,

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    #[serde(rename = "dependsOn")]
    pub depends_on: Vec<String>,
}

pub fn convert_to_cyclonedx(
    sbom: &Sbom,
    supplier_resolver: Option<&SupplierResolver>,
) -> CycloneDXDocument {
    let project_name = sbom
        .project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown-project");

    let serial_number = format!("urn:uuid:{}", uuid::Uuid::new_v4());

    let mut components = Vec::new();
    let mut dependencies = Vec::new();

    // Root component representing the project
    let root_bom_ref = format!("project-{}", project_name);

    // Filter dependencies based on mode
    #[cfg(feature = "internal")]
    let filtered_deps: Vec<&Dependency> = sbom.dependencies.iter().collect::<Vec<_>>();
    #[cfg(not(feature = "internal"))]
    let filtered_deps: Vec<&Dependency> = sbom.dependencies.iter().collect();

    // Convert filtered dependencies to Vec<Dependency> for compatibility
    let filtered_deps_owned: Vec<Dependency> = filtered_deps.into_iter().cloned().collect();

    // Handle ROS multi-package vs flat structure
    if !sbom.ros_packages.is_empty() {
        convert_ros_packages_to_cyclonedx(&sbom.ros_packages, &mut components, &mut dependencies, supplier_resolver);
    } else {
        convert_flat_dependencies_to_cyclonedx(
            &filtered_deps_owned,
            &mut components,
            &mut dependencies,
            supplier_resolver,
        );
    }

    // C2 fix: add root project component to the dependencies array so the
    // dependency graph is connected from root → all components.
    // CycloneDX 1.5 spec requires the root metadata.component to appear in dependencies.
    // For ROS scans, root depends directly on ROS package components only (not leaf deps —
    // those are already expressed via the ros-package-N → dep-* relationships).
    // For flat scans, all components are direct dependencies of the root.
    let root_depends_on: Vec<String> = if !sbom.ros_packages.is_empty() {
        components
            .iter()
            .filter(|c| c.bom_ref.starts_with("ros-package-"))
            .map(|c| c.bom_ref.clone())
            .collect()
    } else {
        components.iter().map(|c| c.bom_ref.clone()).collect()
    };
    dependencies.push(CycloneDXDependency {
        reference: root_bom_ref.clone(),
        depends_on: root_depends_on,
    });

    let mut doc = CycloneDXDocument {
        bom_format: "CycloneDX".to_string(),
        spec_version: "1.5".to_string(),
        serial_number,
        version: 1,
        metadata: create_cyclonedx_metadata(&sbom.generated_at, project_name, &root_bom_ref),
        components,
        dependencies,
    };
    doc
}

fn convert_ros_packages_to_cyclonedx(
    ros_packages: &[RosPackageWithDeps],
    components: &mut Vec<CycloneDXComponent>,
    dependencies: &mut Vec<CycloneDXDependency>,
    supplier_resolver: Option<&SupplierResolver>,
) {
    // C1 fix: track seen deps by (name, version, ecosystem) to avoid duplicate components.
    // CycloneDX 1.5 spec states components MUST NOT be duplicated — a shared library
    // depended on by N ROS packages must appear exactly once.
    // Value: (bom_ref, component_index) so we can update properties on later occurrences.
    let mut seen_deps: std::collections::HashMap<(String, String, String), (String, usize)> =
        std::collections::HashMap::new();

    for (pkg_idx, ros_pkg) in ros_packages.iter().enumerate() {
        let pkg_bom_ref = format!("ros-package-{}", pkg_idx + 1);

        // Create main ROS package component
        let ros_component = CycloneDXComponent {
            component_type: "application".to_string(),
            bom_ref: pkg_bom_ref.clone(),
            name: ros_pkg.metadata.name.clone(),
            version: Some(ros_pkg.metadata.version.clone()),
            purl: Some(format!(
                "pkg:ros/{}@{}",
                ros_pkg.metadata.name, ros_pkg.metadata.version
            )),
            hashes: vec![],
            licenses: vec![],
            properties: vec![],
            model_card: None,
            components: vec![],
        };
        components.push(ros_component);

        let mut depends_on_refs = Vec::new();

        // Convert each dependency to component, deduplicating shared libraries.
        // If the same dep appears with is_dev=false in any ROS package, the component
        // must NOT be marked dev-only (a non-dev consumer overrides a dev-only one).
        for dep in ros_pkg.dependencies.iter() {
            let dep_key = (dep.name.clone(), dep.version.clone(), dep.ecosystem.clone());

            let dep_bom_ref = if let Some((existing_ref, comp_idx)) = seen_deps.get(&dep_key) {
                // Shared dep already has a component — reuse its bom-ref.
                // If this occurrence is non-dev but the existing component is marked dev,
                // remove the dev-dependency property so the component is correctly non-dev.
                if !dep.is_dev {
                    let comp = &mut components[*comp_idx];
                    comp.properties.retain(|p| p.name != "dev-dependency");
                }
                existing_ref.clone()
            } else {
                let new_ref = format!("dep-{}-{}", dep.ecosystem, seen_deps.len() + 1);
                let dep_component = create_dependency_component(dep, &new_ref, supplier_resolver);
                let comp_idx = components.len();
                components.push(dep_component);
                seen_deps.insert(dep_key, (new_ref.clone(), comp_idx));
                new_ref
            };

            if !depends_on_refs.contains(&dep_bom_ref) {
                depends_on_refs.push(dep_bom_ref);
            }
        }

        // Create dependency relationship
        dependencies.push(CycloneDXDependency {
            reference: pkg_bom_ref,
            depends_on: depends_on_refs,
        });
    }
}

fn convert_flat_dependencies_to_cyclonedx(
    deps: &[Dependency],
    components: &mut Vec<CycloneDXComponent>,
    _dependencies: &mut Vec<CycloneDXDependency>,
    supplier_resolver: Option<&SupplierResolver>,
) {
    // Convert each dependency to component
    for (idx, dep) in deps.iter().enumerate() {
        let dep_bom_ref = format!("dep-{}-{}", dep.ecosystem, idx + 1);

        let component = create_dependency_component(dep, &dep_bom_ref, supplier_resolver);
        components.push(component);
    }

    // Note: CycloneDX dependencies show direct relationships only
    // Transitive dependencies are components without being in dependsOn arrays
}

pub fn create_dependency_component(dep: &Dependency, bom_ref: &str, supplier_resolver: Option<&SupplierResolver>) -> CycloneDXComponent {
    let purl = create_package_url(dep);

    let mut properties = Vec::new();

    // Add dev dependency property
    if dep.is_dev {
        properties.push(CycloneDXProperty {
            name: "dev-dependency".to_string(),
            value: "true".to_string(),
        });
    }

    // Add source property
    let source_str = match dep.source {
        DependencySource::Manifest => "manifest",
        DependencySource::LockFile => "lock-file",
        DependencySource::ImportScan => "import-scan",
    };
    properties.push(CycloneDXProperty {
        name: "dependency-source".to_string(),
        value: source_str.to_string(),
    });

    // Add direct/transitive property
    properties.push(CycloneDXProperty {
        name: "dependency-scope".to_string(),
        value: if dep.is_direct {
            "direct"
        } else {
            "transitive"
        }
        .to_string(),
    });

    // Build model_card and AI-specific properties when ai_model_metadata is present
    let model_card = if let Some(meta) = &dep.ai_model_metadata {
        add_ai_model_properties(&mut properties, meta);
        // Indicate when and why hash was skipped
        if let Some(ref reason) = meta.hash_skip_reason {
            properties.push(CycloneDXProperty {
                name: "radeis:ai:hash_status".to_string(),
                value: format!("skipped — {}", reason),
            });
        }
        build_model_card(meta)
    } else {
        None
    };

    // v1.0.15 (Phase 7, OUT-02): emit autosar:layer + autosar:platform properties
    if let Some(ref meta) = dep.autosar_metadata {
        add_autosar_properties(&mut properties, meta);
    }

    // Phase 8 (v1.0.15): Emit autosar:supplier for every AUTOSAR component.
    // Always emit the property — value is NOASSERTION when the resolver is
    // absent or has no entry for this component name (per D-09).
    if dep.autosar_metadata.is_some() {
        let supplier_value = supplier_resolver
            .and_then(|r| r.lookup(&dep.name))
            .unwrap_or("NOASSERTION");
        properties.push(CycloneDXProperty {
            name: "autosar:supplier".to_string(),
            value: supplier_value.to_string(),
        });
    }

    // Build licenses array from dependency license field
    let licenses = dep
        .license
        .as_ref()
        .filter(|l| !l.is_empty())
        .map(|l| {
            let license = if is_known_spdx_id(l) {
                CycloneDXLicense { id: Some(l.clone()), name: None }
            } else {
                CycloneDXLicense { id: None, name: Some(l.clone()) }
            };
            vec![CycloneDXLicenseChoice { license }]
        })
        .unwrap_or_default();

    // Build nested sub-model components for AI models
    let sub_components = if let Some(ref meta) = dep.ai_model_metadata {
        meta.sub_models
            .iter()
            .enumerate()
            .map(|(idx, sm)| {
                let mut sm_props = Vec::new();
                // Helper to reduce repetition when pushing optional properties
                let mut push_str = |name: &str, value: &str| {
                    sm_props.push(CycloneDXProperty {
                        name: format!("radeis:ai:sub_model:{}", name),
                        value: value.to_string(),
                    });
                };
                push_str("modality", &sm.modality);
                if let Some(ref v) = sm.model_type { push_str("model_type", v); }
                if let Some(v) = sm.num_hidden_layers { push_str("num_hidden_layers", &v.to_string()); }
                if let Some(v) = sm.hidden_size { push_str("hidden_size", &v.to_string()); }
                if let Some(v) = sm.num_attention_heads { push_str("num_attention_heads", &v.to_string()); }
                if let Some(v) = sm.num_key_value_heads { push_str("num_key_value_heads", &v.to_string()); }
                if let Some(v) = sm.max_position_embeddings { push_str("max_position_embeddings", &v.to_string()); }
                if let Some(v) = sm.vocab_size { push_str("vocab_size", &v.to_string()); }
                if let Some(ref v) = sm.dtype { push_str("dtype", v); }
                if let Some(v) = sm.intermediate_size { push_str("intermediate_size", &v.to_string()); }
                if let Some(v) = sm.patch_size { push_str("patch_size", &v.to_string()); }
                if let Some(v) = sm.default_output_length { push_str("default_output_length", &v.to_string()); }
                if let Some(v) = sm.conv_kernel_size { push_str("conv_kernel_size", &v.to_string()); }
                if let Some(v) = sm.output_proj_dims { push_str("output_proj_dims", &v.to_string()); }
                CycloneDXComponent {
                    component_type: "machine-learning-model".to_string(),
                    bom_ref: format!("{}-sub-{}", bom_ref, idx),
                    name: sm.model_type.clone().unwrap_or_else(|| sm.modality.clone()),
                    version: None,
                    purl: None,
                    hashes: vec![],
                    licenses: vec![],
                    properties: sm_props,
                    model_card: None,
                    components: vec![],
                }
            })
            .collect()
    } else {
        vec![]
    };

    CycloneDXComponent {
        component_type: if dep.ai_model_metadata.is_some() {
            "machine-learning-model".to_string()
        } else {
            "library".to_string()
        },
        bom_ref: bom_ref.to_string(),
        name: dep.name.clone(),
        version: if dep.version == "detected"
            || dep.version == "unspecified"
            || dep.version == "unknown"
            || dep.version.contains("$(")
        {
            None
        } else {
            Some(dep.version.clone())
        },
        purl: Some(purl),
        hashes: build_cyclonedx_hashes(dep),
        licenses,
        properties,
        model_card,
        components: sub_components,
    }
}

/// Build CycloneDX hashes from dependency checksum fields
fn build_cyclonedx_hashes(dep: &Dependency) -> Vec<CycloneDXHash> {
    let mut hashes = Vec::new();
    if let Some(ref sha256) = dep.checksum_sha256 {
        hashes.push(CycloneDXHash {
            alg: "SHA-256".to_string(),
            content: sha256.clone(),
        });
    }
    if let Some(ref sha512) = dep.checksum_sha512 {
        hashes.push(CycloneDXHash {
            alg: "SHA-512".to_string(),
            content: sha512.clone(),
        });
    }
    hashes
}

/// Add AI model properties (GGUF + Safetensors) to the CycloneDX properties vec
fn add_ai_model_properties(properties: &mut Vec<CycloneDXProperty>, meta: &AIModelMetadata) {
    if let Some(ref arch) = meta.architecture {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:architecture".to_string(),
            value: arch.clone(),
        });
    }
    if let Some(ref quant) = meta.quantization {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:quantization".to_string(),
            value: quant.clone(),
        });
    }
    if let Some(param_count) = meta.parameter_count {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:parameter_count".to_string(),
            value: param_count.to_string(),
        });
    }
    if let Some(computed) = meta.computed_parameter_count {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:computed_parameter_count".to_string(),
            value: computed.to_string(),
        });
        // Flag integrity mismatch if declared != computed
        if let Some(declared) = meta.parameter_count {
            if declared != computed {
                properties.push(CycloneDXProperty {
                    name: "radeis:ai:integrity_warning".to_string(),
                    value: format!(
                        "declared parameter_count ({}) != computed from tensors ({})",
                        declared, computed
                    ),
                });
            }
        }
        // Cross-validate size_label vs computed
        if let Some(ref size_label) = meta.size_label {
            if let Some(expected) = crate::parsers::gguf::parse_size_label_public(size_label) {
                let lower = (expected as f64 * 0.95) as u64;
                let upper = (expected as f64 * 1.05) as u64;
                if computed < lower || computed > upper {
                    properties.push(CycloneDXProperty {
                        name: "radeis:ai:integrity_warning:size_label".to_string(),
                        value: format!(
                            "size_label '{}' (~{} params) != computed from tensors ({})",
                            size_label, expected, computed
                        ),
                    });
                }
            }
        }
    }
    if let Some(tensor_count) = meta.tensor_count {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:tensor_count".to_string(),
            value: tensor_count.to_string(),
        });
    }
    if let Some(ref size_label) = meta.size_label {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:size_label".to_string(),
            value: size_label.clone(),
        });
    }
    if let Some(ref finetune) = meta.finetune {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:finetune".to_string(),
            value: finetune.clone(),
        });
    }
    if !meta.languages.is_empty() {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:languages".to_string(),
            value: meta.languages.join(","),
        });
    }
    for (i, base_model) in meta.base_models.iter().enumerate() {
        if let Some(ref name) = base_model.name {
            properties.push(CycloneDXProperty {
                name: format!("radeis:ai:base_model.{}.name", i),
                value: name.clone(),
            });
        }
    }

    // v1.0.11: Safetensors-specific properties
    if let Some(ref fmt) = meta.safetensors_format {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:safetensors_format".to_string(),
            value: fmt.clone(),
        });
    }
    if let Some(total_size) = meta.total_size_bytes {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:total_size_bytes".to_string(),
            value: total_size.to_string(),
        });
    }
    if let Some(shards) = meta.shard_count {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:shard_count".to_string(),
            value: shards.to_string(),
        });
    }
    if let Some(ref dtype) = meta.torch_dtype {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:torch_dtype".to_string(),
            value: dtype.clone(),
        });
    }
    if let Some(ref tv) = meta.transformers_version {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:transformers_version".to_string(),
            value: tv.clone(),
        });
    }
    if let Some(vocab) = meta.vocab_size {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:vocab_size".to_string(),
            value: vocab.to_string(),
        });
    }

    // v1.0.12: Rich metadata properties
    if let Some(ref val) = meta.model_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:model_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.num_hidden_layers {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:num_hidden_layers".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.hidden_size {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:hidden_size".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.num_attention_heads {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:num_attention_heads".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.max_position_embeddings {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:max_position_embeddings".to_string(),
            value: val.to_string(),
        });
    }
    if meta.has_vision == Some(true) {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:has_vision".to_string(),
            value: "true".to_string(),
        });
    }
    if meta.has_audio == Some(true) {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:has_audio".to_string(),
            value: "true".to_string(),
        });
    }
    if meta.has_video == Some(true) {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:has_video".to_string(),
            value: "true".to_string(),
        });
    }
    if let Some(ref val) = meta.vision_model_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:vision_model_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(ref val) = meta.audio_model_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:audio_model_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.generation_temperature {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:generation:temperature".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.generation_top_k {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:generation:top_k".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.generation_top_p {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:generation:top_p".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(ref val) = meta.processor_class {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:processor_class".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.model_max_length {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:model_max_length".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(ref val) = meta.image_processor_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:image_processor_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.image_seq_length {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:image_seq_length".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(ref val) = meta.audio_feature_extractor_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:audio_feature_extractor_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.audio_sampling_rate {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:audio_sampling_rate".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(val) = meta.audio_seq_length {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:audio_seq_length".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(ref val) = meta.video_processor_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:video_processor_type".to_string(),
            value: val.clone(),
        });
    }
    if let Some(val) = meta.video_num_frames {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:video_num_frames".to_string(),
            value: val.to_string(),
        });
    }
    if let Some(ref val) = meta.model_name {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:model_name".to_string(),
            value: val.clone(),
        });
    }
    if let Some(ref val) = meta.pipeline_tag {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:pipeline_tag".to_string(),
            value: val.clone(),
        });
    }
    if let Some(ref val) = meta.quantized_by {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:quantized_by".to_string(),
            value: val.clone(),
        });
    }
    if let Some(ref val) = meta.prompt_template {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:prompt_template".to_string(),
            value: val.clone(),
        });
    }
    if meta.is_adapter == Some(true) {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:is_adapter".to_string(),
            value: "true".to_string(),
        });
    }
    if let Some(ref val) = meta.adapter_type {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:adapter_type".to_string(),
            value: val.clone(),
        });
    }
    if !meta.datasets.is_empty() {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:datasets".to_string(),
            value: meta.datasets.join(","),
        });
    }
    if !meta.tags.is_empty() {
        properties.push(CycloneDXProperty {
            name: "radeis:ai:tags".to_string(),
            value: meta.tags.join(","),
        });
    }
}

/// Add AUTOSAR BSW properties to the CycloneDX properties vec.
/// v1.0.15 (Phase 7, OUT-02): emits autosar:layer + autosar:platform.
fn add_autosar_properties(properties: &mut Vec<CycloneDXProperty>, meta: &AutosarMetadata) {
    properties.push(CycloneDXProperty {
        name: "autosar:layer".to_string(),
        value: meta.layer.clone(),
    });
    properties.push(CycloneDXProperty {
        name: "autosar:platform".to_string(),
        value: meta.platform.clone(),
    });
}

/// Build a CycloneDX ModelCard from AI model metadata.
/// Returns `None` when there is nothing meaningful to put in the model card
/// (avoids emitting an empty `"modelCard": {}`).
fn build_model_card(meta: &AIModelMetadata) -> Option<CycloneDXModelCard> {
    let datasets: Vec<CycloneDXModelDataset> = meta
        .datasets
        .iter()
        .map(|ds| CycloneDXModelDataset {
            dataset_type: "dataset".to_string(),
            name: ds.clone(),
        })
        .collect();

    if datasets.is_empty() {
        return None;
    }

    Some(CycloneDXModelCard {
        model_parameters: Some(CycloneDXModelParameters {
            approach: None,
            datasets,
        }),
    })
}

pub fn create_cyclonedx_metadata(
    timestamp: &str,
    project_name: &str,
    root_bom_ref: &str,
) -> CycloneDXMetadata {
    CycloneDXMetadata {
        timestamp: timestamp.to_string(),
        tools: CycloneDXToolsContainer {
            components: vec![CycloneDXToolComponent {
                component_type: "application".to_string(),
                name: "radeis_sc2sbom".to_string(),
                version: VERSION.to_string(),
            }],
        },
        component: Some(CycloneDXComponent {
            component_type: "application".to_string(),
            bom_ref: root_bom_ref.to_string(),
            name: project_name.to_string(),
            version: None,
            purl: None,
            hashes: vec![],
            licenses: vec![],
            properties: vec![],
            model_card: None,
            components: vec![],
        }),
    }
}

pub fn print_cyclonedx_json(
    sbom: &Sbom,
    supplier_resolver: Option<&SupplierResolver>,
) -> Result<()> {
    let cdx_doc = convert_to_cyclonedx(
        sbom,
        supplier_resolver,
    );
    let json = serde_json::to_string_pretty(&cdx_doc)?;
    println!("{}", json);
    Ok(())
}

pub fn save_cyclonedx_json(
    sbom: &Sbom,
    path: &str,
    supplier_resolver: Option<&SupplierResolver>,
) -> Result<()> {
    let cdx_doc = convert_to_cyclonedx(
        sbom,
        supplier_resolver,
    );
    let json = serde_json::to_string_pretty(&cdx_doc)?;
    fs::write(path, json)?;
    Ok(())
}
