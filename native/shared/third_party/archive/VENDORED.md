# Vendored archive readers

The import pipeline accepts ZIP, 7-Zip, RAR4/RAR5, TAR and the tar-family filters
(gzip/bzip2/xz/zstd). Extraction is implemented in `native/convert/archive_extract.cpp`
against libarchive; the libraries below are linked into `libbetterendfieldnext_installer.so`.

| Library | Version | License | Notes |
|---|---|---|---|
| libarchive | 3.7.7 | BSD-2-Clause | ZIP / 7-Zip / RAR4 / RAR5 / TAR readers |
| bzip2 | 1.0.8 | BSD-like (bzip2 license) | `libbz2` decoder/encoder |
| xz (liblzma) | 5.6.3 | 0BSD / LGPL-2.1+ (liblzma is public-domain-ish) | `.xz` / `.lzma` filters |
| zstd | repo copy (`native/shared/third_party/zstd`) | BSD-3-Clause / GPLv2 dual | `.zst` filter |

Changes against upstream: only trim of documentation/tests/autotools files that the
CMake build does not use, plus the small CMake glue in this directory that builds
the filters as static libraries. No source logic was modified.

RAR and 7-Zip support is **read-only** via libarchive; the app never creates those
containers.
