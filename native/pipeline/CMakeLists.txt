# ForTMR4A：统一导入入口（C 主链路）
cmake_minimum_required(VERSION 3.22.1)

if(NOT TARGET be_convert OR NOT TARGET be_ir)
    message(FATAL_ERROR "be_pipeline 需要 be_convert 与 be_ir：请先 add_subdirectory native/convert 与 native/ir")
endif()

add_library(be_pipeline STATIC source_pipeline.c)
target_include_directories(be_pipeline PRIVATE
    "${BETTER_ENDFIELD_NEXT_ROOT}/native/shared/include")
target_compile_features(be_pipeline PRIVATE c_std_11)
target_link_libraries(be_pipeline PRIVATE be_convert be_ir)

add_executable(be-source EXCLUDE_FROM_ALL main.c)
target_link_libraries(be-source PRIVATE be_pipeline)
