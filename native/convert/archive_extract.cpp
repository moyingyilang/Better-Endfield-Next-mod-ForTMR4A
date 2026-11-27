// ForTMR4A: libarchive-backed extraction for the import pipeline (host C++).
//
// libarchive covers ZIP, 7-Zip, RAR4/RAR5, TAR and the tar-family filters. The
// extraction policy lives here: names are sanitized, links are skipped, sizes
// and entry counts are capped, encrypted entries are refused.
//
// fd variants let the Android importer hand over a ContentResolver file
// descriptor, so archives are streamed instead of being copied to a staging
// file first (the biggest avoidable IO cost for RAR/7z packages).
#include "BetterEndfieldNext/SourceArchive.h"

#include <archive.h>
#include <archive_entry.h>

#include <sys/stat.h>
#include <unistd.h>

#include <algorithm>
#include <cstdio>
#include <cstring>
#include <filesystem>
#include <string>
#include <vector>

namespace fs = std::filesystem;

namespace {

constexpr unsigned long long kMaxTotalBytes = 8ull * 1024ull * 1024ull * 1024ull;
constexpr int kMaxEntries = 100000;
constexpr size_t kReadBuffer = 1u << 20;   // 1 MiB per read.
constexpr size_t kBlockSize = 1u << 20;    // libarchive read-ahead window.

void set_error(char* err, unsigned cap, const std::string& message) {
    if (!err || cap == 0) return;
    unsigned n = static_cast<unsigned>(message.size());
    if (n >= cap) n = cap - 1;
    std::memcpy(err, message.data(), n);
    err[n] = 0;
}

std::string archive_error(struct archive* handle) {
    const char* text = archive_error_string(handle);
    std::string message = text ? text : "archive error";
    if (archive_errno(handle) != 0) {
        message += " (errno ";
        message += std::to_string(archive_errno(handle));
        message += ")";
    }
    return message;
}

bool safe_relative_path(const char* raw) {
    if (!raw || raw[0] == 0) return false;
    std::string path(raw);
    for (char& ch : path) {
        if (ch == '\\') ch = '/';
    }
    if (path[0] == '/') return false;
    fs::path parsed(path);
    for (const auto& part : parsed) {
        if (part == "..") return false;
        if (part == ".") continue;
        if (part.empty()) return false;
    }
    return true;
}

struct ExtractState {
    unsigned long long total = 0;
    int entries = 0;
    std::string error;
};

int copy_entry(struct archive* reader, const std::string& target, ExtractState& state,
               char* err, unsigned cap) {
    std::FILE* file = std::fopen(target.c_str(), "wb");
    if (!file) {
        set_error(err, cap, "cannot create " + target);
        return 1;
    }
    std::vector<char> buffer(kReadBuffer);
    for (;;) {
        ssize_t read = archive_read_data(reader, buffer.data(), buffer.size());
        if (read < 0) {
            std::fclose(file);
            set_error(err, cap, archive_error(reader));
            return 1;
        }
        if (read == 0) break;
        state.total += static_cast<unsigned long long>(read);
        if (state.total > kMaxTotalBytes) {
            std::fclose(file);
            set_error(err, cap, "archive expands beyond the size limit");
            return 1;
        }
        if (std::fwrite(buffer.data(), 1, static_cast<size_t>(read), file) !=
            static_cast<size_t>(read)) {
            std::fclose(file);
            set_error(err, cap, "cannot write " + target);
            return 1;
        }
    }
    if (std::fclose(file) != 0) {
        set_error(err, cap, "cannot close " + target);
        return 1;
    }
    return 0;
}

int extract_reader(struct archive* reader, const std::string& out_dir,
                   unsigned long long archive_size, char* err, unsigned cap) {
    std::error_code ec;
    const fs::path root(out_dir);
    if (!fs::create_directories(root, ec) && !fs::is_directory(root)) {
        set_error(err, cap, "cannot create output directory");
        return 1;
    }
    ExtractState state;
    int result = 0;
    struct archive_entry* entry = nullptr;
    for (;;) {
        int status = archive_read_next_header(reader, &entry);
        if (status == ARCHIVE_EOF) break;
        if (status != ARCHIVE_OK && status != ARCHIVE_WARN) {
            state.error = archive_error(reader);
            result = 1;
            break;
        }
        if (++state.entries > kMaxEntries) {
            state.error = "too many archive entries";
            result = 1;
            break;
        }
        const char* name = archive_entry_pathname(entry);
        if (archive_entry_is_encrypted(entry)) {
            state.error = "encrypted archive entries are not supported";
            result = 1;
            break;
        }
        if (!safe_relative_path(name)) {
            state.error = std::string("unsafe archive entry: ") + (name ? name : "");
            result = 1;
            break;
        }
        const fs::path target = root / fs::path(name);
        if (archive_entry_filetype(entry) == AE_IFDIR) {
            fs::create_directories(target, ec);
            archive_read_data_skip(reader);
            continue;
        }
        if (archive_entry_filetype(entry) != AE_IFREG) {
            // Symlinks, devices and hardlinks never belong in a model package.
            archive_read_data_skip(reader);
            continue;
        }
        fs::create_directories(target.parent_path(), ec);
        if (archive_size > 0) {
            const unsigned long long generous =
                std::max(512ull * 1024ull * 1024ull, archive_size * 200ull);
            if (state.total > generous) {
                state.error = "suspicious compression ratio";
                result = 1;
                break;
            }
        }
        if (copy_entry(reader, target, state, err, cap) != 0) {
            result = 1;
            break;
        }
    }
    if (result != 0 && state.error.find_first_not_of(" \t") != std::string::npos) {
        set_error(err, cap, state.error);
    }
    return result;
}

int open_fd_archive(struct archive*& reader, int& owned_fd, int fd) {
    owned_fd = dup(fd);
    if (owned_fd < 0) return -1;
    reader = archive_read_new();
    if (!reader) {
        close(owned_fd);
        owned_fd = -1;
        return -1;
    }
    archive_read_support_filter_all(reader);
    archive_read_support_format_all(reader);
    if (archive_read_open_fd(reader, owned_fd, kBlockSize) != ARCHIVE_OK) {
        return -1;
    }
    return 0;
}

void close_fd_archive(struct archive* reader, int owned_fd) {
    if (reader) {
        archive_read_close(reader);
        archive_read_free(reader);
    }
    if (owned_fd >= 0) close(owned_fd);
}

}  // namespace

extern "C" {

int be_archive_probe(const char* path) {
    if (!path || path[0] == 0) return 0;
    struct archive* reader = archive_read_new();
    if (!reader) return 0;
    archive_read_support_filter_all(reader);
    archive_read_support_format_all(reader);
    struct archive_entry* entry = nullptr;
    int recognized = 0;
    if (archive_read_open_filename(reader, path, kBlockSize) == ARCHIVE_OK) {
        if (archive_read_next_header(reader, &entry) == ARCHIVE_OK && entry != nullptr) {
            recognized = 1;
        }
    }
    archive_read_free(reader);
    return recognized;
}

int be_archive_probe_fd(int fd) {
    if (fd < 0) return 0;
    struct archive* reader = nullptr;
    int owned_fd = -1;
    if (open_fd_archive(reader, owned_fd, fd) != 0) {
        close_fd_archive(reader, owned_fd);
        return 0;
    }
    struct archive_entry* entry = nullptr;
    const int recognized =
        archive_read_next_header(reader, &entry) == ARCHIVE_OK && entry != nullptr;
    close_fd_archive(reader, owned_fd);
    return recognized;
}

int be_archive_extract(const char* path, const char* out_dir,
                       char* err, unsigned int err_cap) {
    if (!path || path[0] == 0 || !out_dir || out_dir[0] == 0) {
        set_error(err, err_cap, "path/out_dir must not be empty");
        return 2;
    }
    struct archive* reader = archive_read_new();
    if (!reader) {
        set_error(err, err_cap, "cannot allocate archive reader");
        return 1;
    }
    archive_read_support_filter_all(reader);
    archive_read_support_format_all(reader);
    if (archive_read_open_filename(reader, path, kBlockSize) != ARCHIVE_OK) {
        set_error(err, err_cap, archive_error(reader));
        archive_read_free(reader);
        return 3;
    }
    unsigned long long archive_size = 0;
    std::error_code sizeError;
    const auto fileSize = fs::file_size(path, sizeError);
    if (!sizeError) archive_size = fileSize;
    const int result = extract_reader(reader, out_dir, archive_size, err, err_cap);
    archive_read_close(reader);
    archive_read_free(reader);
    return result;
}

int be_archive_extract_fd(int fd, const char* out_dir,
                          char* err, unsigned int err_cap) {
    if (fd < 0 || !out_dir || out_dir[0] == 0) {
        set_error(err, err_cap, "fd/out_dir must not be empty");
        return 2;
    }
    struct archive* reader = nullptr;
    int owned_fd = -1;
    if (open_fd_archive(reader, owned_fd, fd) != 0) {
        set_error(err, err_cap, reader ? archive_error(reader) : "cannot read archive");
        close_fd_archive(reader, owned_fd);
        return 3;
    }
    unsigned long long archive_size = 0;
    struct stat info {};
    if (fstat(owned_fd, &info) == 0 && info.st_size > 0) {
        archive_size = static_cast<unsigned long long>(info.st_size);
    }
    const int result = extract_reader(reader, out_dir, archive_size, err, err_cap);
    close_fd_archive(reader, owned_fd);
    return result;
}

}  // extern "C"
