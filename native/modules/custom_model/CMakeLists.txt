# ForTMR4A: Android-only. 原 Windows 侧 BemValidate/Bem*Tests/BemCompose 源码已归档到
# legacy/windows/native/；这里只保留双端共用的解码/组合库与 Android 目标需要的部分。
# Platform-neutral package decoding and resource selection, shared by PC/Android.
# Engine calls, process memory access and native layout discovery stay in adapters.
file(GLOB BEM_ZSTD_COMMON CONFIGURE_DEPENDS ../../shared/third_party/zstd/lib/common/*.c)
file(GLOB BEM_ZSTD_DECODE CONFIGURE_DEPENDS ../../shared/third_party/zstd/lib/decompress/*.c)
file(GLOB BEM_ZSTD_COMPRESS CONFIGURE_DEPENDS ../../shared/third_party/zstd/lib/compress/*.c)
add_library(BetterEndfieldNext.BemZstd STATIC ${BEM_ZSTD_COMMON} ${BEM_ZSTD_DECODE} ${BEM_ZSTD_COMPRESS})
target_compile_definitions(BetterEndfieldNext.BemZstd PRIVATE ZSTD_DISABLE_ASM)
set_target_properties(BetterEndfieldNext.BemZstd PROPERTIES POSITION_INDEPENDENT_CODE ON)

# ZIP container support (format specification section 14). miniz is vendored as the
# amalgamated upstream single-file pair; the directory also holds three unresolvable
# corrupt leftovers from a filesystem fault, so this is an explicit source list.
add_library(BetterEndfieldNext.BemMiniz STATIC ../../shared/third_party/miniz/miniz.c)
target_include_directories(BetterEndfieldNext.BemMiniz PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}/../../shared/third_party/miniz")
if(NOT MSVC)
    target_compile_options(BetterEndfieldNext.BemMiniz PRIVATE -w)
endif()
set_target_properties(BetterEndfieldNext.BemMiniz PROPERTIES POSITION_INDEPENDENT_CODE ON)

add_library(BetterEndfieldNext.CustomModelCore STATIC bem.cpp bem_preview.cpp bem_compose.cpp bem_compose_project.cpp mod_registry.cpp)
target_compile_features(BetterEndfieldNext.CustomModelCore PUBLIC cxx_std_20)
target_include_directories(BetterEndfieldNext.CustomModelCore PUBLIC "${CMAKE_CURRENT_SOURCE_DIR}")
target_link_libraries(BetterEndfieldNext.CustomModelCore PUBLIC BetterEndfieldNext.BemZstd BetterEndfieldNext.BemMiniz)
set_target_properties(BetterEndfieldNext.CustomModelCore PROPERTIES POSITION_INDEPENDENT_CODE ON)

if(WIN32)
    set(ASTCENC_CLI OFF CACHE BOOL "" FORCE)
    set(ASTCENC_ISA_SSE2 ON CACHE BOOL "" FORCE)
    set(ASTCENC_ISA_NATIVE OFF CACHE BOOL "" FORCE)
    set(ASTCENC_UNITTEST OFF CACHE BOOL "" FORCE)
    add_subdirectory(../../shared/third_party/astcenc "${CMAKE_CURRENT_BINARY_DIR}/preview-astc")
    target_link_libraries(BetterEndfieldNext.CustomModelCore PUBLIC astcenc-sse2-static)
    add_executable(BetterEndfieldNext.BemCompose ${CMAKE_CURRENT_SOURCE_DIR}/../../../legacy/windows/native/tools/bem_compose_main.cpp)
    target_link_libraries(BetterEndfieldNext.BemCompose PRIVATE BetterEndfieldNext.CustomModelCore)
    target_compile_options(BetterEndfieldNext.BemCompose PRIVATE $<$<CXX_COMPILER_ID:MSVC>:/utf-8>)
    set_target_properties(BetterEndfieldNext.BemCompose PROPERTIES
        RUNTIME_OUTPUT_DIRECTORY "${BETTER_ENDFIELD_NEXT_STAGE_DIR}/tools/BemCompose")
endif()
