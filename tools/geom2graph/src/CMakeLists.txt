file(GLOB_RECURSE lib_sources CONFIGURE_DEPENDS "*.cpp")
list(REMOVE_ITEM lib_sources ${CMAKE_CURRENT_SOURCE_DIR}/main.cpp)

# TODO: Figure out why it takes so damn long to link the existing (and very simple) executable.
add_executable(geom2graph main.cpp)
add_library(geom2graph_lib OBJECT ${lib_sources})

# Enable LTO
set_property(TARGET geom2graph PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
set_property(TARGET geom2graph_lib PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)

target_link_libraries(geom2graph_lib PRIVATE project_options project_warnings)
third_party_target_link_libraries(geom2graph_lib PUBLIC geos)

target_link_libraries(geom2graph PRIVATE project_options project_warnings)
target_link_libraries(geom2graph PRIVATE geom2graph_lib)

# Include third party libraries with -isystem so I don't get clangd or clang-tidy warnings for them.
third_party_target_link_libraries(geom2graph PRIVATE cxxopts)
third_party_target_link_libraries(geom2graph PRIVATE log4cplus)
