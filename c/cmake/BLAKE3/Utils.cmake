
# Function for joining paths known from most languages
#
# SPDX-License-Identifier: (MIT OR CC0-1.0)
# Copyright 2020 Jan Tojnar
# https://github.com/jtojnar/cmake-snips
#
# Modelled after Python’s os.path.join
# https://docs.python.org/3.7/library/os.path.html#os.path.join
# Windows not supported
function(join_paths joined_path first_path_segment)
    set(temp_path "${first_path_segment}")
    foreach(current_segment IN LISTS ARGN)
        if(NOT ("${current_segment}" STREQUAL ""))
            if(IS_ABSOLUTE "${current_segment}")
                set(temp_path "${current_segment}")
            else()
                set(temp_path "${temp_path}/${current_segment}")
            endif()
        endif()
    endforeach()
    set(${joined_path} "${temp_path}" PARENT_SCOPE)
endfunction()

# In-place rewrite a string and and join by `sep`.
#
# TODO: Replace function with list(JOIN) when updating to CMake 3.12
function(join_pkg_config_field sep requires)
  set(_requires "${${requires}}") # avoid shadowing issues, e.g. "${requires}"=len
  list(LENGTH "${requires}" len)
  set(idx 1)
  foreach(req IN LISTS _requires)
    string(APPEND acc "${req}")
    if(idx LESS len)
      string(APPEND acc "${sep}")
    endif()
    math(EXPR idx "${idx} + 1")
  endforeach()
  set("${requires}" "${acc}" PARENT_SCOPE)
endfunction()
