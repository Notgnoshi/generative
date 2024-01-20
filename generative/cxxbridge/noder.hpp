#pragma once
#include "generative/generative/cxxbridge/geometry_collection_ffi.rs.h"
#include "geometry_collection.hpp"

inline void _compile_tester(const GeometryCollectionShim& rust_geoms) noexcept
{
    const auto geos_geoms = copy_rust_collection_to_geos(rust_geoms);
}
