#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "BLAKE3::blake3" for configuration "Release"
set_property(TARGET BLAKE3::blake3 APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(BLAKE3::blake3 PROPERTIES
  IMPORTED_LINK_INTERFACE_LANGUAGES_RELEASE "ASM;C"
  IMPORTED_LOCATION_RELEASE "${_IMPORT_PREFIX}/lib/libblake3.a"
  )

list(APPEND _cmake_import_check_targets BLAKE3::blake3 )
list(APPEND _cmake_import_check_files_for_BLAKE3::blake3 "${_IMPORT_PREFIX}/lib/libblake3.a" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
