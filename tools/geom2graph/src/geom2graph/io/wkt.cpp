#include "geom2graph/io/wkt.h"

#include <geos/io/ParseException.h>
#include <geos/io/WKTReader.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

namespace geom2graph::io {

static auto s_logger = log4cplus::Logger::getInstance("geom2graph.io.wkt");

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
}  // namespace geom2graph::io
