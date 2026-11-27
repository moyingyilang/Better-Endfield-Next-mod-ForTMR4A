// Host harness for RewriteBemTextures' keep/parallel bookkeeping.
//
//   rewrite_parallel_harness <input.bem> <output.bem>
//
// Keeps BC1/BC3/R8/RGBA32 textures untouched, forces every other texture through
// the conversion callback, and changes a manifest field so the "re-encode and
// append payload" branch is exercised. Run twice: byte-identical outputs prove
// the parallel path keeps a deterministic payload order.
#include "bem_rewrite.h"

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

using namespace BetterEndfieldNext::CustomModel;

int main(int argc, char** argv) {
    if (argc != 3) {
        std::fprintf(stderr, "usage: %s <input.bem> <output.bem>\n", argv[0]);
        return 2;
    }
    std::remove(argv[2]);
    unsigned kept = 0, converted = 0;
    auto keep = [&](const BemJson& texture) {
        const int format = texture.value("format", 0);
        const bool result = format == 4 || format == 63 || format == 10 || format == 12;
        if (result) ++kept;
        return result;
    };
    auto transform = [&](const BemJson& manifest, BemJson& texture, std::vector<uint8_t>& bytes) {
        (void)manifest;
        ++converted;
        // Same size, same format: only a marker so the changed branch is taken.
        texture["original_name"] = texture.value("original_name", std::string{}) + "#p";
        if (!bytes.empty()) bytes[0] = static_cast<uint8_t>(bytes[0] ^ 0x5a);
    };
    std::string report, error;
    const bool ok = RewriteBemTextures(argv[1], argv[2], transform, [] {}, report, error, keep);
    if (!ok) {
        std::fprintf(stderr, "rewrite failed: %s\n", error.c_str());
        return 1;
    }
    std::printf("kept=%u converted=%u report=%zu bytes\n", kept, converted, report.size());
    return 0;
}
