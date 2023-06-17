cmake_minimum_required(VERSION 3.9)

project(libblake3
  VERSION 1.5.0
  DESCRIPTION "BLAKE3 C implementation"
  LANGUAGES C ASM
)

include_directories(c)
add_subdirectory(c)
