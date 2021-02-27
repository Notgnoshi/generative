#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/io/wkt-stream-reader.h"
#include "geom2graph/noding/geometry-noder.h"

#include <geos/geom/GeometryCollection.h>
#include <geos/noding/snap/SnappingNoder.h>
#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <algorithm>
#include <string>

static auto s_logger = log4cplus::Logger::getRoot();

int main(int argc, const char* argv[])
{
    log4cplus::Initializer initializer;
    // A basic ConsoleAppender that logs to stderr.
    auto appender = log4cplus::SharedAppenderPtr(new log4cplus::ConsoleAppender(true, true));
    s_logger.addAppender(appender);

    const CmdlineArgs args = CmdlineArgs::parse_args(argc, argv);

    auto factory = geos::geom::GeometryFactory::create();
    auto geom_stream = geom2graph::io::WKTStreamReader(args.input, *factory);

    // Collapse the input stream, because it doesn't appear that a geos::noding::Noder can only node
    // a static collection of geometries all at once instead of on-the-fly, which kind of makes
    // sense, but requires loading the entire stream into memory at once.
    LOG4CPLUS_INFO(s_logger, "Loading geometries...");
    auto geometries = geom_stream.collapse();
    const auto collection = factory->createGeometryCollection(std::move(geometries));

    LOG4CPLUS_INFO(s_logger, "Snapping geometries...");
    //! @todo Read from commandline args.
    const double tolerance = 0.01;
    auto noder = std::make_unique<geos::noding::snap::SnappingNoder>(tolerance);
    const std::unique_ptr<geos::geom::Geometry> noded =
        geom2graph::noding::GeometryNoder::node(*collection, std::move(noder));

    if (noded)
    {
        LOG4CPLUS_INFO(s_logger, "Processing snapped geometries...");
        // The noding should also return a MULTILINESTRING, which doesn't need to be _recursively_
        // flattened, but that's what I implemented, so that's what I'm going to use.
        auto flattener = geom2graph::GeometryFlattener(*noded);

        for (const auto& geometry : flattener)
        {
            LOG4CPLUS_TRACE(s_logger, "Snapped geometry: " << geometry.toString());
            args.output << geometry.toString() << std::endl;
        }
    } else
    {
        LOG4CPLUS_WARN(s_logger, "Failed to snap geometries.");
    }

    return 0;
}
