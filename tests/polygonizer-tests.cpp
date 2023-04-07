#include "generative/io/wkt.h"
#include "generative/noding/geometry-graph.h"
#include "generative/noding/geometry-noder.h"

#include <geos/geom/Geometry.h>
#include <geos/noding/Noder.h>
#include <geos/operation/polygonize/Polygonizer.h>

#include <algorithm>

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;

TEST(PolygonizerTests, SimplePolygon)
{
    const auto geometry = generative::io::from_wkt("POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))");
    ASSERT_TRUE(geometry);
    const auto noded = generative::noding::GeometryNoder::node(*geometry);
    const auto graph = generative::noding::GeometryGraph(*noded);

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
    const auto geometry = generative::io::from_wkt(
        // clang-format off
            "GEOMETRYCOLLECTION("
                "POLYGON((0 0, 0 1, 1 1, 1 0, 0 0)),"
                "LINESTRING(0.5 0.5, 1.5 0.5)"
            ")"
        // clang-format on
    );
    ASSERT_TRUE(geometry);
    const auto noded = generative::noding::GeometryNoder::node(*geometry);
    const auto graph = generative::noding::GeometryGraph(*noded);

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

    const auto poly = generative::io::from_wkt("POLYGON((0 0, 0 1, 1 1, 1 0, 0 0))");
    const auto dangle1 = generative::io::from_wkt("LINESTRING(1 0.5, 1.5 0.5)");
    const auto dangle2 = generative::io::from_wkt("LINESTRING(1 0.5, 0.5 0.5)");

    EXPECT_TRUE(polys.at(0)->equals(poly.get()));
    EXPECT_TRUE(dangles.at(0)->equals(dangle1.get()));
    EXPECT_TRUE(dangles.at(1)->equals(dangle2.get()));
}

TEST(PolygonizerTests, DISABLED_MissingData)
{
    const auto* factory = geos::geom::GeometryFactory::getDefaultInstance();
    auto graph = generative::noding::GeometryGraph(*factory);

    std::vector<generative::noding::GeometryGraph::Node> nodes;
    nodes.emplace_back(
        0, std::unique_ptr<geos::geom::Point>(factory->createPoint(geos::geom::Coordinate(0, 0))));
    nodes.emplace_back(
        1, std::unique_ptr<geos::geom::Point>(factory->createPoint(geos::geom::Coordinate(1, 0))));
    nodes.emplace_back(
        2, std::unique_ptr<geos::geom::Point>(factory->createPoint(geos::geom::Coordinate(2, 0))));
    nodes.emplace_back(
        3, std::unique_ptr<geos::geom::Point>(factory->createPoint(geos::geom::Coordinate(0, 1))));
    nodes.emplace_back(
        4, std::unique_ptr<geos::geom::Point>(factory->createPoint(geos::geom::Coordinate(1, 1))));
    graph.set_nodes(std::move(nodes));

    graph.add_edge(0, 1);
    // TODO: This is the culprit - I think this edge is incorrectly noded.
    graph.add_edge(0, 2);
    graph.add_edge(1, 2);
    graph.add_edge(1, 3);
    graph.add_edge(2, 4);
    graph.add_edge(3, 4);

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

    ASSERT_FALSE(polygonizer.hasInvalidRingLines());

    const auto polys = polygonizer.getPolygons();
    const auto dangles = polygonizer.getDangles();

    EXPECT_THAT(polys, SizeIs(1));
    EXPECT_THAT(dangles, SizeIs(1));
}
