use std::fs;
use std::io::Write;
use tempfile::{NamedTempFile, TempDir};

use radeis_sc2sbom::formats::spdx::create_package_url;
use radeis_sc2sbom::models::Dependency;
use radeis_sc2sbom::parsers::parse_gguf_file;

// -- Helper types and functions for constructing GGUF binary data -----------

enum GgufTestValue<'a> {
    String(&'a str),
    Uint32(u32),
}

fn build_gguf_v3(metadata: &[(&str, GgufTestValue)]) -> Vec<u8> {
    let mut buf = Vec::new();
    // Magic: "GGUF" as little-endian u32 = 0x46554747
    buf.extend_from_slice(&0x4655_4747u32.to_le_bytes());
    // Version 3
    buf.extend_from_slice(&3u32.to_le_bytes());
    // Tensor count = 0 (u64 for v3)
    buf.extend_from_slice(&0u64.to_le_bytes());
    // Metadata KV count (u64 for v3)
    buf.extend_from_slice(&(metadata.len() as u64).to_le_bytes());
    // KV pairs
    for (key, value) in metadata {
        // Key: u64 length + UTF-8 bytes
        buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
        buf.extend_from_slice(key.as_bytes());
        // Value
        match value {
            GgufTestValue::String(s) => {
                buf.extend_from_slice(&8u32.to_le_bytes()); // GGUF_TYPE_STRING
                buf.extend_from_slice(&(s.len() as u64).to_le_bytes());
                buf.extend_from_slice(s.as_bytes());
            }
            GgufTestValue::Uint32(v) => {
                buf.extend_from_slice(&4u32.to_le_bytes()); // GGUF_TYPE_UINT32
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    buf
}

fn build_gguf_v2(metadata: &[(&str, GgufTestValue)]) -> Vec<u8> {
    let mut buf = Vec::new();
    // Magic
    buf.extend_from_slice(&0x4655_4747u32.to_le_bytes());
    // Version 2
    buf.extend_from_slice(&2u32.to_le_bytes());
    // Tensor count = 0 (u32 for v2)
    buf.extend_from_slice(&0u32.to_le_bytes());
    // Metadata KV count (u32 for v2)
    buf.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    // KV pairs (same encoding as v3 for string/uint32 values)
    for (key, value) in metadata {
        buf.extend_from_slice(&(key.len() as u64).to_le_bytes());
        buf.extend_from_slice(key.as_bytes());
        match value {
            GgufTestValue::String(s) => {
                buf.extend_from_slice(&8u32.to_le_bytes());
                buf.extend_from_slice(&(s.len() as u64).to_le_bytes());
                buf.extend_from_slice(s.as_bytes());
            }
            GgufTestValue::Uint32(v) => {
                buf.extend_from_slice(&4u32.to_le_bytes());
                buf.extend_from_slice(&v.to_le_bytes());
            }
        }
    }
    buf
}

fn write_temp_gguf(data: &[u8], suffix: &str) -> NamedTempFile {
    let mut temp = tempfile::Builder::new()
        .suffix(suffix)
        .tempfile()
        .expect("failed to create temp file");
    temp.write_all(data).expect("failed to write temp file");
    temp.flush().expect("failed to flush temp file");
    temp
}

// -- Tests -----------------------------------------------------------------

#[test]
fn test_gguf_magic_bytes_validation() {
    // Write invalid magic bytes
    let mut bad_data = Vec::new();
    bad_data.extend_from_slice(&0xDEADBEEFu32.to_le_bytes());
    bad_data.extend_from_slice(&3u32.to_le_bytes());
    bad_data.extend_from_slice(&0u64.to_le_bytes());
    bad_data.extend_from_slice(&0u64.to_le_bytes());

    let temp = write_temp_gguf(&bad_data, ".gguf");
    let result = parse_gguf_file(temp.path(), 0);
    assert!(result.is_err(), "Expected error for invalid magic bytes");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not a GGUF file"),
        "Error should mention invalid magic, got: {}",
        err_msg
    );
}

#[test]
fn test_gguf_v3_metadata_extraction() {
    let metadata = vec![
        ("general.architecture", GgufTestValue::String("llama")),
        ("general.name", GgufTestValue::String("test-model")),
        ("general.version", GgufTestValue::String("1.0")),
        ("general.author", GgufTestValue::String("Test Author")),
        ("general.license", GgufTestValue::String("Apache-2.0")),
        ("general.file_type", GgufTestValue::Uint32(14)), // Q4_K_M
    ];
    let data = build_gguf_v3(&metadata);
    let temp = write_temp_gguf(&data, ".gguf");

    let deps = parse_gguf_file(temp.path(), 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1, "Should return exactly 1 dependency");

    let dep = &deps[0];
    assert_eq!(dep.name, "test-model");
    assert_eq!(dep.version, "1.0");
    assert_eq!(dep.ecosystem, "gguf");
    assert_eq!(dep.author, Some("Test Author".to_string()));
    assert_eq!(dep.license, Some("Apache-2.0".to_string()));

    let ai_meta = dep
        .ai_model_metadata
        .as_ref()
        .expect("ai_model_metadata should be present");
    assert_eq!(ai_meta.architecture, Some("llama".to_string()));
    assert_eq!(ai_meta.quantization, Some("Q4_K_M".to_string()));
    assert_eq!(ai_meta.gguf_version, Some(3));
    assert_eq!(ai_meta.tensor_count, Some(0));
}

#[test]
fn test_gguf_missing_metadata_fallback() {
    // No metadata KV pairs at all
    let data = build_gguf_v3(&[]);
    let temp = tempfile::Builder::new()
        .prefix("my-fancy-model")
        .suffix(".gguf")
        .tempfile()
        .expect("failed to create temp file");
    {
        let mut f = temp.as_file();
        f.write_all(&data).expect("write failed");
        f.flush().expect("flush failed");
    }

    let deps = parse_gguf_file(temp.path(), 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);

    let dep = &deps[0];
    // Name should fall back to filename stem (without .gguf extension)
    let expected_stem = temp
        .path()
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(dep.name, expected_stem);
    assert_eq!(dep.version, "unknown");
}

#[test]
fn test_gguf_v2_format() {
    let metadata = vec![
        ("general.architecture", GgufTestValue::String("mistral")),
        ("general.name", GgufTestValue::String("v2-model")),
        ("general.version", GgufTestValue::String("2.0")),
    ];
    let data = build_gguf_v2(&metadata);
    let temp = write_temp_gguf(&data, ".gguf");

    let deps = parse_gguf_file(temp.path(), 0).expect("v2 parse should succeed");
    assert_eq!(deps.len(), 1);

    let dep = &deps[0];
    assert_eq!(dep.name, "v2-model");
    assert_eq!(dep.version, "2.0");
    assert_eq!(dep.ecosystem, "gguf");

    let ai_meta = dep
        .ai_model_metadata
        .as_ref()
        .expect("ai_model_metadata should be present");
    assert_eq!(ai_meta.architecture, Some("mistral".to_string()));
    assert_eq!(ai_meta.gguf_version, Some(2));
}

#[test]
fn test_spdx_gguf_purl_generic() {
    let dep = Dependency {
        name: "test-model".to_string(),
        version: "1.0".to_string(),
        ecosystem: "gguf".to_string(),
        repository_url: None,
        ..Default::default()
    };
    let purl = create_package_url(&dep);
    assert_eq!(purl, "pkg:generic/test-model@1.0?type=gguf");
}

#[test]
fn test_spdx_gguf_purl_huggingface() {
    let dep = Dependency {
        name: "test-model".to_string(),
        version: "1.0".to_string(),
        ecosystem: "gguf".to_string(),
        repository_url: Some(
            "https://huggingface.co/TheBloke/Llama-2-7B-GGUF".to_string(),
        ),
        ..Default::default()
    };
    let purl = create_package_url(&dep);
    assert_eq!(purl, "pkg:huggingface/TheBloke/Llama-2-7B-GGUF@1.0");
}

// -- Companion-file and shared tests (v1.0.12) --------------------------------

/// Helper: write a GGUF binary into a TempDir and return the path to the file.
fn write_gguf_in_dir(dir: &TempDir, filename: &str, data: &[u8]) -> std::path::PathBuf {
    let path = dir.path().join(filename);
    fs::write(&path, data).expect("failed to write GGUF file into temp dir");
    path
}

#[test]
fn test_gguf_config_json_full_extraction() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[
        ("general.architecture", GgufTestValue::String("llama")),
    ]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let config = r#"{
    "model_type": "llama",
    "architectures": ["LlamaForCausalLM"],
    "torch_dtype": "float16",
    "transformers_version": "4.40.0",
    "text_config": {
        "num_hidden_layers": 32,
        "hidden_size": 4096,
        "num_attention_heads": 32,
        "vocab_size": 32000,
        "max_position_embeddings": 4096
    }
}"#;
    fs::write(dir.path().join("config.json"), config).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    let meta = dep.ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    // Binary wins for architecture
    assert_eq!(meta.architecture.as_deref(), Some("llama"));
    // config.json fields
    assert_eq!(meta.model_type.as_deref(), Some("llama"));
    assert_eq!(meta.num_hidden_layers, Some(32));
    assert_eq!(meta.hidden_size, Some(4096));
    assert_eq!(meta.torch_dtype.as_deref(), Some("float16"));
    assert_eq!(meta.transformers_version.as_deref(), Some("4.40.0"));
}

#[test]
fn test_gguf_readme_frontmatter() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[
        ("general.name", GgufTestValue::String("TestModel")),
    ]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let readme = "\
---
base_model: mistralai/Mistral-7B-Instruct-v0.2
license: apache-2.0
model_creator: Mistral AI
model_name: Mistral 7B Instruct v0.2
pipeline_tag: text-generation
quantized_by: TheBloke
tags:
- finetuned
language:
- en
datasets:
- cerebras/SlimPajama-627B
---
# Model Card
Some description.
";
    fs::write(dir.path().join("README.md"), readme).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let dep = &deps[0];
    let meta = dep.ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    assert_eq!(meta.model_name.as_deref(), Some("Mistral 7B Instruct v0.2"));
    assert_eq!(meta.pipeline_tag.as_deref(), Some("text-generation"));
    assert_eq!(meta.quantized_by.as_deref(), Some("TheBloke"));

    // base_model
    assert_eq!(meta.base_models.len(), 1);
    assert_eq!(
        meta.base_models[0].repo_url.as_deref(),
        Some("mistralai/Mistral-7B-Instruct-v0.2")
    );

    // license fallback from README (normalized to proper SPDX casing)
    assert_eq!(dep.license.as_deref(), Some("Apache-2.0"));

    // tags, languages, datasets
    assert!(meta.tags.contains(&"finetuned".to_string()), "tags should contain 'finetuned'");
    assert!(meta.languages.contains(&"en".to_string()), "languages should contain 'en'");
    assert!(
        meta.datasets.contains(&"cerebras/SlimPajama-627B".to_string()),
        "datasets should contain 'cerebras/SlimPajama-627B'"
    );
}

#[test]
fn test_readme_frontmatter_base_model_list() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let readme = "\
---
base_model:
- org/model-a
- org/model-b
pipeline_tag: text-generation
---
";
    fs::write(dir.path().join("README.md"), readme).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    assert_eq!(meta.base_models.len(), 2, "Should have 2 base models");
    let urls: Vec<_> = meta.base_models.iter().filter_map(|b| b.repo_url.as_deref()).collect();
    assert!(urls.contains(&"org/model-a"), "Should contain org/model-a");
    assert!(urls.contains(&"org/model-b"), "Should contain org/model-b");
}

#[test]
fn test_readme_crlf_and_lowercase() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    // lowercase readme.md with CRLF line endings
    let readme = "---\r\nmodel_name: CRLF Test Model\r\npipeline_tag: text-generation\r\n---\r\n";
    fs::write(dir.path().join("readme.md"), readme).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    assert_eq!(meta.model_name.as_deref(), Some("CRLF Test Model"));
    assert_eq!(meta.pipeline_tag.as_deref(), Some("text-generation"));
}

#[test]
fn test_adapter_detection() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[
        ("general.architecture", GgufTestValue::String("llama")),
    ]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let adapter_config = r#"{
    "peft_type": "LORA",
    "base_model_name_or_path": "meta-llama/Llama-2-7b-hf"
}"#;
    fs::write(dir.path().join("adapter_config.json"), adapter_config).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    assert_eq!(meta.is_adapter, Some(true));
    assert_eq!(meta.adapter_type.as_deref(), Some("LORA"));
    assert_eq!(meta.base_models.len(), 1);
    assert_eq!(
        meta.base_models[0].repo_url.as_deref(),
        Some("meta-llama/Llama-2-7b-hf")
    );
}

#[test]
fn test_tags_union_merge() {
    let dir = TempDir::new().unwrap();
    // Use GGUF binary without tags (string arrays are complex to encode)
    // and verify that README tags are picked up
    let data = build_gguf_v3(&[
        ("general.architecture", GgufTestValue::String("llama")),
    ]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let readme = "\
---
tags:
- finetuned
- chat
---
";
    fs::write(dir.path().join("README.md"), readme).unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let meta = deps[0].ai_model_metadata.as_ref().expect("should have ai_model_metadata");

    assert!(meta.tags.contains(&"finetuned".to_string()), "tags should contain 'finetuned'");
    assert!(meta.tags.contains(&"chat".to_string()), "tags should contain 'chat'");
}

// ── Sub-model enrichment from companion config.json ─────────────────────────

#[test]
fn test_gguf_sub_model_enrichment() {
    let dir = TempDir::new().unwrap();
    let data = build_gguf_v3(&[
        ("general.architecture", GgufTestValue::String("gemma4")),
    ]);
    let gguf_path = write_gguf_in_dir(&dir, "model.gguf", &data);

    let config = serde_json::json!({
        "model_type": "gemma4",
        "text_config": {
            "model_type": "gemma4_text",
            "num_hidden_layers": 35,
            "hidden_size": 1536
        },
        "vision_config": {
            "model_type": "gemma4_vision",
            "num_hidden_layers": 16,
            "hidden_size": 768,
            "patch_size": 16
        }
    });
    fs::write(
        dir.path().join("config.json"),
        serde_json::to_vec_pretty(&config).unwrap(),
    )
    .unwrap();

    let deps = parse_gguf_file(&gguf_path, 0).expect("parse should succeed");
    assert_eq!(deps.len(), 1);
    let meta = deps[0]
        .ai_model_metadata
        .as_ref()
        .expect("should have ai_model_metadata");

    assert_eq!(meta.sub_models.len(), 2, "expected 2 sub-models (text, vision)");

    let text = &meta.sub_models[0];
    assert_eq!(text.modality, "text");
    assert_eq!(text.model_type.as_deref(), Some("gemma4_text"));
    assert_eq!(text.num_hidden_layers, Some(35));
    assert_eq!(text.hidden_size, Some(1536));

    let vision = &meta.sub_models[1];
    assert_eq!(vision.modality, "vision");
    assert_eq!(vision.model_type.as_deref(), Some("gemma4_vision"));
    assert_eq!(vision.num_hidden_layers, Some(16));
    assert_eq!(vision.hidden_size, Some(768));
    assert_eq!(vision.patch_size, Some(16));
}
