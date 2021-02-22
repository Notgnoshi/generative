#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <gmock/gmock.h>
#include <gtest/gtest.h>

static auto s_logger = log4cplus::Logger::getRoot();

int main(int argc, char* argv[])
{
    log4cplus::Initializer initializer;
    // A basic ConsoleAppender that logs to stderr.
    auto appender = log4cplus::SharedAppenderPtr(new log4cplus::ConsoleAppender(true, true));
    s_logger.setLogLevel(log4cplus::ERROR_LOG_LEVEL);
    s_logger.addAppender(appender);

    //! @todo I don't know if I need to initialize both?
    // testing::InitGoogleTest(&argc, argv);
    testing::InitGoogleMock(&argc, argv);

    // Sure, this is terrible, but it's also for the tests, and is hopefully not used much.
    for (int i = 0; i < argc; i++)
    {
        const std::string arg(argv[i]);
        if (arg == "-l" || arg == "--log-level")
        {
            i++;
            if (i >= argc)
            {
                LOG4CPLUS_FATAL(s_logger, "--log-level requires an argument");
                std::exit(1);
            }
            const std::string log_level = argv[i];
            if (log_level == "TRACE")
            {
                s_logger.setLogLevel(log4cplus::TRACE_LOG_LEVEL);
            } else if (log_level == "DEBUG")
            {
                s_logger.setLogLevel(log4cplus::DEBUG_LOG_LEVEL);
            } else if (log_level == "INFO")
            {
                s_logger.setLogLevel(log4cplus::INFO_LOG_LEVEL);

            } else if (log_level == "WARN")
            {
                s_logger.setLogLevel(log4cplus::WARN_LOG_LEVEL);

            } else if (log_level == "ERROR")
            {
                s_logger.setLogLevel(log4cplus::ERROR_LOG_LEVEL);

            } else if (log_level == "FATAL")
            {
                s_logger.setLogLevel(log4cplus::FATAL_LOG_LEVEL);
            } else
            {
                LOG4CPLUS_ERROR(s_logger, "Unknown log level '" << log_level << "'");
                std::exit(1);
            }
        }
    }

    return RUN_ALL_TESTS();
}
