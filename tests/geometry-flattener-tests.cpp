#include "generative/geometry-flattener.h"
#include "generative/io/wkt.h"

#include <geos/io/ParseException.h>
#include <geos/io/WKTReader.h>

#include <gtest/gtest.h>

using generative::io::from_wkt;

TEST(GeometryFlattenerTests, TestPoint)
{
    const auto geometry = from_wkt("POINT(1 2)");
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);
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
    auto flattener = generative::GeometryFlattener(*geometry);

    size_t e = 0;
    for (const auto& actual : flattener)
    {
        SCOPED_TRACE("Iteration: " + std::to_string(e));
        const auto expected = from_wkt(expected_wkt.at(e));

        ASSERT_EQ(expected->getGeometryType(), actual.getGeometryType());
        ASSERT_TRUE(actual.equals(expected.get()));

        e++;
    }
}

TEST(GeometryFlattenerTests, RecursiveIteratorEquality)
{
    // The multipoint will have a child iterator.
    const auto geometry = from_wkt("GEOMETRYCOLLECTION(MULTIPOINT(1 1, 2 2), POINT(3 3))");
    ASSERT_TRUE(geometry);
    auto flattener = generative::GeometryFlattener(*geometry);
    const auto end = flattener.end();
    auto iter1 = flattener.begin();
    auto iter2 = flattener.begin();

    ASSERT_NE(iter1, end);
    ASSERT_EQ(iter1, iter2);

    ++iter1;
    EXPECT_NE(iter1, iter2);

    ++iter2;
    EXPECT_EQ(iter1, iter2);

    ++iter1;
    ++iter2;
    EXPECT_EQ(iter1, iter2);

    ++iter1;
    ++iter2;
    EXPECT_EQ(iter1, iter2);
    EXPECT_EQ(iter1, end);
}

TEST(GeometryFlattenerTests, DeeplyRecursiveIteratorEquality)
{
    const auto geometry = from_wkt(
        // clang-format off
        "GEOMETRYCOLLECTION("
            "GEOMETRYCOLLECTION("
                "POINT(1 1),"                    // 0
                "GEOMETRYCOLLECTION("
                    "MULTIPOINT((2 2), (3 3)),"  // 1, 2
                    "POINT(4 4)"                 // 3
                "),"
                "MULTIPOINT((5 5))"              // 4
            "),"
            "POINT(6 6),"                        // 5
            "MULTILINESTRING((7 7, 8 8, 9 9))"   // 6, 7, 8
        ")"
        // clang-format on
    );
    ASSERT_TRUE(geometry);

    auto flattener = generative::GeometryFlattener(*geometry);
    const auto end = flattener.end();

    auto iter1 = flattener.begin();
    auto iter2 = flattener.begin();

    ASSERT_NE(iter1, end);
    ASSERT_EQ(iter1, iter2);

    ++iter1;
    ASSERT_NE(iter1, end);
    EXPECT_NE(iter1, iter2);

    ++iter1;
    ASSERT_NE(iter1, end);
    EXPECT_NE(iter1, iter2);

    ++iter1;
    ASSERT_NE(iter1, end);
    EXPECT_NE(iter1, iter2);

    std::advance(iter2, 3);
    ASSERT_NE(iter2, end);
    EXPECT_EQ(iter1, iter2);

    std::advance(iter2, 3);
    ASSERT_NE(iter2, end);
    EXPECT_NE(iter1, iter2);

    std::advance(iter1, 3);
    ASSERT_NE(iter1, end);
    EXPECT_EQ(iter1, iter2);

    std::advance(iter1, 1);
    std::advance(iter2, 1);

    EXPECT_EQ(iter1, end);
    EXPECT_EQ(iter1, iter2);
}
