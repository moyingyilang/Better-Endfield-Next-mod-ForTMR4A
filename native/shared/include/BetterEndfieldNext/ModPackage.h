/* ForTMR4A：mod 包格式桥（C ABI）
 *
 * 上游定义在 ForTMR2 的 `native/crates/efml-core`（Rust，forbid(unsafe_code)）。
 * 本头文件是唯一允许跨 ABI 的表面：opaque 路径字符串 + 错误码 + 调用方缓冲区，
 * 不暴露任何 Rust 结构体布局（见 ForTMR2 LANGUAGE.md 的 FFI 规则）。
 */
#ifndef BETTER_ENDFIELD_NEXT_MOD_PACKAGE_H
#define BETTER_ENDFIELD_NEXT_MOD_PACKAGE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* ABI 版本；签名变化必须递增。 */
#define BE_MOD_ABI_VERSION 1u

/* 返回值。 */
#define BE_MOD_OK 0
#define BE_MOD_INVALID 1
#define BE_MOD_BAD_ARG 2
#define BE_MOD_INCOMPATIBLE 3

/* 返回 ABI 版本。 */
uint32_t be_mod_abi_version(void);

/* 校验一个 mod 包目录（含 mod.toml）。
 * dir    : NUL 结尾 UTF-8 路径
 * err    : 错误信息缓冲区，可为 NULL
 * err_cap: 缓冲区字节数
 * 返回 BE_MOD_*；错误文本始终以 NUL 结尾。 */
int32_t be_mod_validate(const char *dir, char *err, uint32_t err_cap);

/* 读取 mod id 到 out（NUL 结尾）。返回 0 成功，否则 BE_MOD_*。 */
int32_t be_mod_manifest_id(const char *dir, char *out, uint32_t out_cap);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_MOD_PACKAGE_H */
