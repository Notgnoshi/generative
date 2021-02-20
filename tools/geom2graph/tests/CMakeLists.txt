add_executable(tests "")

file(GLOB_RECURSE sources CONFIGURE_DEPENDS "*.cpp")
target_sources(tests PRIVATE ${sources})
unset(sources)

file(GLOB_RECURSE headers CONFIGURE_DEPENDS "*.h")
target_sources(tests PRIVATE ${headers})
unset(headers)

third_party_target_link_libraries(tests PRIVATE gtest_main)
target_link_libraries(tests PRIVATE geom2graph_lib)
