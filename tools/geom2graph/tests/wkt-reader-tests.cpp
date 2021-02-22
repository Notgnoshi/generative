#include "geom2graph/wkt-reader.h"

#include <geos/io/ParseException.h>
#include <geos/io/WKTReader.h>

#include <array>
#include <sstream>

#include <gtest/gtest.h>

std::unique_ptr<geos::geom::Geometry> from_wkt(const std::string& wkt)
{
    // This creates a new GeometryFactory for every geometry.
    geos::io::WKTReader reader;
    try
    {
        return reader.read(wkt);
    }
    catch (geos::io::ParseException& e)
    {
        return nullptr;
    }
}

TEST(WktReaderTests, TestEmptyString)
{
    std::istringstream input_stream{""};
    geom2graph::WKTReader geometries(input_stream);
    auto iter = geometries.begin();
    // You should not be able to get a valid iterator to this stream.
    ASSERT_EQ(iter, geometries.end())
        << "Expected to not get a valid GeometryIterator from an empty stream";
}

TEST(WktReaderTests, TestIteratorSanity)
{
    const std::array<int, 3> values = {1, 2, 3};
    auto iter = values.cbegin();

    EXPECT_NE(iter, values.cend());
    ++iter;
    EXPECT_NE(iter, values.cend());
    ++iter;
    EXPECT_NE(iter, values.cend());
    ++iter;
    EXPECT_EQ(iter, values.cend());
}

TEST(WktReaderTests, TestIteratorEnd)
{
    std::istringstream input_stream{"POINT(1 1)\nPOINT(2 2)\nPOINT(3 3)"};
    geom2graph::WKTReader geometries(input_stream);
    auto iter = geometries.begin();

    EXPECT_NE(iter, geometries.end());
    ++iter;
    EXPECT_NE(iter, geometries.end());
    ++iter;
    EXPECT_NE(iter, geometries.end()) << "Expected the end() iterator to be exclusive";
    ++iter;
    EXPECT_EQ(iter, geometries.end());
}

TEST(WktReaderTests, TestSingleElement)
{
    std::istringstream input_stream{"POINT(0 0)"};
    geom2graph::WKTReader geometries(input_stream);
    auto iter = geometries.begin();

    ASSERT_NE(*iter, nullptr);
    const auto expected = from_wkt("POINT(0 0)");
    ASSERT_NE(expected, nullptr);
    EXPECT_TRUE((*iter)->equalsExact(expected.get()));

    EXPECT_NE(iter, geometries.end());
    ++iter;
    EXPECT_EQ(iter, geometries.end());
}

TEST(WktReaderTests, TestMultipleElements)
{
    std::istringstream input_stream{"POINT(0 0)\nPOINT(1 1)"};
    geom2graph::WKTReader geometries(input_stream);
    auto iter = geometries.begin();
    ASSERT_NE(iter, geometries.end());
    ASSERT_NE(*iter, nullptr);
    EXPECT_TRUE((*iter)->equalsExact(from_wkt("POINT(0 0)").get()));

    ++iter;
    ASSERT_NE(iter, geometries.end()) << "Expected the end iterator to be exclusive";
    ASSERT_NE(*iter, nullptr);
    EXPECT_TRUE((**iter).equalsExact(from_wkt("POINT(1 1)").get()));

    ++iter;
    EXPECT_EQ(iter, geometries.end());
}

TEST(WktReaderTests, TestRangeLoop)
{
    std::istringstream input_stream{"POINT(0 0)\nPOINT(1 2)"};
    std::array<std::string, 2> expected{"POINT(0 0)", "POINT(1 2)"};
    geom2graph::WKTReader geometries(input_stream);

    size_t e = 0;
    for (auto& geometry : geometries)
    {
        SCOPED_TRACE("Iteration: " + std::to_string(e));
        EXPECT_TRUE(geometry->equalsExact(from_wkt(expected.at(e)).get()));

        e++;
    }
    EXPECT_EQ(e, expected.size()) << "Expected to loop over " << expected.size() << " geometries";
}

TEST(GeosWKTReaderTests, TestValidGeom)
{
    const std::string wkt = "POINT(1 1)";
    geos::io::WKTReader reader;

    std::unique_ptr<geos::geom::Geometry> geometry = reader.read(wkt);
    ASSERT_TRUE(geometry) << "Expected non-null geometry";

    auto type = geometry->getGeometryTypeId();
    EXPECT_EQ(type, geos::geom::GeometryTypeId::GEOS_POINT);
}

TEST(GeosWKTReaderTests, TestInvalidGeom)
{
    const std::string wkt = "asdf";
    geos::io::WKTReader reader;

    ASSERT_THROW(reader.read(wkt), geos::io::ParseException);
}

TEST(GeosWKTReaderTests, TestLifetime)
{
    const std::string wkt = "POINT(0 0)";
    std::unique_ptr<geos::geom::Geometry> geometry;
    {
        geos::io::WKTReader reader;
        geometry = reader.read(wkt);
    }

    // Try to use the geometry factory after the reader goes out of scope to test that it remains
    // valid. This was prompted by documentation for one of the WKTReader constructors that takes in
    // a GeoemtryFactory, and notes that you should ensure it's valid for the lifetime of the reader
    // and the geometries.
    ASSERT_TRUE(geometry);
    const auto* factory = geometry->getFactory();
    ASSERT_NE(factory, nullptr);
    auto* geom_raw_ptr = geometry.release();
    factory->destroyGeometry(geom_raw_ptr);
}

// This is a case that comes up frequently at work, because it's easiest to generate WKT with a
// trailing comma.
TEST(GeosWKTReaderTests, TestTrailingComma)
{
    auto geometry = from_wkt("LINESTRING(0 0, 1 1, 2 2)");
    ASSERT_NE(geometry, nullptr);
    geometry = from_wkt("LINESTRING(0 0, 1 1, 2 2,)");
    ASSERT_EQ(geometry, nullptr);
}
