#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/wkt-reader.h"

#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <string>

static auto s_logger = log4cplus::Logger::getRoot();

int main(int argc, const char* argv[])
{
    log4cplus::Initializer initializer;
    // A basic ConsoleAppender that logs to stderr.
    auto appender = log4cplus::SharedAppenderPtr(new log4cplus::ConsoleAppender(true, true));
    s_logger.addAppender(appender);

    const CmdlineArgs args = CmdlineArgs::parse_args(argc, argv);
    auto geometries = geom2graph::WKTReader(args.input);

    for (const auto& geometry : geometries)
    {
        LOG4CPLUS_DEBUG(s_logger, "Flattening " << geometry->toString());
        for (const auto& flat_geometry : geom2graph::GeometryFlattener(*geometry))
        {
            args.output << flat_geometry.toString() << "\n";
        }
    }

    return 0;
}
