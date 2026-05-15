use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::models::{AIModelMetadata, Dependency, DependencyScope, DependencySource, SubModelInfo};

/// Maximum bytes to read from a safetensors binary JSON header (50 MB safety cap).
const MAX_HEADER_BYTES: usize = 50 * 1024 * 1024;

/// Maximum bytes to read from companion files (1 MB safety cap).
pub(crate) const MAX_COMPANION_FILE_BYTES: u64 = 1_048_576;
/// model_max_length values above this cap are treated as "not set" (None).
const MODEL_MAX_LENGTH_CAP: u64 = u32::MAX as u64;
/// Maximum stored length for prompt_template strings.
pub(crate) const MAX_PROMPT_TEMPLATE_LEN: usize = 512;

// ── JSON deserialization helpers ─────────────────────────────────────────────

/// `__metadata__` dict inside a safetensors binary header.
#[derive(Debug, Deserialize, Default)]
struct SafetensorsMetadata {
    format: Option<String>,
    // Other author-defined keys are ignored.
    #[serde(flatten)]
    _extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Top-level JSON header of a safetensors binary file.
/// Keys other than `__metadata__` are tensor descriptors.
#[derive(Debug, Deserialize)]
struct SafetensorsHeader {
    #[serde(rename = "__metadata__", default)]
    metadata: SafetensorsMetadata,
    // Remaining keys are tensor names — we only need their count for now.
    #[serde(flatten)]
    tensors: std::collections::HashMap<String, serde_json::Value>,
}

/// `model.safetensors.index.json` top-level structure.
#[derive(Debug, Deserialize)]
struct SafetensorsIndexJson {
    metadata: Option<SafetensorsIndexMetadata>,
    weight_map: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct SafetensorsIndexMetadata {
    total_size: Option<u64>,
}

/// `config.json` — HuggingFace standard model configuration.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct HuggingFaceConfig {
    pub(crate) model_type: Option<String>,
    pub(crate) architectures: Option<Vec<String>>,
    pub(crate) torch_dtype: Option<String>,
    pub(crate) transformers_version: Option<String>,
    pub(crate) vocab_size: Option<u64>,
    #[serde(rename = "_name_or_path")]
    pub(crate) _name_or_path: Option<String>,
    // v1.0.12 additions:
    pub(crate) dtype: Option<String>,
    pub(crate) text_config: Option<TextConfig>,
    pub(crate) vision_config: Option<serde_json::Value>,
    pub(crate) audio_config: Option<serde_json::Value>,
    // Top-level architecture fields (non-multimodal models like LLaMA put these here
    // instead of inside text_config)
    pub(crate) num_hidden_layers: Option<u32>,
    pub(crate) hidden_size: Option<u32>,
    pub(crate) num_attention_heads: Option<u32>,
    pub(crate) max_position_embeddings: Option<u64>,
}

/// Nested text model configuration within config.json
#[derive(Debug, Deserialize, Default)]
pub(crate) struct TextConfig {
    pub(crate) dtype: Option<String>,
    pub(crate) model_type: Option<String>,
    pub(crate) num_hidden_layers: Option<u32>,
    pub(crate) hidden_size: Option<u32>,
    pub(crate) num_attention_heads: Option<u32>,
    pub(crate) num_key_value_heads: Option<u32>,
    pub(crate) vocab_size: Option<u64>,
    pub(crate) max_position_embeddings: Option<u64>,
    pub(crate) intermediate_size: Option<u32>,
}

/// generation_config.json — inference defaults.
/// NOTE: Do NOT add #[serde(deny_unknown_fields)] — many unknown fields exist (eos_token_id, etc.)
#[derive(Debug, Deserialize, Default)]
pub(crate) struct GenerationConfig {
    pub temperature: Option<f32>,
    pub top_k: Option<u32>,
    pub top_p: Option<f32>,
}

/// tokenizer_config.json — tokenizer settings.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct TokenizerConfig {
    pub processor_class: Option<String>,
    /// Can be astronomically large (e.g. 1e30 in Gemma-4); parsed as Value, then capped.
    pub model_max_length: Option<serde_json::Value>,
}

/// preprocessor_config.json — multimodal processor settings.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct PreprocessorConfig {
    pub processor_class: Option<String>,
    pub image_seq_length: Option<u32>,
    pub audio_seq_length: Option<u32>,
    pub image_processor: Option<ImageProcessorConfig>,
    pub feature_extractor: Option<FeatureExtractorConfig>,
    pub video_processor: Option<VideoProcessorConfig>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct ImageProcessorConfig {
    pub image_processor_type: Option<String>,
    pub image_seq_length: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct FeatureExtractorConfig {
    pub feature_extractor_type: Option<String>,
    pub sampling_rate: Option<u32>,
}

#[derive(Debug, Deserialize, Default)]
pub(crate) struct VideoProcessorConfig {
    pub video_processor_type: Option<String>,
    pub num_frames: Option<u32>,
}

/// Deserialize a field that can be either a string or a list of strings.
/// Used for HuggingFace README frontmatter `base_model` which is a string in
/// older repos and a list in merged/DARE-merged models.
fn deserialize_string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct StringOrVec;

    impl<'de> de::Visitor<'de> for StringOrVec {
        type Value = Vec<String>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or a list of strings")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Vec<String>, E> {
            Ok(vec![v.to_string()])
        }

        fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Vec<String>, A::Error> {
            let mut vec = Vec::new();
            while let Some(val) = seq.next_element::<String>()? {
                vec.push(val);
            }
            Ok(vec)
        }

        fn visit_unit<E: de::Error>(self) -> Result<Vec<String>, E> {
            Ok(Vec::new())
        }
    }

    deserializer.deserialize_any(StringOrVec)
}

/// README.md YAML frontmatter — model card metadata.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct ReadmeFrontmatter {
    #[serde(default, deserialize_with = "deserialize_string_or_vec")]
    pub base_model: Vec<String>,
    pub license: Option<String>,
    pub model_creator: Option<String>,
    pub model_name: Option<String>,
    pub pipeline_tag: Option<String>,
    pub quantized_by: Option<String>,
    pub prompt_template: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub language: Vec<String>,
    #[serde(default)]
    pub datasets: Vec<String>,
}

/// adapter_config.json — PEFT adapter detection.
#[derive(Debug, Deserialize, Default)]
pub(crate) struct AdapterConfig {
    pub peft_type: Option<String>,
    pub base_model_name_or_path: Option<String>,
}

// ── Binary header reader ─────────────────────────────────────────────────────

/// Read and parse the JSON header from a safetensors binary file.
fn read_safetensors_header(path: &Path) -> Result<SafetensorsHeader> {
    let file = File::open(path)
        .with_context(|| format!("failed to open safetensors file: {}", path.display()))?;
    let mut reader = BufReader::new(file);

    // First 8 bytes: u64 little-endian header size
    let mut size_buf = [0u8; 8];
    reader
        .read_exact(&mut size_buf)
        .context("failed to read safetensors header size")?;
    let header_size = u64::from_le_bytes(size_buf) as usize;

    if header_size == 0 || header_size > MAX_HEADER_BYTES {
        anyhow::bail!(
            "safetensors header size {} is out of range (max {})",
            header_size,
            MAX_HEADER_BYTES
        );
    }

    let mut json_buf = vec![0u8; header_size];
    reader
        .read_exact(&mut json_buf)
        .context("failed to read safetensors header JSON")?;

    let header: SafetensorsHeader = serde_json::from_slice(&json_buf)
        .context("failed to parse safetensors header JSON")?;

    Ok(header)
}

// ── Model directory detection ────────────────────────────────────────────────

/// Returns true if `dir` looks like a safetensors model directory.
/// Detection criteria (any one suffices):
/// 1. Contains `model.safetensors.index.json`
/// 2. Contains `model.safetensors`
/// 3. Contains `config.json` + at least one `*.safetensors` file
pub fn is_safetensors_model_dir(dir: &Path) -> bool {
    if dir.join("model.safetensors.index.json").is_file() {
        return true;
    }
    if dir.join("model.safetensors").is_file() {
        return true;
    }
    if dir.join("config.json").is_file() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.ends_with(".safetensors") {
                    return true;
                }
            }
        }
    }
    false
}

/// Recursively find all safetensors model directories under `root`.
/// Stops descending into a directory once it is identified as a model dir
/// (models are not expected to be nested inside each other).
/// Used in tests and available as a public library utility.
#[allow(dead_code)]
pub fn find_safetensors_model_dirs(root: &Path) -> Vec<PathBuf> {
    let mut result = Vec::new();
    find_safetensors_model_dirs_inner(root, &mut result);
    result
}

#[allow(dead_code)]
fn find_safetensors_model_dirs_inner(dir: &Path, result: &mut Vec<PathBuf>) {
    if is_safetensors_model_dir(dir) {
        result.push(dir.to_path_buf());
        // Don't descend further — models don't nest
        return;
    }
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            find_safetensors_model_dirs_inner(&path, result);
        }
    }
}

/// Collect all absolute paths to `*.safetensors` files in `dir` (non-recursive).
pub fn collect_safetensors_shard_paths(dir: &Path) -> HashSet<PathBuf> {
    let mut paths = HashSet::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().and_then(|e| e.to_str()) == Some("safetensors") {
                if let Ok(canonical) = p.canonicalize() {
                    paths.insert(canonical);
                } else {
                    paths.insert(p);
                }
            }
        }
    }
    paths
}

// ── SHA-256 helper ────────────────────────────────────────────────────────────

fn sha256_file(path: &Path, max_bytes: u64) -> (Option<String>, Option<String>) {
    let file_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    if max_bytes > 0 && file_size > max_bytes {
        let reason = format!(
            "file size ({:.1} GB) exceeds --max-hash-size-gb limit",
            file_size as f64 / 1_073_741_824.0
        );
        return (None, Some(reason));
    }
    match File::open(path) {
        Ok(mut f) => {
            let mut hasher = Sha256::new();
            let mut buf = [0u8; 8192];
            loop {
                match f.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => hasher.update(&buf[..n]),
                    Err(e) => return (None, Some(format!("I/O error: {}", e))),
                }
            }
            (Some(format!("{:x}", hasher.finalize())), None)
        }
        Err(e) => (None, Some(format!("failed to open: {}", e))),
    }
}

// ── Companion-file helpers ───────────────────────────────────────────────────

/// Read a companion file from a model directory with a 1 MB size cap.
/// Returns None if the file doesn't exist, is too large, or cannot be read.
pub(crate) fn read_companion_file(dir: &Path, name: &str) -> Option<String> {
    let path = dir.join(name);
    if !path.is_file() {
        return None;
    }
    let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
    if size > MAX_COMPANION_FILE_BYTES {
        eprintln!(
            "Note: Skipping oversized companion file {} ({:.1} KB > {} KB limit)",
            path.display(),
            size as f64 / 1024.0,
            MAX_COMPANION_FILE_BYTES / 1024
        );
        return None;
    }
    fs::read_to_string(&path).ok()
}

/// Find README.md with case-insensitive filename matching.
/// Probes: README.md, readme.md, Readme.md — uses first that exists.
pub(crate) fn find_readme(dir: &Path) -> Option<String> {
    for name in &["README.md", "readme.md", "Readme.md"] {
        let path = dir.join(name);
        if !path.is_file() {
            continue;
        }
        let size = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if size > MAX_COMPANION_FILE_BYTES {
            return None;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            // Normalize line endings (handles Windows \r\n)
            return Some(content.replace("\r\n", "\n"));
        }
    }
    None
}

/// Parse README.md YAML frontmatter (between --- delimiters).
pub(crate) fn parse_readme_frontmatter(dir: &Path) -> Option<ReadmeFrontmatter> {
    let content = find_readme(dir)?;
    if !content.starts_with("---\n") {
        return None;
    }
    let end = content[4..].find("\n---")?;
    let yaml_block = &content[4..4 + end];
    serde_yaml::from_str(yaml_block).ok()
}

/// Parse model_max_length from a serde_json::Value, capping astronomical values.
/// Returns None if the value exceeds MODEL_MAX_LENGTH_CAP or is not a number.
pub(crate) fn parse_model_max_length(v: &serde_json::Value) -> Option<u64> {
    let val = v.as_u64().or_else(|| {
        let f = v.as_f64()?;
        // Reject NaN, infinity, and negative values — casting these with `as u64`
        // would saturate to 0 or u64::MAX, producing incorrect results.
        if !f.is_finite() || f < 0.0 {
            return None;
        }
        Some(f as u64)
    })?;
    if val > MODEL_MAX_LENGTH_CAP {
        None
    } else {
        Some(val)
    }
}

/// Merge two Vec<String> with deduplication (preserving order of `base` first, then `extra`).
pub(crate) fn merge_string_vecs(base: &mut Vec<String>, extra: &[String]) {
    let mut existing: HashSet<String> = base.iter().cloned().collect();
    for item in extra {
        if existing.insert(item.clone()) {
            base.push(item.clone());
        }
    }
}

/// Extract sub-model components from a multimodal HuggingFace config.
/// Only emits sub-models when the model is truly composite (text_config present
/// alongside at least one of vision_config or audio_config).
pub(crate) fn extract_sub_models(hf_config: &HuggingFaceConfig) -> Vec<SubModelInfo> {
    // Guard: only decompose genuinely multimodal models
    let tc = match &hf_config.text_config {
        Some(tc) if hf_config.vision_config.is_some() || hf_config.audio_config.is_some() => tc,
        _ => return Vec::new(),
    };

    let mut sub_models = Vec::new();

    // Helper to extract typed values from serde_json::Value
    let get_str = |v: &serde_json::Value, key: &str| -> Option<String> {
        v.get(key).and_then(|x| x.as_str()).map(|s| s.to_string())
    };
    let get_u32 = |v: &serde_json::Value, key: &str| -> Option<u32> {
        v.get(key).and_then(|x| x.as_u64()).map(|x| x as u32)
    };
    let get_u64 = |v: &serde_json::Value, key: &str| -> Option<u64> {
        v.get(key).and_then(|x| x.as_u64())
    };

    // Text sub-model
    sub_models.push(SubModelInfo {
        model_type: tc.model_type.clone(),
        modality: "text".to_string(),
        num_hidden_layers: tc.num_hidden_layers,
        hidden_size: tc.hidden_size,
        num_attention_heads: tc.num_attention_heads,
        num_key_value_heads: tc.num_key_value_heads,
        max_position_embeddings: tc.max_position_embeddings,
        vocab_size: tc.vocab_size,
        dtype: tc.dtype.clone(),
        intermediate_size: tc.intermediate_size,
        ..Default::default()
    });

    // Vision sub-model
    if let Some(ref vc) = hf_config.vision_config {
        sub_models.push(SubModelInfo {
            model_type: get_str(vc, "model_type"),
            modality: "vision".to_string(),
            num_hidden_layers: get_u32(vc, "num_hidden_layers"),
            hidden_size: get_u32(vc, "hidden_size"),
            num_attention_heads: get_u32(vc, "num_attention_heads"),
            num_key_value_heads: get_u32(vc, "num_key_value_heads"),
            max_position_embeddings: get_u64(vc, "max_position_embeddings"),
            dtype: get_str(vc, "dtype"),
            intermediate_size: get_u32(vc, "intermediate_size"),
            patch_size: get_u32(vc, "patch_size"),
            default_output_length: get_u32(vc, "default_output_length"),
            ..Default::default()
        });
    }

    // Audio sub-model
    if let Some(ref ac) = hf_config.audio_config {
        sub_models.push(SubModelInfo {
            model_type: get_str(ac, "model_type"),
            modality: "audio".to_string(),
            num_hidden_layers: get_u32(ac, "num_hidden_layers"),
            hidden_size: get_u32(ac, "hidden_size"),
            num_attention_heads: get_u32(ac, "num_attention_heads"),
            num_key_value_heads: get_u32(ac, "num_key_value_heads"),
            dtype: get_str(ac, "dtype"),
            intermediate_size: get_u32(ac, "intermediate_size"),
            conv_kernel_size: get_u32(ac, "conv_kernel_size"),
            output_proj_dims: get_u32(ac, "output_proj_dims"),
            ..Default::default()
        });
    }

    sub_models
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse a safetensors model directory and return a single `Dependency`.
///
/// Handles both sharded models (`model.safetensors.index.json`) and
/// single-file models (`model.safetensors` or `*.safetensors`).
/// Always produces **one** `Dependency` regardless of shard count.
///
/// `max_hash_size_gb`: max file size in GB for SHA-256 hashing.
/// 0 = unlimited (hash all). Files above limit skip hashing but still parse metadata.
pub fn parse_safetensors_dir(dir: &Path, max_hash_size_gb: u64) -> Result<Vec<Dependency>> {
    let max_hash_bytes = if max_hash_size_gb == 0 {
        0u64 // 0 = unlimited in sha256_file
    } else {
        max_hash_size_gb.saturating_mul(1024 * 1024 * 1024)
    };

    let mut metadata = AIModelMetadata::default();

    // ── 1. Parse model.safetensors.index.json (sharded model) ────────────────
    let index_path = dir.join("model.safetensors.index.json");
    // Cache result to avoid TOCTOU double probe (used again in source_file attribution below).
    let index_exists = index_path.is_file();
    let mut shard_files: Vec<PathBuf> = Vec::new();
    let mut hash_target: Option<PathBuf> = None;

    if index_exists {
        let content = fs::read_to_string(&index_path)
            .with_context(|| format!("failed to read {}", index_path.display()))?;
        let index: SafetensorsIndexJson = serde_json::from_str(&content)
            .with_context(|| format!("failed to parse {}", index_path.display()))?;

        if let Some(ref idx_meta) = index.metadata {
            metadata.total_size_bytes = idx_meta.total_size;
        }

        // shard_count comes from the declared weight_map (consistent with total_size_bytes,
        // which is also from the index, not from actual file sizes). This keeps the two fields
        // semantically aligned even when a model is only partially downloaded.
        // shard_files is populated separately with only the files present on disk (for parsing).
        if let Some(ref weight_map) = index.weight_map {
            let unique_shard_names: HashSet<&str> =
                weight_map.values().map(|s| s.as_str()).collect();
            // Declared shard count — matches total_size_bytes semantics.
            let declared_count = unique_shard_names.len() as u32;
            metadata.shard_count = if declared_count > 0 { Some(declared_count) } else { None };

            // Collect present files for header parsing (sorted for determinism).
            let mut present: Vec<PathBuf> = unique_shard_names
                .iter()
                .map(|name| dir.join(name))
                .filter(|p| p.is_file())
                .collect();
            present.sort();
            shard_files = present;
        }

        // Hash the index file (small JSON, always fast)
        hash_target = Some(index_path.clone());

        // Try to get header metadata from the first shard (format only; tensor_count is not
        // estimated for sharded models because each shard contains distinct tensors).
        if let Some(first_shard) = shard_files.first() {
            if let Ok(header) = read_safetensors_header(first_shard) {
                metadata.safetensors_format = header.metadata.format.clone();
                // tensor_count is intentionally omitted for sharded models: multiplying the
                // first-shard count by shard_count would be wrong (shards hold different layers).
            }
        }
    } else {
        // ── 2. Single-file or bare *.safetensors ─────────────────────────────
        let single = dir.join("model.safetensors");
        let safetensors_file = if single.is_file() {
            Some(single.clone())
        } else {
            // Find any *.safetensors file — sort entries for deterministic selection.
            let mut candidates: Vec<PathBuf> = fs::read_dir(dir)
                .map(|entries| {
                    entries
                        .flatten()
                        .map(|e| e.path())
                        .filter(|p| {
                            p.extension().and_then(|x| x.to_str()) == Some("safetensors")
                        })
                        .collect()
                })
                .unwrap_or_default();
            candidates.sort();
            candidates.into_iter().next()
        };

        if let Some(ref sf_path) = safetensors_file {
            shard_files.push(sf_path.clone());
            metadata.shard_count = Some(1);
            hash_target = Some(sf_path.clone());

            // Capture file size unconditionally — available even if the header is malformed.
            if let Ok(fmeta) = fs::metadata(sf_path) {
                metadata.total_size_bytes = Some(fmeta.len());
            }

            if let Ok(header) = read_safetensors_header(sf_path) {
                metadata.safetensors_format = header.metadata.format.clone();
                metadata.tensor_count = Some(header.tensors.len() as u64);
            }
        }
    }

    // ── 3. Load config.json ──────────────────────────────────────────────────
    let config_path = dir.join("config.json");
    let mut hf_config = HuggingFaceConfig::default();
    if config_path.is_file() {
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(cfg) = serde_json::from_str::<HuggingFaceConfig>(&content) {
                hf_config = cfg;
            }
        }
    }

    // Map config.json fields onto AIModelMetadata
    metadata.architecture = hf_config
        .architectures
        .as_ref()
        .and_then(|a| a.first())
        .cloned()
        .or(hf_config.model_type.clone());
    metadata.torch_dtype = hf_config.torch_dtype.clone();
    metadata.transformers_version = hf_config.transformers_version.clone();
    metadata.vocab_size = hf_config.vocab_size;

    // v1.0.12: Extended config.json extraction
    metadata.model_type = hf_config.model_type.clone();

    // Dtype priority: torch_dtype (top-level) > dtype (top-level) > text_config.dtype
    if metadata.torch_dtype.is_none() {
        metadata.torch_dtype = hf_config.dtype.clone().or_else(|| {
            hf_config.text_config.as_ref().and_then(|tc| tc.dtype.clone())
        });
    }

    // Architecture fields: text_config (multimodal) > top-level (non-multimodal like LLaMA)
    if let Some(ref tc) = hf_config.text_config {
        metadata.num_hidden_layers = tc.num_hidden_layers;
        metadata.hidden_size = tc.hidden_size;
        metadata.num_attention_heads = tc.num_attention_heads;
        metadata.max_position_embeddings = tc.max_position_embeddings;
        if metadata.vocab_size.is_none() {
            metadata.vocab_size = tc.vocab_size;
        }
    }
    // Top-level fallback for non-multimodal models (e.g. LLaMA, Mistral)
    if metadata.num_hidden_layers.is_none() {
        metadata.num_hidden_layers = hf_config.num_hidden_layers;
    }
    if metadata.hidden_size.is_none() {
        metadata.hidden_size = hf_config.hidden_size;
    }
    if metadata.num_attention_heads.is_none() {
        metadata.num_attention_heads = hf_config.num_attention_heads;
    }
    if metadata.max_position_embeddings.is_none() {
        metadata.max_position_embeddings = hf_config.max_position_embeddings;
    }

    // Multimodal detection from config.json
    if hf_config.vision_config.is_some() {
        metadata.has_vision = Some(true);
        metadata.vision_model_type = hf_config
            .vision_config
            .as_ref()
            .and_then(|v| v.get("model_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }
    if hf_config.audio_config.is_some() {
        metadata.has_audio = Some(true);
        metadata.audio_model_type = hf_config
            .audio_config
            .as_ref()
            .and_then(|v| v.get("model_type"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
    }

    // v1.0.13: Extract sub-model components for multimodal models
    metadata.sub_models = extract_sub_models(&hf_config);

    // ── 3b. Parse generation_config.json ────────────────────────────────────
    if let Some(content) = read_companion_file(dir, "generation_config.json") {
        if let Ok(gen) = serde_json::from_str::<GenerationConfig>(&content) {
            metadata.generation_temperature = gen.temperature;
            metadata.generation_top_k = gen.top_k;
            metadata.generation_top_p = gen.top_p;
        }
    }

    // ── 3c. Parse tokenizer_config.json ─────────────────────────────────────
    let mut tok_processor_class: Option<String> = None;
    if let Some(content) = read_companion_file(dir, "tokenizer_config.json") {
        if let Ok(tok) = serde_json::from_str::<TokenizerConfig>(&content) {
            tok_processor_class = tok.processor_class.clone();
            metadata.processor_class = tok.processor_class;
            metadata.model_max_length = tok.model_max_length.as_ref().and_then(parse_model_max_length);
        }
    }

    // ── 3d. Parse preprocessor_config.json ──────────────────────────────────
    if let Some(content) = read_companion_file(dir, "preprocessor_config.json") {
        if let Ok(preproc) = serde_json::from_str::<PreprocessorConfig>(&content) {
            // processor_class: preprocessor wins, fallback to tokenizer (use clone, NOT take)
            metadata.processor_class = preproc.processor_class.or_else(|| tok_processor_class.clone());
            metadata.image_seq_length = preproc.image_seq_length;
            metadata.audio_seq_length = preproc.audio_seq_length;

            if let Some(ref ip) = preproc.image_processor {
                metadata.image_processor_type = ip.image_processor_type.clone();
                if metadata.image_seq_length.is_none() {
                    metadata.image_seq_length = ip.image_seq_length;
                }
            }
            if let Some(ref fe) = preproc.feature_extractor {
                metadata.audio_feature_extractor_type = fe.feature_extractor_type.clone();
                metadata.audio_sampling_rate = fe.sampling_rate;
            }
            if let Some(ref vp) = preproc.video_processor {
                metadata.video_processor_type = vp.video_processor_type.clone();
                metadata.video_num_frames = vp.num_frames;
                metadata.has_video = Some(true);
            }
        }
    }

    // ── 3e. Parse README.md frontmatter ─────────────────────────────────────
    let mut dep_license: Option<String> = None;
    let mut dep_author: Option<String> = None;
    if let Some(fm) = parse_readme_frontmatter(dir) {
        metadata.model_name = fm.model_name;
        metadata.pipeline_tag = fm.pipeline_tag;
        metadata.quantized_by = fm.quantized_by;

        // Truncate prompt_template to MAX_PROMPT_TEMPLATE_LEN (char-safe for UTF-8)
        metadata.prompt_template = fm.prompt_template.map(|pt| {
            if pt.chars().count() > MAX_PROMPT_TEMPLATE_LEN {
                format!("{}...", pt.chars().take(MAX_PROMPT_TEMPLATE_LEN).collect::<String>())
            } else {
                pt
            }
        });

        // base_model entries → push into base_models as BaseModelInfo
        if metadata.base_models.is_empty() && !fm.base_model.is_empty() {
            for repo in &fm.base_model {
                metadata.base_models.push(crate::models::BaseModelInfo {
                    name: None,
                    author: None,
                    version: None,
                    organization: None,
                    url: None,
                    repo_url: Some(repo.clone()),
                });
            }
        }

        // Fallback license (normalized to proper SPDX casing) and author
        dep_license = fm.license.map(|l| crate::parsers::gguf::normalize_spdx_license(&l));
        dep_author = fm.model_creator;

        // Deduplicated union merge for tags, languages, datasets
        merge_string_vecs(&mut metadata.tags, &fm.tags);
        merge_string_vecs(&mut metadata.languages, &fm.language);
        merge_string_vecs(&mut metadata.datasets, &fm.datasets);
    }

    // ── 3f. Detect adapter_config.json ──────────────────────────────────────
    if let Some(content) = read_companion_file(dir, "adapter_config.json") {
        if let Ok(adapter) = serde_json::from_str::<AdapterConfig>(&content) {
            metadata.is_adapter = Some(true);
            metadata.adapter_type = adapter.peft_type;
            // Push base model from adapter config if base_models is empty
            if metadata.base_models.is_empty() {
                if let Some(ref base) = adapter.base_model_name_or_path {
                    metadata.base_models.push(crate::models::BaseModelInfo {
                        name: None,
                        author: None,
                        version: None,
                        organization: None,
                        url: None,
                        repo_url: Some(base.clone()),
                    });
                }
            }
        }
    }

    // ── 4. Compute SHA-256 hash ──────────────────────────────────────────────
    let (checksum_sha256, hash_skip_reason) = if let Some(ref target) = hash_target {
        sha256_file(target, max_hash_bytes)
    } else {
        (None, Some("no hashable file found".to_string()))
    };
    metadata.hash_skip_reason = hash_skip_reason;

    // ── 5. Determine model name ──────────────────────────────────────────────
    let name = hf_config
        ._name_or_path
        .as_deref()
        .and_then(|n| {
            // _name_or_path is often "google/gemma-3-27b-it" — extract last component
            // but keep if it looks like an org/model slug
            if n.contains('/') {
                Some(n.to_string())
            } else if !n.is_empty() && !n.starts_with('/') {
                Some(n.to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| {
            dir.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown-model")
                .to_string()
        });

    // ── 6. Repository URL from tokenizer_config.json or model card URL ───────
    // Many HuggingFace models store their repo URL in tokenizer_config.json or
    // model card. We do a best-effort lookup.
    let repository_url = try_extract_hf_repo_url(dir, &name);

    // ── 7. Source file attribution ───────────────────────────────────────────
    // Use the specific entry-point file when available: index.json for sharded models,
    // the .safetensors file for single-file models. Fall back to the directory only if
    // no file was identified (e.g. empty dir that still passes is_safetensors_model_dir).
    let source_file = crate::parsers::format_source_info(
        "safetensors/model",
        hash_target.as_deref().unwrap_or(dir),
        None,
        false,
    );

    let dep = Dependency {
        name,
        version: "unknown".to_string(),
        ecosystem: "safetensors".to_string(),
        source: DependencySource::Manifest,
        is_direct: true,
        is_dev: false,
        license: dep_license,
        author: dep_author,
        repository_url,
        source_file: Some(source_file),
        checksum_sha256,
        ai_model_metadata: Some(metadata),
        scope: DependencyScope::Runtime,
        scope_confidence: 0.9,
        scope_reason: "AI model file (safetensors)".to_string(),
        ..Default::default()
    };

    Ok(vec![dep])
}

/// Try to extract a HuggingFace repository URL from common companion files.
fn try_extract_hf_repo_url(dir: &Path, model_name: &str) -> Option<String> {
    // Try tokenizer_config.json for `_name_or_path`
    let tok_config_path = dir.join("tokenizer_config.json");
    if tok_config_path.is_file() {
        if let Ok(content) = fs::read_to_string(&tok_config_path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(url) = val.get("_name_or_path").and_then(|v| v.as_str()) {
                    if url.contains("huggingface.co") {
                        return Some(url.to_string());
                    }
                }
            }
        }
    }

    // If model_name looks like "org/model", construct the HuggingFace URL
    if model_name.contains('/') && !model_name.starts_with('/') {
        let parts: Vec<&str> = model_name.splitn(2, '/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Some(format!("https://huggingface.co/{}/{}", parts[0], parts[1]));
        }
    }

    None
}
