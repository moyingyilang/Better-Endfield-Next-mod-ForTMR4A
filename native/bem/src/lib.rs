//! `be-bem`：BEM 容器的读取与业务校验（CRX 的 Rust 叶子）。
//!
//! 对应 `docs/custom_model/BEM_FORMAT_SPEC.md`：
//! - §2 容器：40 字节头、UTF-8 JSON manifest、32 字节 Payload 目录、连续 Payload 字节；
//! - §3 Manifest 通用规则：schema / package_id / required_capabilities / 数组上限；
//! - §5–§7 的基本引用关系：payload 编号必须存在，顶点流与索引长度必须自洽。
//!
//! 设计边界：
//! - **纯叶子**：不碰游戏运行时，不做 JNI，不接管控制流；由 C/C++ 侧的导入链调用；
//! - **C ABI**：见 `native/shared/include/BetterEndfieldNext/BemPackage.h`；
//! - `panic = "abort"`，入口不 panic，错误走 `Result` → C 错误码。
//!
//! 与既有实现的对应关系：Python `tools/CustomModel/bem_v1.py` 系列是写入器与校验的
//! 参考实现，C++ `native/modules/custom_model/bem.cpp` 是读取器。本 crate 先覆盖
//! “容器 + 结构 + payload 引用”这条业务链，后续切片再补材质/驱动器等语义。

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::io::Read;
use std::path::Path;

use serde::Serialize;
use serde_json::Value;

pub const MAGIC: [u8; 8] = *b"BEM\0PKG\0";
pub const HEADER_SIZE: u64 = 40;
pub const MAX_FILE_SIZE: u64 = 2 * 1024 * 1024 * 1024;
pub const MAX_MANIFEST_SIZE: u64 = 4 * 1024 * 1024;
pub const MAX_PAYLOAD_SIZE: u64 = 512 * 1024 * 1024;
pub const MAX_ARRAY: usize = 4096;
pub const MAX_NESTING: usize = 48;

const BASE_CAPABILITIES: [&str; 3] = ["native-materials", "palette-u8", "indices-u32"];
const KNOWN_CAPABILITIES: [&str; 14] = [
    "native-materials",
    "palette-u8",
    "indices-u32",
    "fixed-appearances",
    "composable-options",
    "keep-material-textures",
    "texture-slots",
    "resource-bone-aliases",
    "body-parameters",
    "mesh-position-deltas",
    "multi-resource-targets",
    "static-meshes",
    "material-overrides-v1",
    "root-offset-v1",
];

/// 错误类型。文案面向包作者，包含字段与文件位置。
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O 失败: {0}")]
    Io(#[from] std::io::Error),
    #[error("不是 BEM 包：magic 不匹配")]
    Magic,
    #[error("BEM 版本不支持：{0}.{1}（当前接受 1.0–1.5）")]
    Version(u16, u16),
    #[error("header 字段非法：{0}")]
    Header(String),
    #[error("Payload 目录非法：{0}")]
    PayloadDir(String),
    #[error("Payload {0} 解压/读取失败：{1}")]
    Payload(u32, String),
    #[error("manifest 解析失败：{0}")]
    Manifest(String),
    #[error("校验失败：{0}")]
    Validate(String),
    #[error("写入失败：{0}")]
    Write(String),
}

impl Error {
    /// 映射到 C ABI 错误码：0 成功，1 包内容不合法，2 参数/IO，3 版本不支持。
    pub fn code(&self) -> i32 {
        match self {
            Error::Magic
            | Error::Header(_)
            | Error::PayloadDir(_)
            | Error::Payload(..)
            | Error::Manifest(_)
            | Error::Validate(_)
            | Error::Write(_) => 1,
            Error::Io(_) => 2,
            Error::Version(..) => 3,
        }
    }
}

/// Payload 目录项（§2）。
#[derive(Debug, Clone, Copy)]
pub struct PayloadMeta {
    pub codec: u32,
    pub offset: u64,
    pub stored_size: u64,
    pub decoded_size: u64,
}

/// 已打开并完成结构校验的 BEM 包。
#[derive(Debug)]
pub struct BemPackage {
    pub minor: u16,
    pub manifest: Value,
    pub payloads: Vec<PayloadMeta>,
    manifest_bytes: Vec<u8>,
    data: Vec<u8>,
}

/// 校验报告（`inspect` 与 C ABI 的 JSON 输出）。
#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub format_version: String,
    pub package_id: String,
    pub name: String,
    pub payload_count: usize,
    pub stored_bytes: u64,
    pub decoded_bytes: u64,
    pub referenced_payloads: usize,
    pub meshes: usize,
    pub textures: usize,
    pub warnings: Vec<String>,
}

fn u16_at(data: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([data[off], data[off + 1]])
}
fn u32_at(data: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]])
}
fn u64_at(data: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&data[off..off + 8]);
    u64::from_le_bytes(b)
}

impl BemPackage {
    /// 打开并做容器层校验（头、目录、连续性、codec、长度）。
    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let data = fs::read(path)?;
        Self::from_bytes(data)
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Self, Error> {
        let len = data.len() as u64;
        if len < HEADER_SIZE {
            return Err(Error::Magic);
        }
        if data[..8] != MAGIC {
            return Err(Error::Magic);
        }
        let major = u16_at(&data, 8);
        let minor = u16_at(&data, 10);
        if major != 1 || minor > 5 {
            return Err(Error::Version(major, minor));
        }
        let header_size = u32_at(&data, 12) as u64;
        let file_size = u64_at(&data, 16);
        let manifest_size = u64_at(&data, 24);
        let payload_count = u32_at(&data, 32) as usize;
        let flags = u32_at(&data, 36);

        if header_size != HEADER_SIZE {
            return Err(Error::Header(format!("header_size={header_size}，应为 40")));
        }
        if flags != 0 {
            return Err(Error::Header(format!("flags={flags}，应为 0")));
        }
        if file_size != len {
            return Err(Error::Header(format!(
                "file_size={file_size} 与实际长度 {len} 不一致"
            )));
        }
        if file_size > MAX_FILE_SIZE {
            return Err(Error::Header("文件超过 2 GiB".into()));
        }
        if manifest_size == 0 || manifest_size > MAX_MANIFEST_SIZE {
            return Err(Error::Header(format!(
                "manifest_size={manifest_size} 超出 1 字节..4 MiB"
            )));
        }
        if payload_count > MAX_ARRAY {
            return Err(Error::PayloadDir(format!(
                "payload_count={payload_count} 超过 {MAX_ARRAY}"
            )));
        }

        let dir_start = HEADER_SIZE + manifest_size;
        let dir_bytes = (payload_count as u64) * 32;
        if dir_start + dir_bytes > len {
            return Err(Error::PayloadDir("Payload 目录越界".into()));
        }
        let manifest_end = (HEADER_SIZE + manifest_size) as usize;
        let manifest_text = std::str::from_utf8(&data[HEADER_SIZE as usize..manifest_end])
            .map_err(|e| Error::Manifest(format!("manifest 不是 UTF-8：{e}")))?;
        let manifest: Value =
            serde_json::from_str(manifest_text).map_err(|e| Error::Manifest(e.to_string()))?;
        depth_check(&manifest, 1)?;

        let mut payloads = Vec::with_capacity(payload_count);
        let mut cursor = dir_start + dir_bytes;
        for index in 0..payload_count {
            let base = (dir_start as usize) + index * 32;
            let codec = u32_at(&data, base);
            let reserved = u32_at(&data, base + 4);
            let offset = u64_at(&data, base + 8);
            let stored_size = u64_at(&data, base + 16);
            let decoded_size = u64_at(&data, base + 24);
            if codec > 1 {
                return Err(Error::PayloadDir(format!("Payload {index} codec={codec}")));
            }
            if reserved != 0 {
                return Err(Error::PayloadDir(format!(
                    "Payload {index} reserved={reserved}，应为 0"
                )));
            }
            if stored_size == 0 || stored_size > MAX_PAYLOAD_SIZE || decoded_size > MAX_PAYLOAD_SIZE {
                return Err(Error::PayloadDir(format!(
                    "Payload {index} 长度非法（stored={stored_size}, decoded={decoded_size}）"
                )));
            }
            if codec == 0 && stored_size != decoded_size {
                return Err(Error::PayloadDir(format!(
                    "Payload {index} codec=raw 但 stored_size != decoded_size"
                )));
            }
            if offset != cursor {
                return Err(Error::PayloadDir(format!(
                    "Payload {index} offset={offset}，应为 {cursor}（不允许间隙/重叠）"
                )));
            }
            if offset + stored_size > len {
                return Err(Error::PayloadDir(format!("Payload {index} 越界")));
            }
            cursor = offset + stored_size;
            payloads.push(PayloadMeta {
                codec,
                offset,
                stored_size,
                decoded_size,
            });
        }
        if cursor != len {
            return Err(Error::PayloadDir(format!(
                "文件尾有 {} 字节尾随数据",
                (len - cursor) as i64
            )));
        }

        Ok(Self {
            minor,
            manifest,
            manifest_bytes: data[HEADER_SIZE as usize..manifest_end].to_vec(),
            payloads,
            data,
        })
    }

    /// 读取并解压一个 Payload（校验 decoded_size）。
    pub fn read_payload(&self, id: u32) -> Result<Vec<u8>, Error> {
        let meta = self
            .payloads
            .get(id as usize)
            .ok_or_else(|| Error::Payload(id, "payload 编号不存在".into()))?;
        let start = meta.offset as usize;
        let end = start + meta.stored_size as usize;
        let stored = &self.data[start..end];
        let out = if meta.codec == 0 {
            stored.to_vec()
        } else {
            let mut decoder = ruzstd::decoding::StreamingDecoder::new(stored)
                .map_err(|e| Error::Payload(id, format!("zstd 头错误：{e}")))?;
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|e| Error::Payload(id, format!("zstd 解压失败：{e}")))?;
            out
        };
        if out.len() as u64 != meta.decoded_size {
            return Err(Error::Payload(
                id,
                format!(
                    "decoded_size={} 与声明 {} 不一致",
                    out.len(),
                    meta.decoded_size
                ),
            ));
        }
        Ok(out)
    }

    /// 业务校验：manifest 结构 + 数组上限 + payload 引用 + 顶点/索引长度自洽。
    pub fn validate(&self) -> Result<Report, Error> {
        let m = &self.manifest;
        let mut warnings = Vec::new();

        let schema = m.get("schema").and_then(Value::as_u64);
        if schema != Some(1) {
            return Err(Error::Validate("schema 必须为 1".into()));
        }
        let package_id = m
            .get("package_id")
            .and_then(Value::as_str)
            .ok_or_else(|| Error::Validate("package_id 缺失".into()))?;
        if !valid_stable_id(package_id) {
            return Err(Error::Validate(format!(
                "package_id 非法（应为 [A-Za-z0-9][A-Za-z0-9_.-]{{0,95}}）：{package_id}"
            )));
        }
        let name = m.get("name").and_then(Value::as_str).unwrap_or("");

        let capabilities = m
            .get("required_capabilities")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::Validate("required_capabilities 缺失".into()))?;
        let mut caps = BTreeSet::new();
        for cap in capabilities {
            let cap = cap
                .as_str()
                .ok_or_else(|| Error::Validate("required_capabilities 含非字符串".into()))?;
            if !caps.insert(cap.to_string()) {
                return Err(Error::Validate(format!("能力重复：{cap}")));
            }
            if !KNOWN_CAPABILITIES.contains(&cap) {
                return Err(Error::Validate(format!("未知能力：{cap}")));
            }
        }
        for base in BASE_CAPABILITIES {
            if !caps.contains(base) {
                return Err(Error::Validate(format!("缺少必需能力：{base}")));
            }
        }

        let meshes = array(m, "meshes")?;
        let textures = array(m, "textures")?;
        check_len(meshes, "meshes")?;
        check_len(textures, "textures")?;

        let mut referenced = BTreeSet::new();

        for (mesh_index, mesh) in meshes.iter().enumerate() {
            let vertex_count = u64_field(mesh, "vertex_count", "mesh.vertex_count")?;
            let index_count = u64_field(mesh, "index_count", "mesh.index_count")?;
            let index_size = u64_field(mesh, "index_size", "mesh.index_size")?;
            if index_size != 2 && index_size != 4 {
                return Err(Error::Validate(format!(
                    "meshes[{mesh_index}].index_size 必须为 2 或 4"
                )));
            }
            let streams = mesh
                .get("streams")
                .and_then(Value::as_array)
                .ok_or_else(|| Error::Validate(format!("meshes[{mesh_index}].streams 缺失")))?;
            if streams.len() != 3 {
                return Err(Error::Validate(format!(
                    "meshes[{mesh_index}].streams 必须为 3 项"
                )));
            }
            for (stream_index, stream) in streams.iter().enumerate() {
                let stride = u64_field(stream, "stride", "stream.stride")?;
                if !(1..=64).contains(&stride) {
                    return Err(Error::Validate(format!(
                        "meshes[{mesh_index}].streams[{stream_index}].stride 超出 1..64"
                    )));
                }
                let payload = payload_id(stream, "meshes", mesh_index)?;
                referenced.insert(payload);
                let expected = vertex_count * stride;
                self.expect_payload_size(payload, expected, "顶点流")?;
            }
            // 1.0–1.2：网格级 indices；1.1+：draw 级 indices。两者都可能在包里。
            if let Some(indices) = mesh.get("indices").and_then(Value::as_u64) {
                if indices > u32::MAX as u64 {
                    return Err(Error::Validate(format!(
                        "meshes[{mesh_index}].indices 超出 UInt32"
                    )));
                }
                referenced.insert(indices as u32);
                self.expect_payload_size(
                    indices as u32,
                    index_count * index_size,
                    "网格级索引",
                )?;
            }
            if let Some(draws) = mesh.get("draws").and_then(Value::as_array) {
                for (draw_index, draw) in draws.iter().enumerate() {
                    if let Some(indices) = draw.get("indices").and_then(Value::as_u64) {
                        if indices > u32::MAX as u64 {
                            return Err(Error::Validate(format!(
                                "meshes[{mesh_index}].draws[{draw_index}].indices 超出 UInt32"
                            )));
                        }
                        referenced.insert(indices as u32);
                        let count = u64_field(draw, "count", "draw.count")?;
                        self.expect_payload_size(
                            indices as u32,
                            count * index_size,
                            "draw 索引",
                        )?;
                    }
                }
            }
        }

        for (texture_index, texture) in textures.iter().enumerate() {
            let payload = payload_id(texture, "textures", texture_index)?;
            referenced.insert(payload);
            let mips = u64_field(texture, "mips", "texture.mips")?;
            if !(1..=16).contains(&mips) {
                return Err(Error::Validate(format!(
                    "textures[{texture_index}].mips 超出 1..16"
                )));
            }
            // 具体字节数依赖像素格式表，切片 2 再补；先要求 payload 非空。
            let meta = self
                .payloads
                .get(payload as usize)
                .ok_or_else(|| Error::Validate(format!("Payload {payload} 不存在")))?;
            if meta.decoded_size == 0 {
                return Err(Error::Validate(format!(
                    "textures[{texture_index}] 的 Payload 为空"
                )));
            }
        }

        // appearances / 默认外观（1.0+）
        if let Some(appearances) = m.get("appearances").and_then(Value::as_array) {
            check_len(appearances, "appearances")?;
            let mut ids = BTreeSet::new();
            for (appearance_index, appearance) in appearances.iter().enumerate() {
                let id = appearance
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| Error::Validate("appearance.id 缺失".into()))?;
                if !ids.insert(id.to_string()) {
                    return Err(Error::Validate(format!("appearance.id 重复：{id}")));
                }
                if let Some(components) = appearance.get("components").and_then(Value::as_array) {
                    for component in components {
                        if let Some(target) = component.get("target").and_then(Value::as_u64) {
                            let target_count = m
                                .get("target")
                                .and_then(|t| t.get("components"))
                                .and_then(Value::as_array)
                                .map(|c| c.len())
                                .unwrap_or(0);
                            if target_count > 0 && target as usize >= target_count {
                                return Err(Error::Validate(format!(
                                    "appearances[{appearance_index}] target={target} 越界（目标组件 {target_count}）"
                                )));
                            }
                        }
                        if let Some(mesh) = component.get("mesh").and_then(Value::as_u64) {
                            if mesh as usize >= meshes.len() {
                                return Err(Error::Validate(format!(
                                    "appearances[{appearance_index}] mesh={mesh} 越界"
                                )));
                            }
                        }
                    }
                }
            }
            if let Some(default_id) = m.get("default_appearance_id").and_then(Value::as_str) {
                if !ids.contains(default_id) {
                    return Err(Error::Validate(format!(
                        "default_appearance_id={default_id} 不在 appearances 中"
                    )));
                }
            }
        } else if self.minor == 0 {
            warnings.push("1.0 包缺少 appearances（规范要求固定外观）".into());
        }

        // 1.1+ 的组合选项引用
        if let Some(groups) = m.get("option_groups").and_then(Value::as_array) {
            check_len(groups, "option_groups")?;
        }

        // 所有被引用的 payload 都必须存在
        for payload in &referenced {
            if *payload as usize >= self.payloads.len() {
                return Err(Error::Validate(format!(
                    "引用了不存在的 Payload {payload}（共 {} 个）",
                    self.payloads.len()
                )));
            }
        }

        // 解压全部 payload，确认 zstd 帧与 decoded_size 一致（参照 C++ 读取器行为）
        for id in 0..self.payloads.len() as u32 {
            if let Err(err) = self.read_payload(id) {
                let meta = self.payloads[id as usize];
                warnings.push(format!(
                    "Payload {id}（codec={}）解压校验失败：{err}",
                    meta.codec
                ));
            }
        }

        let stored_bytes = self.payloads.iter().map(|p| p.stored_size).sum();
        let decoded_bytes = self.payloads.iter().map(|p| p.decoded_size).sum();
        Ok(Report {
            format_version: format!("1.{}", self.minor),
            package_id: package_id.to_string(),
            name: name.to_string(),
            payload_count: self.payloads.len(),
            stored_bytes,
            decoded_bytes,
            referenced_payloads: referenced.len(),
            meshes: meshes.len(),
            textures: textures.len(),
            warnings,
        })
    }

    fn expect_payload_size(&self, id: u32, expected: u64, what: &str) -> Result<(), Error> {
        let meta = self
            .payloads
            .get(id as usize)
            .ok_or_else(|| Error::Validate(format!("{what}引用了不存在的 Payload {id}")))?;
        if meta.decoded_size != expected {
            return Err(Error::Validate(format!(
                "{what} Payload {id} 长度 {}，按 manifest 应为 {expected}",
                meta.decoded_size
            )));
        }
        Ok(())
    }
}

fn array<'a>(m: &'a Value, key: &str) -> Result<&'a [Value], Error> {
    match m.get(key).and_then(Value::as_array) {
        Some(value) => Ok(value.as_slice()),
        None => Err(Error::Validate(format!("{key} 缺失或不是数组"))),
    }
}

fn check_len(values: &[Value], what: &str) -> Result<(), Error> {
    if values.len() > MAX_ARRAY {
        return Err(Error::Validate(format!(
            "{what} 项数 {} 超过 {MAX_ARRAY}",
            values.len()
        )));
    }
    Ok(())
}

fn u64_field(value: &Value, key: &str, what: &str) -> Result<u64, Error> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::Validate(format!("{what} 缺失或不是 UInt32/UInt64")))
}

fn payload_id(value: &Value, what: &str, index: usize) -> Result<u32, Error> {
    let id = value
        .get("payload")
        .and_then(Value::as_u64)
        .ok_or_else(|| Error::Validate(format!("{what}[{index}].payload 缺失")))?;
    if id > u32::MAX as u64 {
        return Err(Error::Validate(format!(
            "{what}[{index}].payload 超出 UInt32"
        )));
    }
    Ok(id as u32)
}

fn valid_stable_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes.len() > 96 {
        return false;
    }
    let first = bytes[0] as char;
    if !first.is_ascii_alphanumeric() {
        return false;
    }
    bytes[1..]
        .iter()
        .all(|b| b.is_ascii_alphanumeric() || matches!(*b, b'_' | b'.' | b'-'))
}

fn depth_check(value: &Value, depth: usize) -> Result<(), Error> {
    if depth > MAX_NESTING {
        return Err(Error::Manifest(format!("嵌套超过 {MAX_NESTING} 层")));
    }
    match value {
        Value::Array(items) => {
            for item in items {
                depth_check(item, depth + 1)?;
            }
        }
        Value::Object(map) => {
            for (_, item) in map {
                depth_check(item, depth + 1)?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl fmt::Display for PayloadMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "codec={} off={} stored={} decoded={}",
            self.codec, self.offset, self.stored_size, self.decoded_size
        )
    }
}


// ------------------------------------------------------------------ writer

/// 以 raw（codec=0）Payload 写回 BEM 容器。
///
/// 为什么先只做 raw：转换器当前也写 raw；raw 不依赖压缩器，跨平台最稳。
/// zstd 压缩留给后续切片（`ruzstd` 只解码，不编码）。
pub fn write_bem_raw(minor: u16, manifest: &[u8], payloads: &[Vec<u8>]) -> Result<Vec<u8>, Error> {
    if minor > 5 {
        return Err(Error::Version(1, minor));
    }
    if manifest.is_empty() || manifest.len() as u64 > MAX_MANIFEST_SIZE {
        return Err(Error::Write("manifest 长度必须在 1 字节..4 MiB".into()));
    }
    let value: Value =
        serde_json::from_slice(manifest).map_err(|e| Error::Write(format!("manifest JSON 非法：{e}")))?;
    depth_check(&value, 1)?;
    if payloads.len() > MAX_ARRAY {
        return Err(Error::Write(format!(
            "payload 数 {} 超过 {MAX_ARRAY}",
            payloads.len()
        )));
    }

    let dir_end = HEADER_SIZE + manifest.len() as u64 + payloads.len() as u64 * 32;
    let mut metas = Vec::with_capacity(payloads.len());
    let mut cursor = dir_end;
    for (index, payload) in payloads.iter().enumerate() {
        if payload.is_empty() || payload.len() as u64 > MAX_PAYLOAD_SIZE {
            return Err(Error::Write(format!(
                "payload {index} 长度必须在 1 字节..512 MiB"
            )));
        }
        metas.push((cursor, payload.len() as u64));
        cursor += payload.len() as u64;
    }
    if cursor > MAX_FILE_SIZE {
        return Err(Error::Write("输出超过 2 GiB".into()));
    }

    let mut out = Vec::with_capacity(cursor as usize);
    out.extend_from_slice(&MAGIC);
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&minor.to_le_bytes());
    out.extend_from_slice(&(HEADER_SIZE as u32).to_le_bytes());
    out.extend_from_slice(&cursor.to_le_bytes());
    out.extend_from_slice(&(manifest.len() as u64).to_le_bytes());
    out.extend_from_slice(&(payloads.len() as u32).to_le_bytes());
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(manifest);
    for (offset, size) in &metas {
        out.extend_from_slice(&0u32.to_le_bytes()); // codec raw
        out.extend_from_slice(&0u32.to_le_bytes()); // reserved
        out.extend_from_slice(&offset.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
        out.extend_from_slice(&size.to_le_bytes());
    }
    for payload in payloads {
        out.extend_from_slice(payload);
    }
    Ok(out)
}

impl BemPackage {
    /// 读→写：保留 manifest，把所有 Payload 解出来后用 raw 重新封装。
    pub fn repack_raw(&self) -> Result<Vec<u8>, Error> {
        let mut payloads = Vec::with_capacity(self.payloads.len());
        for id in 0..self.payloads.len() as u32 {
            payloads.push(self.read_payload(id)?);
        }
        write_bem_raw(self.minor, &self.manifest_bytes, &payloads)
    }

    pub fn manifest_bytes(&self) -> &[u8] {
        &self.manifest_bytes
    }
}

// ------------------------------------------------------------------ C ABI

use std::ffi::{c_char, CStr};

/// ABI 版本；签名变化必须递增。
pub const BE_BEM_ABI_VERSION: u32 = 1;
pub const BE_BEM_OK: i32 = 0;
pub const BE_BEM_INVALID: i32 = 1;
pub const BE_BEM_BAD_ARG: i32 = 2;
pub const BE_BEM_INCOMPATIBLE: i32 = 3;

unsafe fn write_buf(out: *mut c_char, cap: usize, text: &str) -> usize {
    if out.is_null() || cap == 0 {
        return 0;
    }
    let bytes = text.as_bytes();
    let n = bytes.len().min(cap - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), out as *mut u8, n);
    *out.add(n) = 0;
    n
}

/// 返回 ABI 版本。
#[no_mangle]
pub extern "C" fn be_bem_abi_version() -> u32 {
    BE_BEM_ABI_VERSION
}

/// 校验一个 BEM 文件；失败原因写入 `err`（可为空）。
///
/// # Safety
/// `path` 必须是 NUL 结尾 UTF-8 C 字符串；`err` 要么为空，要么指向 ≥ `err_cap` 字节可写内存。
#[no_mangle]
pub unsafe extern "C" fn be_bem_validate(path: *const c_char, err: *mut c_char, err_cap: usize) -> i32 {
    if path.is_null() {
        write_buf(err, err_cap, "path is null");
        return BE_BEM_BAD_ARG;
    }
    let raw = match CStr::from_ptr(path).to_str() {
        Ok(value) => value,
        Err(_) => {
            write_buf(err, err_cap, "path is not valid UTF-8");
            return BE_BEM_BAD_ARG;
        }
    };
    match BemPackage::open(raw).and_then(|package| package.validate()) {
        Ok(_) => BE_BEM_OK,
        Err(cause) => {
            write_buf(err, err_cap, &cause.to_string());
            cause.code().max(BE_BEM_INVALID)
        }
    }
}

/// 读入 BEM 并以 raw payload 重写到 `output`（读→改→写链路的最小入口）。
///
/// # Safety
/// `input` / `output` 必须是 NUL 结尾 UTF-8 C 字符串；`err` 规则同 `be_bem_validate`。
#[no_mangle]
pub unsafe extern "C" fn be_bem_repack(
    input: *const c_char,
    output: *const c_char,
    err: *mut c_char,
    err_cap: usize,
) -> i32 {
    if input.is_null() || output.is_null() {
        write_buf(err, err_cap, "input/output is null");
        return BE_BEM_BAD_ARG;
    }
    let (src, dst) = match (CStr::from_ptr(input).to_str(), CStr::from_ptr(output).to_str()) {
        (Ok(src), Ok(dst)) => (src, dst),
        _ => {
            write_buf(err, err_cap, "path is not valid UTF-8");
            return BE_BEM_BAD_ARG;
        }
    };
    let package = match BemPackage::open(src) {
        Ok(package) => package,
        Err(cause) => {
            write_buf(err, err_cap, &cause.to_string());
            return cause.code().max(BE_BEM_INVALID);
        }
    };
    let bytes = match package.repack_raw() {
        Ok(bytes) => bytes,
        Err(cause) => {
            write_buf(err, err_cap, &cause.to_string());
            return cause.code().max(BE_BEM_INVALID);
        }
    };
    if let Err(cause) = std::fs::write(dst, &bytes) {
        write_buf(err, err_cap, &format!("写入失败：{cause}"));
        return BE_BEM_BAD_ARG;
    }
    BE_BEM_OK
}

/// 生成 `inspect` JSON；`out_cap` 不足时返回所需字节数（不含结尾 NUL），内容截断。
///
/// # Safety
/// 同 [`be_bem_validate`]。
#[no_mangle]
pub unsafe extern "C" fn be_bem_info_json(
    path: *const c_char,
    out: *mut c_char,
    out_cap: usize,
) -> i64 {
    if path.is_null() {
        return -(BE_BEM_BAD_ARG as i64);
    }
    let raw = match CStr::from_ptr(path).to_str() {
        Ok(value) => value,
        Err(_) => return -(BE_BEM_BAD_ARG as i64),
    };
    match BemPackage::open(raw).and_then(|package| package.validate()) {
        Ok(report) => match serde_json::to_string(&report) {
            Ok(text) => {
                let need = text.len();
                write_buf(out, out_cap, &text);
                need as i64
            }
            Err(_) => -(BE_BEM_INVALID as i64),
        },
        Err(cause) => {
            write_buf(out, out_cap, &cause.to_string());
            -(cause.code().max(BE_BEM_INVALID) as i64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic(minor: u16, manifest: &str, payload: &[u8]) -> Vec<u8> {
        let manifest = manifest.as_bytes();
        let mut data = Vec::new();
        data.extend_from_slice(&MAGIC);
        data.extend_from_slice(&1u16.to_le_bytes());
        data.extend_from_slice(&minor.to_le_bytes());
        data.extend_from_slice(&40u32.to_le_bytes());
        let total = 40 + manifest.len() as u64 + 32 + payload.len() as u64;
        data.extend_from_slice(&total.to_le_bytes());
        data.extend_from_slice(&(manifest.len() as u64).to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes()); // payload_count
        data.extend_from_slice(&0u32.to_le_bytes()); // flags
        data.extend_from_slice(manifest);
        let offset = 40 + manifest.len() as u64 + 32;
        data.extend_from_slice(&0u32.to_le_bytes()); // codec raw
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&offset.to_le_bytes());
        data.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        data.extend_from_slice(&(payload.len() as u64).to_le_bytes());
        data.extend_from_slice(payload);
        data
    }

    const MINIMAL: &str = r#"{"schema":1,"package_id":"test.pkg","name":"t","author":"a","version":"1","required_capabilities":["native-materials","palette-u8","indices-u32"],"target":{"character_id":"c"},"meshes":[],"textures":[]}"#;

    #[test]
    fn opens_minimal_package() {
        let bytes = synthetic(0, MINIMAL, &[1, 2, 3, 4]);
        let package = BemPackage::from_bytes(bytes).expect("open");
        let report = package.validate().expect("validate");
        assert_eq!(report.package_id, "test.pkg");
        assert_eq!(report.payload_count, 1);
        assert_eq!(package.read_payload(0).expect("payload").len(), 4);
    }

    #[test]
    fn rejects_bad_magic() {
        let mut bytes = synthetic(0, MINIMAL, &[0]);
        bytes[0] = b'X';
        assert!(matches!(BemPackage::from_bytes(bytes), Err(Error::Magic)));
    }

    #[test]
    fn rejects_file_size_mismatch() {
        let mut bytes = synthetic(0, MINIMAL, &[0]);
        bytes[16] ^= 0xFF;
        assert!(matches!(BemPackage::from_bytes(bytes), Err(Error::Header(_))));
    }

    #[test]
    fn rejects_dangling_payload_reference() {
        let manifest = r#"{"schema":1,"package_id":"test.pkg","required_capabilities":["native-materials","palette-u8","indices-u32"],"target":{"character_id":"c"},"meshes":[{"vertex_count":1,"index_count":0,"index_size":2,"streams":[{"stride":4,"payload":7},{"stride":4,"payload":7},{"stride":4,"payload":7}],"indices":0}],"textures":[]}"#;
        let bytes = synthetic(0, manifest, &[0, 0, 0, 0]);
        let package = BemPackage::from_bytes(bytes).expect("open");
        assert!(matches!(
            package.validate(),
            Err(Error::Validate(_)) | Err(Error::Payload(..))
        ));
    }

    #[test]
    fn writes_and_reopens_package() {
        let bytes = write_bem_raw(3, MINIMAL.as_bytes(), &[vec![9, 8, 7]]).expect("write");
        let package = BemPackage::from_bytes(bytes).expect("reopen");
        assert_eq!(package.minor, 3);
        assert_eq!(package.read_payload(0).expect("payload"), vec![9, 8, 7]);
        package.validate().expect("validate");
    }

    #[test]
    fn repack_fixture_round_trips() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../android/app/src/androidTest/assets/installer-fixture.bem");
        if !path.is_file() {
            return;
        }
        let before = BemPackage::open(&path).expect("open fixture");
        let bytes = before.repack_raw().expect("repack");
        let after = BemPackage::from_bytes(bytes).expect("reopen repacked");
        assert_eq!(before.minor, after.minor);
        assert_eq!(before.manifest_bytes(), after.manifest_bytes());
        assert_eq!(before.payloads.len(), after.payloads.len());
        for id in 0..before.payloads.len() as u32 {
            assert_eq!(
                before.read_payload(id).expect("before payload"),
                after.read_payload(id).expect("after payload")
            );
        }
        after.validate().expect("validate repacked");
    }

    #[test]
    fn rejects_invalid_manifest_on_write() {
        let result = write_bem_raw(0, b"not json", &[vec![1]]);
        assert!(matches!(result, Err(Error::Write(_))));
    }

    #[test]
    fn reads_fixture() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../android/app/src/androidTest/assets/installer-fixture.bem");
        if !path.is_file() {
            return;
        }
        let package = BemPackage::open(&path).expect("open fixture");
        let report = package.validate().expect("validate fixture");
        assert_eq!(report.format_version, "1.0");
        assert_eq!(report.package_id, "test.package");
        assert_eq!(report.payload_count, 7);
        assert_eq!(report.meshes, 1);
        assert_eq!(report.textures, 2);
    }
}
