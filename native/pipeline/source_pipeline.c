// ForTMR4A：统一导入入口（CRX 的 C 主链路）
//
// 编排顺序：探测 →（EFMI/EFF/普通 ZIP）进程内转 BEM → BEM→mod.toml/IR → 清理临时文件。
// 所有解析/转换能力来自 C ABI 叶子，本文件只负责控制流、临时文件与错误传播。
#include "BetterEndfieldNext/SourcePipeline.h"

#include <dirent.h>
#include <errno.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <unistd.h>

#include "BetterEndfieldNext/IrConvert.h"
#include "BetterEndfieldNext/SourceConvert.h"

#define BE_TMP_NAME "/.be-source.tmp.bem"
#define BE_UNPACK_NAME "/.be-source-unpacked"

static void set_error(char* err, unsigned cap, const char* message) {
    if (!err || cap == 0) return;
    unsigned n = (unsigned)strlen(message);
    if (n >= cap) n = cap - 1;
    memcpy(err, message, n);
    err[n] = 0;
}

static int append(char* out, unsigned cap, const char* prefix, const char* message) {
    if (!out || cap == 0) return 0;
    int written = snprintf(out, cap, "%s%s", prefix, message);
    return written < 0 ? 0 : 1;
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

static int rm_rf(const char* path) {
    struct stat info;
    if (lstat(path, &info) != 0) return errno == ENOENT ? 0 : -1;
    if (!S_ISDIR(info.st_mode)) return unlink(path);
    DIR* dir = opendir(path);
    if (!dir) return -1;
    int result = 0;
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        char child[4096];
        int written = snprintf(child, sizeof child, "%s/%s", path, entry->d_name);
        if (written < 0 || written >= (int)sizeof child || rm_rf(child) != 0) {
            result = -1;
        }
    }
    closedir(dir);
    if (result == 0 && rmdir(path) != 0) result = -1;
    return result;
}

/* ZIP 常见“单一顶层目录”布局：解包后下探一层。 */
static int single_child_dir(const char* root, char* out, size_t out_size) {
    DIR* dir = opendir(root);
    if (!dir) return 0;
    int children = 0;
    int only_dir = 0;
    char name[512] = {0};
    struct dirent* entry;
    while ((entry = readdir(dir)) != NULL) {
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0) {
            continue;
        }
        children++;
        char path[4096];
        snprintf(path, sizeof path, "%s/%s", root, entry->d_name);
        struct stat info;
        int is_dir = lstat(path, &info) == 0 && S_ISDIR(info.st_mode);
        if (is_dir) only_dir = 1;
        snprintf(name, sizeof name, "%s", entry->d_name);
    }
    closedir(dir);
    if (children != 1 || !only_dir) return 0;
    int written = snprintf(out, out_size, "%s/%s", root, name);
    return written > 0 && written < (int)out_size;
}

static int has_bem_magic(const char* path) {
    static const unsigned char magic[8] = {'B', 'E', 'M', 0, 'P', 'K', 'G', 0};
    FILE* file = fopen(path, "rb");
    if (!file) return 0;
    unsigned char buffer[8] = {0};
    size_t read = fread(buffer, 1, sizeof buffer, file);
    fclose(file);
    return read == sizeof buffer && memcmp(buffer, magic, sizeof magic) == 0;
}

static int find_bem_file(const char* dir, char* out, size_t out_size) {
    DIR* handle = opendir(dir);
    if (!handle) return 0;
    struct dirent* entry;
    int found = 0;
    while ((entry = readdir(handle)) != NULL) {
        if (entry->d_name[0] == '.') continue;
        char path[4096];
        int written = snprintf(path, sizeof path, "%s/%s", dir, entry->d_name);
        if (written < 0 || written >= (int)sizeof path) continue;
        if (!has_bem_magic(path)) continue;
        int copied = snprintf(out, out_size, "%s", path);
        if (copied > 0 && copied < (int)out_size) {
            found = 1;
            break;
        }
    }
    closedir(handle);
    return found;
}

int be_pipeline_abi_version(void) { return 1; }

static int convert_bem(const char* bem_path, const char* catalog_dir,
                       const char* out_mod_dir, char* err, unsigned cap) {
    char detail[512] = {0};
    int rc = catalog_dir && *catalog_dir
                 ? be_ir_convert_bem_with_catalog(bem_path, catalog_dir, out_mod_dir,
                                                  detail, (unsigned)sizeof detail)
                 : be_ir_convert_bem(bem_path, out_mod_dir, detail, (unsigned)sizeof detail);
    if (rc != BE_IR_OK) {
        append(err, cap, "BEM→IR 失败：", detail);
        return BE_PIPELINE_ERR;
    }
    return BE_PIPELINE_OK;
}

/* EFMI/EFF 目录：先转 BEM，再走 IR。 */
static int convert_dir(const char* dir, const char* catalog_dir, const char* out_mod_dir,
                       char* err, unsigned cap) {
    const int kind = be_source_probe(dir);
    if (kind != BE_SRC_EFMI && kind != BE_SRC_EFF3) {
        set_error(err, cap, "目录既不是 EFMI 也不是 EFF3");
        return BE_PIPELINE_UNSUPPORTED;
    }
    if (!catalog_dir || !*catalog_dir) {
        set_error(err, cap, "EFMI/EFF 来源需要 catalog_dir");
        return BE_PIPELINE_BAD_ARG;
    }
    char tmp[4096];
    if (snprintf(tmp, sizeof tmp, "%s%s", out_mod_dir, BE_TMP_NAME) >= (int)sizeof tmp) {
        set_error(err, cap, "临时路径过长");
        return BE_PIPELINE_ERR;
    }
    unlink(tmp);
    char detail[512] = {0};
    int converted = be_source_convert_to_bem(dir, catalog_dir, tmp, detail, (unsigned)sizeof detail);
    if (converted != 0) {
        unlink(tmp);
        append(err, cap, "旧格式→BEM 失败：", detail);
        return converted == BE_PIPELINE_BAD_ARG ? BE_PIPELINE_BAD_ARG : BE_PIPELINE_ERR;
    }
    int result = convert_bem(tmp, catalog_dir, out_mod_dir, err, cap);
    unlink(tmp);
    return result;
}

/* ZIP：先当 BEM 原生容器试；失败则解包成目录，按里面的实际格式走。 */
static int import_zip(const char* src, const char* catalog_dir, const char* out_mod_dir,
                      char* err, unsigned cap) {
    char detail[512] = {0};
    int zrc = catalog_dir && *catalog_dir
                  ? be_ir_convert_bem_zip_with_catalog(src, catalog_dir, out_mod_dir,
                                                       detail, (unsigned)sizeof detail)
                  : be_ir_convert_bem_zip(src, out_mod_dir, detail, (unsigned)sizeof detail);
    if (zrc == BE_IR_OK) return BE_PIPELINE_OK;

    char unpacked[4096];
    if (snprintf(unpacked, sizeof unpacked, "%s%s", out_mod_dir, BE_UNPACK_NAME) >=
        (int)sizeof unpacked) {
        set_error(err, cap, "临时路径过长");
        return BE_PIPELINE_ERR;
    }
    rm_rf(unpacked);
    if (mkdir_p(unpacked) != 0) {
        set_error(err, cap, "创建解包目录失败");
        return BE_PIPELINE_ERR;
    }
    char zerr[512] = {0};
    if (be_zip_extract(src, unpacked, zerr, (unsigned)sizeof zerr) != BE_IR_OK) {
        append(err, cap, "ZIP 解包失败：", zerr);
        rm_rf(unpacked);
        return BE_PIPELINE_ERR;
    }

    char child[4096];
    const char* root = single_child_dir(unpacked, child, sizeof child) ? child : unpacked;
    int result;
    const int inner = be_source_probe(root);
    if (inner == BE_SRC_EFMI || inner == BE_SRC_EFF3) {
        result = convert_dir(root, catalog_dir, out_mod_dir, err, cap);
    } else {
        char bem_path[4096];
        if (find_bem_file(root, bem_path, sizeof bem_path)) {
            result = convert_bem(bem_path, catalog_dir, out_mod_dir, err, cap);
        } else {
            append(err, cap, "ZIP 内容既不是 BEM 容器，也不是 EFMI/EFF：", detail);
            result = BE_PIPELINE_UNSUPPORTED;
        }
    }
    rm_rf(unpacked);
    return result;
}

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

    if (kind == BE_SRC_BEM) {
        return convert_bem(src, catalog_dir, out_mod_dir, err, err_cap);
    }
    if (kind == BE_SRC_ZIP) {
        return import_zip(src, catalog_dir, out_mod_dir, err, err_cap);
    }
    return convert_dir(src, catalog_dir, out_mod_dir, err, err_cap);
}
