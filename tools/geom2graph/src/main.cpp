#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/io/wkt-stream-reader.h"
#include "geom2graph/noding/geometry-noder.h"
#include "geom2graph/noding/geometry-graph.h"

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

    auto factory = geos::geom::GeometryFactory::create();
    auto geom_stream = geom2graph::io::WKTStreamReader(args.input, *factory);

    // Collapse the input stream, because it doesn't appear that a geos::noding::Noder can only node
    // a static collection of geometries all at once instead of on-the-fly, which kind of makes
    // sense, but requires loading the entire stream into memory at once.
    LOG4CPLUS_INFO(s_logger, "Loading geometries...");
    auto geometries = geom_stream.collapse();
    const auto collection = factory->createGeometryCollection(std::move(geometries));

    LOG4CPLUS_INFO(s_logger, "Snapping geometries with tolerance " << args.tolerance << "...");
    auto noder = std::make_unique<geos::noding::snap::SnappingNoder>(args.tolerance);
    const std::unique_ptr<geos::geom::Geometry> noded =
        geom2graph::noding::GeometryNoder::node(*collection, std::move(noder));

    if (noded)
    {
        const auto graph = geom2graph::noding::GeometryGraph(*noded);
        const auto& nodes = graph.get_graph();

        //! @todo Move TGF output to geom2graph::io::TGFWriter
        geos::io::WKTWriter writer;
        writer.setTrim(true);
        writer.setOutputDimension(3);

        for (const auto& node : nodes)
        {
            const auto point = std::unique_ptr<geos::geom::Point>(factory->createPoint(node.coord));
            args.output << node.id << "\t" << writer.write(point.get()) << std::endl;
        }
        args.output << "#\n";
        for (const auto& node : nodes)
        {
            for(const auto& idx : node.adjacencies)
            {
                //! @todo Is it worth labeling each edge with a LINESTRING?
                args.output << node.id << "\t" << idx << "\n";
            }
        }
    } else
    {
        LOG4CPLUS_WARN(s_logger, "Failed to snap geometries.");
    }

    return 0;
}
