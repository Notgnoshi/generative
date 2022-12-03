# TODO: Figure out why it takes so damn long to link the existing (and very
# simple) executable.
add_executable(geom2graph geom2graph.cpp)
install(TARGETS geom2graph)

# Enable LTO
if(GENERATIVE_ENABLE_LTO)
  set_property(TARGET geom2graph PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
endif()

target_link_libraries(geom2graph PRIVATE project_options project_warnings)
target_link_libraries(geom2graph PRIVATE generative)

# Include third party libraries with -isystem so I don't get clangd or
# clang-tidy warnings for them.
third_party_target_link_libraries(geom2graph PRIVATE cxxopts)
third_party_target_link_libraries(geom2graph PRIVATE log4cplus)
