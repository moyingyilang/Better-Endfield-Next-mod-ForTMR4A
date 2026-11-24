/* ForTMR4A：统一导入入口（C ABI）
 *
 * 一条调用完成：源探测 →（必要时）Converter4BENeXt 转 BEM → be-bem 校验 → be-ir 转 mod.toml/IR。
 * 这是 CRX 的“C 主链路”：编排、临时文件生命周期、错误传播都在 C 里；解析/转换交给叶子。
 */
#ifndef BETTER_ENDFIELD_NEXT_SOURCE_PIPELINE_H
#define BETTER_ENDFIELD_NEXT_SOURCE_PIPELINE_H

#ifdef __cplusplus
extern "C" {
#endif

#define BE_PIPELINE_OK 0
#define BE_PIPELINE_ERR 1
#define BE_PIPELINE_BAD_ARG 2
#define BE_PIPELINE_UNSUPPORTED 3

/* 返回统一入口 ABI 版本。 */
int be_pipeline_abi_version(void);

/* 把任意受支持的来源转换成 ForTMR2 mod 目录。
 * src        : BEM 文件 / EFMI 目录 / EFF3 目录 / 含 mod.toml 的目录
 * catalog_dir: EFMI/EFF 转换需要的 BEN catalog（BEM 源可为 NULL）
 * out_mod_dir: 输出 mod 目录（会被创建；已存在时按文件覆盖）
 * err/err_cap: 错误文本缓冲区（可为 NULL）
 * 返回 BE_PIPELINE_*；mod.toml 来源返回 BE_PIPELINE_UNSUPPORTED（已是原生格式）。 */
int be_source_to_mod(const char *src, const char *catalog_dir,
                     const char *out_mod_dir, char *err, unsigned int err_cap);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_SOURCE_PIPELINE_H */
