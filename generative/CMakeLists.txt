file(GLOB_RECURSE lib_sources CONFIGURE_DEPENDS "./*.cpp")

add_library(generative STATIC ${lib_sources})
target_include_directories(generative PUBLIC include)

if(GENERATIVE_ENABLE_LTO)
  set_property(TARGET generative PROPERTY INTERPROCEDURAL_OPTIMIZATION TRUE)
endif()

target_link_libraries(generative PRIVATE project_options project_warnings)
third_party_target_link_libraries(generative PUBLIC geos)
third_party_target_link_libraries(generative PRIVATE log4cplus)
