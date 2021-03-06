#include "geom2graph/io/wkt.h"
#include "geom2graph/noding/geometry-graph.h"
#include "geom2graph/noding/geometry-noder.h"

#include <geos/geom/Geometry.h>
#include <geos/noding/Noder.h>
#include <geos/operation/polygonize/Polygonizer.h>

#include <algorithm>

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;

TEST(PolygonizerTests, SimplePolygon)
{
    const auto geometry = geom2graph::io::from_wkt("POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))");
    ASSERT_TRUE(geometry);
    const auto noded = geom2graph::noding::GeometryNoder::node(*geometry);
    const auto graph = geom2graph::noding::GeometryGraph(*noded);

    const auto owned_edges = graph.get_edges();
    std::vector<const geos::geom::Geometry*> edges;
    std::transform(
        owned_edges.begin(),
        owned_edges.end(),
        std::back_inserter(edges),
        [](const std::unique_ptr<geos::geom::LineString>& edge) -> const geos::geom::Geometry* {
            return edge.get();
        });

    auto polygonizer = geos::operation::polygonize::Polygonizer();
    polygonizer.add(&edges);

    const auto polys = polygonizer.getPolygons();
    const auto dangles = polygonizer.getDangles();

    EXPECT_THAT(polys, SizeIs(1));
    EXPECT_THAT(dangles, SizeIs(0));

    EXPECT_TRUE(polys.at(0)->equals(geometry.get()));
}

TEST(PolygonizerTests, SimplePolygonWithDangles)
{
    const auto geometry = geom2graph::io::from_wkt(
        // clang-format off
            "GEOMETRYCOLLECTION("
                "POLYGON((0 0, 0 1, 1 1, 1 0, 0 0)),"
                "LINESTRING(0.5 0.5, 1.5 0.5)"
            ")"
        // clang-format on
    );
    ASSERT_TRUE(geometry);
    const auto noded = geom2graph::noding::GeometryNoder::node(*geometry);
    const auto graph = geom2graph::noding::GeometryGraph(*noded);

    const auto owned_edges = graph.get_edges();
    std::vector<const geos::geom::Geometry*> edges;
    std::transform(
        owned_edges.begin(),
        owned_edges.end(),
        std::back_inserter(edges),
        [](const std::unique_ptr<geos::geom::LineString>& edge) -> const geos::geom::Geometry* {
            return edge.get();
        });

    auto polygonizer = geos::operation::polygonize::Polygonizer();
    polygonizer.add(&edges);

    const auto polys = polygonizer.getPolygons();
    const auto dangles = polygonizer.getDangles();

    EXPECT_THAT(polys, SizeIs(1));
    EXPECT_THAT(dangles, SizeIs(2));

    const auto poly = geom2graph::io::from_wkt("POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))");
    const auto dangle1 = geom2graph::io::from_wkt("LINESTRING(1 0.5, 1.5 0.5)");
    const auto dangle2 = geom2graph::io::from_wkt("LINESTRING(1 0.5, 0.5 0.5)");

    EXPECT_TRUE(polys.at(0)->equals(poly.get()));
    EXPECT_TRUE(dangles.at(0)->equals(dangle1.get()));
    EXPECT_TRUE(dangles.at(1)->equals(dangle2.get()));
}
