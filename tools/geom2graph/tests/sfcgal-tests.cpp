#include <SFCGAL/io/wkt.h>
#include <SFCGAL/Geometry.h>

#include <gtest/gtest.h>

TEST(SFCGALTests, TestWkt)
{
    const std::string wkt = "POINT ( 0 1 )";
    //! @todo Do I get sketchy and require the version of SFCGAL that uses unique_ptr instead of auto_ptr?
    //! That's just another way to make dependency installation non-trivial...
    const std::unique_ptr<SFCGAL::Geometry> geometry(SFCGAL::io::readWkt(wkt).release());

    ASSERT_TRUE(geometry);
}
