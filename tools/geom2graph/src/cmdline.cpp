#include "cmdline.h"

#include <cxxopts.hpp>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

static auto s_logger = log4cplus::Logger::getInstance("cmdline");
static log4cplus::LogLevel to_log_level(const std::string& level)
{
    if (level == "TRACE")
    {
        return log4cplus::TRACE_LOG_LEVEL;
    }
    if (level == "DEBUG")
    {
        return log4cplus::DEBUG_LOG_LEVEL;
    }
    if (level == "INFO")
    {
        return log4cplus::INFO_LOG_LEVEL;
    }
    if (level == "WARN")
    {
        return log4cplus::WARN_LOG_LEVEL;
    }
    if (level == "ERROR")
    {
        return log4cplus::ERROR_LOG_LEVEL;
    }
    if (level == "FATAL")
    {
        return log4cplus::FATAL_LOG_LEVEL;
    }
    LOG4CPLUS_ERROR(s_logger,
                    "Unknown log level: '"
                        << level << "'. Must be one of TRACE, DEBUG, INFO, WARN, ERROR, or FATAL."
                        << "\n"
                        << "Defaulting to WARN");
    return log4cplus::WARN_LOG_LEVEL;
}

CmdlineArgs CmdlineArgs::parse_args(int argc, const char* argv[])
{
    cxxopts::Options options(
        "geom2graph",
        "A C++ CLI application to convert a set of WKT geometries to a graph representation.");
    // clang-format off
        options.add_options()
            ("h,help",      "show this help message and exit")
            ("l,log-level", "TRACE, DEBUG, INFO, WARN, ERROR, or FATAL.", cxxopts::value<std::string>()->default_value("WARN"))
            ("i,input",     "File to read WKT geometries from",           cxxopts::value<std::string>()->default_value("-"))
            ("o,output",    "File to write graph to",                     cxxopts::value<std::string>()->default_value("-"))
        ;
    // clang-format on

    try
    {
        auto result = options.parse(argc, argv);

        if (result.count("help") > 0)
        {
            std::cout << options.help() << "\n";
            std::exit(0);
        }

        log4cplus::Logger::getRoot().setLogLevel(
            to_log_level(result["log-level"].as<std::string>()));

        const std::string input_filename = result["input"].as<std::string>();
        const std::string output_filename = result["output"].as<std::string>();
        return CmdlineArgs(input_filename, output_filename);
    }
    catch (cxxopts::OptionException& e)
    {
        LOG4CPLUS_ERROR(s_logger, "Failed to parse commandline options: " << e.what());
        std::cout << options.help() << "\n";
        std::exit(1);
    }
    return CmdlineArgs();
}
