# ForTMR4A：BEM 包读取/校验（可选 Rust 叶子）
#
# 与 native/mod 相同的接入模式：默认关闭，打开后 cargo 产出 libbe_bem.a 并链进 Android 运行时。
cmake_minimum_required(VERSION 3.22.1)

get_filename_component(BETTER_ENDFIELD_NEXT_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

find_program(BE_BEM_CARGO_EXECUTABLE cargo)
if(NOT BE_BEM_CARGO_EXECUTABLE)
    message(FATAL_ERROR "cargo not found; install Rust or disable BETTER_ENDFIELD_NEXT_BEM_RUST")
endif()

set(BE_BEM_RUST_TARGET "aarch64-linux-android" CACHE STRING "Rust target triple")
set(BE_BEM_CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
set(BE_BEM_ARTIFACT
    "${BE_BEM_CRATE_DIR}/target/${BE_BEM_RUST_TARGET}/release/libbe_bem.a")

file(GLOB_RECURSE BE_BEM_RUST_SOURCES CONFIGURE_DEPENDS
    "${BE_BEM_CRATE_DIR}/src/*.rs")
list(APPEND BE_BEM_RUST_SOURCES "${BE_BEM_CRATE_DIR}/Cargo.toml")

add_custom_command(
    OUTPUT "${BE_BEM_ARTIFACT}"
    COMMAND ${CMAKE_COMMAND} -E env
        "CC_aarch64_linux_android=${CMAKE_C_COMPILER}"
        "CXX_aarch64_linux_android=${CMAKE_CXX_COMPILER}"
        "AR_aarch64_linux_android=${CMAKE_AR}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${CMAKE_C_COMPILER}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_AR=${CMAKE_AR}"
        "${BE_BEM_CARGO_EXECUTABLE}" build --release --locked
            --target "${BE_BEM_RUST_TARGET}"
            --manifest-path "${BE_BEM_CRATE_DIR}/Cargo.toml"
    DEPENDS ${BE_BEM_RUST_SOURCES}
    WORKING_DIRECTORY "${BE_BEM_CRATE_DIR}"
    COMMENT "cargo build (aarch64-linux-android) be-bem"
    VERBATIM)

add_custom_target(be_bem_rust DEPENDS "${BE_BEM_ARTIFACT}")

add_library(be_bem STATIC IMPORTED GLOBAL)
set_target_properties(be_bem PROPERTIES
    IMPORTED_LOCATION "${BE_BEM_ARTIFACT}"
    INTERFACE_INCLUDE_DIRECTORIES
        "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
