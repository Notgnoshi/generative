add_executable(tests "")

file(GLOB_RECURSE sources CONFIGURE_DEPENDS "*.cpp")
target_sources(tests PRIVATE ${sources})
unset(sources)

file(GLOB_RECURSE headers CONFIGURE_DEPENDS "*.h")
target_sources(tests PRIVATE ${headers})
unset(headers)

target_link_libraries(tests PRIVATE geom2graph_lib)
target_link_libraries(tests PRIVATE project_options)
third_party_target_link_libraries(tests PRIVATE gmock)
third_party_target_link_libraries(tests PRIVATE gtest)
third_party_target_link_libraries(tests PRIVATE log4cplus)
