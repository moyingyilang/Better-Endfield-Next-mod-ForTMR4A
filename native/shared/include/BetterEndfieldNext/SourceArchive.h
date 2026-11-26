/* ForTMR4A: archive extraction (C ABI, libarchive-backed).
 *
 * The import path accepts any archive libarchive can read: ZIP, 7-Zip, RAR4,
 * RAR5, TAR and tar-family filters (gzip/bzip2/xz/zstd/lz4...). Extraction is
 * deliberately conservative: no absolute paths, no `..`, no symlinks, size and
 * entry limits, no encrypted entries.
 */
#ifndef BETTER_ENDFIELD_NEXT_SOURCE_ARCHIVE_H
#define BETTER_ENDFIELD_NEXT_SOURCE_ARCHIVE_H

#ifdef __cplusplus
extern "C" {
#endif

/* 1 when libarchive can open the file and read its first entry, 0 otherwise. */
int be_archive_probe(const char *path);
int be_archive_probe_fd(int fd);

/* Extract into out_dir. Returns 0 on success, 1 on error, 2 on bad arguments,
 * 3 when the archive is not supported. Error text goes to err. */
int be_archive_extract(const char *path, const char *out_dir,
                       char *err, unsigned int err_cap);
int be_archive_extract_fd(int fd, const char *out_dir,
                          char *err, unsigned int err_cap);

#ifdef __cplusplus
}
#endif

#endif /* BETTER_ENDFIELD_NEXT_SOURCE_ARCHIVE_H */
