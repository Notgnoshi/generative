#include <cxxopts.hpp>
#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <fstream>
#include <iostream>
#include <memory>

static auto s_logger = log4cplus::Logger::getRoot();

struct Arguments
{
private:
    // The ifstream needs somewhere to live so that the std::istream& reference remains valid.
    // I honestly can't believe that it's this difficult to transparently switch between reading
    // from a file or stdin.
    std::unique_ptr<std::ifstream> m_input_file;
    std::unique_ptr<std::ofstream> m_output_file;

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
                            << level
                            << "'. Must be one of TRACE, DEBUG, INFO, WARN, ERROR, or FATAL."
                            << "\n"
                            << "Defaulting to WARN");
        return log4cplus::WARN_LOG_LEVEL;
    }

public:
    std::istream& input = std::cin;
    std::ostream& output = std::cout;

    Arguments(const std::string& input_filename, const std::string& output_filename) :
        m_input_file((input_filename.empty() || input_filename == "-")
                         ? nullptr
                         : new std::ifstream(input_filename)),
        m_output_file((output_filename.empty() || output_filename == "-")
                          ? nullptr
                          : new std::ofstream(output_filename)),
        input(m_input_file ? *m_input_file : std::cin),
        output(m_output_file ? *m_output_file : std::cout)
    {
    }

    Arguments() = default;

    static Arguments parse_args(int argc, const char* argv[])
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

            if (result.count("help"))
            {
                std::cout << options.help() << "\n";
                std::exit(0);
            }

            log4cplus::Logger::getRoot().setLogLevel(
                to_log_level(result["log-level"].as<std::string>()));

            const std::string input_filename = result["input"].as<std::string>();
            const std::string output_filename = result["output"].as<std::string>();
            return Arguments(input_filename, output_filename);
        }
        catch (cxxopts::OptionException& e)
        {
            LOG4CPLUS_ERROR(s_logger, "Failed to parse commandline options: " << e.what());
            std::cout << options.help() << "\n";
            std::exit(1);
        }
        return Arguments();
    }
};

int main(int argc, const char* argv[])
{
    log4cplus::Initializer initializer;
    // A basic ConsoleAppender that logs to stderr.
    auto appender = log4cplus::SharedAppenderPtr(new log4cplus::ConsoleAppender(true, true));
    s_logger.addAppender(appender);

    const Arguments args = Arguments::parse_args(argc, argv);

    // Read the geometries.
    std::string line;
    while (std::getline(args.input, line))
    {
        // A pretty dumb program so far, but it demonstrates that IO works.
        args.output << line << "\n";
    }

    return 0;
}
