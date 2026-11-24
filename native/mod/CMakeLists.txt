# ForTMR4A：Rust mod-format bridge（可选）
#
# 把 ForTMR2 的 efml-core（Rust 叶子）编成 Android 静态库，供 JNI/宿主通过
# BetterEndfieldNext/ModPackage.h 调用。默认不参与构建（BETTER_ENDFIELD_NEXT_MOD_RUST=OFF），
# 因为需要 cargo + Android target 的 NDK 工具链配置。
#
# 开关：在 android/app/src/main/cpp/CMakeLists.txt 传
#   -DBETTER_ENDFIELD_NEXT_MOD_RUST=ON
cmake_minimum_required(VERSION 3.22.1)

get_filename_component(BETTER_ENDFIELD_NEXT_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

find_program(BE_CARGO_EXECUTABLE cargo)
if(NOT BE_CARGO_EXECUTABLE)
    message(FATAL_ERROR "cargo not found; install Rust or disable BETTER_ENDFIELD_NEXT_MOD_RUST")
endif()

set(BE_MOD_RUST_TARGET "aarch64-linux-android" CACHE STRING "Rust target triple")
set(BE_MOD_CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
set(BE_MOD_ARTIFACT
    "${BE_MOD_CRATE_DIR}/target/${BE_MOD_RUST_TARGET}/release/libbe_mod.a")

file(GLOB_RECURSE BE_MOD_RUST_SOURCES CONFIGURE_DEPENDS
    "${BE_MOD_CRATE_DIR}/src/*.rs")
list(APPEND BE_MOD_RUST_SOURCES "${BE_MOD_CRATE_DIR}/Cargo.toml")

# NDK 的 clang 同时充当 linker，cargo 只认环境变量。
add_custom_command(
    OUTPUT "${BE_MOD_ARTIFACT}"
    COMMAND ${CMAKE_COMMAND} -E env
        "CC_aarch64_linux_android=${CMAKE_C_COMPILER}"
        "CXX_aarch64_linux_android=${CMAKE_CXX_COMPILER}"
        "AR_aarch64_linux_android=${CMAKE_AR}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${CMAKE_C_COMPILER}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_AR=${CMAKE_AR}"
        "${BE_CARGO_EXECUTABLE}" build --release --locked
            --target "${BE_MOD_RUST_TARGET}"
            --manifest-path "${BE_MOD_CRATE_DIR}/Cargo.toml"
    DEPENDS ${BE_MOD_RUST_SOURCES}
    WORKING_DIRECTORY "${BE_MOD_CRATE_DIR}"
    COMMENT "cargo build (aarch64-linux-android) be-mod-ffi"
    VERBATIM)

add_custom_target(be_mod_rust DEPENDS "${BE_MOD_ARTIFACT}")

add_library(be_mod STATIC IMPORTED GLOBAL)
set_target_properties(be_mod PROPERTIES
    IMPORTED_LOCATION "${BE_MOD_ARTIFACT}"
    INTERFACE_INCLUDE_DIRECTORIES
        "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
