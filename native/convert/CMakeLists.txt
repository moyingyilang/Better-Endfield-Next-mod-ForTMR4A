# ForTMR4A：多格式源接入（Converter4BENeXt vendored + C ABI）
#
# 满足两条约束：
#   1. BEM 不淘汰 —— BEM 是原生格式，直接走 BEM 读取/校验链；
#   2. EFMI / EFF Format 3 在进程内实时转换 —— 不再依赖外部 CLI。
# 默认关闭，打开后链接进 betterendfieldnext_android。
cmake_minimum_required(VERSION 3.22.1)

get_filename_component(BETTER_ENDFIELD_NEXT_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)
set(BE_VENDOR_C4BEN "${BETTER_ENDFIELD_NEXT_ROOT}/third_party/Converter4BENeXt")

find_package(ZLIB REQUIRED)

add_library(c4ben_vendor STATIC
    "${BE_VENDOR_C4BEN}/src/main.cpp"
    "${BE_VENDOR_C4BEN}/src/third_party/cJSON.c")
target_compile_definitions(c4ben_vendor PRIVATE C4BEN_NO_MAIN)
target_include_directories(c4ben_vendor PRIVATE
    "${BE_VENDOR_C4BEN}/src"
    "${BE_VENDOR_C4BEN}/src/third_party")
target_compile_features(c4ben_vendor PRIVATE cxx_std_17)
target_compile_options(c4ben_vendor PRIVATE -fexceptions)
target_link_libraries(c4ben_vendor PUBLIC ZLIB::ZLIB)

add_library(be_convert STATIC source_convert.cpp)
target_include_directories(be_convert PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}"
    "${BE_VENDOR_C4BEN}/src"
    "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
target_compile_features(be_convert PRIVATE cxx_std_17)
target_link_libraries(be_convert PRIVATE c4ben_vendor)

add_executable(be-convert-probe EXCLUDE_FROM_ALL tests/probe_harness.c)
target_include_directories(be-convert-probe PRIVATE
    "${CMAKE_CURRENT_SOURCE_DIR}"
    "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
target_link_libraries(be-convert-probe PRIVATE be_convert)
