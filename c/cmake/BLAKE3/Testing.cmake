
find_package(Python COMPONENTS Interpreter)

add_executable(blake3-test EXCLUDE_FROM_ALL main.c)
target_link_libraries(blake3-test PRIVATE BLAKE3::blake3)

# use a function to scope the variables
function(blake3_add_tests)
  set(_TEST_CONFIGS "DEFAULT")
  set(_TEST_CONFIG_DEFAULT_NAME "default")
  set(_TEST_CONFIG_DEFAULT_C_FLAGS "")

  if(BLAKE3_SIMD_TYPE STREQUAL "amd64-asm" OR BLAKE3_SIMD_TYPE STREQUAL "x86-intrinsics")
    set(_SIMD_FLAVORS SSE2 SSE41 AVX2 AVX512)
    foreach(_SIMD_FLAVOR IN LISTS _SIMD_FLAVORS)
      list(APPEND _TEST_CONFIGS ${_SIMD_FLAVOR})
      string(TOLOWER ${_SIMD_FLAVOR} _TEST_CONFIG_${_SIMD_FLAVOR}_NAME)
      foreach(_EXCLUDED IN LISTS _SIMD_FLAVORS)
        if(NOT _EXCLUDED STREQUAL _SIMD_FLAVOR)
          list(APPEND _TEST_CONFIG_${_SIMD_FLAVOR}_C_FLAGS "-DBLAKE3_NO_${_EXCLUDED}=1")
        endif()
      endforeach()
      join_pkg_config_field(" " _TEST_CONFIG_${_SIMD_FLAVOR}_C_FLAGS)
    endforeach()
  endif()

  set(_TEST_C_FLAGS
    -DBLAKE3_TESTING=1
  )

  foreach(CONFIG IN LISTS _TEST_CONFIGS)
    set(_TEST_BINARY_DIR "${CMAKE_CURRENT_BINARY_DIR}/${_TEST_CONFIG_${CONFIG}_NAME}")
    file(MAKE_DIRECTORY "${_TEST_BINARY_DIR}")
    add_test(NAME blake3-test-${_TEST_CONFIG_${CONFIG}_NAME}
      COMMAND "${CMAKE_CTEST_COMMAND}"
        -C $<CONFIG>
        --verbose
        --extra-verbose
        --build-and-test "${CMAKE_CURRENT_SOURCE_DIR}" "${_TEST_BINARY_DIR}"
        --build-generator "${CMAKE_GENERATOR}"
        --build-makeprogram "${CMAKE_MAKE_PROGRAM}"
        --build-project libblake3
        --build-target blake3-test
        --build-options
          "-DCMAKE_C_COMPILER=${CMAKE_C_COMPILER}"
          "-DCMAKE_CXX_COMPILER=${CMAKE_CXX_COMPILER}"
          "-DBUILD_TESTING=ON"
          "-DBUILD_SHARED_LIBS=${BUILD_SHARED_LIBS}"
          "-DBLAKE3_FROM_CTEST=ON"
          "-DBLAKE3_SIMD_TYPE=${BLAKE3_SIMD_TYPE}"
          "-DBLAKE3_USE_TBB=${BLAKE3_USE_TBB}"
          "-DCMAKE_C_FLAGS=${CMAKE_C_FLAGS} ${_TEST_C_FLAGS} ${_TEST_CONFIG_${CONFIG}_C_FLAGS}"
          "-DCMAKE_CXX_FLAGS=${CMAKE_CXX_FLAGS} ${_TEST_C_FLAGS} ${_TEST_CONFIG_${CONFIG}_C_FLAGS}"
          "-DCMAKE_EXE_LINKER_FLAGS=${CMAKE_EXE_LINKER_FLAGS}"
        --test-command
          $<TARGET_FILE:Python::Interpreter> "${CMAKE_SOURCE_DIR}/test.py" "${_TEST_BINARY_DIR}/$<CONFIG>/$<TARGET_FILE_NAME:blake3-test>"
    )
  endforeach()
endfunction()

if (NOT BLAKE3_FROM_CTEST)
  blake3_add_tests()
endif()
