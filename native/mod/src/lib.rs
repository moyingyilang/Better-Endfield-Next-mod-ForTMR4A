//! `be-mod-ffi`：ForTMR2 `efml-core` 的 C ABI 适配层。
//!
//! 为什么单独一个 crate：上游 `efml-core` 显式 `#![forbid(unsafe_code)]`，
//! C ABI 的 `unsafe` 不能写进去；按 ForTMR2 `LANGUAGE.md` 的 FFI 规则，这里是
//! 薄 shim：opaque 输入 + 显式错误码 + 调用方提供的错误缓冲区，不跨 FFI 暴露
//! Rust 结构体布局。
//!
//! `panic = "abort"`：不允许 unwinding 穿过 FFI；本文件所有入口都不 panic，
//! 输入先做空指针/UTF-8 校验。

use std::ffi::{c_char, CStr};
use std::path::Path;

/// ABI 版本。任何导出签名变化都必须递增。
pub const BE_MOD_ABI_VERSION: u32 = 1;

/// 成功。
pub const BE_MOD_OK: i32 = 0;
/// mod 包结构/完整性校验失败（`mod.toml`、缺文件、校验和不符等）。
pub const BE_MOD_INVALID: i32 = 1;
/// 参数不可用：空指针、非 UTF-8 路径。
pub const BE_MOD_BAD_ARG: i32 = 2;
/// 与当前 loader 版本不兼容（`min_loader_version`）。
pub const BE_MOD_INCOMPATIBLE: i32 = 3;

/// 把错误信息写进调用方缓冲区，始终以 NUL 结尾。
///
/// # Safety
/// `err` 要么为空，要么指向至少 `cap` 字节的可写内存。
unsafe fn write_error(err: *mut c_char, cap: usize, message: &str) {
    if err.is_null() || cap == 0 {
        return;
    }
    let bytes = message.as_bytes();
    let n = bytes.len().min(cap - 1);
    std::ptr::copy_nonoverlapping(bytes.as_ptr(), err as *mut u8, n);
    *err.add(n) = 0;
}

/// 返回 ABI 版本。
#[no_mangle]
pub extern "C" fn be_mod_abi_version() -> u32 {
    BE_MOD_ABI_VERSION
}

/// 校验一个 mod 包目录（含 `mod.toml`）。
///
/// 返回 [`BE_MOD_OK`] 或错误码；失败原因写入 `err`（可为空）。
///
/// # Safety
/// - `dir` 必须是 NUL 结尾的 UTF-8 C 字符串；
/// - `err` 要么为空，要么指向至少 `err_cap` 字节的可写内存。
#[no_mangle]
pub unsafe extern "C" fn be_mod_validate(
    dir: *const c_char,
    err: *mut c_char,
    err_cap: usize,
) -> i32 {
    if dir.is_null() {
        write_error(err, err_cap, "dir is null");
        return BE_MOD_BAD_ARG;
    }
    let raw = match CStr::from_ptr(dir).to_str() {
        Ok(value) => value,
        Err(_) => {
            write_error(err, err_cap, "dir is not valid UTF-8");
            return BE_MOD_BAD_ARG;
        }
    };
    let path = Path::new(raw);

    let package = match efml_core::ModPackage::open(path) {
        Ok(package) => package,
        Err(cause) => {
            write_error(err, err_cap, &cause.to_string());
            return BE_MOD_INVALID;
        }
    };
    if let Err(cause) = package.ensure_loader_compatible() {
        write_error(err, err_cap, &cause.to_string());
        return BE_MOD_INCOMPATIBLE;
    }
    match package.validate() {
        Ok(_report) => BE_MOD_OK,
        Err(cause) => {
            write_error(err, err_cap, &cause.to_string());
            BE_MOD_INVALID
        }
    }
}

/// 读取 mod 包 id，写入调用方缓冲区（NUL 结尾）。
///
/// 返回 0 表示成功，否则返回 [`BE_MOD_BAD_ARG`] / [`BE_MOD_INVALID`]。
///
/// # Safety
/// 同 [`be_mod_validate`]；`out` 指向至少 `out_cap` 字节的可写内存。
#[no_mangle]
pub unsafe extern "C" fn be_mod_manifest_id(
    dir: *const c_char,
    out: *mut c_char,
    out_cap: usize,
) -> i32 {
    if dir.is_null() || out.is_null() || out_cap == 0 {
        return BE_MOD_BAD_ARG;
    }
    let raw = match CStr::from_ptr(dir).to_str() {
        Ok(value) => value,
        Err(_) => return BE_MOD_BAD_ARG,
    };
    match efml_core::ModPackage::open(Path::new(raw)) {
        Ok(package) => {
            write_error(out, out_cap, &package.manifest().id);
            0
        }
        Err(_) => BE_MOD_INVALID,
    }
}
