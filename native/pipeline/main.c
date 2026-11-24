/* be-source：统一导入入口的宿主 CLI。
 *
 * 用法：be-source <src> <catalog|-> <out-mod-dir>
 *   src     : BEM 文件 / EFMI 目录 / EFF3 目录 / 含 mod.toml 的目录
 *   catalog : EFMI/EFF 需要的 BEN catalog 目录；BEM 源写 "-"
 */
#include <stdio.h>
#include <string.h>

#include "BetterEndfieldNext/SourcePipeline.h"

int main(int argc, char** argv) {
    if (argc != 4) {
        fprintf(stderr, "用法: %s <src> <catalog|-> <out-mod-dir>\n", argv[0]);
        return 2;
    }
    const char* catalog = strcmp(argv[2], "-") == 0 ? NULL : argv[2];
    char err[512] = {0};
    int rc = be_source_to_mod(argv[1], catalog, argv[3], err, (unsigned)sizeof err);
    if (rc == BE_PIPELINE_OK) {
        printf("OK: %s -> %s\n", argv[1], argv[3]);
    } else {
        fprintf(stderr, "FAIL(%d): %s\n", rc, err[0] ? err : "(no detail)");
    }
    return rc == BE_PIPELINE_OK ? 0 : 1;
}
