#include "geom2graph/geometry-flattener.h"

#include <geos/io/ParseException.h>
#include <geos/io/WKTReader.h>

#include <gtest/gtest.h>

static std::unique_ptr<geos::geom::Geometry> from_wkt(const std::string& wkt)
{
    // This creates a new GeometryFactory for every geometry.
    geos::io::WKTReader reader;
    try
    {
        return reader.read(wkt);
    } catch (geos::io::ParseException& e)
    {
        return nullptr;
    }
}

TEST(GeometryFlattenerTests, TestPoint)
{
    const auto geometry = from_wkt("POINT(1 2)");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);

    ++iter;
    EXPECT_EQ(iter, flattener.cend());
    EXPECT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestMultiPoint)
{
    const auto geometry = from_wkt("MULTIPOINT((1 1), (2 2))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(2 2)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestSingleMultiPoint)
{
    const auto geometry = from_wkt("MULTIPOINT((1 1))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestSingleCollection)
{
    const auto geometry = from_wkt("GEOMETRYCOLLECTION(POINT(1 1))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestCollection)
{
    const auto geometry = from_wkt("GEOMETRYCOLLECTION(POINT(1 1), POINT(2 2), POINT(3 3))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(2 2)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(3 3)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

// End the collection with a multi-geometry to test the edge case where you can't mark the root as
// exhausted until you exhaust _all_ of the child.
TEST(GeometryFlattenerTests, TestCollectionWithMultiGeometry)
{
    const auto geometry = from_wkt("GEOMETRYCOLLECTION(POINT(1 1), MULTIPOINT((2 2), (3 3)))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(2 2)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(3 3)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestNestedCollection)
{
    const auto geometry =
        from_wkt("GEOMETRYCOLLECTION(GEOMETRYCOLLECTION(POINT(1 1), POINT(2 2)), POINT(3 3))");
    auto flattener = geom2graph::GeometryFlattener(*geometry);
    auto iter = flattener.cbegin();

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(1 1)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(2 2)").get()));

    ++iter;

    ASSERT_NE(iter, flattener.cend());
    ASSERT_TRUE(iter);
    ASSERT_EQ(iter->getGeometryTypeId(), geos::geom::GeometryTypeId::GEOS_POINT);
    EXPECT_TRUE(iter->equals(from_wkt("POINT(3 3)").get()));

    ++iter;

    ASSERT_EQ(iter, flattener.cend());
    ASSERT_FALSE(iter);
}

TEST(GeometryFlattenerTests, TestDeeplyNestedCollection)
{
    const auto geometry = from_wkt(
        // clang-format off
        "GEOMETRYCOLLECTION("
            "GEOMETRYCOLLECTION("
                "POINT(1 1),"
                "GEOMETRYCOLLECTION("
                    "MULTIPOINT((2 2), (3 3)),"
                    "POINT(4 4)"
                "),"
                "MULTIPOINT((5 5))"
            "),"
            "POINT(6 6),"
            "MULTILINESTRING((7 7, 8 8, 9 9))"
        ")"
        // clang-format on
    );
    const std::array<std::string, 7> expected_wkt = {
        "POINT(1 1)",
        "POINT(2 2)",
        "POINT(3 3)",
        "POINT(4 4)",
        "POINT(5 5)",
        "POINT(6 6)",
        "LINESTRING(7 7, 8 8, 9 9)",
    };
    auto flattener = geom2graph::GeometryFlattener(*geometry);

    size_t e = 0;
    for(const auto& actual : flattener)
    {
        SCOPED_TRACE("Iteration: " + std::to_string(e));
        const auto expected = from_wkt(expected_wkt.at(e));

        ASSERT_EQ(expected->getGeometryType(), actual.getGeometryType());
        ASSERT_TRUE(actual.equals(expected.get()));

        e++;
    }
}
