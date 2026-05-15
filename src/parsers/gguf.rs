use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

use crate::models::{AIModelMetadata, BaseModelInfo, Dependency, DependencyScope, DependencySource};
use super::safetensors::{
    read_companion_file, parse_readme_frontmatter, parse_model_max_length,
    merge_string_vecs, extract_sub_models, GenerationConfig, TokenizerConfig,
    PreprocessorConfig, AdapterConfig, HuggingFaceConfig, MAX_PROMPT_TEMPLATE_LEN,
};

// GGUF value type constants
const GGUF_TYPE_UINT8: u32 = 0;
const GGUF_TYPE_INT8: u32 = 1;
const GGUF_TYPE_UINT16: u32 = 2;
const GGUF_TYPE_INT16: u32 = 3;
const GGUF_TYPE_UINT32: u32 = 4;
const GGUF_TYPE_INT32: u32 = 5;
const GGUF_TYPE_FLOAT32: u32 = 6;
const GGUF_TYPE_BOOL: u32 = 7;
const GGUF_TYPE_STRING: u32 = 8;
const GGUF_TYPE_ARRAY: u32 = 9;
const GGUF_TYPE_UINT64: u32 = 10;
const GGUF_TYPE_INT64: u32 = 11;
const GGUF_TYPE_FLOAT64: u32 = 12;

/// Maximum bytes we'll read from the header (50 MB safety cap)
const MAX_HEADER_BYTES: u64 = 50 * 1024 * 1024;

/// GGUF magic bytes: "GGUF" in little-endian u32 = 0x46554747
const GGUF_MAGIC: u32 = 0x4655_4747;

/// Wrapper around a reader that tracks total bytes consumed and enforces a cap.
struct CappedReader<R: Read> {
    inner: R,
    bytes_read: u64,
    cap: u64,
}

impl<R: Read> CappedReader<R> {
    fn new(inner: R, cap: u64) -> Self {
        Self {
            inner,
            bytes_read: 0,
            cap,
        }
    }
}

impl<R: Read> Read for CappedReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.bytes_read >= self.cap {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "GGUF header read cap exceeded ({} bytes)",
                    self.cap
                ),
            ));
        }
        let remaining = (self.cap - self.bytes_read) as usize;
        let max_read = buf.len().min(remaining);
        let n = self.inner.read(&mut buf[..max_read])?;
        self.bytes_read += n as u64;
        Ok(n)
    }
}

// ── Low-level reading helpers ───────────────────────────────────────────────

fn read_u8(r: &mut impl Read) -> Result<u8> {
    let mut buf = [0u8; 1];
    r.read_exact(&mut buf).context("failed to read u8")?;
    Ok(buf[0])
}

fn read_u16_le(r: &mut impl Read) -> Result<u16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf).context("failed to read u16")?;
    Ok(u16::from_le_bytes(buf))
}

fn read_u32_le(r: &mut impl Read) -> Result<u32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).context("failed to read u32")?;
    Ok(u32::from_le_bytes(buf))
}

fn read_u64_le(r: &mut impl Read) -> Result<u64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).context("failed to read u64")?;
    Ok(u64::from_le_bytes(buf))
}

fn read_i8(r: &mut impl Read) -> Result<i8> {
    Ok(read_u8(r)? as i8)
}

fn read_i16_le(r: &mut impl Read) -> Result<i16> {
    let mut buf = [0u8; 2];
    r.read_exact(&mut buf).context("failed to read i16")?;
    Ok(i16::from_le_bytes(buf))
}

fn read_i32_le(r: &mut impl Read) -> Result<i32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).context("failed to read i32")?;
    Ok(i32::from_le_bytes(buf))
}

fn read_i64_le(r: &mut impl Read) -> Result<i64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).context("failed to read i64")?;
    Ok(i64::from_le_bytes(buf))
}

fn read_f32_le(r: &mut impl Read) -> Result<f32> {
    let mut buf = [0u8; 4];
    r.read_exact(&mut buf).context("failed to read f32")?;
    Ok(f32::from_le_bytes(buf))
}

fn read_f64_le(r: &mut impl Read) -> Result<f64> {
    let mut buf = [0u8; 8];
    r.read_exact(&mut buf).context("failed to read f64")?;
    Ok(f64::from_le_bytes(buf))
}

/// Read a GGUF-format string: u64 length + UTF-8 bytes (not null-terminated).
fn read_gguf_string(r: &mut impl Read) -> Result<String> {
    let len = read_u64_le(r).context("failed to read string length")?;
    if len > MAX_HEADER_BYTES {
        bail!("GGUF string length {} exceeds safety cap", len);
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf).context("failed to read string data")?;
    String::from_utf8(buf).context("GGUF string is not valid UTF-8")
}

// ── Value representation (only what we need to keep) ────────────────────────

/// A parsed GGUF metadata value. We only retain values for `general.*` keys;
/// everything else is read-and-discarded.
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum GgufValue {
    Uint8(u8),
    Int8(i8),
    Uint16(u16),
    Int16(i16),
    Uint32(u32),
    Int32(i32),
    Float32(f32),
    Bool(bool),
    String(String),
    Array(Vec<GgufValue>),
    Uint64(u64),
    Int64(i64),
    Float64(f64),
}

impl GgufValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            GgufValue::String(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_u64(&self) -> Option<u64> {
        match self {
            GgufValue::Uint8(v) => Some(*v as u64),
            GgufValue::Uint16(v) => Some(*v as u64),
            GgufValue::Uint32(v) => Some(*v as u64),
            GgufValue::Uint64(v) => Some(*v),
            GgufValue::Int8(v) if *v >= 0 => Some(*v as u64),
            GgufValue::Int16(v) if *v >= 0 => Some(*v as u64),
            GgufValue::Int32(v) if *v >= 0 => Some(*v as u64),
            GgufValue::Int64(v) if *v >= 0 => Some(*v as u64),
            _ => None,
        }
    }

    fn as_string_array(&self) -> Option<Vec<String>> {
        match self {
            GgufValue::Array(items) => {
                let mut out = Vec::with_capacity(items.len());
                for item in items {
                    out.push(item.as_str()?.to_string());
                }
                Some(out)
            }
            _ => None,
        }
    }
}

/// Read a single GGUF value by type tag.
/// `depth` limits recursion for nested arrays; callers should pass 0.
fn read_gguf_value(r: &mut impl Read, value_type: u32, depth: u32) -> Result<GgufValue> {
    if depth > 4 {
        bail!("GGUF nested array depth exceeds limit (>4)");
    }
    match value_type {
        GGUF_TYPE_UINT8 => Ok(GgufValue::Uint8(read_u8(r)?)),
        GGUF_TYPE_INT8 => Ok(GgufValue::Int8(read_i8(r)?)),
        GGUF_TYPE_UINT16 => Ok(GgufValue::Uint16(read_u16_le(r)?)),
        GGUF_TYPE_INT16 => Ok(GgufValue::Int16(read_i16_le(r)?)),
        GGUF_TYPE_UINT32 => Ok(GgufValue::Uint32(read_u32_le(r)?)),
        GGUF_TYPE_INT32 => Ok(GgufValue::Int32(read_i32_le(r)?)),
        GGUF_TYPE_FLOAT32 => Ok(GgufValue::Float32(read_f32_le(r)?)),
        GGUF_TYPE_BOOL => {
            let b = read_u8(r)?;
            Ok(GgufValue::Bool(b != 0))
        }
        GGUF_TYPE_STRING => Ok(GgufValue::String(read_gguf_string(r)?)),
        GGUF_TYPE_ARRAY => {
            let elem_type = read_u32_le(r)?;
            let count = read_u64_le(r)?;
            // Tight cap for materialized arrays (only general.* keys use this path;
            // their arrays are small — tags, languages, datasets). The CappedReader
            // provides a secondary 50MB I/O limit.
            if count > 1_000_000 {
                bail!("array count too large for materialized read: {} (max 1M)", count);
            }
            let mut items = Vec::with_capacity(count.min(64 * 1024) as usize);
            for _ in 0..count {
                items.push(read_gguf_value(r, elem_type, depth + 1)?);
            }
            Ok(GgufValue::Array(items))
        }
        GGUF_TYPE_UINT64 => Ok(GgufValue::Uint64(read_u64_le(r)?)),
        GGUF_TYPE_INT64 => Ok(GgufValue::Int64(read_i64_le(r)?)),
        GGUF_TYPE_FLOAT64 => Ok(GgufValue::Float64(read_f64_le(r)?)),
        other => bail!("unknown GGUF value type: {}", other),
    }
}

/// Skip (read and discard) a single GGUF value by type tag.
/// `depth` limits recursion for nested arrays; callers should pass 0.
fn skip_gguf_value(r: &mut impl Read, value_type: u32, depth: u32) -> Result<()> {
    if depth > 4 {
        bail!("GGUF nested array depth exceeds limit (>4)");
    }
    match value_type {
        GGUF_TYPE_UINT8 | GGUF_TYPE_INT8 | GGUF_TYPE_BOOL => {
            let mut buf = [0u8; 1];
            r.read_exact(&mut buf)?;
        }
        GGUF_TYPE_UINT16 | GGUF_TYPE_INT16 => {
            let mut buf = [0u8; 2];
            r.read_exact(&mut buf)?;
        }
        GGUF_TYPE_UINT32 | GGUF_TYPE_INT32 | GGUF_TYPE_FLOAT32 => {
            let mut buf = [0u8; 4];
            r.read_exact(&mut buf)?;
        }
        GGUF_TYPE_UINT64 | GGUF_TYPE_INT64 | GGUF_TYPE_FLOAT64 => {
            let mut buf = [0u8; 8];
            r.read_exact(&mut buf)?;
        }
        GGUF_TYPE_STRING => {
            let len = read_u64_le(r)?;
            // Discard string bytes in chunks to avoid huge allocations
            let mut remaining = len;
            let mut discard = [0u8; 8192];
            while remaining > 0 {
                let chunk = (remaining as usize).min(discard.len());
                r.read_exact(&mut discard[..chunk])?;
                remaining -= chunk as u64;
            }
        }
        GGUF_TYPE_ARRAY => {
            let elem_type = read_u32_le(r)?;
            let count = read_u64_le(r)?;
            if count > 100_000_000 {
                bail!("array count too large: {}", count);
            }
            for _ in 0..count {
                skip_gguf_value(r, elem_type, depth + 1)?;
            }
        }
        other => bail!("unknown GGUF value type: {}", other),
    }
    Ok(())
}

// ── file_type → quantization mapping ────────────────────────────────────────

fn file_type_to_quantization(file_type: u64) -> String {
    match file_type {
        0 => "F32".to_string(),
        1 => "F16".to_string(),
        2 => "Q4_0".to_string(),
        3 => "Q4_1".to_string(),
        7 => "Q8_0".to_string(),
        8 => "Q8_1".to_string(),
        9 => "Q2_K".to_string(),
        10 => "Q3_K_S".to_string(),
        11 => "Q3_K_M".to_string(),
        12 => "Q3_K_L".to_string(),
        13 => "Q4_K_S".to_string(),
        14 => "Q4_K_M".to_string(),
        15 => "Q5_K_S".to_string(),
        16 => "Q5_K_M".to_string(),
        17 => "Q6_K".to_string(),
        18 => "IQ2_XXS".to_string(),
        19 => "IQ2_XS".to_string(),
        20 => "IQ3_XXS".to_string(),
        21 => "IQ1_S".to_string(),
        22 => "IQ4_NL".to_string(),
        23 => "IQ3_S".to_string(),
        24 => "IQ2_S".to_string(),
        25 => "IQ4_XS".to_string(),
        26 => "IQ1_M".to_string(),
        27 => "BF16".to_string(),
        28 => "Q4_0_4_4".to_string(),
        29 => "Q4_0_4_8".to_string(),
        30 => "Q4_0_8_8".to_string(),
        _ => format!("UNKNOWN({})", file_type),
    }
}

/// Parse a human-readable size label like "476M", "4.6B", "7B" into an approximate parameter count.
fn parse_size_label(label: &str) -> Option<u64> {
    parse_size_label_public(label)
}

/// Public version of parse_size_label for use in console report integrity checks.
pub fn parse_size_label_public(label: &str) -> Option<u64> {
    let label = label.trim();
    let (num_str, multiplier) = if let Some(n) = label.strip_suffix('B') {
        (n, 1_000_000_000u64)
    } else if let Some(n) = label.strip_suffix('M') {
        (n, 1_000_000u64)
    } else if let Some(n) = label.strip_suffix('K') {
        (n, 1_000u64)
    } else {
        return None;
    };
    let num: f64 = num_str.parse().ok()?;
    if num <= 0.0 {
        return None;
    }
    Some((num * multiplier as f64) as u64)
}

/// Normalize a license string to proper SPDX identifier casing.
/// GGUF files often store licenses in lowercase (e.g. "apache-2.0")
/// but SPDX identifiers use specific casing (e.g. "Apache-2.0").
pub(crate) fn normalize_spdx_license(license: &str) -> String {
    // Common SPDX license mappings (lowercase → proper SPDX ID)
    match license.to_lowercase().as_str() {
        "apache-2.0" => "Apache-2.0".to_string(),
        "mit" => "MIT".to_string(),
        "gpl-2.0" => "GPL-2.0-only".to_string(),
        "gpl-3.0" => "GPL-3.0-only".to_string(),
        "lgpl-2.1" => "LGPL-2.1-only".to_string(),
        "lgpl-3.0" => "LGPL-3.0-only".to_string(),
        "bsd-2-clause" => "BSD-2-Clause".to_string(),
        "bsd-3-clause" => "BSD-3-Clause".to_string(),
        "mpl-2.0" => "MPL-2.0".to_string(),
        "isc" => "ISC".to_string(),
        "cc-by-4.0" => "CC-BY-4.0".to_string(),
        "cc-by-sa-4.0" => "CC-BY-SA-4.0".to_string(),
        "cc-by-nc-4.0" => "CC-BY-NC-4.0".to_string(),
        "cc-by-nc-sa-4.0" => "CC-BY-NC-SA-4.0".to_string(),
        "cc-by-nc-nd-4.0" => "CC-BY-NC-ND-4.0".to_string(),
        "agpl-3.0" => "AGPL-3.0-only".to_string(),
        "artistic-2.0" => "Artistic-2.0".to_string(),
        "wtfpl" => "WTFPL".to_string(),
        "zlib" => "Zlib".to_string(),
        "unlicense" => "Unlicense".to_string(),
        "0bsd" => "0BSD".to_string(),
        _ => license.to_string(), // pass through unknown licenses as-is
    }
}

/// Extract quantization type from a GGUF filename.
/// Matches common patterns like "model-Q4_K_M", "model-UD-IQ2_M", "mmproj-F16", "model-BF16".
fn extract_quantization_from_filename(stem: &str) -> Option<String> {
    // Known quantization patterns (order matters — check longer patterns first)
    const PATTERNS: &[&str] = &[
        "Q4_0_8_8", "Q4_0_4_8", "Q4_0_4_4",
        "IQ2_XXS", "IQ3_XXS", "IQ2_XS", "IQ4_XS",
        "IQ4_NL", "IQ3_S", "IQ2_S", "IQ2_M", "IQ1_S",
        "Q3_K_XL", "Q4_K_XL", "Q5_K_XL", "Q6_K_XL", "Q8_K_XL", "Q2_K_XL",
        "Q3_K_S", "Q3_K_M", "Q3_K_L",
        "Q4_K_S", "Q4_K_M",
        "Q5_K_S", "Q5_K_M",
        "Q2_K", "Q6_K", "Q8_K",
        "Q4_0", "Q4_1", "Q8_0", "Q8_1",
        "BF16", "F32", "F16",
    ];
    let upper = stem.to_uppercase();
    for pat in PATTERNS {
        if upper.contains(pat) {
            return Some(pat.to_string());
        }
    }
    None
}

// ── Public API ──────────────────────────────────────────────────────────────

/// Parse a GGUF binary file and return a single `Dependency` representing the AI model.
///
/// Supports GGUF format versions 2 and 3. Only `general.*` metadata keys are
/// retained; tokenizer and LLM configuration keys are skipped to save memory.
/// A 50 MB cap is enforced on total bytes read from the header section.
/// `max_hash_size_gb`: maximum file size in GB for SHA-256 hashing.
/// 0 means unlimited (hash all files). Files above the limit skip hashing
/// but still have their metadata parsed from the header.
pub fn parse_gguf_file(path: &Path, max_hash_size_gb: u64) -> Result<Vec<Dependency>> {
    let file = File::open(path)
        .with_context(|| format!("failed to open GGUF file: {}", path.display()))?;
    let buf = BufReader::new(file);
    let mut reader = CappedReader::new(buf, MAX_HEADER_BYTES);

    // 1. Magic bytes
    let magic = read_u32_le(&mut reader).context("failed to read GGUF magic")?;
    if magic != GGUF_MAGIC {
        bail!(
            "not a GGUF file: expected magic 0x{:08X}, got 0x{:08X}",
            GGUF_MAGIC,
            magic
        );
    }

    // 2. Version
    let version = read_u32_le(&mut reader).context("failed to read GGUF version")?;
    if version != 2 && version != 3 {
        bail!("unsupported GGUF version: {} (only v2 and v3 supported)", version);
    }

    // 3. Tensor count and metadata KV count (u64 for v3, u32 for v2)
    let tensor_count: u64;
    let metadata_kv_count: u64;
    if version >= 3 {
        tensor_count = read_u64_le(&mut reader).context("failed to read tensor count")?;
        metadata_kv_count = read_u64_le(&mut reader).context("failed to read metadata KV count")?;
    } else {
        tensor_count = read_u32_le(&mut reader).context("failed to read tensor count")? as u64;
        metadata_kv_count =
            read_u32_le(&mut reader).context("failed to read metadata KV count")? as u64;
    }

    // 4. Read metadata KV pairs — only store general.* keys
    //    Tolerate read errors on non-general.* keys (e.g. truncated files,
    //    large tokenizer arrays that exceed the read cap). All general.* keys
    //    are typically at the start of the metadata section, so we can still
    //    produce a useful result even if we can't read the entire header.
    //    Cap metadata_kv_count to prevent DoS from crafted files.
    const MAX_METADATA_KV_COUNT: u64 = 100_000;
    let metadata_kv_count = metadata_kv_count.min(MAX_METADATA_KV_COUNT);
    let mut general_kv: HashMap<String, GgufValue> = HashMap::new();

    for _ in 0..metadata_kv_count {
        let key = match read_gguf_string(&mut reader) {
            Ok(k) => k,
            Err(_) => break, // truncated or cap reached
        };
        let value_type = match read_u32_le(&mut reader) {
            Ok(v) => v,
            Err(_) => break,
        };

        if key.starts_with("general.") {
            match read_gguf_value(&mut reader, value_type, 0) {
                Ok(value) => {
                    general_kv.insert(key, value);
                }
                Err(_) => break, // truncated mid-value
            }
        } else {
            if skip_gguf_value(&mut reader, value_type, 0).is_err() {
                break; // truncated or cap reached — use what we have
            }
        }
    }

    // 4b. Read tensor descriptors to compute total parameter count.
    //     Each tensor: name (string), n_dimensions (u32), dimensions (u64[n]),
    //     type (u32), offset (u64).
    //     Parameter count = sum of product(dimensions) across all tensors.
    //     Tolerates read errors (truncated files) — uses what we parsed so far.
    let mut computed_params: u64 = 0;
    let mut tensors_read: u64 = 0;
    for _ in 0..tensor_count {
        // name
        if read_gguf_string(&mut reader).is_err() {
            break;
        }
        // n_dimensions (real tensors rarely exceed 4-5 dims; cap at 16 for safety)
        let n_dims = match read_u32_le(&mut reader) {
            Ok(n) if n <= 16 => n,
            Ok(n) => {
                eprintln!("Warning: GGUF tensor has unreasonable n_dims={}, stopping tensor scan", n);
                break;
            }
            Err(_) => break,
        };
        // dimensions
        let mut element_count: u64 = 1;
        for _ in 0..n_dims {
            match read_u64_le(&mut reader) {
                Ok(dim) => {
                    element_count = element_count.saturating_mul(dim);
                }
                Err(_) => {
                    element_count = 0;
                    break;
                }
            }
        }
        // type (u32)
        if read_u32_le(&mut reader).is_err() {
            break;
        }
        // offset (u64)
        if read_u64_le(&mut reader).is_err() {
            break;
        }
        computed_params = computed_params.saturating_add(element_count);
        tensors_read += 1;
    }

    // Only set computed_parameter_count if we successfully read ALL tensor descriptors
    let computed_parameter_count = if tensors_read == tensor_count {
        Some(computed_params)
    } else {
        None // truncated file — can't trust partial sum
    };

    // 5. Build AIModelMetadata from collected general.* keys
    let mut metadata = AIModelMetadata::default();
    metadata.tensor_count = Some(tensor_count);
    metadata.gguf_version = Some(version);
    metadata.computed_parameter_count = computed_parameter_count;

    // Helper closures for extracting typed values
    let get_str = |key: &str| -> Option<String> {
        general_kv.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    };
    let get_u64 = |key: &str| -> Option<u64> {
        general_kv.get(key).and_then(|v| v.as_u64())
    };
    let get_string_array = |key: &str| -> Vec<String> {
        general_kv
            .get(key)
            .and_then(|v| v.as_string_array())
            .unwrap_or_default()
    };

    metadata.architecture = get_str("general.architecture");
    metadata.description = get_str("general.description");
    metadata.organization = get_str("general.organization");
    metadata.basename = get_str("general.basename");
    metadata.finetune = get_str("general.finetune");
    metadata.size_label = get_str("general.size_label");
    metadata.parameter_count = get_u64("general.parameter_count");
    metadata.source_url = get_str("general.source.url");
    metadata.source_repo_url = get_str("general.source.repo_url");
    metadata.license_name = get_str("general.license.name");
    metadata.license_link = get_str("general.license.link");
    metadata.datasets = get_string_array("general.datasets");
    metadata.tags = get_string_array("general.tags");
    metadata.languages = get_string_array("general.languages");

    // Quantization: derive from file_type metadata, fall back to filename pattern
    // Many newer GGUF files (e.g. Gemma-4) don't include general.file_type;
    // the quantization is only encoded in the filename (e.g. "model-Q4_K_M.gguf").
    if let Some(ft) = get_u64("general.file_type") {
        metadata.quantization = Some(file_type_to_quantization(ft));
    } else {
        // Try to extract quantization from filename: look for known patterns
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            metadata.quantization = extract_quantization_from_filename(stem);
        }
    }

    // Base models (cap at 64 to prevent DoS from crafted metadata)
    let base_model_count = get_u64("general.base_model.count").unwrap_or(0).min(64);
    for idx in 0..base_model_count {
        let prefix = format!("general.base_model.{}.", idx);
        let info = BaseModelInfo {
            name: get_str(&format!("{}name", prefix)),
            author: get_str(&format!("{}author", prefix)),
            version: get_str(&format!("{}version", prefix)),
            organization: get_str(&format!("{}organization", prefix)),
            url: get_str(&format!("{}url", prefix)),
            repo_url: get_str(&format!("{}repo_url", prefix)),
        };
        metadata.base_models.push(info);
    }

    // ── 5a. Companion file enrichment (v1.0.12) ────────────────────────────
    // Binary KV metadata always wins. Companion files fill gaps only.
    let mut dep_license_fallback: Option<String> = None;
    let mut dep_author_fallback: Option<String> = None;

    if let Some(model_dir) = path.parent() {
        // config.json — full HuggingFaceConfig extraction
        if let Some(content) = read_companion_file(model_dir, "config.json") {
            if let Ok(hf_config) = serde_json::from_str::<HuggingFaceConfig>(&content) {
                // model_type: supplement architecture if not already set
                if metadata.model_type.is_none() {
                    metadata.model_type = hf_config.model_type.clone();
                }
                // Architecture fallback from config.json
                if metadata.architecture.is_none() {
                    metadata.architecture = hf_config
                        .architectures
                        .as_ref()
                        .and_then(|a| a.first())
                        .cloned()
                        .or(hf_config.model_type.clone());
                }
                // Dtype: binary doesn't have this, so always fill from config.json
                // Priority: torch_dtype > dtype > text_config.dtype
                if metadata.torch_dtype.is_none() {
                    metadata.torch_dtype = hf_config
                        .torch_dtype
                        .clone()
                        .or(hf_config.dtype.clone())
                        .or_else(|| {
                            hf_config.text_config.as_ref().and_then(|tc| tc.dtype.clone())
                        });
                }
                // Architecture fields: text_config (multimodal) > top-level (non-multimodal)
                if let Some(ref tc) = hf_config.text_config {
                    if metadata.num_hidden_layers.is_none() {
                        metadata.num_hidden_layers = tc.num_hidden_layers;
                    }
                    if metadata.hidden_size.is_none() {
                        metadata.hidden_size = tc.hidden_size;
                    }
                    if metadata.num_attention_heads.is_none() {
                        metadata.num_attention_heads = tc.num_attention_heads;
                    }
                    if metadata.max_position_embeddings.is_none() {
                        metadata.max_position_embeddings = tc.max_position_embeddings;
                    }
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
                // Multimodal detection
                if hf_config.vision_config.is_some() && metadata.has_vision.is_none() {
                    metadata.has_vision = Some(true);
                    if metadata.vision_model_type.is_none() {
                        metadata.vision_model_type = hf_config
                            .vision_config
                            .as_ref()
                            .and_then(|v| v.get("model_type"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
                if hf_config.audio_config.is_some() && metadata.has_audio.is_none() {
                    metadata.has_audio = Some(true);
                    if metadata.audio_model_type.is_none() {
                        metadata.audio_model_type = hf_config
                            .audio_config
                            .as_ref()
                            .and_then(|v| v.get("model_type"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                    }
                }
                // v1.0.13: Extract sub-model components
                metadata.sub_models = extract_sub_models(&hf_config);
                // Transformers version
                if metadata.transformers_version.is_none() {
                    metadata.transformers_version = hf_config.transformers_version;
                }
            }
        }

        // generation_config.json
        if let Some(content) = read_companion_file(model_dir, "generation_config.json") {
            if let Ok(gen) = serde_json::from_str::<GenerationConfig>(&content) {
                if metadata.generation_temperature.is_none() { metadata.generation_temperature = gen.temperature; }
                if metadata.generation_top_k.is_none() { metadata.generation_top_k = gen.top_k; }
                if metadata.generation_top_p.is_none() { metadata.generation_top_p = gen.top_p; }
            }
        }

        // tokenizer_config.json
        let mut tok_processor_class: Option<String> = None;
        if let Some(content) = read_companion_file(model_dir, "tokenizer_config.json") {
            if let Ok(tok) = serde_json::from_str::<TokenizerConfig>(&content) {
                tok_processor_class = tok.processor_class.clone();
                if metadata.processor_class.is_none() { metadata.processor_class = tok.processor_class; }
                if metadata.model_max_length.is_none() {
                    metadata.model_max_length = tok.model_max_length.as_ref().and_then(parse_model_max_length);
                }
            }
        }

        // preprocessor_config.json
        if let Some(content) = read_companion_file(model_dir, "preprocessor_config.json") {
            if let Ok(preproc) = serde_json::from_str::<PreprocessorConfig>(&content) {
                if metadata.processor_class.is_none() {
                    metadata.processor_class = preproc.processor_class.or_else(|| tok_processor_class.clone());
                }
                if metadata.image_seq_length.is_none() { metadata.image_seq_length = preproc.image_seq_length; }
                if metadata.audio_seq_length.is_none() { metadata.audio_seq_length = preproc.audio_seq_length; }
                if let Some(ref ip) = preproc.image_processor {
                    if metadata.image_processor_type.is_none() { metadata.image_processor_type = ip.image_processor_type.clone(); }
                    if metadata.image_seq_length.is_none() { metadata.image_seq_length = ip.image_seq_length; }
                }
                if let Some(ref fe) = preproc.feature_extractor {
                    if metadata.audio_feature_extractor_type.is_none() { metadata.audio_feature_extractor_type = fe.feature_extractor_type.clone(); }
                    if metadata.audio_sampling_rate.is_none() { metadata.audio_sampling_rate = fe.sampling_rate; }
                }
                if let Some(ref vp) = preproc.video_processor {
                    if metadata.video_processor_type.is_none() { metadata.video_processor_type = vp.video_processor_type.clone(); }
                    if metadata.video_num_frames.is_none() { metadata.video_num_frames = vp.num_frames; }
                    if metadata.has_video.is_none() { metadata.has_video = Some(true); }
                }
            }
        }

        // README.md frontmatter
        if let Some(fm) = parse_readme_frontmatter(model_dir) {
            if metadata.model_name.is_none() { metadata.model_name = fm.model_name; }
            if metadata.pipeline_tag.is_none() { metadata.pipeline_tag = fm.pipeline_tag; }
            if metadata.quantized_by.is_none() { metadata.quantized_by = fm.quantized_by; }
            if metadata.prompt_template.is_none() {
                metadata.prompt_template = fm.prompt_template.map(|pt| {
                    if pt.chars().count() > MAX_PROMPT_TEMPLATE_LEN {
                        format!("{}...", pt.chars().take(MAX_PROMPT_TEMPLATE_LEN).collect::<String>())
                    } else {
                        pt
                    }
                });
            }
            // base_model: fill only if binary didn't provide any
            if metadata.base_models.is_empty() && !fm.base_model.is_empty() {
                for repo in &fm.base_model {
                    metadata.base_models.push(BaseModelInfo {
                        name: None, author: None, version: None,
                        organization: None, url: None,
                        repo_url: Some(repo.clone()),
                    });
                }
            }
            // Deduplicated union merge for tags, languages, datasets
            merge_string_vecs(&mut metadata.tags, &fm.tags);
            merge_string_vecs(&mut metadata.languages, &fm.language);
            merge_string_vecs(&mut metadata.datasets, &fm.datasets);
            // Fallbacks for dependency-level fields
            dep_license_fallback = fm.license;
            dep_author_fallback = fm.model_creator;
        }

        // adapter_config.json
        if let Some(content) = read_companion_file(model_dir, "adapter_config.json") {
            if let Ok(adapter) = serde_json::from_str::<AdapterConfig>(&content) {
                metadata.is_adapter = Some(true);
                metadata.adapter_type = adapter.peft_type;
                if metadata.base_models.is_empty() {
                    if let Some(ref base) = adapter.base_model_name_or_path {
                        metadata.base_models.push(BaseModelInfo {
                            name: None, author: None, version: None,
                            organization: None, url: None,
                            repo_url: Some(base.clone()),
                        });
                    }
                }
            }
        }
    }

    // 5b. Integrity validation: compare declared vs computed parameter count
    if let (Some(declared), Some(computed)) = (metadata.parameter_count, metadata.computed_parameter_count) {
        if declared != computed {
            eprintln!(
                "Warning: GGUF integrity mismatch in {}: declared parameter_count={} but computed from tensors={}. \
                 The model file may be corrupted or tampered with.",
                path.display(), declared, computed
            );
        }
    }

    // 5c. Cross-validate size_label against computed parameter count
    //     e.g. size_label "476M" should be consistent with computed 475,729,088
    if let (Some(ref size_label), Some(computed)) = (&metadata.size_label, metadata.computed_parameter_count) {
        if let Some(expected) = parse_size_label(size_label) {
            // Allow 5% tolerance for rounding differences
            let lower = (expected as f64 * 0.95) as u64;
            let upper = (expected as f64 * 1.05) as u64;
            if computed < lower || computed > upper {
                eprintln!(
                    "Warning: GGUF size_label mismatch in {}: size_label='{}' (~{} params) but computed from tensors={}. \
                     The model file may be corrupted or tampered with.",
                    path.display(), size_label, expected, computed
                );
            }
        }
    }

    // 5d. Compute SHA-256 hash of the entire file (streaming, 8KB chunks).
    //     When max_hash_size_gb > 0, skip hashing for files exceeding the limit.
    //     When max_hash_size_gb == 0, hash all files (unlimited).
    let max_hash_bytes = if max_hash_size_gb == 0 {
        u64::MAX
    } else {
        max_hash_size_gb.saturating_mul(1024 * 1024 * 1024)
    };
    let mut hash_skip_reason: Option<String> = None;
    let checksum_sha256 = match std::fs::metadata(path) {
        Ok(meta) if meta.len() > max_hash_bytes => {
            let reason = format!(
                "file size ({:.1} GB) exceeds --max-hash-size-gb {} limit",
                meta.len() as f64 / 1_073_741_824.0,
                max_hash_size_gb
            );
            eprintln!("Note: Skipping SHA-256 hash for {}: {}", path.display(), reason);
            hash_skip_reason = Some(reason);
            None
        }
        _ => match File::open(path) {
            Ok(mut file) => {
                let mut hasher = Sha256::new();
                let mut buf = [0u8; 8192];
                let mut read_error = false;
                loop {
                    match file.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => hasher.update(&buf[..n]),
                        Err(e) => {
                            let reason = format!("I/O error: {}", e);
                            eprintln!("Warning: Skipping SHA-256 hash for {}: {}", path.display(), reason);
                            hash_skip_reason = Some(reason);
                            read_error = true;
                            break;
                        }
                    }
                }
                if read_error {
                    None
                } else {
                    Some(format!("{:x}", hasher.finalize()))
                }
            }
            Err(e) => {
                hash_skip_reason = Some(format!("failed to open file: {}", e));
                None
            }
        },
    };
    metadata.hash_skip_reason = hash_skip_reason;

    // 6. Construct the Dependency
    let name = get_str("general.name").unwrap_or_else(|| {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-model")
            .to_string()
    });
    // Use explicit version if present; otherwise use quantization as a qualifier
    // so different quantized variants (Q4_K_M, Q8_0, etc.) are not deduplicated
    // into a single entry.
    let dep_version = get_str("general.version").unwrap_or_else(|| {
        metadata
            .quantization
            .as_deref()
            .map(|q| q.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    });
    let author = get_str("general.author").or(dep_author_fallback);
    let license = get_str("general.license")
        .or(dep_license_fallback)
        .map(|l| normalize_spdx_license(&l));
    let homepage_url = get_str("general.url");
    let repository_url = get_str("general.repo_url")
        .or_else(|| metadata.source_repo_url.clone())
        .or_else(|| metadata.source_url.clone());

    let source_file =
        crate::parsers::format_source_info("gguf/model", path, None, false);

    let dep = Dependency {
        name,
        version: dep_version,
        ecosystem: "gguf".to_string(),
        source: DependencySource::Manifest,
        is_direct: true,
        is_dev: false,
        license,
        author,
        repository_url,
        homepage_url,
        source_file: Some(source_file),
        checksum_sha256,
        ai_model_metadata: Some(metadata),
        scope: DependencyScope::Runtime,
        scope_confidence: 0.9,
        scope_reason: "AI model file".to_string(),
        ..Default::default()
    };

    Ok(vec![dep])
}
