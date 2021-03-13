#include "cmdline.h"
#include "geom2graph/geometry-flattener.h"
#include "geom2graph/io/tgf-graph-reader.h"
#include "geom2graph/io/tgf-graph-writer.h"
#include "geom2graph/io/wkt-stream-reader.h"
#include "geom2graph/noding/geometry-graph.h"
#include "geom2graph/noding/geometry-noder.h"

#include <geos/geom/GeometryCollection.h>
#include <geos/io/WKTWriter.h>
#include <geos/noding/snap/SnappingNoder.h>
#include <geos/operation/polygonize/Polygonizer.h>
#include <log4cplus/consoleappender.h>
#include <log4cplus/initializer.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <algorithm>
#include <string>

static auto s_logger = log4cplus::Logger::getRoot();

static int _geom2graph(const CmdlineArgs& args)
{
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

    std::unique_ptr<geom2graph::io::GraphWriter> writer;
    switch (args.graph_format)
    {
    case CmdlineArgs::GraphFormat::TGF:
        writer = std::make_unique<geom2graph::io::TGFGraphWriter>(args.output);
        break;
    }
    writer->write(graph);
    return 0;
}

static int _graph2geom(const CmdlineArgs& args)
{
    LOG4CPLUS_INFO(s_logger, "Reading graph...");
    auto factory = geos::geom::GeometryFactory::create();
    std::unique_ptr<geom2graph::io::GraphReader> reader;
    switch (args.graph_format)
    {
    case CmdlineArgs::GraphFormat::TGF:
        reader = std::make_unique<geom2graph::io::TGFGraphReader>(args.input, *factory);
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
