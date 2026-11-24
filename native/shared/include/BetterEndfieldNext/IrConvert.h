/* ForTMR4A：BEM → mod.toml/IR 转换（C ABI，CRX 叶子）
 *
 * 实现：native/ir（Rust，`be-ir`）。BEM 不淘汰：BEM 是输入格式，输出为 ForTMR2 的
 * mod 目录（mod.toml + 数据文件），生成后由 efml-core 自校验。
 */
#ifndef BETTER_ENDFIELD_NEXT_IR_CONVERT_H
#define BETTER_ENDFIELD_NEXT_IR_CONVERT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BE_IR_ABI_VERSION 1u
#define BE_IR_OK 0
#define BE_IR_INVALID 1
#define BE_IR_BAD_ARG 2

uint32_t be_ir_abi_version(void);

/* BEM 文件 → mod 目录（默认 il2cpp-hook / arknights-endfield / arm64-v8a）。 */
int32_t be_ir_convert_bem(const char *bem_path, const char *out_dir,
                          char *err, uint32_t err_cap);

/* BEM 原生 ZIP 容器（manifest.json + payloads/NNNN.bin）→ mod 目录。 */
int32_t be_ir_convert_bem_zip(const char *zip_path, const char *out_dir,
                              char *err, uint32_t err_cap);

/* 解压普通 ZIP 到目录（拒绝绝对路径/..），供 EFMI/EFF 压缩包先解包。 */
int32_t be_zip_extract(const char *zip_path, const char *dest_dir,
                       char *err, uint32_t err_cap);

/* 同上，但用 BEN catalog 补 skeleton.map / material.override。 */
int32_t be_ir_convert_bem_with_catalog(const char *bem_path, const char *catalog_dir,
                                       const char *out_dir, char *err, uint32_t err_cap);
int32_t be_ir_convert_bem_zip_with_catalog(const char *zip_path, const char *catalog_dir,
                                           const char *out_dir, char *err, uint32_t err_cap);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_IR_CONVERT_H */
