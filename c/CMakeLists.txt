cmake_minimum_required(VERSION 3.9)

project(libblake3
  VERSION 0.1.0
  DESCRIPTION "BLAKE3 C implementation"
  LANGUAGES C ASM
)

include(GNUInstallDirs)

# default SIMD compiler flag configuration (can be overriden by toolchains or CLI)
if(CMAKE_C_COMPILER_ID STREQUAL "MSVC")
  set(BLAKE3_CFLAGS_SSE2 "/arch:SSE2" CACHE STRING "the compiler flags to enable SSE2")
  # MSVC has no dedicated sse4.1 flag (see https://learn.microsoft.com/en-us/cpp/build/reference/arch-x86?view=msvc-170)
  set(BLAKE3_CFLAGS_SSE4.1 "/arch:AVX" CACHE STRING "the compiler flags to enable SSE4.1")
  set(BLAKE3_CFLAGS_AVX2 "/arch:AVX2" CACHE STRING "the compiler flags to enable AVX2")
  set(BLAKE3_CFLAGS_AVX512 "/arch:AVX512" CACHE STRING "the compiler flags to enable AVX512")

elseif(CMAKE_C_COMPILER_ID STREQUAL "GNU"
       OR CMAKE_C_COMPILER_ID STREQUAL "Clang"
       OR CMAKE_C_COMPILER_ID STREQUAL "AppleClang")
  set(BLAKE3_CFLAGS_SSE2 "-msse2" CACHE STRING "the compiler flags to enable SSE2")
  set(BLAKE3_CFLAGS_SSE4.1 "-msse4.1" CACHE STRING "the compiler flags to enable SSE4.1")
  set(BLAKE3_CFLAGS_AVX2 "-mavx2" CACHE STRING "the compiler flags to enable AVX2")
  set(BLAKE3_CFLAGS_AVX512 "-mavx512f -mavx512vl" CACHE STRING "the compiler flags to enable AVX512")
  set(BLAKE3_CFLAGS_NEON "-mfpu=neon" CACHE STRING "the compiler flags to enable NEON")
endif()

# library target
add_library(blake3
  blake3.c
  blake3_dispatch.c
  blake3_portable.c
)
add_library(BLAKE3::blake3 ALIAS blake3)

# library configuration
target_include_directories(blake3 PUBLIC $<INSTALL_INTERFACE:${CMAKE_INSTALL_INCLUDEDIR}>)
set_target_properties(blake3 PROPERTIES
  VERSION ${PROJECT_VERSION}
  SOVERSION 0
)

# optional SIMD sources
if(CMAKE_SYSTEM_PROCESSOR STREQUAL "x86_64"
   OR CMAKE_SYSTEM_PROCESSOR STREQUAL "AMD64")
    if(CMAKE_C_COMPILER_ID STREQUAL "MSVC")
    enable_language(ASM_MASM)
    target_sources(blake3 PRIVATE
      blake3_avx2_x86-64_windows_msvc.asm
      blake3_avx512_x86-64_windows_msvc.asm
      blake3_sse2_x86-64_windows_msvc.asm
      blake3_sse41_x86-64_windows_msvc.asm
    )

  elseif(CMAKE_C_COMPILER_ID STREQUAL "GNU"
         OR CMAKE_C_COMPILER_ID STREQUAL "Clang"
         OR CMAKE_C_COMPILER_ID STREQUAL "AppleClang")
    if (WIN32)
      target_sources(blake3 PRIVATE
        blake3_avx2_x86-64_windows_gnu.S
        blake3_avx512_x86-64_windows_gnu.S
        blake3_sse2_x86-64_windows_gnu.S
        blake3_sse41_x86-64_windows_gnu.S
      )

    elseif(UNIX)
      target_sources(blake3 PRIVATE
        blake3_avx2_x86-64_unix.S
        blake3_avx512_x86-64_unix.S
        blake3_sse2_x86-64_unix.S
        blake3_sse41_x86-64_unix.S
      )
    endif()
  endif()

elseif((CMAKE_SYSTEM_PROCESSOR STREQUAL "i686"
          OR CMAKE_SYSTEM_PROCESSOR STREQUAL "X86")
       AND DEFINED BLAKE3_CFLAGS_SSE2
       AND DEFINED BLAKE3_CFLAGS_SSE4.1
       AND DEFINED BLAKE3_CFLAGS_AVX2
       AND DEFINED BLAKE3_CFLAGS_AVX512)
  target_sources(blake3 PRIVATE
    blake3_avx2.c
    blake3_avx512.c
    blake3_sse2.c
    blake3_sse41.c
  )
  set_source_files_properties(blake3_avx2.c PROPERTIES COMPILE_FLAGS "${BLAKE3_CFLAGS_AVX2}")
  set_source_files_properties(blake3_avx512.c PROPERTIES COMPILE_FLAGS "${BLAKE3_CFLAGS_AVX512}")
  set_source_files_properties(blake3_sse2.c PROPERTIES COMPILE_FLAGS "${BLAKE3_CFLAGS_SSE2}")
  set_source_files_properties(blake3_sse41.c PROPERTIES COMPILE_FLAGS "${BLAKE3_CFLAGS_SSE4.1}")

elseif((ANDROID_ABI STREQUAL "armeabi-v7a"
          OR CMAKE_SYSTEM_PROCESSOR STREQUAL "aarch64"
          OR CMAKE_SYSTEM_PROCESSOR STREQUAL "arm64") # For M1 macs
       AND DEFINED BLAKE3_CFLAGS_NEON)
  target_sources(blake3 PRIVATE
    blake3_neon.c
  )
  set_source_files_properties(blake3_dispatch.c PROPERTIES COMPILE_DEFINITIONS BLAKE3_USE_NEON=1)
  set_source_files_properties(blake3_neon.c PROPERTIES COMPILE_FLAGS "${BLAKE3_CFLAGS_NEON}")
endif()

# cmake install support
install(FILES blake3.h DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
install(TARGETS blake3 EXPORT blake3-targets)
install(EXPORT blake3-targets
  NAMESPACE BLAKE3::
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/blake3"
)

include(CMakePackageConfigHelpers)
configure_package_config_file(blake3-config.cmake.in
    "${CMAKE_CURRENT_BINARY_DIR}/blake3-config.cmake"

    INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/blake3"
)
write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/blake3-config-version.cmake"
    VERSION ${libblake3_VERSION}
    COMPATIBILITY SameMinorVersion # as long as libblake3_VERSION < 1.0.0
)
install(FILES
        "${CMAKE_CURRENT_BINARY_DIR}/blake3-config.cmake"
        "${CMAKE_CURRENT_BINARY_DIR}/blake3-config-version.cmake"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/blake3"
)

configure_file(libblake3.pc.in libblake3.pc @ONLY)
install(FILES "${CMAKE_BINARY_DIR}/libblake3.pc"
  DESTINATION "${CMAKE_INSTALL_LIBDIR}/pkgconfig")
