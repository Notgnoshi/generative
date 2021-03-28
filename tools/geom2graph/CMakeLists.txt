cmake_minimum_required(VERSION 3.15)
# TODO: What's the actual minimum version?

project(
    geom2graph
    VERSION 0.1
    DESCRIPTION "A C++ CLI application to convert a set of WKT geometries to a graph representation"
    LANGUAGES CXX
)
# TODO: Add the description to a macro, and replace the string literal in the application source.

include(cmake/standard-settings.cmake)
include(cmake/prevent-in-source-builds.cmake)
include(cmake/third-party-target-link-libraries.cmake)

# TODO: Add UB and other sanitizers
# TODO: Add static analyzers like clang-tidy, and cppcheck

add_library(project_options INTERFACE)
target_compile_features(project_options INTERFACE cxx_std_17)

add_library(project_warnings INTERFACE)
include(cmake/compiler-warnings.cmake)
set_project_warnings(project_warnings)

include(cmake/doxygen.cmake)
enable_doxygen()

option(ENABLE_PCH "Enable Precompiled Headers" ON)
if(ENABLE_PCH)
    target_precompile_headers(
        project_options
        INTERFACE
        <vector>
        <string>
        <map>
        <utility>
        <unordered_set>
        <set>
        <memory>
    )
endif()

set(ROOT depends/geos)
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_LIST_DIR}/depends/geos/cmake")
set(GEOS_BUILD_DEVELOPER OFF CACHE BOOL "enable geos warnings" FORCE)
set(BUILD_TESTING OFF CACHE BOOL "enable geos tests" FORCE)
set(BUILD_BENCHMARKS OFF CACHE BOOL "enable geos benchmarks" FORCE)
add_subdirectory(depends/geos)
set(ROOT ../../)

# SFCGAL doesn't provide a SFCGALConfig.cmake file when installed.
# See: https://packages.ubuntu.com/focal/libsfcgal-dev
#find_package(SFCGAL)
find_package(Boost REQUIRED COMPONENTS serialization)

# cxxopts has some very reasonable configuration that detects if it's being built by a third party,
# and does the right thing so as to not break said party.
add_subdirectory(depends/cxxopts)

set(LOG4CPLUS_BUILD_TESTING OFF CACHE BOOL "enable log4cplus tests" FORCE)  # Prevents duplicate target failure :(
set(WITH_UNIT_TESTS OFF CACHE BOOL "enable log4cplus tests" FORCE)
add_subdirectory(depends/log4cplus)

# Needs to be set before we add the src/ directory.
include_directories(include)
add_subdirectory(src)

# TODO: Overwrite the geos-defined 'make check' target to run the tests.
option(ENABLE_TESTING "Enable unit tests" ON)
if(ENABLE_TESTING)
    # TODO: Decide if CTest is worth using
    # TODO: Add coverage instrumentation
    add_subdirectory(depends/googletest)
    add_subdirectory(tests)
endif()
