cmake_minimum_required(VERSION 3.9)

project(libblake3 VERSION 0.1.0 DESCRIPTION "BLAKE3 C implementation")
enable_language(C ASM)

include(GNUInstallDirs)

option(BLAKE3_STATIC ON)

set(blake3_SOURCES)
list(APPEND blake3_SOURCES
  blake3.c
  blake3_dispatch.c
  blake3_portable.c)

if(CMAKE_SYSTEM_PROCESSOR STREQUAL x86_64)
  list(APPEND blake3_SOURCES
    blake3_avx2_x86-64_unix.S
    blake3_avx512_x86-64_unix.S
    blake3_sse2_x86-64_unix.S
    blake3_sse41_x86-64_unix.S)

elseif(CMAKE_SYSTEM_PROCESSOR STREQUAL i686)
  list(APPEND blake3_SOURCES
    blake3_avx2.c
    blake3_avx512.c
    blake3_sse2.c
    blake3_sse41.c)
  set_source_files_properties(blake3_avx2.c PROPERTIES COMPILE_FLAGS -mavx2)
  set_source_files_properties(blake3_avx512.c PROPERTIES COMPILE_FLAGS "-mavx512f -mavx512vl")
  set_source_files_properties(blake3_sse2.c PROPERTIES COMPILE_FLAGS -msse2)
  set_source_files_properties(blake3_sse41.c PROPERTIES COMPILE_FLAGS -msse4.1)

elseif((ANDROID_ABI STREQUAL armeabi-v7a) OR
       (CMAKE_SYSTEM_PROCESSOR STREQUAL aarch64) OR 
       (CMAKE_SYSTEM_PROCESSOR STREQUAL arm64) # For M1 macs
      )
  list(APPEND blake3_SOURCES blake3_neon.c)
  set_source_files_properties(blake3_dispatch.c PROPERTIES COMPILE_FLAGS -DBLAKE3_USE_NEON=1)
  set_source_files_properties(blake3_neon.c PROPERTIES COMPILE_FLAGS -mfpu=neon)
endif()

configure_file(libblake3.pc.in libblake3.pc @ONLY)
install(FILES ${CMAKE_BINARY_DIR}/libblake3.pc
  DESTINATION ${CMAKE_INSTALL_LIBDIR}/pkgconfig)

if(BLAKE3_STATIC)
  add_library(blake3-static STATIC ${blake3_SOURCES})
  set_target_properties(blake3-static PROPERTIES OUTPUT_NAME blake3)
  set_target_properties(blake3-static PROPERTIES VERSION ${PROJECT_VERSION})
  set_target_properties(blake3-static PROPERTIES SOVERSION 0)
  set_target_properties(blake3-static PROPERTIES PUBLIC_HEADER blake3.h)
  set_target_properties(blake3-static PROPERTIES COMPILE_FLAGS -fPIC)
  target_include_directories(blake3-static PUBLIC .)

  install(TARGETS blake3-static
  LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
  PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

else()

  add_library(blake3-shared SHARED ${blake3_SOURCES})
  set_target_properties(blake3-shared PROPERTIES VERSION ${PROJECT_VERSION})
  set_target_properties(blake3-shared PROPERTIES SOVERSION 0)
  set_target_properties(blake3-shared PROPERTIES PUBLIC_HEADER blake3.h)
  target_include_directories(blake3-shared PUBLIC .)
  
  install(TARGETS blake3-shared
    LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
    PUBLIC_HEADER DESTINATION ${CMAKE_INSTALL_INCLUDEDIR})

endif(BLAKE3_STATIC)

unset(BLAKE3_STATIC)