//! `be-ir`：把 BEN 的 BEM 包转换成 ForTMR2 的 `mod.toml` + IR（CRX 叶子）。
//!
//! 定位：BEM 是 BEN 的创作物格式（不淘汰），ForTMR2 的 `mod.toml`/IR 是运行时格式。
//! 本 crate 负责两者之间的**结构与数据转换**：
//!
//! - BEM 的 3 条顶点流（position/normal/tangent、uv/color、skin）交错成 IR 的单顶点缓冲；
//! - BEM 网格级/按 draw 的索引写成 IR 子网格；
//! - BEM 贴图 payload 落盘为 IR 的贴图源；识别策略用 `asset-path`（`original_name`）；
//! - BEM 特有信息（外观、骨骼表、能力）写进 `[ext.ben.*]`（`efml-core` 忽略未知字段）；
//! - 生成后用 `efml-core` 自校验，保证产物是合法 mod 包。
//!
//! 未映射（切片 3 范围外）：骨骼 bindpose、材质槽名/关键字覆盖、形态键、驱动器、
//! 1.4 多资源目标。这些需要游戏 catalog 知识，留给后续切片或由 Converter4BENeXt 补。

use std::ffi::{c_char, CStr};
use std::fs;
use std::path::Path;

use efml_core::identify::Identify;
use efml_core::ir::{
    IndexFormat, MeshReplace, MeshSection, ModIr, SubMesh, TextureReplace, TextureSection,
    VertexAttribute, VertexFormat, VertexLayout,
};
use efml_core::manifest::{Backend, ModManifest, ModTarget};
use efml_core::ModPackage;
use serde::Serialize;
use serde_json::Value;

/// 顶点属性语义（Unity `VertexAttribute` 枚举）。
fn semantic_name(id: u64) -> String {
    match id {
        0 => "POSITION".into(),
        1 => "NORMAL".into(),
        2 => "TANGENT".into(),
        3 => "COLOR".into(),
        4..=11 => format!("TEXCOORD{}", id - 4),
        12 => "BLENDINDICES".into(),
        13 => "BLENDWEIGHTS".into(),
        other => format!("SEMANTIC{other}"),
    }
}

/// Unity `VertexAttributeFormat` + dimension → IR 顶点格式名。
fn vertex_format_name(format: u64, dimension: u64) -> String {
    let dim = dimension.clamp(1, 4);
    let comps = match dim {
        1 => "R",
        2 => "RG",
        3 => "RGB",
        _ => "RGBA",
    };
    let (bits, kind) = match format {
        0 => (32, "SFLOAT"),
        1 => (16, "SFLOAT"),
        2 => (8, "UNORM"),
        3 => (8, "SNORM"),
        4 => (16, "UNORM"),
        5 => (16, "SNORM"),
        6 => (8, "UINT"),
        7 => (8, "SINT"),
        8 => (16, "UINT"),
        9 => (16, "SINT"),
        10 => (32, "UINT"),
        11 => (32, "SINT"),
        _ => (32, "SFLOAT"),
    };
    let mut name = String::new();
    for ch in comps.chars() {
        name.push(ch);
        name.push_str(&bits.to_string());
    }
    name.push('_');
    name.push_str(kind);
    name
}

/// Unity `TextureFormat` → IR 贴图格式提示。
fn texture_format_name(format: u64, srgb: bool) -> String {
    let suffix = if srgb { "_SRGB" } else { "" };
    match format {
        4 => "R8G8B8A8_UNORM".into(),
        10 => format!("BC1_UNORM{suffix}"),
        12 => format!("BC3_UNORM{suffix}"),
        25 => format!("BC7_UNORM{suffix}"),
        26 => "BC4_UNORM".into(),
        27 => "BC5_UNORM".into(),
        48 => "ASTC_4x4_UNORM".into(),
        49 => "ASTC_5x5_UNORM".into(),
        50 => "ASTC_6x6_UNORM".into(),
        63 => "R8_UNORM".into(),
        other => format!("UNKNOWN_{other}"),
    }
}

/// 把 BEM 的 `package_id` 归一成 IR 允许的 mod id（小写/数字/`-`/`_`，≤64）。
fn sanitize_id(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    out.push_str("ben-");
    for ch in raw.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_lowercase() || lower.is_ascii_digit() || lower == '-' || lower == '_' {
            out.push(lower);
        } else {
            out.push('-');
        }
    }
    while out.ends_with('-') || out.ends_with('_') {
        out.pop();
    }
    if out.len() > 64 {
        out.truncate(64);
    }
    if out.is_empty() || out == "ben-" || out == "ben" {
        "ben-package".into()
    } else {
        out
    }
}

fn normalize_version(raw: &str) -> (String, Option<String>) {
    if semver::Version::parse(raw).is_ok() {
        return (raw.to_string(), None);
    }
    (
        "0.1.0".to_string(),
        Some(format!("BEM version `{raw}` 不是 semver，已归一为 0.1.0")),
    )
}

/// 转换选项。
#[derive(Debug, Clone)]
pub struct Options {
    pub backend: Backend,
    pub game: String,
    pub game_version: Option<String>,
    pub abi: Option<String>,
    pub hot_reload: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            backend: Backend::Il2CppHook,
            game: "arknights-endfield".into(),
            game_version: None,
            abi: Some("arm64-v8a".into()),
            hot_reload: true,
        }
    }
}

/// 转换报告。
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub mod_id: String,
    pub mod_dir: String,
    pub meshes: usize,
    pub submeshes: usize,
    pub textures: usize,
    pub vertex_bytes: u64,
    pub index_bytes: u64,
    pub warnings: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("BEM 读取失败：{0}")]
    Bem(#[from] be_bem::Error),
    #[error("BEM 校验失败：{0}")]
    BemValidate(String),
    #[error("manifest 字段缺失：{0}")]
    Missing(String),
    #[error("I/O 失败：{0}")]
    Io(#[from] std::io::Error),
    #[error("IR 生成失败：{0}")]
    Ir(String),
    #[error("产物自校验失败：{0}")]
    Validate(String),
}

/// 转换入口：BEM → mod 包目录。
pub fn convert_bem_to_mod(
    bem_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    options: &Options,
) -> Result<Report, Error> {
    let bem = be_bem::BemPackage::open(bem_path.as_ref())?;
    bem.validate().map_err(|e| Error::BemValidate(e.to_string()))?;
    convert_package(&bem, out_dir.as_ref(), options)
}

fn convert_package(bem: &be_bem::BemPackage, out_dir: &Path, options: &Options) -> Result<Report, Error> {
    let manifest = &bem.manifest;
    let mut warnings = Vec::new();

    let package_id = manifest
        .get("package_id")
        .and_then(Value::as_str)
        .ok_or_else(|| Error::Missing("package_id".into()))?;
    let name = manifest
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or(package_id);
    let raw_version = manifest.get("version").and_then(Value::as_str).unwrap_or("0.1.0");
    let (version, version_warning) = normalize_version(raw_version);
    if let Some(warning) = version_warning {
        warnings.push(warning);
    }
    let mod_id = sanitize_id(package_id);

    let meshes = manifest
        .get("meshes")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Missing("meshes".into()))?;
    let textures = manifest
        .get("textures")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Missing("textures".into()))?;

    fs::create_dir_all(out_dir.join("mesh"))?;
    fs::create_dir_all(out_dir.join("texture"))?;

    let mut mesh_replace = Vec::new();
    let mut vertex_bytes = 0u64;
    let mut index_bytes = 0u64;
    let mut submesh_count = 0usize;

    for (mesh_index, mesh) in meshes.iter().enumerate() {
        let replaced = convert_mesh(bem, mesh, mesh_index, out_dir, options, &mut warnings)?;
        vertex_bytes += replaced.vertex_bytes;
        index_bytes += replaced.index_bytes;
        submesh_count += replaced.submeshes;
        mesh_replace.push(replaced.replace);
    }

    let mut texture_replace = Vec::new();
    for (texture_index, texture) in textures.iter().enumerate() {
        let source = format!("texture/{texture_index}.bin");
        let payload = payload_id(texture, "textures", texture_index)?;
        let bytes = bem.read_payload(payload)?;
        fs::write(out_dir.join(&source), bytes)?;

        let asset_name = texture
            .get("original_name")
            .and_then(Value::as_str)
            .unwrap_or("texture")
            .to_string();
        let format_id = texture.get("format").and_then(Value::as_u64).unwrap_or(0);
        let srgb = texture.get("srgb").and_then(Value::as_bool).unwrap_or(false);
        let format = texture_format_name(format_id, srgb);
        if format.starts_with("UNKNOWN_") {
            warnings.push(format!(
                "textures[{texture_index}] 的 Unity TextureFormat={format_id} 没有 IR 映射，已原样写入格式提示"
            ));
        }
        texture_replace.push(TextureReplace {
            identify: Identify::AssetPath { path: asset_name },
            source,
            format: Some(format),
        });
    }

    let manifest_ir = ModIr {
        mesh: MeshSection { replace: mesh_replace },
        texture: TextureSection { replace: texture_replace },
        ..ModIr::default()
    };

    let mod_manifest = ModManifest {
        format_version: efml_core::version::SUPPORTED_FORMAT_VERSION,
        id: mod_id.clone(),
        name: name.to_string(),
        version,
        authors: vec!["Better Endfield Next (BEM)".into()],
        license: None,
        description: Some(format!(
            "Converted from BEM package `{package_id}` (BEM 1.{})",
            bem.minor
        )),
        target: ModTarget {
            game: options.game.clone(),
            game_version: options.game_version.clone(),
            abi: options.abi.clone(),
        },
        min_loader_version: None,
        backends: vec![options.backend],
        entry: None,
        ir: manifest_ir,
    };
    mod_manifest
        .validate()
        .map_err(|e| Error::Ir(e.to_string()))?;

    let mut toml_text = toml::to_string_pretty(&mod_manifest).map_err(|e| Error::Ir(e.to_string()))?;
    toml_text.push_str("\n# BEN 扩展：efml-core 忽略未知字段，运行时/工具可读\n[ext.ben]\n");
    toml_text.push_str(&format!(
        "source_package_id = {}\nbem_version = \"1.{}\"\npayload_count = {}\n",
        toml_string(package_id),
        bem.minor,
        bem.payloads.len()
    ));
    let appearances = manifest
        .get("appearances")
        .and_then(Value::as_array)
        .map(|items| items.len())
        .unwrap_or(0);
    toml_text.push_str(&format!("appearances = {appearances}\n"));
    let bones: usize = meshes
        .iter()
        .filter_map(|mesh| mesh.get("bones").and_then(Value::as_array))
        .map(|items| items.len())
        .sum();
    toml_text.push_str(&format!("bones = {bones}\n"));
    toml_text.push_str(
        "note = \"skeleton bindposes / material overrides / blendshapes are not converted yet\"\n",
    );
    fs::write(out_dir.join("mod.toml"), &toml_text)?;

    // checksums.txt：按 efml-core 的 `<sha256>  <相对路径>` 格式生成，加载器可校验完整性。
    let mut checksum_lines = String::new();
    let mut checksums = Vec::new();
    for relative in mod_manifest.referenced_files() {
        let path = out_dir.join(&relative);
        let digest = sha256_file(&path)?;
        checksums.push((relative, digest));
    }
    checksums.sort();
    for (relative, digest) in &checksums {
        checksum_lines.push_str(&format!("{digest}  {relative}\n"));
    }
    fs::write(out_dir.join("checksums.txt"), checksum_lines)?;

    // 产物自校验：efml-core 能打开并结构校验通过，才认为可用。
    let package = ModPackage::open(out_dir).map_err(|e| Error::Validate(e.to_string()))?;
    package
        .validate()
        .map_err(|e| Error::Validate(e.to_string()))?;

    Ok(Report {
        mod_id,
        mod_dir: out_dir.display().to_string(),
        meshes: meshes.len(),
        submeshes: submesh_count,
        textures: textures.len(),
        vertex_bytes,
        index_bytes,
        warnings,
    })
}

struct ConvertedMesh {
    replace: MeshReplace,
    vertex_bytes: u64,
    index_bytes: u64,
    submeshes: usize,
}

fn convert_mesh(
    bem: &be_bem::BemPackage,
    mesh: &Value,
    mesh_index: usize,
    out_dir: &Path,
    options: &Options,
    warnings: &mut Vec<String>,
) -> Result<ConvertedMesh, Error> {
    let vertex_count = u64_field(mesh, "vertex_count", "meshes")?;
    let index_size = u64_field(mesh, "index_size", "meshes")?;
    if index_size != 2 && index_size != 4 {
        return Err(Error::Ir(format!("meshes[{mesh_index}].index_size 必须为 2/4")));
    }
    let index_format = if index_size == 2 { IndexFormat::U16 } else { IndexFormat::U32 };

    let streams = mesh
        .get("streams")
        .and_then(Value::as_array)
        .ok_or_else(|| Error::Missing(format!("meshes[{mesh_index}].streams")))?;
    if streams.len() != 3 {
        return Err(Error::Ir(format!("meshes[{mesh_index}].streams 必须为 3 项")));
    }
    let strides: Vec<u64> = streams
        .iter()
        .map(|stream| u64_field(stream, "stride", "streams"))
        .collect::<Result<_, _>>()?;
    let mut payloads: Vec<Vec<u8>> = Vec::with_capacity(3);
    for (stream_index, stream) in streams.iter().enumerate() {
        let payload = payload_id(stream, "streams", stream_index)?;
        let bytes = bem.read_payload(payload)?;
        if bytes.len() as u64 != vertex_count * strides[stream_index] {
            return Err(Error::Ir(format!(
                "meshes[{mesh_index}].streams[{stream_index}] 长度与 vertex_count×stride 不一致"
            )));
        }
        payloads.push(bytes);
    }

    // 3 条流交错成 IR 的单顶点缓冲。
    let merged_stride: u64 = strides.iter().sum();
    let mut interleaved = Vec::with_capacity((vertex_count * merged_stride) as usize);
    for vertex in 0..vertex_count as usize {
        for (stream_index, stride) in strides.iter().enumerate() {
            let start = vertex * (*stride as usize);
            interleaved.extend_from_slice(&payloads[stream_index][start..start + *stride as usize]);
        }
    }
    let vertex_source = format!("mesh/{mesh_index}.vertex.bin");
    fs::write(out_dir.join(&vertex_source), &interleaved)?;

    // 属性表：BEM 的 [semantic, format, dimension, stream, offset] → IR 的合并偏移。
    let stream_offsets: Vec<u64> = {
        let mut offsets = Vec::with_capacity(3);
        let mut cursor = 0;
        for stride in &strides {
            offsets.push(cursor);
            cursor += stride;
        }
        offsets
    };
    let mut attributes = Vec::new();
    if let Some(attrs) = mesh.get("attributes").and_then(Value::as_array) {
        for (attr_index, attr) in attrs.iter().enumerate() {
            let list = attr
                .as_array()
                .ok_or_else(|| Error::Ir(format!("meshes[{mesh_index}].attributes[{attr_index}] 不是数组")))?;
            if list.len() != 5 {
                return Err(Error::Ir(format!(
                    "meshes[{mesh_index}].attributes[{attr_index}] 必须为 5 元组"
                )));
            }
            let semantic = list[0].as_u64().unwrap_or(u64::MAX);
            let format = list[1].as_u64().unwrap_or(0);
            let dimension = list[2].as_u64().unwrap_or(1);
            let stream = list[3].as_u64().unwrap_or(0) as usize;
            let offset = list[4].as_u64().unwrap_or(0);
            if stream >= 3 {
                return Err(Error::Ir(format!("meshes[{mesh_index}].attributes[{attr_index}] stream 越界")));
            }
            let merged_offset = stream_offsets[stream] + offset;
            let semantic_name = semantic_name(semantic);
            if semantic_name.starts_with("SEMANTIC") {
                warnings.push(format!(
                    "meshes[{mesh_index}] 的 semantic={semantic} 没有标准 IR 语义，已原样命名"
                ));
            }
            attributes.push(VertexAttribute {
                semantic: semantic_name,
                format: VertexFormat::new(vertex_format_name(format, dimension)),
                offset: merged_offset as u32,
                location: Some(attr_index as u32),
            });
        }
    }
    if attributes.is_empty() {
        return Err(Error::Ir(format!("meshes[{mesh_index}] 没有顶点属性")));
    }
    let vertex_layout = VertexLayout {
        stride: merged_stride as u32,
        attributes,
    };

    // 索引：1.0 是网格级 `indices`；1.1+ 每条 draw 自带 `indices`。
    let mut submeshes = Vec::new();
    let mut index_bytes = 0u64;
    if let Some(indices) = mesh.get("indices").and_then(Value::as_u64) {
        let count = u64_field(mesh, "index_count", "meshes")?;
        let source = format!("mesh/{mesh_index}.index.bin");
        let bytes = bem.read_payload(indices as u32)?;
        index_bytes += bytes.len() as u64;
        fs::write(out_dir.join(&source), bytes)?;
        submeshes.push(SubMesh {
            index_count: count as u32,
            index_format,
            source,
            topology: None,
        });
    } else if let Some(draws) = mesh.get("draws").and_then(Value::as_array) {
        for (draw_index, draw) in draws.iter().enumerate() {
            let Some(indices) = draw.get("indices").and_then(Value::as_u64) else {
                continue;
            };
            let count = u64_field(draw, "count", "draws")?;
            let source = format!("mesh/{mesh_index}.draw{draw_index}.index.bin");
            let bytes = bem.read_payload(indices as u32)?;
            index_bytes += bytes.len() as u64;
            fs::write(out_dir.join(&source), bytes)?;
            submeshes.push(SubMesh {
                index_count: count as u32,
                index_format,
                source,
                topology: None,
            });
        }
    }
    if submeshes.is_empty() {
        warnings.push(format!("meshes[{mesh_index}] 没有索引数据，已跳过子网格"));
    }

    let identify = Identify::VertexSignature {
        stride: merged_stride as u32,
        attributes: vertex_layout
            .attributes
            .iter()
            .map(|attr| attr.semantic.clone())
            .collect(),
    };

    Ok(ConvertedMesh {
        replace: MeshReplace {
            identify,
            vertex_layout: Some(vertex_layout),
            vertex_count: vertex_count as u32,
            submeshes: submeshes.clone(),
            source: vertex_source,
            hot_reload: options.hot_reload,
        },
        vertex_bytes: interleaved.len() as u64,
        index_bytes,
        submeshes: submeshes.len(),
    })
}

fn payload_id(value: &Value, what: &str, index: usize) -> Result<u32, Error> {
    value
        .get("payload")
        .and_then(Value::as_u64)
        .map(|id| id as u32)
        .ok_or_else(|| Error::Missing(format!("{what}[{index}].payload")))
}

fn u64_field(value: &Value, key: &str, what: &str) -> Result<u64, Error> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::Missing(format!("{what}.{key}")))
}

fn sha256_file(path: &Path) -> Result<String, Error> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Ok(hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

fn toml_string(value: &str) -> String {
    format!("{:?}", value)
}


// ------------------------------------------------------------------ BEM ZIP 容器（规范 §14）

fn infer_minor(manifest: &Value) -> u16 {
    if let Some(hint) = manifest.get("format_version") {
        let text = match hint {
            Value::String(text) => Some(text.clone()),
            Value::Number(number) => number.as_u64().map(|value| format!("1.{value}")),
            _ => None,
        };
        if let Some(text) = text {
            if let Some((_, minor)) = text.split_once('.') {
                if let Ok(minor) = minor.trim().parse::<u16>() {
                    if minor <= 5 {
                        return minor;
                    }
                }
            }
        }
    }
    if manifest.get("drivers").is_some()
        || manifest.get("root_offset").is_some()
        || manifest.get("visual_root_offset").is_some()
        || json_has_material_overrides(manifest)
    {
        return 5;
    }
    if manifest.get("parameters").is_some() || manifest.get("mesh_deformations").is_some() {
        return 3;
    }
    if manifest
        .get("target")
        .and_then(|target| target.get("resources"))
        .is_some()
    {
        return 4;
    }
    if manifest.get("texture_slots").is_some() {
        return 2;
    }
    if manifest.get("option_groups").is_some() || manifest.get("component_rules").is_some() {
        return 1;
    }
    0
}

fn json_has_material_overrides(value: &Value) -> bool {
    match value {
        Value::Array(items) => items.iter().any(json_has_material_overrides),
        Value::Object(map) => {
            map.contains_key("material_overrides") || map.values().any(json_has_material_overrides)
        }
        _ => false,
    }
}

/// 读 BEM 原生 ZIP 容器：`manifest.json` + `payloads/NNNN.bin`（可被单一顶层目录包裹）。
fn read_bem_zip(zip_path: &Path) -> Result<(String, Vec<Vec<u8>>, u16), Error> {
    use std::collections::{BTreeMap, BTreeSet};
    use zip::ZipArchive;

    let file = fs::File::open(zip_path)?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| Error::Ir(format!("ZIP 打开失败：{e}")))?;

    let mut names = Vec::with_capacity(archive.len());
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|e| Error::Ir(format!("ZIP 条目读取失败：{e}")))?;
        if entry.is_dir() {
            continue;
        }
        names.push((index, entry.name().to_string()));
    }

    // 顶层目录容错：只有一个顶层目录时剥掉它。
    let roots: BTreeSet<String> = names
        .iter()
        .map(|(_, name)| name.split('/').next().unwrap_or("").to_string())
        .collect();
    let prefix = if roots.len() == 1 && names.iter().any(|(_, name)| name.contains('/')) {
        format!("{}/", roots.iter().next().unwrap())
    } else {
        String::new()
    };

    let mut manifest: Option<String> = None;
    let mut payloads: BTreeMap<u32, Vec<u8>> = BTreeMap::new();
    for (index, name) in &names {
        let relative = name.strip_prefix(prefix.as_str()).unwrap_or(name);
        if relative == "manifest.json" {
            let entry = archive
                .by_index(*index)
                .map_err(|e| Error::Ir(format!("ZIP 条目读取失败：{e}")))?;
            let mut text = String::new();
            use std::io::Read;
            let mut reader = entry;
            reader
                .read_to_string(&mut text)
                .map_err(|e| Error::Ir(format!("manifest.json 读取失败：{e}")))?;
            manifest = Some(text);
        } else if let Some(rest) = relative.strip_prefix("payloads/") {
            if let Some(stem) = rest.strip_suffix(".bin") {
                if let Ok(id) = stem.parse::<u32>() {
                    let mut entry = archive
                        .by_index(*index)
                        .map_err(|e| Error::Ir(format!("ZIP 条目读取失败：{e}")))?;
                    let mut bytes = Vec::new();
                    use std::io::Read;
                    entry
                        .read_to_end(&mut bytes)
                        .map_err(|e| Error::Ir(format!("payload {id} 读取失败：{e}")))?;
                    payloads.insert(id, bytes);
                }
            }
        }
    }

    let manifest = manifest.ok_or_else(|| {
        Error::Ir("ZIP 不是 BEM 容器：缺少 manifest.json（可能只是普通压缩包）".into())
    })?;
    let value: Value = serde_json::from_str(&manifest)
        .map_err(|e| Error::Ir(format!("ZIP manifest.json 非法：{e}")))?;
    let minor = infer_minor(&value);

    let count = payloads.len() as u32;
    let mut ordered = Vec::with_capacity(payloads.len());
    for id in 0..count {
        let bytes = payloads.remove(&id).ok_or_else(|| {
            Error::Ir(format!("ZIP package is missing payload {id}"))
        })?;
        ordered.push(bytes);
    }
    if let Some(extra) = payloads.keys().next() {
        return Err(Error::Ir(format!("ZIP payload 编号不连续：出现 {extra}")));
    }
    Ok((manifest, ordered, minor))
}

/// BEM 原生 ZIP 容器 → mod 目录。
pub fn convert_bem_zip_to_mod(
    zip_path: impl AsRef<Path>,
    out_dir: impl AsRef<Path>,
    options: &Options,
) -> Result<Report, Error> {
    let (manifest, payloads, minor) = read_bem_zip(zip_path.as_ref())?;
    let bytes = be_bem::write_bem_raw(minor, manifest.as_bytes(), &payloads)?;
    let package = be_bem::BemPackage::from_bytes(bytes)?;
    package.validate().map_err(|e| Error::BemValidate(e.to_string()))?;
    convert_package(&package, out_dir.as_ref(), options)
}

// ------------------------------------------------------------------ C ABI

pub const BE_IR_ABI_VERSION: u32 = 1;
pub const BE_IR_OK: i32 = 0;
pub const BE_IR_INVALID: i32 = 1;
pub const BE_IR_BAD_ARG: i32 = 2;

unsafe fn write_buf(out: *mut c_char, cap: usize, text: &str) {
    if out.is_null() || cap == 0 {
        return;
    }
    let bytes = text.as_bytes();
    let n = bytes.len().min(cap - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, n);
    *out.add(n) = 0;
}

/// BEM → mod 目录（默认选项：il2cpp-hook、arknights-endfield、arm64-v8a）。
///
/// # Safety
/// `bem_path` / `out_dir` 为 NUL 结尾 UTF-8；`err` 规则同其它 C ABI。
#[no_mangle]
pub unsafe extern "C" fn be_ir_convert_bem(
    bem_path: *const c_char,
    out_dir: *const c_char,
    err: *mut c_char,
    err_cap: usize,
) -> i32 {
    if bem_path.is_null() || out_dir.is_null() {
        write_buf(err, err_cap, "bem_path/out_dir is null");
        return BE_IR_BAD_ARG;
    }
    let (src, dst) = match (CStr::from_ptr(bem_path).to_str(), CStr::from_ptr(out_dir).to_str()) {
        (Ok(src), Ok(dst)) => (src, dst),
        _ => {
            write_buf(err, err_cap, "path is not valid UTF-8");
            return BE_IR_BAD_ARG;
        }
    };
    match convert_bem_to_mod(src, dst, &Options::default()) {
        Ok(_) => BE_IR_OK,
        Err(cause) => {
            write_buf(err, err_cap, &cause.to_string());
            BE_IR_INVALID
        }
    }
}

#[no_mangle]
pub extern "C" fn be_ir_abi_version() -> u32 {
    BE_IR_ABI_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn fixture() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../android/app/src/androidTest/assets/installer-fixture.bem")
    }

    #[test]
    fn sanitizes_ids() {
        assert_eq!(sanitize_id("test.package"), "ben-test-package");
        assert_eq!(sanitize_id("A B"), "ben-a-b");
        assert_eq!(sanitize_id("___"), "ben-package");
    }

    #[test]
    fn maps_vertex_and_texture_formats() {
        assert_eq!(vertex_format_name(0, 3), "R32G32B32_SFLOAT");
        assert_eq!(vertex_format_name(6, 4), "R8G8B8A8_UINT");
        assert_eq!(texture_format_name(25, true), "BC7_UNORM_SRGB");
        assert_eq!(texture_format_name(10, false), "BC1_UNORM");
    }

    #[test]
    fn converts_bem_zip_container() {
        if !fixture().is_file() {
            return;
        }
        let package = be_bem::BemPackage::open(fixture()).expect("open fixture");
        let dir = tempfile::tempdir().expect("tempdir");
        let zip_path = dir.path().join("fixture.bem.zip");
        {
            use std::io::Write;
            let file = fs::File::create(&zip_path).expect("create zip");
            let mut writer = zip::ZipWriter::new(file);
            let stored: zip::write::SimpleFileOptions =
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored);
            writer.start_file("manifest.json", stored).expect("manifest entry");
            writer
                .write_all(package.manifest_bytes())
                .expect("manifest write");
            for (id, _) in package.payloads.iter().enumerate() {
                writer
                    .start_file(format!("payloads/{id:04}.bin"), stored)
                    .expect("payload entry");
                let bytes = package.read_payload(id as u32).expect("payload");
                writer.write_all(&bytes).expect("payload write");
            }
            writer.finish().expect("finish zip");
        }
        let out = dir.path().join("mod");
        let report = convert_bem_zip_to_mod(&zip_path, &out, &Options::default()).expect("zip convert");
        assert_eq!(report.meshes, 1);
        ModPackage::open(&out).expect("open generated").validate().expect("validate");
    }

    #[test]
    fn converts_fixture_to_valid_mod() {
        if !fixture().is_file() {
            return;
        }
        let dir = tempfile::tempdir().expect("tempdir");
        let report = convert_bem_to_mod(fixture(), dir.path(), &Options::default()).expect("convert");
        assert_eq!(report.meshes, 1);
        assert_eq!(report.textures, 2);
        assert!(report.submeshes >= 1);
        let toml = fs::read_to_string(dir.path().join("mod.toml")).expect("mod.toml");
        assert!(toml.contains("ben-test-package"));
        assert!(toml.contains("[[mesh.replace]]"));
        assert!(toml.contains("[ext.ben]"));
        // efml-core 能重新打开并校验
        let package = ModPackage::open(dir.path()).expect("open generated");
        package.validate().expect("validate generated");
    }
}
