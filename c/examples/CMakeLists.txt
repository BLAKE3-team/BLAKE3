cmake_minimum_required(VERSION 3.9 FATAL_ERROR)

# respect C_EXTENSIONS OFF without explicitly setting C_STANDARD
if (POLICY CMP0128)
  cmake_policy(SET CMP0128 NEW)
endif()
# mark_as_advanced does not implicitly create UNINITIALIZED cache entries
if (POLICY CMP0102)
  cmake_policy(SET CMP0102 NEW)
endif()

project(blake3-examples
  VERSION 1.8.1
  DESCRIPTION "BLAKE3 C examples"
  LANGUAGES C CXX ASM
)

find_package(BLAKE3)

if(TARGET BLAKE3::blake3)
  if(NOT WIN32)
    add_executable(blake3-example
      example.c)
    target_link_libraries(blake3-example PUBLIC BLAKE3::blake3)
    install(TARGETS blake3-example)

    if(BLAKE3_USE_TBB)
      add_executable(blake3-example-tbb
        example_tbb.c)
      target_link_libraries(blake3-example-tbb PUBLIC BLAKE3::blake3)
      install(TARGETS blake3-example-tbb)
    endif()
  endif()
endif()
