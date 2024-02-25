#pragma once
#include "generative/generative/cxxbridge/geometry_graph_ffi.rs.h"
#include "geometry_collection.hpp"
#include <generative/noding/geometry-graph.h>
#include <generative/noding/geometry-noder.h>

#include <geos/noding/snap/SnappingNoder.h>

#include <memory>

[[nodiscard]] inline std::unique_ptr<GeometryGraphShim>
node(const GeometryCollectionShim& rust_geoms, double tolerance) noexcept
{
    const auto geos_geoms = copy_rust_collection_to_geos(rust_geoms);
    auto noded = std::unique_ptr<geos::geom::Geometry>(nullptr);
    if (tolerance == 0.0)
    {
        noded = generative::noding::GeometryNoder::node(*geos_geoms, nullptr);
    } else
    {
        auto noder = std::make_unique<geos::noding::snap::SnappingNoder>(tolerance);
        noded = generative::noding::GeometryNoder::node(*geos_geoms, std::move(noder));
    }
    auto graph = generative::noding::GeometryGraph(*noded);

    auto graph_shim = std::make_unique<GeometryGraphShim>(std::move(graph));

    return graph_shim;
}
