/* ForTMR4A：多格式源接入的宿主自检 harness。
 *
 * 用法：be-convert-probe <path...>
 * 打印每个路径的探测结果；可选（第二个参数为 --convert）执行转换：
 *   be-convert-probe convert <src> <catalog> <out.bem>
 */
#include <stdio.h>
#include <string.h>

#include "BetterEndfieldNext/SourceConvert.h"

static const char* kind_name(int kind) {
    switch (kind) {
        case BE_SRC_BEM: return "BEM";
        case BE_SRC_MOD_TOML: return "mod.toml";
        case BE_SRC_EFMI: return "EFMI";
        case BE_SRC_EFF3: return "EFF3";
        case BE_SRC_ZIP: return "ZIP";
        default: return "UNKNOWN";
    }
}

int main(int argc, char** argv) {
    if (argc < 2) {
        fprintf(stderr, "用法: %s <path...>\n       %s convert <src> <catalog> <out.bem>\n", argv[0], argv[0]);
        return 2;
    }
    if (strcmp(argv[1], "convert") == 0) {
        if (argc < 5) {
            fprintf(stderr, "用法: %s convert <src> <catalog> <out.bem>\n", argv[0]);
            return 2;
        }
        char err[512] = {0};
        int rc = be_source_convert_to_bem(argv[2], argv[3], argv[4], err, (unsigned)sizeof err);
        printf("convert rc=%d err='%s'\n", rc, err);
        return rc == 0 ? 0 : 1;
    }
    for (int i = 1; i < argc; i++) {
        printf("%-12s %s\n", kind_name(be_source_probe(argv[i])), argv[i]);
    }
    return 0;
}
