# Vendored archive readers for the import pipeline (CRX: C libraries, no shell-out).
#
# libarchive provides ZIP / 7-Zip / RAR4 / RAR5 / TAR and the tar-family filters;
# bzip2, liblzma and zstd are built from the in-tree copies so the APK works on
# a stock NDK without extra host packages.
cmake_minimum_required(VERSION 3.22.1)

get_filename_component(TMR4A_ARCHIVE_ROOT "${CMAKE_CURRENT_SOURCE_DIR}" ABSOLUTE)
get_filename_component(TMR4A_REPO_ROOT "${TMR4A_ARCHIVE_ROOT}/../../../.." ABSOLUTE)

set(BUILD_SHARED_LIBS OFF CACHE BOOL "" FORCE)
set(CMAKE_POSITION_INDEPENDENT_CODE ON)

# bzip2 (BSD-like).
set(TMR4A_BZIP2_DIR "${TMR4A_ARCHIVE_ROOT}/bzip2-1.0.8")
add_library(bz2_static STATIC
    "${TMR4A_BZIP2_DIR}/blocksort.c"
    "${TMR4A_BZIP2_DIR}/huffman.c"
    "${TMR4A_BZIP2_DIR}/crctable.c"
    "${TMR4A_BZIP2_DIR}/randtable.c"
    "${TMR4A_BZIP2_DIR}/compress.c"
    "${TMR4A_BZIP2_DIR}/decompress.c"
    "${TMR4A_BZIP2_DIR}/bzlib.c")
target_include_directories(bz2_static PUBLIC "${TMR4A_BZIP2_DIR}")
target_compile_options(bz2_static PRIVATE -w)
set_target_properties(bz2_static PROPERTIES POSITION_INDEPENDENT_CODE ON)

# liblzma (xz; the decoder side is what libarchive needs).
set(TMR4A_LZMA_DIR "${TMR4A_ARCHIVE_ROOT}/xz-5.6.3")
add_subdirectory("${TMR4A_LZMA_DIR}" "${CMAKE_CURRENT_BINARY_DIR}/xz" EXCLUDE_FROM_ALL)
add_library(lzma_static INTERFACE)
target_link_libraries(lzma_static INTERFACE liblzma)

# zstd (already vendored at the repo root for BEM).
set(TMR4A_ZSTD_DIR "${TMR4A_REPO_ROOT}/native/shared/third_party/zstd")
file(GLOB TMR4A_ZSTD_SOURCES CONFIGURE_DEPENDS
    "${TMR4A_ZSTD_DIR}/lib/common/*.c"
    "${TMR4A_ZSTD_DIR}/lib/compress/*.c"
    "${TMR4A_ZSTD_DIR}/lib/decompress/*.c")
add_library(zstd_archive STATIC ${TMR4A_ZSTD_SOURCES})
target_include_directories(zstd_archive PUBLIC "${TMR4A_ZSTD_DIR}/lib")
target_compile_definitions(zstd_archive PRIVATE ZSTD_DISABLE_ASM)
target_compile_options(zstd_archive PRIVATE -w)
set_target_properties(zstd_archive PROPERTIES POSITION_INDEPENDENT_CODE ON)

# libarchive itself (BSD-2-Clause). Only the library is built.
set(TMR4A_LIBARCHIVE_DIR "${TMR4A_ARCHIVE_ROOT}/libarchive-3.7.7")
set(ENABLE_TEST OFF CACHE BOOL "" FORCE)
set(ENABLE_TAR OFF CACHE BOOL "" FORCE)
set(ENABLE_CPIO OFF CACHE BOOL "" FORCE)
set(ENABLE_CAT OFF CACHE BOOL "" FORCE)
set(ENABLE_UNZIP OFF CACHE BOOL "" FORCE)
set(ENABLE_OPENSSL OFF CACHE BOOL "" FORCE)
set(ENABLE_MBEDTLS OFF CACHE BOOL "" FORCE)
set(ENABLE_NETTLE OFF CACHE BOOL "" FORCE)
set(ENABLE_LIBXML2 OFF CACHE BOOL "" FORCE)
set(ENABLE_EXPAT OFF CACHE BOOL "" FORCE)
set(ENABLE_LZ4 OFF CACHE BOOL "" FORCE)
set(ENABLE_LZO OFF CACHE BOOL "" FORCE)
set(ENABLE_LIBB2 OFF CACHE BOOL "" FORCE)
set(ENABLE_ZLIB ON CACHE BOOL "" FORCE)
set(ENABLE_BZip2 OFF CACHE BOOL "" FORCE)
set(ENABLE_LZMA OFF CACHE BOOL "" FORCE)
set(ENABLE_ZSTD OFF CACHE BOOL "" FORCE)
set(ENABLE_ACL OFF CACHE BOOL "" FORCE)
set(ENABLE_XATTR OFF CACHE BOOL "" FORCE)
set(ENABLE_ICONV OFF CACHE BOOL "" FORCE)
add_subdirectory("${TMR4A_LIBARCHIVE_DIR}" "${CMAKE_CURRENT_BINARY_DIR}/libarchive" EXCLUDE_FROM_ALL)
set_target_properties(archive_static PROPERTIES POSITION_INDEPENDENT_CODE ON)

set(TMR4A_LIBARCHIVE_TARGET archive_static)
target_include_directories(${TMR4A_LIBARCHIVE_TARGET} PUBLIC
    "${TMR4A_LIBARCHIVE_DIR}/libarchive")
