if(WIN32 AND CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  message(FATAL_ERROR "BLAKE3_USE_LLFIO requires building with Clang or MSVC on Windows")
  set(BLAKE3_USE_LLFIO OFF)
endif()

if(CMAKE_VERSION VERSION_GREATER_EQUAL 3.11)
  include(FetchContent)

  if(NOT TARGET ${BLAKE3_USE_LLFIO_TARGET} AND BLAKE3_USE_LLFIO)
    set(CMAKE_C_STANDARD 99)
    set(CMAKE_C_EXTENSIONS OFF)

    set(CMAKE_CXX_STANDARD 20)
    set(CMAKE_CXX_EXTENSIONS OFF)

    if(BLAKE3_FETCH_LLFIO)
      FetchContent_Declare(
        llfio
        GIT_REPOSITORY https://github.com/ned14/llfio
        GIT_TAG 20250206
        GIT_SHALLOW TRUE
        EXCLUDE_FROM_ALL
      )
      FetchContent_MakeAvailable(llfio)
    elseif(${BLAKE3_USE_LLFIO_DIR})
      add_subdirectory(${BLAKE3_USE_LLFIO_DIR} EXCLUDE_FROM_ALL)
    endif()
  endif()

  if(TARGET ${BLAKE3_USE_LLFIO_TARGET})
    if(${BLAKE3_USE_LLFIO_TARGET} STREQUAL "llfio_hl") 
      if(WIN32 AND NOT ${BLAKE3_USE_LLFIO_EXPERIMENTAL_STATUS_CODE})
        message(FATAL_ERROR 
          "llfio as a header library without SG14 status codes is an unsupported configuration on Windows.\n"
          "Select to link llfio as a dynamic or static library instead by setting `BLAKE3_USE_LLFIO_TARGET` to \"llfio_dl\" or \"llfio_sl\".")
      endif()
    endif()
    if(${BLAKE3_USE_LLFIO_EXPERIMENTAL_STATUS_CODE})
      target_compile_definitions(${BLAKE3_USE_LLFIO_TARGET}
        INTERFACE
          # Enable experimental SG14 `status_code` to work around unreliable Windows semantics.
          # See here for more details: https://github.com/ned14/llfio/blob/develop/Build.md
          LLFIO_EXPERIMENTAL_STATUS_CODE)
    endif()
    if(WIN32)
      target_compile_definitions(${BLAKE3_USE_LLFIO_TARGET}
        INTERFACE
          # Silence excessive CXX20 deprecation warning noise from MSVC
          _SILENCE_ALL_CXX20_DEPRECATION_WARNINGS)
    endif()
  endif()
endif()
