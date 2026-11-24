// ForTMR4A：统一导入入口（CRX 的 C 主链路）
//
// 编排顺序：探测 →（EFMI/EFF）进程内转 BEM → BEM→mod.toml/IR → 清理临时文件。
// 所有解析/转换能力来自 C ABI 叶子，本文件只负责控制流、临时文件与错误传播。
#include "BetterEndfieldNext/SourcePipeline.h"

#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include "BetterEndfieldNext/IrConvert.h"
#include "BetterEndfieldNext/SourceConvert.h"

#define BE_TMP_NAME "/.be-source.tmp.bem"

static void set_error(char* err, unsigned cap, const char* message) {
    if (!err || cap == 0) return;
    unsigned n = (unsigned)strlen(message);
    if (n >= cap) n = cap - 1;
    memcpy(err, message, n);
    err[n] = 0;
}

/* mkdir -p，尽量小：只处理绝对/相对的普通路径，不处理 “..” 语义特例。 */
static int mkdir_p(const char* path) {
    char buffer[4096];
    size_t len = strlen(path);
    if (len == 0 || len >= sizeof buffer) return -1;
    memcpy(buffer, path, len + 1);
    for (size_t i = 1; i < len; i++) {
        if (buffer[i] != '/') continue;
        buffer[i] = 0;
        if (mkdir(buffer, 0777) != 0 && errno != EEXIST) return -1;
        buffer[i] = '/';
    }
    if (mkdir(buffer, 0777) != 0 && errno != EEXIST) return -1;
    return 0;
}

static int append(char* out, unsigned cap, const char* prefix, const char* message) {
    if (!out || cap == 0) return 0;
    int written = snprintf(out, cap, "%s%s", prefix, message);
    return written < 0 ? 0 : 1;
}

int be_pipeline_abi_version(void) { return 1; }

int be_source_to_mod(const char* src, const char* catalog_dir,
                     const char* out_mod_dir, char* err, unsigned int err_cap) {
    if (!src || !*src || !out_mod_dir || !*out_mod_dir) {
        set_error(err, err_cap, "src/out_mod_dir 不能为空");
        return BE_PIPELINE_BAD_ARG;
    }

    const int kind = be_source_probe(src);
    if (kind == BE_SRC_MOD_TOML) {
        set_error(err, err_cap, "来源已是 mod.toml（ForTMR2 原生格式），无需转换");
        return BE_PIPELINE_UNSUPPORTED;
    }
    if (kind == BE_SRC_UNKNOWN) {
        set_error(err, err_cap, "无法识别的来源：既不是 BEM/ZIP，也不是 EFMI/EFF3/mod.toml");
        return BE_PIPELINE_UNSUPPORTED;
    }

    if (mkdir_p(out_mod_dir) != 0) {
        append(err, err_cap, "创建输出目录失败：", out_mod_dir);
        return BE_PIPELINE_ERR;
    }

    /* BEM 原生 ZIP 容器：be-ir 内部解包成 BEM 再进 IR 链。 */
    if (kind == BE_SRC_ZIP) {
        char detail[512] = {0};
        int zip_rc = be_ir_convert_bem_zip(src, out_mod_dir, detail, (unsigned)sizeof detail);
        if (zip_rc != BE_IR_OK) {
            append(err, err_cap,
                   "ZIP 不是 BEM 原生容器（manifest.json + payloads/NNNN.bin）：", detail);
            return BE_PIPELINE_ERR;
        }
        return BE_PIPELINE_OK;
    }

    /* BEM 直接进 IR 链。 */
    if (kind == BE_SRC_BEM) {
        char detail[512] = {0};
        int rc = be_ir_convert_bem(src, out_mod_dir, detail, (unsigned)sizeof detail);
        if (rc != BE_IR_OK) {
            append(err, err_cap, "BEM→IR 失败：", detail);
            return BE_PIPELINE_ERR;
        }
        return BE_PIPELINE_OK;
    }

    /* EFMI / EFF3：先转 BEM（进程内），再走同一条 IR 链。 */
    if (!catalog_dir || !*catalog_dir) {
        set_error(err, err_cap, "EFMI/EFF 来源需要 catalog_dir");
        return BE_PIPELINE_BAD_ARG;
    }
    char tmp[4096];
    if (snprintf(tmp, sizeof tmp, "%s%s", out_mod_dir, BE_TMP_NAME) >= (int)sizeof tmp) {
        set_error(err, err_cap, "临时路径过长");
        return BE_PIPELINE_ERR;
    }
    unlink(tmp);

    char detail[512] = {0};
    int converted = be_source_convert_to_bem(src, catalog_dir, tmp, detail, (unsigned)sizeof detail);
    if (converted != 0) {
        unlink(tmp);
        append(err, err_cap, "旧格式→BEM 失败：", detail);
        return converted == BE_PIPELINE_BAD_ARG ? BE_PIPELINE_BAD_ARG : BE_PIPELINE_ERR;
    }

    memset(detail, 0, sizeof detail);
    int rc = be_ir_convert_bem(tmp, out_mod_dir, detail, (unsigned)sizeof detail);
    unlink(tmp);
    if (rc != BE_IR_OK) {
        append(err, err_cap, "BEM→IR 失败：", detail);
        return BE_PIPELINE_ERR;
    }
    return BE_PIPELINE_OK;
}
