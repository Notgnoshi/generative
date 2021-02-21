#include "cmdline.h"

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

    // Read the geometries.
    std::string line;
    while (std::getline(args.input, line))
    {
        // A pretty dumb program so far, but it demonstrates that IO works.
        args.output << line << "\n";
    }

    return 0;
}
