#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/io/tgf-graph-writer.h"
#include "geom2graph/io/wkt-stream-reader.h"
#include "geom2graph/noding/geometry-graph.h"
#include "geom2graph/noding/geometry-noder.h"

#include <geos/geom/GeometryCollection.h>
#include <geos/io/WKTWriter.h>
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

    LOG4CPLUS_INFO(s_logger, "Reading geometries...");
    auto factory = geos::geom::GeometryFactory::create();
    auto geom_stream = geom2graph::io::WKTStreamReader(args.input, *factory);

    // Collapse the input stream, because it doesn't appear that a geos::noding::Noder can only node
    // a static collection of geometries all at once instead of on-the-fly, which kind of makes
    // sense, but requires loading the entire stream into memory at once.
    LOG4CPLUS_INFO(s_logger, "Loading geometries...");
    auto geometries = geom_stream.collapse();
    const auto collection = factory->createGeometryCollection(std::move(geometries));

    // Find all intersections, and break geometries into non-intersecting (except at the vertices)
    // linestrings, where vertices sufficiently close together are fuzzily snapped.
    LOG4CPLUS_INFO(s_logger, "Snapping geometries with tolerance " << args.tolerance << "...");
    auto noder = std::make_unique<geos::noding::snap::SnappingNoder>(args.tolerance);
    const auto noded = geom2graph::noding::GeometryNoder::node(*collection, std::move(noder));
    if (!noded)
    {
        LOG4CPLUS_ERROR(s_logger, "Failed to snap geometries.");
        return 1;
    }

    LOG4CPLUS_INFO(s_logger, "Building geometry graph...");
    const auto graph = geom2graph::noding::GeometryGraph(*noded);

    LOG4CPLUS_INFO(s_logger, "Writing geometry graph...");
    //! @todo Read graph output format from commandline arguments.
    std::unique_ptr<geom2graph::io::GraphWriter> writer =
        std::make_unique<geom2graph::io::TGFGraphWriter>(args.output, *factory);
    writer->write(graph);

    return 0;
}
