use std::collections::HashMap;
use std::path::PathBuf;

use super::sbom::{RosPackageMetadata, RosPackageWithDeps};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum DependencySource {
    Manifest,
    LockFile,
    ImportScan,
}

/// Dependency scope classification (v1.0.6)
///
/// Classifies dependencies by their role in the software lifecycle:
/// - Runtime: Required at runtime in production
/// - Build: Required only during build process
/// - Test: Required only for testing
/// - Development: Required only for development (linters, formatters, etc.)
/// - Optional: Optional features that may or may not be included
/// - Provided: Provided by the environment (e.g., system libraries, submodules)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash)]
pub enum DependencyScope {
    Runtime,
    Build,
    Test,
    Development,
    Optional,
    Provided,
}

impl Default for DependencyScope {
    fn default() -> Self {
        DependencyScope::Runtime
    }
}

/// AI model metadata extracted from GGUF files (v1.0.9)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIModelMetadata {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub architecture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub quantization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub basename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub finetune: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub size_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub parameter_count: Option<u64>,
    /// Computed parameter count from summing tensor dimensions (for integrity validation)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub computed_parameter_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub tensor_count: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub gguf_version: Option<u32>,
    /// Base model provenance — also receives base_model entries parsed from README frontmatter
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub base_models: Vec<BaseModelInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub datasets: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub languages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub license_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub license_link: Option<String>,
    /// Reason SHA-256 hash was skipped (e.g. "file exceeds limit", "I/O error")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hash_skip_reason: Option<String>,

    // v1.0.11: Safetensors-specific fields
    /// `__metadata__.format` from safetensors header (e.g. "pt")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub safetensors_format: Option<String>,
    /// Total model size in bytes (from model.safetensors.index.json `metadata.total_size`)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub total_size_bytes: Option<u64>,
    /// Number of .safetensors shard files
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub shard_count: Option<u32>,
    /// PyTorch dtype from config.json (e.g. "bfloat16", "float16", "float32")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub torch_dtype: Option<String>,
    /// HuggingFace Transformers version from config.json
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub transformers_version: Option<String>,
    /// Vocabulary size from config.json
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vocab_size: Option<u64>,

    // v1.0.12: Rich metadata from companion files

    // config.json — model architecture
    /// Model type identifier (e.g. "gemma4", "llama", "mistral")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub num_hidden_layers: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hidden_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub num_attention_heads: Option<u32>,
    /// Architectural context window size
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_position_embeddings: Option<u64>,

    // config.json — multimodal detection
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub has_vision: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub has_audio: Option<bool>,
    /// Detected from processor_config
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub has_video: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vision_model_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub audio_model_type: Option<String>,

    // v1.0.13: Multimodal sub-model decomposition
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sub_models: Vec<SubModelInfo>,

    // generation_config.json
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub generation_temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub generation_top_k: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub generation_top_p: Option<f32>,

    // tokenizer_config.json + preprocessor_config.json
    /// Processor class name (e.g. "Gemma4Processor")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub processor_class: Option<String>,
    /// Maximum sequence length (capped at MODEL_MAX_LENGTH_CAP)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_max_length: Option<u64>,

    // preprocessor_config.json — multimodal processors
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub image_processor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub image_seq_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub audio_feature_extractor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub audio_sampling_rate: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub audio_seq_length: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub video_processor_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub video_num_frames: Option<u32>,

    // README.md frontmatter (both Safetensors and GGUF)
    // NOTE: base_model from README is pushed into `base_models: Vec<BaseModelInfo>` above
    /// Human-readable model name
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_name: Option<String>,
    /// Pipeline tag (e.g. "text-generation", "image-text-to-text")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub pipeline_tag: Option<String>,
    /// Supply chain: who quantized the model
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub quantized_by: Option<String>,
    /// Prompt template (capped at MAX_PROMPT_TEMPLATE_LEN)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub prompt_template: Option<String>,

    // Adapter detection
    /// True if adapter_config.json is present
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub is_adapter: Option<bool>,
    /// Adapter type (e.g. "LORA", "QLORA")
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub adapter_type: Option<String>,
}

/// AUTOSAR BSW component classification metadata (v1.0.15, Phase 7)
///
/// Populated by classifier::autosar::classify_autosar_components when a
/// dependency name matches a curated BSW module entry. Emitted as
/// CycloneDX properties (autosar:layer, autosar:platform) and SPDX
/// ExternalRefs (referenceCategory: OTHER).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutosarMetadata {
    /// Canonical module name from the BSW config (e.g. "NvM"). Preserves
    /// original casing from the config file even when the input dep name
    /// was different case.
    pub module_name: String,
    /// AUTOSAR R22-11 layer name. One of: MCAL, BSW-Services,
    /// BSW-Communication, BSW-Memory, RTE, OS, Complex-Drivers.
    pub layer: String,
    /// AUTOSAR platform: "Classic" or "Adaptive".
    pub platform: String,
}

/// Base model provenance information for AI model lineage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseModelInfo {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub repo_url: Option<String>,
}

/// Sub-model component within a multimodal AI model (v1.0.13)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubModelInfo {
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub model_type: Option<String>,
    pub modality: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub num_hidden_layers: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hidden_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub num_attention_heads: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub num_key_value_heads: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub max_position_embeddings: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub vocab_size: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub dtype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub intermediate_size: Option<u32>,
    // Vision-specific
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub patch_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub default_output_length: Option<u32>,
    // Audio-specific
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub conv_kernel_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub output_proj_dims: Option<u32>,
}

impl Default for SubModelInfo {
    fn default() -> Self {
        Self {
            model_type: None,
            modality: "unknown".to_string(),
            num_hidden_layers: None,
            hidden_size: None,
            num_attention_heads: None,
            num_key_value_heads: None,
            max_position_embeddings: None,
            vocab_size: None,
            dtype: None,
            intermediate_size: None,
            patch_size: None,
            default_output_length: None,
            conv_kernel_size: None,
            output_proj_dims: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    #[serde(default = "default_source")]
    pub source: DependencySource,
    #[serde(default)]
    pub is_dev: bool,
    #[serde(default = "default_is_direct")]
    pub is_direct: bool,
    // v0.8.0: Metadata fields for license, supplier/originator, and source tracking
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub license: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub author: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub maintainers: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub repository_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub homepage_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub source_file: Option<String>,

    // v0.9.0: Package checksums for integrity verification
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksum_sha256: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub checksum_sha512: Option<String>,

    // v1.0.9: AI model metadata (GGUF); v1.0.11: extended for Safetensors
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ai_model_metadata: Option<AIModelMetadata>,

    /// v1.0.15 (Phase 7): AUTOSAR BSW classification metadata. Set by
    /// classifier::autosar::classify_autosar_components when name matches
    /// a BSW module entry. None for non-AUTOSAR components.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub autosar_metadata: Option<AutosarMetadata>,

    // v1.0.6: Dependency scope classification
    #[serde(default)]
    pub scope: DependencyScope,

    #[serde(skip)]
    pub scope_confidence: f32,

    #[serde(skip)]
    pub scope_reason: String,
}

pub fn default_source() -> DependencySource {
    DependencySource::Manifest
}

pub fn default_is_direct() -> bool {
    true
}

impl Default for Dependency {
    fn default() -> Self {
        Dependency {
            name: String::new(),
            version: String::new(),
            ecosystem: String::new(),
            source: default_source(),
            is_dev: false,
            is_direct: default_is_direct(),
            license: None,
            author: None,
            maintainers: None,
            repository_url: None,
            homepage_url: None,
            source_file: None,
            checksum_sha256: None,
            checksum_sha512: None,
            ai_model_metadata: None,
            autosar_metadata: None,
            scope: DependencyScope::default(),
            scope_confidence: 0.0,
            scope_reason: String::new(),
        }
    }
}

impl Dependency {
    /// Create a new Dependency with minimal required fields
    /// Other fields use sensible defaults and can be modified after creation
    #[allow(dead_code)]
    pub fn new(name: String, version: String, ecosystem: String) -> Self {
        Dependency {
            name,
            version,
            ecosystem,
            ..Default::default()
        }
    }

    /// Builder method to set source
    #[allow(dead_code)]
    pub fn with_source(mut self, source: DependencySource) -> Self {
        self.source = source;
        self
    }

    /// Builder method to set source_file
    #[allow(dead_code)]
    pub fn with_source_file(mut self, source_file: String) -> Self {
        self.source_file = Some(source_file);
        self
    }

    /// Builder method to set is_dev
    #[allow(dead_code)]
    pub fn with_is_dev(mut self, is_dev: bool) -> Self {
        self.is_dev = is_dev;
        self
    }

    /// Builder method to set is_direct
    #[allow(dead_code)]
    pub fn with_is_direct(mut self, is_direct: bool) -> Self {
        self.is_direct = is_direct;
        self
    }

    /// Builder method to set dependency scope classification (v1.0.6)
    #[allow(dead_code)]
    pub fn with_scope(mut self, scope: DependencyScope, confidence: f32, reason: &str) -> Self {
        self.scope = scope;
        self.scope_confidence = confidence;
        self.scope_reason = reason.to_string();
        self
    }
}

#[derive(Debug, Clone)]
pub struct DependencyRelationship {
    pub parent_id: String,
    pub child_names: Vec<String>,
}

#[derive(Debug)]
pub struct LockFileData {
    pub dependencies: Vec<Dependency>,
    pub relationships: Vec<DependencyRelationship>,
}

#[derive(Debug)]
pub struct ScanContext {
    pub dependencies: Vec<Dependency>,
    pub npm_relationships: Vec<DependencyRelationship>,
    pub cargo_relationships: Vec<DependencyRelationship>,
    pub python_lockfile_relationships: Vec<DependencyRelationship>,
    pub ros_metadata: Option<RosPackageMetadata>,
    pub ros_packages: Vec<RosPackageWithDeps>,
    /// v1.0.0: Git submodule relationships (submodule -> nested dependencies)
    pub git_submodule_relationships: Vec<DependencyRelationship>,
    /// v1.0.15: AUTOSAR project detection flag — set by detect_autosar() pre-pass (Phase 6, DET-01/02/03)
    pub is_autosar: bool,
    /// v1.0.16 (Phase 11, D-01/D-02/D-03): (name, ecosystem) -> source-tree directory mapping
    /// for C/C++ components. Used by the lexical CWE scanner (gated behind `feature = "internal"`)
    /// to scope file walking to component-mapped directories only (SCAN-05).
    /// The field itself is unconditional (stdlib types, zero-cost when empty); only the
    /// scanner consumer is feature-gated. Components without a recorded directory
    /// (e.g., so-scanner discoveries) have no entry and are skipped, not guessed.
    pub component_dirs: HashMap<(String, String), PathBuf>,
}
