# ForTMR4A：BEM → mod.toml/IR 转换（Rust 叶子）
cmake_minimum_required(VERSION 3.22.1)

get_filename_component(BETTER_ENDFIELD_NEXT_ROOT
    "${CMAKE_CURRENT_SOURCE_DIR}/../.." ABSOLUTE)

find_program(BE_IR_CARGO_EXECUTABLE cargo)
if(NOT BE_IR_CARGO_EXECUTABLE)
    message(FATAL_ERROR "cargo not found; install Rust or disable BETTER_ENDFIELD_NEXT_IR_RUST")
endif()

set(BE_IR_RUST_TARGET "aarch64-linux-android" CACHE STRING "Rust target triple")
set(BE_IR_CRATE_DIR "${CMAKE_CURRENT_SOURCE_DIR}")
set(BE_IR_ARTIFACT "${BE_IR_CRATE_DIR}/target/${BE_IR_RUST_TARGET}/release/libbe_ir.a")

file(GLOB_RECURSE BE_IR_RUST_SOURCES CONFIGURE_DEPENDS "${BE_IR_CRATE_DIR}/src/*.rs")
list(APPEND BE_IR_RUST_SOURCES "${BE_IR_CRATE_DIR}/Cargo.toml")

add_custom_command(
    OUTPUT "${BE_IR_ARTIFACT}"
    COMMAND ${CMAKE_COMMAND} -E env
        "CC_aarch64_linux_android=${CMAKE_C_COMPILER}"
        "CXX_aarch64_linux_android=${CMAKE_CXX_COMPILER}"
        "AR_aarch64_linux_android=${CMAKE_AR}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=${CMAKE_C_COMPILER}"
        "CARGO_TARGET_AARCH64_LINUX_ANDROID_AR=${CMAKE_AR}"
        "${BE_IR_CARGO_EXECUTABLE}" build --release --locked
            --target "${BE_IR_RUST_TARGET}"
            --manifest-path "${BE_IR_CRATE_DIR}/Cargo.toml"
    DEPENDS ${BE_IR_RUST_SOURCES}
    WORKING_DIRECTORY "${BE_IR_CRATE_DIR}"
    COMMENT "cargo build (aarch64-linux-android) be-ir"
    VERBATIM)

add_custom_target(be_ir_rust DEPENDS "${BE_IR_ARTIFACT}")

add_library(be_ir STATIC IMPORTED GLOBAL)
set_target_properties(be_ir PROPERTIES
    IMPORTED_LOCATION "${BE_IR_ARTIFACT}"
    INTERFACE_INCLUDE_DIRECTORIES "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
