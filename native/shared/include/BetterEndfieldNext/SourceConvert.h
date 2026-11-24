/* ForTMR4A：多格式源接入（C ABI）
 *
 * 统一入口：BEM（原生）/ EFMI / EFF Format 3 / ForTMR2 mod.toml。
 * EFMI/EFF 走 Converter4BENeXt（vendored，AGPL-3.0-or-later）进程内转换到 BEM，
 * 不再需要外部 CLI；BEM 与 mod.toml 是原生格式，原样交给后续校验链。
 */
#ifndef BETTER_ENDFIELD_NEXT_SOURCE_CONVERT_H
#define BETTER_ENDFIELD_NEXT_SOURCE_CONVERT_H

#ifdef __cplusplus
extern "C" {
#endif

#define BE_SRC_UNKNOWN 0
#define BE_SRC_BEM 1
#define BE_SRC_MOD_TOML 2
#define BE_SRC_EFMI 3
#define BE_SRC_EFF3 4
#define BE_SRC_ZIP 5

/* 探测来源类型；目录或文件都可以。 */
int be_source_probe(const char *path);

/* 转换为 BEM。
 * - BEM 源：out_path 可为 NULL（只做存在性/结构判断），否则复制到 out_path；
 * - mod.toml 源：返回 BE_SRC_MOD_TOML（由 IR 链处理，不做 BEM 转换）；
 * - EFMI/EFF：需要 catalog_dir 与 out_path，调用 Converter4BENeXt。
 * 返回值：0 成功；1 转换失败；2 参数错误；3 不支持。错误文本写入 err。 */
int be_source_convert_to_bem(const char *path, const char *catalog_dir,
                             const char *out_path, char *err, unsigned int err_cap);

/* 适配层 ABI 版本。 */
int be_source_abi_version(void);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_SOURCE_CONVERT_H */
