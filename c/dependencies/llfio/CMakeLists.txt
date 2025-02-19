if(WIN32 AND CMAKE_CXX_COMPILER_ID STREQUAL "GNU")
  message(FATAL_ERROR "BLAKE3_USE_LLFIO requires building with Clang or MSVC on Windows")
  set(BLAKE3_USE_LLFIO OFF)
endif()

if(CMAKE_VERSION VERSION_GREATER_EQUAL 3.11)
  include(FetchContent)

  if(NOT TARGET llfio::hl AND BLAKE3_USE_LLFIO)
    set(CMAKE_C_STANDARD 99)
    set(CMAKE_C_EXTENSIONS OFF)

    set(CMAKE_CXX_STANDARD 20)
    set(CMAKE_CXX_EXTENSIONS OFF)

    FetchContent_Declare(
      llfio
      GIT_REPOSITORY https://github.com/ned14/llfio
      GIT_TAG 20250206
      GIT_SHALLOW TRUE
      EXCLUDE_FROM_ALL
    )

    FetchContent_MakeAvailable(llfio)
  endif()

  if(TARGET llfio::hl)
    target_compile_definitions(llfio_hl
      INTERFACE
        # Enable experimental SG14 `status_code` to work around unreliable Windows semantics.
        # See here for more details: https://github.com/ned14/llfio/blob/develop/Build.md
        LLFIO_EXPERIMENTAL_STATUS_CODE)
    if(WIN32)
      target_compile_definitions(llfio_hl
        INTERFACE
          # Enable experimental SG14 `status_code` to work around unreliable Windows semantics.
          # See here for more details: https://github.com/ned14/llfio/blob/develop/Build.md
          LLFIO_EXPERIMENTAL_STATUS_CODE
          # Silence excessive CXX20 deprecation warning noise from MSVC
          _SILENCE_ALL_CXX20_DEPRECATION_WARNINGS)
    endif()
  endif()
endif()
