#include "generative/io/wkt.h"

#include <geos/io/ParseException.h>
#include <geos/io/WKTReader.h>
#include <geos/io/WKTWriter.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

namespace generative::io {

static auto s_logger = log4cplus::Logger::getInstance("generative.io.wkt");

std::unique_ptr<geos::geom::Geometry> from_wkt(const std::string& wkt)
{
    // Will create a new GeometryFactory for every geometry.
    geos::io::WKTReader reader;
    try
    {
        return reader.read(wkt);
    } catch (const geos::io::ParseException& e)
    {
        LOG4CPLUS_WARN(s_logger, "Failed to parse '" << wkt << "' as WKT");
        return nullptr;
    }
}

std::ostream& operator<<(std::ostream& out, const std::unique_ptr<geos::geom::Geometry>& geom)
{
    if (geom)
    {
        return generative::io::operator<<(out, *geom);
    }
    return out;
}

std::ostream& operator<<(std::ostream& out, const std::unique_ptr<geos::geom::Point>& point)
{
    if (point)
    {
        const auto* geom = static_cast<geos::geom::Geometry*>(point.get());
        return generative::io::operator<<(out, *geom);
    }
    return out;
}

std::ostream& operator<<(std::ostream& out, const geos::geom::Geometry& geom)
{
    geos::io::WKTWriter writer;
    writer.setTrim(true);
    writer.setOutputDimension(3);
    return out << writer.write(&geom);
}
}  // namespace generative::io
