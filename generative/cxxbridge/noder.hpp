#pragma once
#include "generative/generative/cxxbridge/coord_ffi.rs.h"
#include "generative/generative/cxxbridge/geometry_graph_ffi.rs.h"
#include "geometry_collection.hpp"
#include "geometry_graph.hpp"
#include <generative/noding/geometry-graph.h>
#include <generative/noding/geometry-noder.h>

#include <geos/geom/Geometry.h>
#include <geos/noding/snap/SnappingNoder.h>
#include <geos/operation/overlay/snap/GeometrySnapper.h>
#include <geos/operation/polygonize/Polygonizer.h>

#include <iterator>
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
    if (!noded)
    {
        return nullptr;
    }

    auto graph = generative::noding::GeometryGraph(*noded);

    auto graph_shim = std::make_unique<GeometryGraphShim>(std::move(graph));

    return graph_shim;
}

[[nodiscard]] inline PolygonizationResult polygonize(const GeometryGraphShim& graph) noexcept
{
    const auto owned_edges = graph.inner().get_edges();
    std::vector<const geos::geom::Geometry*> non_owned_edges;
    non_owned_edges.reserve(owned_edges.size());
    std::transform(
        owned_edges.cbegin(),
        owned_edges.cend(),
        std::back_inserter(non_owned_edges),
        [](const std::unique_ptr<geos::geom::LineString>& edge) -> const geos::geom::Geometry* {
            return edge.get();
        });

    auto polygonizer = geos::operation::polygonize::Polygonizer();
    // Adding the edges doesn't actually polygonize. That's deferred to the first time the polygons
    // or dangles are accessed.
    polygonizer.add(&non_owned_edges);

    const auto polys = polygonizer.getPolygons();
    // The dangles are just pointers back to the LineString pointers we passed in?
    const auto dangles = polygonizer.getDangles();

    PolygonizationResult retval;
    retval.polygons.reserve(polys.size());
    retval.dangles.reserve(dangles.size());

    for (const auto& poly : polys)
    {
        LineStringShim result;
        const auto* shell = poly->getExteriorRing();
        result.vec.reserve(shell->getNumPoints());
        const auto* coords = shell->getCoordinatesRO();
        for (size_t i = 0; i < coords->size(); i++)
        {
            const auto coord = coords->getAt(i);
            result.vec.push_back(CoordShim{coord.x, coord.y});
        }
        retval.polygons.push_back(result);
    }

    for (const auto* dangle : dangles)
    {
        LineStringShim result;
        const auto* coords = dangle->getCoordinatesRO();
        result.vec.reserve(coords->size());
        for (size_t i = 0; i < coords->size(); i++)
        {
            const auto coord = coords->getAt(i);
            result.vec.push_back(CoordShim{coord.x, coord.y});
        }
        retval.dangles.push_back(result);
    }

    return retval;
}
