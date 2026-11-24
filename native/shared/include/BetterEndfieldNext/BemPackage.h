/* ForTMR4A：BEM 包读取/校验的 C ABI（CRX 叶子）
 *
 * 实现：native/bem（Rust，`be-bem`）。规范见 docs/custom_model/BEM_FORMAT_SPEC.md。
 * 只读叶子：不碰游戏运行时，不跨 FFI 暴露 Rust 结构体。
 */
#ifndef BETTER_ENDFIELD_NEXT_BEM_PACKAGE_H
#define BETTER_ENDFIELD_NEXT_BEM_PACKAGE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define BE_BEM_ABI_VERSION 1u

#define BE_BEM_OK 0
#define BE_BEM_INVALID 1
#define BE_BEM_BAD_ARG 2
#define BE_BEM_INCOMPATIBLE 3

/* 返回 ABI 版本。 */
uint32_t be_bem_abi_version(void);

/* 校验 BEM 文件：容器 + manifest 结构 + payload 引用。
 * 返回 0 或错误码；错误文本写入 err（可为 NULL），始终 NUL 结尾。 */
int32_t be_bem_validate(const char *path, char *err, uint32_t err_cap);

/* 生成 inspect JSON。out_cap 不足时截断并返回所需字节数（不含 NUL）；
 * 出错时返回负的错误码。 */
int64_t be_bem_info_json(const char *path, char *out, uint32_t out_cap);

/* 读入 input.bem 并以 raw payload 重写到 output.bem（读→改→写最小入口）。 */
int32_t be_bem_repack(const char *input, const char *output, char *err, uint32_t err_cap);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_BEM_PACKAGE_H */
