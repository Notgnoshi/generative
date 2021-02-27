#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/io/wkt-stream-reader.h"

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
    auto geom_stream = geom2graph::io::WKTStreamReader(args.input);

    // Collapse the input stream, because it doesn't appear that a geos::noding::Noder can only node
    // a static collection of geometries all at once instead of on-the-fly, which kind of makes
    // sense, but requires loading the entire stream into memory at once.
    const auto geometries = geom_stream.collapse();

    for (const auto& geometry : geometries)
    {
        //! @todo The noding process doesn't require flattened geometries.
        LOG4CPLUS_DEBUG(s_logger, "Flattening " << geometry->toString());
        for (const auto& flat_geometry : geom2graph::GeometryFlattener(*geometry))
        {
            args.output << flat_geometry.toString() << "\n";
        }
    }

    return 0;
}
