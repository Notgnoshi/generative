#include "generative/geometry-flattener.h"
#include "generative/io/tgf-graph-reader.h"
#include "generative/io/tgf-graph-writer.h"
#include "generative/io/wkt-stream-reader.h"
#include "generative/io/wkt.h"
#include "generative/noding/geometry-graph.h"
#include "generative/noding/geometry-noder.h"

#include <cxxopts.hpp>
#include <geos/geom/GeometryCollection.h>
#include <geos/io/WKTWriter.h>
#include <geos/noding/snap/SnappingNoder.h>
#include <geos/operation/polygonize/Polygonizer.h>
#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <algorithm>
#include <fstream>
#include <iostream>
#include <memory>
#include <string>

using generative::io::operator<<;

static auto s_logger = log4cplus::Logger::getRoot();

struct CmdlineArgs
{
private:
    // The ifstream needs somewhere to live so that the std::istream& reference remains valid.
    // I honestly can't believe that it's this difficult to transparently switch between reading
    // from a file or stdin.
    std::unique_ptr<std::ifstream> m_input_file;
    std::unique_ptr<std::ofstream> m_output_file;

public:
    enum class GraphFormat
    {
        TGF,
    };

    std::istream& input = std::cin;
    std::ostream& output = std::cout;
    //! @brief The snapping tolerance in the graph generation.
    double tolerance = 0.0;
    GraphFormat graph_format = GraphFormat::TGF;
    //! @brief Indicates we're reading a previouslly generated graph and converting it back to geoms
    bool graph2geom = false;

    CmdlineArgs(const std::string& input_filename, const std::string& output_filename) :
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

    CmdlineArgs() = default;
    static CmdlineArgs parse_args(int argc, const char* argv[]);
};

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

static CmdlineArgs::GraphFormat to_graph_format(std::string format)
{
    std::transform(format.begin(), format.end(), format.begin(), [](char c) {
        return std::tolower(c);
    });
    if (format == "tgf")
    {
        return CmdlineArgs::GraphFormat::TGF;
    }

    throw cxxopts::exceptions::incorrect_argument_type("Unknown graph format: " + format);
}

CmdlineArgs CmdlineArgs::parse_args(int argc, const char* argv[])
{
    cxxopts::Options options(
        "geom2graph",
        "A C++ CLI application to convert a set of WKT geometries to a graph representation.");
    // clang-format off
        options.add_options()
            ("h,help",         "show this help message and exit")
            ("l,log-level",    "TRACE, DEBUG, INFO, WARN, ERROR, or FATAL.", cxxopts::value<std::string>()->default_value("WARN"))
            ("i,input",        "File to read WKT geometries from",           cxxopts::value<std::string>()->default_value("-"))
            ("o,output",       "File to write graph to",                     cxxopts::value<std::string>()->default_value("-"))
            ("t,tolerance",    "Vertex snapping tolerance",                  cxxopts::value<double>()->default_value("0.00001"))
            ("f,graph-format", "The graph format",                           cxxopts::value<std::string>()->default_value("tgf"))
            ("graph2geom",     "Whether to convert an existing graph back to geometries")
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
        const double tolerance = result["tolerance"].as<double>();
        const std::string graph_format = result["graph-format"].as<std::string>();
        const bool graph2geom = result.count("graph2geom") > 0;

        auto args = CmdlineArgs(input_filename, output_filename);
        args.tolerance = tolerance;
        args.graph_format = to_graph_format(graph_format);
        args.graph2geom = graph2geom;

        return args;
    } catch (cxxopts::exceptions::exception& e)
    {
        LOG4CPLUS_ERROR(s_logger, "Failed to parse commandline options: " << e.what());
        std::cout << options.help() << "\n";
        std::exit(1);
    }
    return CmdlineArgs();
}

static int _geom2graph(const CmdlineArgs& args)
{
    LOG4CPLUS_INFO(s_logger, "Reading geometries...");
    auto factory = geos::geom::GeometryFactory::create();
    auto geom_stream = generative::io::WKTStreamReader(args.input, *factory);

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
    const auto noded = generative::noding::GeometryNoder::node(*collection, std::move(noder));
    if (!noded)
    {
        LOG4CPLUS_ERROR(s_logger, "Failed to snap geometries.");
        return 1;
    }

    LOG4CPLUS_TRACE(s_logger, "Snapped geometries: " << noded);
    LOG4CPLUS_INFO(s_logger, "Building geometry graph...");
    const auto graph = generative::noding::GeometryGraph(*noded);

    LOG4CPLUS_INFO(s_logger, "Writing geometry graph...");

    std::unique_ptr<generative::io::GraphWriter> writer;
    switch (args.graph_format)
    {
    case CmdlineArgs::GraphFormat::TGF:
        writer = std::make_unique<generative::io::TGFGraphWriter>(args.output);
        break;
    }
    writer->write(graph);
    return 0;
}

static int _graph2geom(const CmdlineArgs& args)
{
    LOG4CPLUS_INFO(s_logger, "Reading graph...");
    auto factory = geos::geom::GeometryFactory::create();
    std::unique_ptr<generative::io::GraphReader> reader;
    switch (args.graph_format)
    {
    case CmdlineArgs::GraphFormat::TGF:
        reader = std::make_unique<generative::io::TGFGraphReader>(args.input, *factory);
    }
    const auto graph = reader->read();

    LOG4CPLUS_INFO(s_logger, "Read graph with " << graph.get_nodes().size() << " nodes");
    // Expensive, as it has to create a new LINESTRING geometry for every edge.
    const auto owned_edges = graph.get_edges();
    LOG4CPLUS_INFO(s_logger, "Read " << owned_edges.size() << " edges from graph");

    // The polygonizer takes raw pointers, so we have to get a non-owning view.
    std::vector<const geos::geom::Geometry*> edges;
    edges.reserve(owned_edges.size());
    std::transform(
        owned_edges.begin(),
        owned_edges.end(),
        std::back_inserter(edges),
        [](const std::unique_ptr<geos::geom::LineString>& edge) -> const geos::geom::Geometry* {
            return edge.get();
        });

    auto polygonizer = geos::operation::polygonize::Polygonizer();
    // Adding the edges doesn't actually polygonize. That's deferred to the first time the polygons
    // or dangles are accessed.
    polygonizer.add(&edges);

    LOG4CPLUS_INFO(s_logger, "Polygonizing graph...");
    const auto polys = polygonizer.getPolygons();
    const auto dangles = polygonizer.getDangles();
    LOG4CPLUS_INFO(s_logger,
                   "Got " << polys.size() << " polygons and " << dangles.size() << " dangles.");

    geos::io::WKTWriter writer;
    writer.setTrim(true);
    writer.setOutputDimension(3);

    for (const auto& poly : polys)
    {
        const std::string wkt = writer.write(poly.get());
        args.output << wkt << std::endl;
    }
    for (const auto& dangle : dangles)
    {
        const std::string wkt = writer.write(dangle);
        args.output << wkt << std::endl;
    }

    return 0;
}

int main(int argc, const char* argv[])
{
    log4cplus::Initializer initializer;
    // A basic ConsoleAppender that logs to stderr.
    auto appender = log4cplus::SharedAppenderPtr(new log4cplus::ConsoleAppender(true, true));
    s_logger.addAppender(appender);

    const CmdlineArgs args = CmdlineArgs::parse_args(argc, argv);

    if (args.graph2geom)
    {
        LOG4CPLUS_INFO(s_logger, "Converting graph to geometries...");
        return _graph2geom(args);
    }
    LOG4CPLUS_INFO(s_logger, "Converting geometries to a graph...");
    return _geom2graph(args);
}
