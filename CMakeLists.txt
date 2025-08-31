cmake_minimum_required(VERSION 3.15)
# TODO: What's the actual minimum version?

project(
  generative
  VERSION 0.1
  DESCRIPTION "A C++ generative art library"
  LANGUAGES CXX)
# TODO: Add the description to a macro, and replace the string literal in the
# application source.

include(cmake/standard-settings.cmake)
include(cmake/prevent-in-source-builds.cmake)
include(cmake/third-party-target-link-libraries.cmake)

# TODO: Add UB and other sanitizers TODO: Add static analyzers like clang-tidy,
# and cppcheck

add_library(project_options INTERFACE)
target_compile_features(project_options INTERFACE cxx_std_17)

add_library(project_warnings INTERFACE)
include(cmake/compiler-warnings.cmake)
set_project_warnings(project_warnings)

option(GENERATIVE_BUILD_DOCS "Build Doxygen documentation" ON)
if(GENERATIVE_BUILD_DOCS)
  include(cmake/doxygen.cmake)
  enable_doxygen()
endif()

option(GENERATIVE_ENABLE_PCH "Enable Precompiled Headers" ON)
if(GENERATIVE_ENABLE_PCH)
  target_precompile_headers(
    project_options
    INTERFACE
    <vector>
    <string>
    <map>
    <utility>
    <unordered_set>
    <set>
    <memory>)
endif()

option(GENERATIVE_ENABLE_LTO "Enable link time optimization" ON)

set(ROOT submodules/geos)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/submodules/geos/cmake")
set(GEOS_BUILD_DEVELOPER
    OFF
    CACHE BOOL "enable geos warnings" FORCE)
set(BUILD_TESTING
    OFF
    CACHE BOOL "enable geos tests" FORCE)
set(BUILD_BENCHMARKS
    OFF
    CACHE BOOL "enable geos benchmarks" FORCE)
add_subdirectory(submodules/geos)
set(ROOT ../../)

# Prevents duplicate target failure :(
set(LOG4CPLUS_BUILD_TESTING
    OFF
    CACHE BOOL "enable log4cplus tests" FORCE)
set(WITH_UNIT_TESTS
    OFF
    CACHE BOOL "enable log4cplus tests" FORCE)
add_subdirectory(submodules/log4cplus)

if(DEFINED GENERATIVE_TOOL_INSTALL_RPATH)
  set(CMAKE_INSTALL_RPATH "${GENERATIVE_TOOL_INSTALL_RPATH}")
else()
  set(CMAKE_INSTALL_RPATH "$ORIGIN/../lib/")
endif()

add_subdirectory(generative/)

# TODO: Overwrite the geos-defined 'make check' target to run the tests.
option(GENERATIVE_ENABLE_TESTING "Enable unit tests" ON)
if(GENERATIVE_ENABLE_TESTING)
  # TODO: Decide if CTest is worth using TODO: Add coverage instrumentation
  add_subdirectory(submodules/googletest)
  add_subdirectory(tests)
endif()
