// ForTMR4A：多格式源接入的 C++ 宿主适配（CRX：宿主层，C ABI 出边界）。
//
// - BEM 是原生格式，不淘汰（历史包与 Converter4BENeXt 的输出都走同一条读取链）；
// - EFMI / EFF Format 3 通过 vendored Converter4BENeXt 在进程内转换到 BEM；
// - ForTMR2 的 mod.toml 是另一条原生格式，交给 native/mod 的 efml-core 链。
#include "BetterEndfieldNext/SourceConvert.h"

#include <cstdio>
#include <cstring>
#include <filesystem>
#include <string>
#include <vector>

#include "c4ben.h"

namespace fs = std::filesystem;

namespace {

constexpr unsigned char kMagicBem[8] = {'B', 'E', 'M', 0, 'P', 'K', 'G', 0};

void set_error(char* err, unsigned cap, const std::string& message) {
    if (!err || cap == 0) return;
    unsigned n = (unsigned)message.size();
    if (n >= cap) n = cap - 1;
    std::memcpy(err, message.data(), n);
    err[n] = 0;
}

bool file_starts_with(const std::string& path, const void* magic, size_t size) {
    std::FILE* file = std::fopen(path.c_str(), "rb");
    if (!file) return false;
    unsigned char buffer[8] = {0};
    const size_t read = std::fread(buffer, 1, size, file);
    std::fclose(file);
    return read == size && std::memcmp(buffer, magic, size) == 0;
}

bool is_zip(const std::string& path) {
    const unsigned char pk[4] = {'P', 'K', 3, 4};
    return file_starts_with(path, pk, sizeof pk);
}

int c4ben_kind_to_source(int kind) {
    switch (kind) {
        case 1:
            return BE_SRC_EFMI;
        case 2:
            return BE_SRC_EFF3;
        default:
            return BE_SRC_UNKNOWN;
    }
}

}  // namespace

extern "C" {

int be_source_abi_version(void) { return 1; }

int be_source_probe(const char* path) {
    if (!path || !*path) return BE_SRC_UNKNOWN;
    std::error_code ec;
    const std::string target(path);
    if (fs::is_regular_file(target, ec)) {
        if (file_starts_with(target, kMagicBem, sizeof kMagicBem)) return BE_SRC_BEM;
        if (is_zip(target)) return BE_SRC_ZIP;
        return BE_SRC_UNKNOWN;
    }
    if (!fs::is_directory(target, ec)) return BE_SRC_UNKNOWN;
    if (fs::is_regular_file(target + "/mod.toml", ec)) return BE_SRC_MOD_TOML;
    return c4ben_kind_to_source(c4ben_probe(path));
}

int be_source_convert_to_bem(const char* path, const char* catalog_dir,
                             const char* out_path, char* err, unsigned int err_cap) {
    if (!path || !*path) {
        set_error(err, err_cap, "path is null");
        return 2;
    }
    const int kind = be_source_probe(path);
    switch (kind) {
        case BE_SRC_BEM: {
            if (!out_path || !*out_path) return 0;
            std::error_code ec;
            if (std::string(path) == std::string(out_path)) return 0;
            fs::copy_file(path, out_path, fs::copy_options::overwrite_existing, ec);
            if (ec) {
                set_error(err, err_cap, "copy BEM failed: " + ec.message());
                return 1;
            }
            return 0;
        }
        case BE_SRC_MOD_TOML:
            set_error(err, err_cap, "mod.toml is handled by the IR chain, not BEM conversion");
            return 3;
        case BE_SRC_EFMI:
        case BE_SRC_EFF3: {
            if (!catalog_dir || !*catalog_dir || !out_path || !*out_path) {
                set_error(err, err_cap, "EFMI/EFF conversion needs catalog_dir and out_path");
                return 2;
            }
            return c4ben_convert(path, catalog_dir, out_path, err, err_cap);
        }
        default:
            set_error(err, err_cap, "unsupported source (not BEM/EFMI/EFF3/mod.toml)");
            return 3;
    }
}

}  // extern "C"
