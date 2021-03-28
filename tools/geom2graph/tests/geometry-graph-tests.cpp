#include "geom2graph/io/wkt.h"
#include "geom2graph/noding/geometry-graph.h"

#include <geos/geom/CoordinateSequence.h>
#include <geos/geom/Geometry.h>

#include <gmock/gmock.h>
#include <gtest/gtest.h>

using namespace ::testing;

TEST(GeometryGraphTests, SingleLinestring)
{
    const auto geometry = geom2graph::io::from_wkt("LINESTRING(0 0, 1 1, 2 2)");
    ASSERT_TRUE(geometry);
    const auto coords = geometry->getCoordinates();
    ASSERT_TRUE(coords);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(3));
    const auto& first = nodes.at(0);
    const auto& second = nodes.at(1);
    const auto& third = nodes.at(2);

    // As an implementation detail, expect the IDs to be generated in order of discovery.
    EXPECT_EQ(*first.point->getCoordinate(), coords->getAt(0));
    EXPECT_EQ(*second.point->getCoordinate(), coords->getAt(1));
    EXPECT_EQ(*third.point->getCoordinate(), coords->getAt(2));

    EXPECT_THAT(first.adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(second.adjacencies, UnorderedElementsAre(0, 2));
    EXPECT_THAT(third.adjacencies, UnorderedElementsAre(1));
}

TEST(GeometryGraphTests, Single3DLinestring)
{
    const auto geometry = geom2graph::io::from_wkt("LINESTRING Z(0 0 0, 1 1 1, 2 2 2)");
    ASSERT_TRUE(geometry);
    const auto coords = geometry->getCoordinates();
    ASSERT_TRUE(coords);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(3));
    const auto& first = nodes.at(0);
    const auto& second = nodes.at(1);
    const auto& third = nodes.at(2);

    EXPECT_EQ(*first.point->getCoordinate(), coords->getAt(0));
    EXPECT_EQ(*second.point->getCoordinate(), coords->getAt(1));
    EXPECT_EQ(*third.point->getCoordinate(), coords->getAt(2));

    EXPECT_THAT(first.adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(second.adjacencies, UnorderedElementsAre(0, 2));
    EXPECT_THAT(third.adjacencies, UnorderedElementsAre(1));
}

TEST(GeometryGraphTests, MayaLineString)
{
    // The first linestring from the maya-tree-2 example.
    const auto geometry = geom2graph::io::from_wkt(
        "LINESTRING Z (0 0 0, 0 0 1, 0 -0.7071067811865476 1.707106781186547)");
    ASSERT_TRUE(geometry);
    ASSERT_EQ(geometry->getNumPoints(), 3);
    ASSERT_EQ(geometry->getCoordinateDimension(), 3);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(3));
}

TEST(GeometryGraphTests, DifferInZCoord)
{
    const auto geometry = geom2graph::io::from_wkt("LINESTRING Z (0 0 1, 0 0 2, 0 0 3)");
    ASSERT_TRUE(geometry);
    ASSERT_EQ(geometry->getNumPoints(), 3);
    ASSERT_EQ(geometry->getCoordinateDimension(), 3);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(3));
}

TEST(GeometryGraphTests, ClosedPolygon)
{
    const auto geometry = geom2graph::io::from_wkt("POLYGON((0 0, 1 1, 2 2, 0 0))");
    ASSERT_TRUE(geometry);
    const auto coords = geometry->getCoordinates();
    ASSERT_TRUE(coords);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(3));

    const auto& first = nodes.at(0);
    const auto& second = nodes.at(1);
    const auto& third = nodes.at(2);

    // As an implementation detail, expect the IDs to be generated in order of discovery.
    EXPECT_EQ(*first.point->getCoordinate(), coords->getAt(0));
    EXPECT_EQ(*second.point->getCoordinate(), coords->getAt(1));
    EXPECT_EQ(*third.point->getCoordinate(), coords->getAt(2));

    EXPECT_THAT(first.adjacencies, UnorderedElementsAre(1, 2));
    EXPECT_THAT(second.adjacencies, UnorderedElementsAre(0, 2));
    EXPECT_THAT(third.adjacencies, UnorderedElementsAre(1, 0));
}

TEST(GeometryGraphTests, DisjointMultiLinestring)
{
    const auto geometry = geom2graph::io::from_wkt(
        // clang-format off
        "MULTILINESTRING("
            "(0 0, 1 0, 2 0),"
            "(0 1, 1 1, 2 1)" // Disjoint from the one above.
        ")"
        // clang-format on
    );
    ASSERT_TRUE(geometry);
    const auto coords = geometry->getCoordinates();
    ASSERT_TRUE(coords);
    ASSERT_THAT(coords, Pointee(SizeIs(6)));

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    ASSERT_THAT(nodes, SizeIs(6));
    // As an implementation detail, expect the IDs to be generated in order of discovery.
    EXPECT_EQ(*nodes.at(0).point->getCoordinate(), coords->getAt(0));
    EXPECT_EQ(*nodes.at(1).point->getCoordinate(), coords->getAt(1));
    EXPECT_EQ(*nodes.at(2).point->getCoordinate(), coords->getAt(2));
    EXPECT_EQ(*nodes.at(3).point->getCoordinate(), coords->getAt(3));
    EXPECT_EQ(*nodes.at(4).point->getCoordinate(), coords->getAt(4));
    EXPECT_EQ(*nodes.at(5).point->getCoordinate(), coords->getAt(5));

    EXPECT_THAT(nodes.at(0).adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(nodes.at(1).adjacencies, UnorderedElementsAre(0, 2));
    EXPECT_THAT(nodes.at(2).adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(nodes.at(3).adjacencies, UnorderedElementsAre(4));
    EXPECT_THAT(nodes.at(4).adjacencies, UnorderedElementsAre(3, 5));
    EXPECT_THAT(nodes.at(5).adjacencies, UnorderedElementsAre(4));
}

TEST(GeometryGraphTests, MultiLinestring)
{
    const auto geometry = geom2graph::io::from_wkt(
        // clang-format off
        "MULTILINESTRING("
            "(0 0, 1 0, 2 0),"       // 0 1 2
            "(1 0, 2 0.00001, 3 3)"  // 1 3 4
        ")"
        // clang-format on
    );
    ASSERT_TRUE(geometry);
    const auto coords = geometry->getCoordinates();
    ASSERT_TRUE(coords);
    ASSERT_THAT(coords, Pointee(SizeIs(6)));

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    // The two geometries share a point, so the numbers of points don't match.
    ASSERT_THAT(nodes, SizeIs(5));
    // As an implementation detail, expect the IDs to be generated in order of discovery.
    EXPECT_EQ(*nodes.at(0).point->getCoordinate(), coords->getAt(0));
    EXPECT_EQ(*nodes.at(1).point->getCoordinate(), coords->getAt(1));
    EXPECT_EQ(*nodes.at(2).point->getCoordinate(), coords->getAt(2));
    EXPECT_EQ(*nodes.at(3).point->getCoordinate(),
              coords->getAt(4));  // Skipping 3, because it's a duplicate!
    EXPECT_EQ(*nodes.at(4).point->getCoordinate(), coords->getAt(5));

    EXPECT_THAT(nodes.at(0).adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(nodes.at(1).adjacencies, UnorderedElementsAre(0, 2, 3));
    EXPECT_THAT(nodes.at(2).adjacencies, UnorderedElementsAre(1));
    EXPECT_THAT(nodes.at(3).adjacencies, UnorderedElementsAre(1, 4));
    EXPECT_THAT(nodes.at(4).adjacencies, UnorderedElementsAre(3));
}

TEST(GeometryGraphTests, LonePoint)
{
    const auto geometry = geom2graph::io::from_wkt("POINT(0 0)");
    ASSERT_TRUE(geometry);

    const auto graph = geom2graph::noding::GeometryGraph(*geometry);
    const auto& nodes = graph.get_nodes();

    // Point isn't added to the graph, because we iterate over the coordinates of a geometry
    // pairwise.
    ASSERT_THAT(nodes, SizeIs(0));
}
