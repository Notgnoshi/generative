#pragma once
#include "generative/generative/cxxbridge/coord_ffi.rs.h"
#include "generative/generative/cxxbridge/geometry_collection_ffi.rs.h"

#include <geos/geom/Coordinate.h>
#include <geos/geom/CoordinateSequence.h>
#include <geos/geom/Geometry.h>
#include <geos/geom/GeometryCollection.h>
#include <geos/geom/GeometryFactory.h>
#include <geos/geom/LinearRing.h>

#include <memory>
#include <vector>

/// @brief Convert each of the Rust geo::Geometry's to a C++ geos::Geometry
[[nodiscard]] inline std::vector<std::unique_ptr<geos::geom::Geometry>>
get_geos_geoms_from_rust(const GeometryCollectionShim& rust_geoms,
                         geos::geom::GeometryFactory::Ptr& factory) noexcept
{
    std::vector<std::unique_ptr<geos::geom::Geometry>> geos_geoms;
    geos_geoms.reserve(rust_geoms.get_total_geoms());

    const auto points = rust_geoms.get_points();
    for (const auto& point : points)
    {
        const auto coord = geos::geom::CoordinateXY{point.x, point.y};
        factory->createPoint(coord);
    }

    const auto linestrings = rust_geoms.get_linestrings();
    for (const auto& linestring : linestrings)
    {
        auto cs = std::make_unique<geos::geom::CoordinateSequence>(linestring.vec.size(), 2);
        for (auto coord : linestring.vec)
        {
            cs->add(geos::geom::CoordinateXY{coord.x, coord.y});
        }

        auto geos_line = factory->createLineString(std::move(cs));
        geos_geoms.push_back(std::move(geos_line));
    }

    const auto polygons = rust_geoms.get_polygons();
    for (const auto& polygon : polygons)
    {
        std::vector<std::unique_ptr<geos::geom::LinearRing>> holes;
        std::unique_ptr<geos::geom::LinearRing> shell = nullptr;
        for (const auto& ring : polygon.vec)
        {
            auto cs = std::make_unique<geos::geom::CoordinateSequence>(ring.vec.size(), 2);
            for (auto coord : ring.vec)
            {
                cs->add(geos::geom::CoordinateXY{coord.x, coord.y});
            }
            auto geos_ring = factory->createLinearRing(std::move(cs));
            if (shell == nullptr)
            {
                shell.swap(geos_ring);
            } else
            {
                holes.push_back(std::move(geos_ring));
            }
        }

        if (shell != nullptr)
        {
            auto geos_polygon = factory->createPolygon(std::move(shell), std::move(holes));
            geos_geoms.push_back(std::move(geos_polygon));
        }
    }

    return geos_geoms;
}

/// @brief Convert a Rust GeometryCollectionShim to a Geos Geometry
///
/// @note The generated GeometryCollection isn't passed back to the Rust side. It's entirely kept on
/// the C++ side, and passed only to the Noder.
[[nodiscard]] inline std::unique_ptr<geos::geom::GeometryCollection>
copy_rust_collection_to_geos(const GeometryCollectionShim& rust_geoms) noexcept
{
    auto factory = geos::geom::GeometryFactory::create();
    auto geos_geoms = get_geos_geoms_from_rust(rust_geoms, factory);
    return factory->createGeometryCollection(std::move(geos_geoms));
}
