#include "geom2graph/io/wkt.h"
#include "geom2graph/noding/geometry-noder.h"

#include <geos/geom/Geometry.h>

#include <gtest/gtest.h>

using geom2graph::io::from_wkt;

TEST(GeometryNoderTests, DisjointPoint)
{
    const std::unique_ptr<geos::geom::Geometry> geometries = from_wkt(
        // clang-format off
        "GEOMETRYCOLLECTION("
            "LINESTRING(0 0, 2 2),"
            "POINT(1 1)," // midpoint of line
            "POINT(0 1)"  // not on line
        ")"
        // clang-format on
    );
    ASSERT_TRUE(geometries);
    // Noding ignores POINTs, even if they're on an existing edge.
    const std::unique_ptr<geos::geom::Geometry> expected = from_wkt("MULTILINESTRING((0 0, 2 2))");
    ASSERT_TRUE(expected);

    const std::unique_ptr<geos::geom::Geometry> noded = geom2graph::noding::GeometryNoder::node(*geometries);
    ASSERT_TRUE(noded);
    // std::cerr << noded->toString() << std::endl;

    EXPECT_EQ(noded->getGeometryType(), expected->getGeometryType());
    EXPECT_TRUE(noded->equals(expected.get()));
}

TEST(GeometryNoderTests, SimpleRectangle)
{
    const std::unique_ptr<geos::geom::Geometry> rectangle = from_wkt(
        // clang-format off
        "GEOMETRYCOLLECTION("
            "LINESTRING(2 0, 2 8),"  // left vertical
            "LINESTRING(6 0, 6 8),"  // right vertical
            "LINESTRING(0 2, 8 2),"  // bottom horizontal
            "LINESTRING(0 6, 8 6)"  // top horizontal
        ")"
        // clang-format on
    );
    const std::unique_ptr<geos::geom::Geometry> expected = from_wkt(
        // clang-format off
        "MULTILINESTRING ("
            "(2 0, 2 2),"
            "(2 2, 2 6),"
            "(2 6, 2 8),"
            "(6 0, 6 2),"
            "(6 2, 6 6),"
            "(6 6, 6 8),"
            "(0 2, 2 2),"
            "(2 2, 6 2),"
            "(6 2, 8 2),"
            "(0 6, 2 6),"
            "(2 6, 6 6),"
            "(6 6, 8 6)"
        ")"
        // clang-format on
    );

    ASSERT_TRUE(rectangle);
    //! @todo This crashes if the provided rectangle is null. Fix.
    const std::unique_ptr<geos::geom::Geometry> noded =
        geom2graph::noding::GeometryNoder::node(*rectangle);

    ASSERT_TRUE(noded);
    // std::cerr << noded->toString() << std::endl;

    EXPECT_EQ(noded->getGeometryType(), expected->getGeometryType());
    EXPECT_TRUE(noded->equals(expected.get()));
}

TEST(GeometryNoderTests, ProvideOwnNoder)
{
    //! @todo Allow providing your own noder.
}

TEST(GeometryNoderTests, SnappingNoder)
{
    //! @todo Try out the snapping noder to see how it works
}

TEST(GeometryNoderTests, SnapRoundingNoder)
{
    //! @todo Try out the snap-rounding noder, to see how it's different from the snapping noder.
}

//! @todo Use google benchmark.
TEST(GeometryNoderTests, DISABLED_SnappingNoderPerformance)
{
}

TEST(GeometryNoderTests, DISABLED_SnapRoundingNoderPerformance)
{
}
