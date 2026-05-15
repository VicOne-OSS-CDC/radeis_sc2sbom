use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

use radeis_sc2sbom::formats::spdx::create_package_url;
use radeis_sc2sbom::parsers::safetensors::{
    find_safetensors_model_dirs, is_safetensors_model_dir, parse_safetensors_dir,
};

// ── Binary helpers ────────────────────────────────────────────────────────────

/// Build a minimal valid safetensors binary with the given metadata dict.
/// Format: [u64 header_size][JSON header bytes]
fn build_safetensors_binary(metadata: &serde_json::Value, tensors: &serde_json::Value) -> Vec<u8> {
    let mut header = serde_json::Map::new();
    if let Some(obj) = metadata.as_object() {
        header.insert("__metadata__".to_string(), serde_json::Value::Object(obj.clone()));
    } else {
        header.insert("__metadata__".to_string(), serde_json::json!({}));
    }
    // Add any tensors
    if let Some(t_obj) = tensors.as_object() {
        for (k, v) in t_obj {
            header.insert(k.clone(), v.clone());
        }
    }
    let json_bytes = serde_json::to_vec(&serde_json::Value::Object(header)).unwrap();
    let header_size = json_bytes.len() as u64;
    let mut buf = Vec::new();
    buf.extend_from_slice(&header_size.to_le_bytes());
    buf.extend_from_slice(&json_bytes);
    buf
}

fn write_file(dir: &TempDir, name: &str, contents: &[u8]) {
    let path = dir.path().join(name);
    fs::write(&path, contents).unwrap();
}

fn write_json(dir: &TempDir, name: &str, value: &serde_json::Value) {
    let path = dir.path().join(name);
    fs::write(&path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

// ── Test 1: Binary header parse ───────────────────────────────────────────────

#[test]
fn test_safetensors_binary_header_parse() {
    let dir = TempDir::new().unwrap();
    let metadata = serde_json::json!({ "format": "pt" });
    let tensors = serde_json::json!({
        "model.embed_tokens.weight": { "dtype": "BF16", "shape": [256000, 2048], "data_offsets": [0, 1048576000] }
    });
    let binary = build_safetensors_binary(&metadata, &tensors);
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "gemma3",
            "torch_dtype": "bfloat16",
            "vocab_size": 256000
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "safetensors");

    let meta = dep.ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.safetensors_format.as_deref(), Some("pt"));
    assert_eq!(meta.shard_count, Some(1));
    assert_eq!(meta.torch_dtype.as_deref(), Some("bfloat16"));
    assert_eq!(meta.vocab_size, Some(256000));
}

// ── Test 2: Index JSON parse ──────────────────────────────────────────────────

#[test]
fn test_safetensors_index_json_parse() {
    let dir = TempDir::new().unwrap();

    // Write two shard files (minimal binary)
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model-00001-of-00002.safetensors", &binary);
    write_file(&dir, "model-00002-of-00002.safetensors", &binary);

    let index = serde_json::json!({
        "metadata": { "total_size": 67_173_949_440u64 },
        "weight_map": {
            "model.layers.0.self_attn.q_proj.weight": "model-00001-of-00002.safetensors",
            "model.layers.1.self_attn.q_proj.weight": "model-00002-of-00002.safetensors"
        }
    });
    write_json(&dir, "model.safetensors.index.json", &index);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "llama",
            "architectures": ["LlamaForCausalLM"],
            "torch_dtype": "bfloat16",
            "transformers_version": "4.40.0",
            "vocab_size": 128256
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1, "should produce exactly one Dependency");
    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "safetensors");

    let meta = dep.ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.total_size_bytes, Some(67_173_949_440));
    assert_eq!(meta.shard_count, Some(2));
    assert_eq!(meta.torch_dtype.as_deref(), Some("bfloat16"));
    assert_eq!(meta.transformers_version.as_deref(), Some("4.40.0"));
    assert_eq!(meta.vocab_size, Some(128256));
}

// ── Test 3: config.json parse ─────────────────────────────────────────────────

#[test]
fn test_config_json_parse() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "mistral",
            "architectures": ["MistralForCausalLM"],
            "torch_dtype": "float16",
            "transformers_version": "4.38.0",
            "vocab_size": 32768,
            "_name_or_path": "mistralai/Mistral-7B-v0.1"
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];

    let meta = dep.ai_model_metadata.as_ref().unwrap();
    // architectures takes precedence over model_type
    assert_eq!(meta.architecture.as_deref(), Some("MistralForCausalLM"));
    assert_eq!(meta.torch_dtype.as_deref(), Some("float16"));
    assert_eq!(meta.transformers_version.as_deref(), Some("4.38.0"));
    assert_eq!(meta.vocab_size, Some(32768));

    // Name from _name_or_path
    assert_eq!(dep.name, "mistralai/Mistral-7B-v0.1");
}

// ── Test 4: Single file model ─────────────────────────────────────────────────

#[test]
fn test_single_file_model() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(
        &serde_json::json!({ "format": "pt" }),
        &serde_json::json!({
            "weight": { "dtype": "F32", "shape": [512, 512], "data_offsets": [0, 1048576] }
        }),
    );
    write_file(&dir, "model.safetensors", &binary);

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "safetensors");
    let meta = dep.ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.shard_count, Some(1));
    assert_eq!(meta.safetensors_format.as_deref(), Some("pt"));
    // total_size_bytes set from file metadata
    assert!(meta.total_size_bytes.is_some());
}

// ── Test 5: Sharded model deduplication ──────────────────────────────────────

#[test]
fn test_sharded_model_dedup() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model-00001-of-00002.safetensors", &binary);
    write_file(&dir, "model-00002-of-00002.safetensors", &binary);

    let index = serde_json::json!({
        "metadata": { "total_size": 2048u64 },
        "weight_map": {
            "layer.0.weight": "model-00001-of-00002.safetensors",
            "layer.1.weight": "model-00002-of-00002.safetensors"
        }
    });
    write_json(&dir, "model.safetensors.index.json", &index);

    // Only 1 Dependency should be emitted — not one per shard
    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(
        deps.len(),
        1,
        "sharded model should produce exactly one Dependency, got {}",
        deps.len()
    );
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.shard_count, Some(2));
}

// ── Test 6: SPDX PURL — pkg:huggingface from name ────────────────────────────

#[test]
fn test_spdx_huggingface_purl_safetensors() {
    use radeis_sc2sbom::models::{AIModelMetadata, Dependency, DependencyScope, DependencySource};

    let dep = Dependency {
        name: "google/gemma-3-27b-it".to_string(),
        version: "unknown".to_string(),
        ecosystem: "safetensors".to_string(),
        source: DependencySource::Manifest,
        is_direct: true,
        is_dev: false,
        repository_url: Some("https://huggingface.co/google/gemma-3-27b-it".to_string()),
        ai_model_metadata: Some(AIModelMetadata::default()),
        scope: DependencyScope::Runtime,
        ..Default::default()
    };

    let purl = create_package_url(&dep);
    assert_eq!(purl, "pkg:huggingface/google/gemma-3-27b-it");
}

// ── Test 7: CycloneDX component type ─────────────────────────────────────────

#[cfg(feature = "internal")]
#[test]
fn test_cyclonedx_safetensors_component() {
    use radeis_sc2sbom::formats::cyclonedx::convert_to_cyclonedx;
    use radeis_sc2sbom::models::{AIModelMetadata, Dependency, DependencyScope, DependencySource, Sbom};

    let mut meta = AIModelMetadata::default();
    meta.architecture = Some("MistralForCausalLM".to_string());
    meta.shard_count = Some(2);
    meta.total_size_bytes = Some(14_000_000_000);
    meta.torch_dtype = Some("bfloat16".to_string());

    let dep = Dependency {
        name: "mistralai/Mistral-7B-v0.1".to_string(),
        version: "unknown".to_string(),
        ecosystem: "safetensors".to_string(),
        source: DependencySource::Manifest,
        is_direct: true,
        is_dev: false,
        repository_url: Some("https://huggingface.co/mistralai/Mistral-7B-v0.1".to_string()),
        ai_model_metadata: Some(meta),
        scope: DependencyScope::Runtime,
        ..Default::default()
    };

    let sbom = Sbom {
        project_path: PathBuf::from("/tmp/test"),
        generated_at: "2026-04-13T00:00:00Z".to_string(),
        dependencies: vec![dep],
        ros_package: None,
        ros_packages: vec![],
        scope_statistics: None,
    };

    let cdx_doc = convert_to_cyclonedx(&sbom, None, #[cfg(feature = "internal")] &[]);
    let output = serde_json::to_string_pretty(&cdx_doc).unwrap();
    let json: serde_json::Value = serde_json::from_str(&output).unwrap();

    let components = json["components"].as_array().unwrap();
    assert_eq!(components.len(), 1);
    let comp = &components[0];

    assert_eq!(
        comp["type"].as_str().unwrap(),
        "machine-learning-model",
        "safetensors component should have type machine-learning-model"
    );

    // Check properties contain safetensors-specific fields
    let props = comp["properties"].as_array().unwrap();
    let prop_names: Vec<&str> = props
        .iter()
        .filter_map(|p| p["name"].as_str())
        .collect();
    assert!(
        prop_names.contains(&"radeis:ai:shard_count"),
        "expected radeis:ai:shard_count property, got: {:?}",
        prop_names
    );
    assert!(
        prop_names.contains(&"radeis:ai:torch_dtype"),
        "expected radeis:ai:torch_dtype property, got: {:?}",
        prop_names
    );
    assert!(
        prop_names.contains(&"radeis:ai:total_size_bytes"),
        "expected radeis:ai:total_size_bytes property, got: {:?}",
        prop_names
    );
}

// ── Test 8: is_safetensors_model_dir detection ────────────────────────────────

#[test]
fn test_is_safetensors_model_dir() {
    let dir = TempDir::new().unwrap();

    // Empty dir — not a model dir
    assert!(!is_safetensors_model_dir(dir.path()));

    // With model.safetensors.index.json
    let index_path = dir.path().join("model.safetensors.index.json");
    fs::write(&index_path, b"{}").unwrap();
    assert!(is_safetensors_model_dir(dir.path()));

    // Remove index, add model.safetensors
    fs::remove_file(&index_path).unwrap();
    let model_path = dir.path().join("model.safetensors");
    fs::write(&model_path, b"dummy").unwrap();
    assert!(is_safetensors_model_dir(dir.path()));

    // Remove model.safetensors, add config.json + *.safetensors
    fs::remove_file(&model_path).unwrap();
    fs::write(dir.path().join("config.json"), b"{}").unwrap();
    fs::write(dir.path().join("model-00001-of-00002.safetensors"), b"dummy").unwrap();
    assert!(is_safetensors_model_dir(dir.path()));
}

// ── Test 9: find_safetensors_model_dirs ──────────────────────────────────────

#[test]
fn test_find_safetensors_model_dirs() {
    let root = TempDir::new().unwrap();

    // Create two model sub-dirs and one non-model dir
    let model1 = root.path().join("model_a");
    let model2 = root.path().join("nested/model_b");
    let other = root.path().join("src");
    fs::create_dir_all(&model1).unwrap();
    fs::create_dir_all(&model2).unwrap();
    fs::create_dir_all(&other).unwrap();

    fs::write(model1.join("model.safetensors"), b"dummy").unwrap();
    fs::write(model2.join("model.safetensors.index.json"), b"{}").unwrap();
    fs::write(other.join("main.rs"), b"fn main() {}").unwrap();

    let dirs = find_safetensors_model_dirs(root.path());
    assert_eq!(dirs.len(), 2, "expected 2 model dirs, got {:?}", dirs);
}

// ── Test 10: Malformed/oversized binary header recovery ──────────────────────

#[test]
fn test_malformed_safetensors_header_recovery() {
    let dir = TempDir::new().unwrap();

    // Write a file with header_size = 0 (invalid) — should be silently ignored
    let mut buf_zero = Vec::new();
    buf_zero.extend_from_slice(&0u64.to_le_bytes());
    write_file(&dir, "model.safetensors", &buf_zero);

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1, "should still produce one Dependency for malformed shard");
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    // Header parse failed silently — these fields stay None
    assert_eq!(meta.safetensors_format, None);
    assert_eq!(meta.tensor_count, None);

    // Write a file claiming a header_size larger than MAX_HEADER_BYTES (50MB cap)
    let dir2 = TempDir::new().unwrap();
    let oversized_header_claim = (51u64 * 1024 * 1024).to_le_bytes();
    let mut buf_big = Vec::new();
    buf_big.extend_from_slice(&oversized_header_claim);
    buf_big.extend_from_slice(&[0u8; 64]); // minimal body (won't be read)
    write_file(&dir2, "model.safetensors", &buf_big);

    let deps2 = parse_safetensors_dir(dir2.path(), 0).unwrap();
    assert_eq!(deps2.len(), 1, "should still produce one Dependency for oversized header");
    let meta2 = deps2[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta2.safetensors_format, None);
}

// ── Test 11: SPDX PURL via name split (no repository_url) ────────────────────

#[test]
fn test_spdx_huggingface_purl_from_name_split() {
    use radeis_sc2sbom::formats::spdx::create_package_url;
    use radeis_sc2sbom::models::{AIModelMetadata, Dependency, DependencyScope, DependencySource};

    // No repository_url — PURL must be derived from the "org/model" name pattern
    let dep = Dependency {
        name: "mistralai/Mistral-7B-v0.1".to_string(),
        version: "unknown".to_string(),
        ecosystem: "safetensors".to_string(),
        source: DependencySource::Manifest,
        is_direct: true,
        is_dev: false,
        repository_url: None,
        ai_model_metadata: Some(AIModelMetadata::default()),
        scope: DependencyScope::Runtime,
        ..Default::default()
    };

    let purl = create_package_url(&dep);
    assert_eq!(purl, "pkg:huggingface/mistralai/Mistral-7B-v0.1");
}

// ── Test 12: parse_safetensors_dir on criterion-(c) directory ─────────────────

#[test]
fn test_criterion_c_directory_parse() {
    // Criterion (c): config.json + *.safetensors (non-standard shard name, no index.json)
    let dir = TempDir::new().unwrap();

    let binary = build_safetensors_binary(
        &serde_json::json!({ "format": "pt" }),
        &serde_json::json!({
            "weight": { "dtype": "F32", "shape": [768, 768], "data_offsets": [0, 2359296] }
        }),
    );
    write_file(&dir, "encoder.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "bert",
            "architectures": ["BertForMaskedLM"],
            "torch_dtype": "float32",
            "vocab_size": 30522,
            "_name_or_path": "bert-base-uncased"
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "safetensors");
    assert_eq!(dep.name, "bert-base-uncased");

    let meta = dep.ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.architecture.as_deref(), Some("BertForMaskedLM"));
    assert_eq!(meta.torch_dtype.as_deref(), Some("float32"));
    assert_eq!(meta.vocab_size, Some(30522));
    assert_eq!(meta.shard_count, Some(1));
    assert_eq!(meta.safetensors_format.as_deref(), Some("pt"));
}

// ── Test 13: Real model — google/gemma-4-E2B-it (single-file, 5.1 GB) ──────────
//
// This test uses the actual model files downloaded from HuggingFace:
//   https://huggingface.co/google/gemma-4-E2B-it/tree/main
//
// To run (files must be pre-fetched — only the header portion of model.safetensors
// is needed, not the full 5 GB weights):
//
//   mkdir -p /tmp/gemma4-e2b-it-test
//   # Fetch config.json and tokenizer_config.json (small, download in full)
//   curl -sL https://huggingface.co/google/gemma-4-E2B-it/resolve/main/config.json \
//        -o /tmp/gemma4-e2b-it-test/config.json
//   curl -sL https://huggingface.co/google/gemma-4-E2B-it/resolve/main/tokenizer_config.json \
//        -o /tmp/gemma4-e2b-it-test/tokenizer_config.json
//   # Fetch just the safetensors header via HTTP Range (263960 bytes = 8-byte size prefix + 257 KB JSON header)
//   curl -sL https://huggingface.co/google/gemma-4-E2B-it/resolve/main/model.safetensors \
//        -H "Range: bytes=0-263959" \
//        -o /tmp/gemma4-e2b-it-test/model.safetensors
//
//   cargo test test_real_gemma4_e2b_it -- --ignored
//
// Known values (verified from actual model files):
//   - model_type:            gemma4
//   - architectures:         ["Gemma4ForConditionalGeneration"]
//   - transformers_version:  5.5.0.dev0
//   - torch_dtype:           not set at top level (multimodal sub-configs use bfloat16)
//   - vocab_size:            not set at top level
//   - _name_or_path:         not set → name falls back to directory name "gemma4-e2b-it-test"
//   - safetensors_format:    "pt"  (from __metadata__.format)
//   - tensor_count:          2011
//   - shard_count:           1  (single file, no index.json)
//   - total_size_bytes:      263960 (truncated header-only fixture; real file is ~5.1 GB)
//   - repository_url:        https://huggingface.co/gemma4-e2b-it-test  (name has no '/' so no URL)
//
#[test]
#[ignore = "requires pre-fetched model files in /tmp/gemma4-e2b-it-test — see comment above"]
fn test_real_gemma4_e2b_it() {
    let model_dir = std::path::Path::new("/tmp/gemma4-e2b-it-test");
    if !model_dir.exists() {
        panic!(
            "Model files not found at /tmp/gemma4-e2b-it-test. \
             Run the curl commands in the test comment to fetch them."
        );
    }

    assert!(
        is_safetensors_model_dir(model_dir),
        "should be detected as a safetensors model dir"
    );

    let deps = parse_safetensors_dir(model_dir, 0).unwrap();
    assert_eq!(deps.len(), 1, "single-file model must produce exactly one Dependency");

    let dep = &deps[0];
    assert_eq!(dep.ecosystem, "safetensors");
    // _name_or_path is null in config.json — name falls back to directory name
    assert_eq!(dep.name, "gemma4-e2b-it-test");

    let meta = dep.ai_model_metadata.as_ref().unwrap();
    // Architecture from architectures[0], takes precedence over model_type
    assert_eq!(
        meta.architecture.as_deref(),
        Some("Gemma4ForConditionalGeneration"),
        "architecture from architectures[0]"
    );
    // model_type from top-level config.json
    // (architecture field is set from architectures[], model_type available via fallback)
    assert_eq!(
        meta.transformers_version.as_deref(),
        Some("5.5.0.dev0"),
        "transformers_version from config.json"
    );
    // torch_dtype is not set at the top-level config (multimodal model — sub-configs carry dtype)
    assert_eq!(meta.torch_dtype, None, "torch_dtype absent at top level for multimodal model");
    // vocab_size is not set at the top level either
    assert_eq!(meta.vocab_size, None, "vocab_size absent at top level for multimodal model");

    // Single-file model
    assert_eq!(meta.shard_count, Some(1), "single file → shard_count = 1");
    // Format from __metadata__.format in the binary header
    assert_eq!(
        meta.safetensors_format.as_deref(),
        Some("pt"),
        "format from __metadata__.format"
    );
    // Tensor count from the JSON header (2011 tensor entries in the real model)
    assert_eq!(meta.tensor_count, Some(2011), "tensor count from binary header");
    // total_size_bytes set from fs::metadata (header-only fixture, not real file size)
    assert!(meta.total_size_bytes.is_some(), "total_size_bytes from fs::metadata");
    // SHA-256 should be computed (file is under default size limit)
    assert!(
        dep.checksum_sha256.is_some(),
        "SHA-256 checksum should be computed for the fixture file"
    );
}

// ── Test 14: generation_config.json parse ────────────────────────────────────

#[test]
fn test_generation_config_parse() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(&dir, "config.json", &serde_json::json!({"model_type": "llama"}));
    write_json(
        &dir,
        "generation_config.json",
        &serde_json::json!({
            "temperature": 0.7,
            "top_k": 40,
            "top_p": 0.9
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.generation_temperature, Some(0.7));
    assert_eq!(meta.generation_top_k, Some(40));
    assert_eq!(meta.generation_top_p, Some(0.9));
}

// ── Test 15: tokenizer_config.json parse ─────────────────────────────────────

#[test]
fn test_tokenizer_config_parse() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(&dir, "config.json", &serde_json::json!({"model_type": "llama"}));
    write_json(
        &dir,
        "tokenizer_config.json",
        &serde_json::json!({
            "processor_class": "LlamaProcessor",
            "model_max_length": 4096
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.processor_class.as_deref(), Some("LlamaProcessor"));
    assert_eq!(meta.model_max_length, Some(4096));
}

// ── Test 16: model_max_length astronomical value cap ─────────────────────────

#[test]
fn test_model_max_length_cap() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(&dir, "config.json", &serde_json::json!({"model_type": "gemma2"}));
    write_json(
        &dir,
        "tokenizer_config.json",
        &serde_json::json!({
            "model_max_length": 1000000000000000019884624838656.0_f64
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(
        meta.model_max_length, None,
        "astronomical model_max_length should be discarded"
    );
}

// ── Test 17: preprocessor_config.json parse ──────────────────────────────────

#[test]
fn test_preprocessor_config_parse() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(&dir, "config.json", &serde_json::json!({"model_type": "gemma4"}));
    write_json(
        &dir,
        "preprocessor_config.json",
        &serde_json::json!({
            "processor_class": "Gemma4Processor",
            "image_seq_length": 280,
            "audio_seq_length": 750,
            "image_processor": {
                "image_processor_type": "Gemma4ImageProcessor",
                "image_seq_length": 280
            },
            "feature_extractor": {
                "feature_extractor_type": "Gemma4AudioFeatureExtractor",
                "sampling_rate": 16000
            },
            "video_processor": {
                "video_processor_type": "Gemma4VideoProcessor",
                "num_frames": 32
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.processor_class.as_deref(), Some("Gemma4Processor"));
    assert_eq!(meta.image_processor_type.as_deref(), Some("Gemma4ImageProcessor"));
    assert_eq!(meta.image_seq_length, Some(280));
    assert_eq!(meta.audio_feature_extractor_type.as_deref(), Some("Gemma4AudioFeatureExtractor"));
    assert_eq!(meta.audio_sampling_rate, Some(16000));
    assert_eq!(meta.audio_seq_length, Some(750));
    assert_eq!(meta.video_processor_type.as_deref(), Some("Gemma4VideoProcessor"));
    assert_eq!(meta.video_num_frames, Some(32));
    assert_eq!(meta.has_video, Some(true));
}

// ── Test 18: multimodal detection from config.json ───────────────────────────

#[test]
fn test_multimodal_detection() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "gemma4",
            "architectures": ["Gemma4ForConditionalGeneration"],
            "vision_config": {"model_type": "gemma4_vision"},
            "audio_config": {"model_type": "gemma4_audio"}
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.has_vision, Some(true));
    assert_eq!(meta.has_audio, Some(true));
    assert_eq!(meta.vision_model_type.as_deref(), Some("gemma4_vision"));
    assert_eq!(meta.audio_model_type.as_deref(), Some("gemma4_audio"));
}

// ── Test 19: text_config extraction from config.json ─────────────────────────

#[test]
fn test_text_config_extraction() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "gemma4",
            "text_config": {
                "num_hidden_layers": 35,
                "hidden_size": 1536,
                "num_attention_heads": 8,
                "vocab_size": 262144,
                "max_position_embeddings": 131072
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert_eq!(meta.num_hidden_layers, Some(35));
    assert_eq!(meta.hidden_size, Some(1536));
    assert_eq!(meta.num_attention_heads, Some(8));
    assert_eq!(meta.max_position_embeddings, Some(131072));
    // vocab_size falls back from text_config when not at top level
    assert_eq!(meta.vocab_size, Some(262144));
}

// ── Test 20: torch_dtype fallback chain ──────────────────────────────────────

#[test]
fn test_torch_dtype_fallback_chain() {
    // Case A: torch_dtype present at top level — use it
    {
        let dir = TempDir::new().unwrap();
        let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
        write_file(&dir, "model.safetensors", &binary);
        write_json(
            &dir,
            "config.json",
            &serde_json::json!({
                "torch_dtype": "float16",
                "dtype": "bfloat16",
                "text_config": {"dtype": "float32"}
            }),
        );

        let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
        let meta = deps[0].ai_model_metadata.as_ref().unwrap();
        assert_eq!(
            meta.torch_dtype.as_deref(),
            Some("float16"),
            "Case A: torch_dtype at top level should take precedence"
        );
    }

    // Case B: only top-level dtype — use it
    {
        let dir = TempDir::new().unwrap();
        let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
        write_file(&dir, "model.safetensors", &binary);
        write_json(
            &dir,
            "config.json",
            &serde_json::json!({
                "dtype": "bfloat16",
                "text_config": {"dtype": "float32"}
            }),
        );

        let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
        let meta = deps[0].ai_model_metadata.as_ref().unwrap();
        assert_eq!(
            meta.torch_dtype.as_deref(),
            Some("bfloat16"),
            "Case B: top-level dtype should be used when torch_dtype is absent"
        );
    }

    // Case C: only text_config.dtype — use it
    {
        let dir = TempDir::new().unwrap();
        let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
        write_file(&dir, "model.safetensors", &binary);
        write_json(
            &dir,
            "config.json",
            &serde_json::json!({
                "text_config": {"dtype": "float32"}
            }),
        );

        let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
        let meta = deps[0].ai_model_metadata.as_ref().unwrap();
        assert_eq!(
            meta.torch_dtype.as_deref(),
            Some("float32"),
            "Case C: text_config.dtype should be used as last fallback"
        );
    }
}

// ── Test 21: full companion file extraction ──────────────────────────────────

#[test]
fn test_full_companion_file_extraction() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(
        &serde_json::json!({"format": "pt"}),
        &serde_json::json!({}),
    );
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "gemma4",
            "architectures": ["Gemma4ForConditionalGeneration"],
            "torch_dtype": "bfloat16",
            "transformers_version": "5.5.0",
            "vocab_size": 262144,
            "_name_or_path": "google/gemma-4-test",
            "text_config": {
                "num_hidden_layers": 35,
                "hidden_size": 1536,
                "num_attention_heads": 8,
                "max_position_embeddings": 131072
            },
            "vision_config": {"model_type": "gemma4_vision"},
            "audio_config": {"model_type": "gemma4_audio"}
        }),
    );
    write_json(
        &dir,
        "generation_config.json",
        &serde_json::json!({
            "temperature": 0.8,
            "top_k": 50,
            "top_p": 0.95
        }),
    );
    write_json(
        &dir,
        "tokenizer_config.json",
        &serde_json::json!({
            "processor_class": "Gemma4Processor",
            "model_max_length": 8192
        }),
    );
    write_json(
        &dir,
        "preprocessor_config.json",
        &serde_json::json!({
            "image_processor": {
                "image_processor_type": "Gemma4ImageProcessor",
                "image_seq_length": 280
            },
            "feature_extractor": {
                "feature_extractor_type": "Gemma4AudioFeatureExtractor",
                "sampling_rate": 16000
            },
            "video_processor": {
                "video_processor_type": "Gemma4VideoProcessor",
                "num_frames": 32
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    assert_eq!(dep.name, "google/gemma-4-test");
    assert_eq!(dep.ecosystem, "safetensors");

    let meta = dep.ai_model_metadata.as_ref().unwrap();

    // config.json fields
    assert_eq!(meta.model_type.as_deref(), Some("gemma4"));
    assert_eq!(meta.architecture.as_deref(), Some("Gemma4ForConditionalGeneration"));
    assert_eq!(meta.torch_dtype.as_deref(), Some("bfloat16"));
    assert_eq!(meta.transformers_version.as_deref(), Some("5.5.0"));
    assert_eq!(meta.vocab_size, Some(262144));

    // text_config fields
    assert_eq!(meta.num_hidden_layers, Some(35));
    assert_eq!(meta.hidden_size, Some(1536));
    assert_eq!(meta.num_attention_heads, Some(8));
    assert_eq!(meta.max_position_embeddings, Some(131072));

    // multimodal detection
    assert_eq!(meta.has_vision, Some(true));
    assert_eq!(meta.has_audio, Some(true));
    assert_eq!(meta.vision_model_type.as_deref(), Some("gemma4_vision"));
    assert_eq!(meta.audio_model_type.as_deref(), Some("gemma4_audio"));

    // generation_config.json
    assert_eq!(meta.generation_temperature, Some(0.8));
    assert_eq!(meta.generation_top_k, Some(50));
    assert_eq!(meta.generation_top_p, Some(0.95));

    // tokenizer_config.json
    assert_eq!(meta.processor_class.as_deref(), Some("Gemma4Processor"));
    assert_eq!(meta.model_max_length, Some(8192));

    // preprocessor_config.json
    assert_eq!(meta.image_processor_type.as_deref(), Some("Gemma4ImageProcessor"));
    assert_eq!(meta.image_seq_length, Some(280));
    assert_eq!(meta.audio_feature_extractor_type.as_deref(), Some("Gemma4AudioFeatureExtractor"));
    assert_eq!(meta.audio_sampling_rate, Some(16000));
    assert_eq!(meta.video_processor_type.as_deref(), Some("Gemma4VideoProcessor"));
    assert_eq!(meta.video_num_frames, Some(32));
    assert_eq!(meta.has_video, Some(true));

    // safetensors binary header
    assert_eq!(meta.safetensors_format.as_deref(), Some("pt"));
    assert_eq!(meta.shard_count, Some(1));
}

// ── Test 22: sub-model extraction — multimodal (text + vision + audio) ──────

#[test]
fn test_sub_model_extraction_multimodal() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "gemma4",
            "architectures": ["Gemma4ForConditionalGeneration"],
            "text_config": {
                "model_type": "gemma4_text",
                "num_hidden_layers": 35,
                "hidden_size": 1536,
                "num_attention_heads": 8,
                "num_key_value_heads": 1,
                "vocab_size": 262144,
                "max_position_embeddings": 131072,
                "dtype": "bfloat16",
                "intermediate_size": 6144
            },
            "vision_config": {
                "model_type": "gemma4_vision",
                "num_hidden_layers": 16,
                "hidden_size": 768,
                "num_attention_heads": 12,
                "num_key_value_heads": 12,
                "dtype": "bfloat16",
                "intermediate_size": 3072,
                "patch_size": 16,
                "default_output_length": 280
            },
            "audio_config": {
                "model_type": "gemma4_audio",
                "num_hidden_layers": 12,
                "hidden_size": 1024,
                "num_attention_heads": 8,
                "dtype": "bfloat16",
                "conv_kernel_size": 5,
                "output_proj_dims": 1536
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();

    assert_eq!(meta.sub_models.len(), 3, "expected 3 sub-models (text, vision, audio)");

    // Text sub-model
    let text = &meta.sub_models[0];
    assert_eq!(text.modality, "text");
    assert_eq!(text.model_type.as_deref(), Some("gemma4_text"));
    assert_eq!(text.num_hidden_layers, Some(35));
    assert_eq!(text.hidden_size, Some(1536));
    assert_eq!(text.num_attention_heads, Some(8));
    assert_eq!(text.vocab_size, Some(262144));
    assert_eq!(text.max_position_embeddings, Some(131072));

    // Vision sub-model
    let vision = &meta.sub_models[1];
    assert_eq!(vision.modality, "vision");
    assert_eq!(vision.model_type.as_deref(), Some("gemma4_vision"));
    assert_eq!(vision.num_hidden_layers, Some(16));
    assert_eq!(vision.hidden_size, Some(768));
    assert_eq!(vision.patch_size, Some(16));
    assert_eq!(vision.default_output_length, Some(280));

    // Audio sub-model
    let audio = &meta.sub_models[2];
    assert_eq!(audio.modality, "audio");
    assert_eq!(audio.model_type.as_deref(), Some("gemma4_audio"));
    assert_eq!(audio.num_hidden_layers, Some(12));
    assert_eq!(audio.hidden_size, Some(1024));
    assert_eq!(audio.conv_kernel_size, Some(5));
    assert_eq!(audio.output_proj_dims, Some(1536));
}

// ── Test 23: text-only config — no sub-models generated ─────────────────────

#[test]
fn test_sub_model_text_only_no_sub_models() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "llama",
            "text_config": {
                "model_type": "llama",
                "num_hidden_layers": 32,
                "hidden_size": 4096
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert!(
        meta.sub_models.is_empty(),
        "text-only model should not generate sub-models"
    );
}

// ── Test 24: no text_config — no sub-models ─────────────────────────────────

#[test]
fn test_sub_model_no_text_config() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "bert",
            "num_hidden_layers": 12,
            "hidden_size": 768
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();
    assert!(
        meta.sub_models.is_empty(),
        "model without text_config should not generate sub-models"
    );
}

// ── Test 25: vision + text only sub-models (LLaVA-style) ────────────────────

#[test]
fn test_sub_model_vision_text_only() {
    let dir = TempDir::new().unwrap();
    let binary = build_safetensors_binary(&serde_json::json!({}), &serde_json::json!({}));
    write_file(&dir, "model.safetensors", &binary);
    write_json(
        &dir,
        "config.json",
        &serde_json::json!({
            "model_type": "llava",
            "text_config": {
                "model_type": "llama",
                "num_hidden_layers": 32,
                "hidden_size": 4096,
                "num_attention_heads": 32,
                "vocab_size": 32000
            },
            "vision_config": {
                "model_type": "clip_vision_model",
                "num_hidden_layers": 24,
                "hidden_size": 1024,
                "num_attention_heads": 16,
                "patch_size": 14
            }
        }),
    );

    let deps = parse_safetensors_dir(dir.path(), 0).unwrap();
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().unwrap();

    assert_eq!(meta.sub_models.len(), 2, "expected 2 sub-models (text, vision)");

    let text = &meta.sub_models[0];
    assert_eq!(text.modality, "text");
    assert_eq!(text.model_type.as_deref(), Some("llama"));

    let vision = &meta.sub_models[1];
    assert_eq!(vision.modality, "vision");
    assert_eq!(vision.model_type.as_deref(), Some("clip_vision_model"));
    assert_eq!(vision.patch_size, Some(14));
}
