#include "geom2graph/noding/geometry-graph.h"

#include "geom2graph/geometry-flattener.h"

#include <geos/geom/CoordinateSequence.h>
#include <geos/geom/Geometry.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <map>

static auto s_logger = log4cplus::Logger::getInstance("geom2graph.noding.geometrygraph");

namespace geom2graph::noding {

// TIL: anonymous namespaces are equivalent to marking these methods as static.
namespace {
    using Nodes_t = std::map<geos::geom::Coordinate, std::size_t, geos::geom::CoordinateLessThen>;
    using Graph_t = std::vector<GeometryGraph::Node>;

    GeometryGraph::Node&
    find_or_insert(Graph_t& graph, Nodes_t& coords, const Nodes_t::key_type& coord)
    {
        auto iter = coords.find(coord);

        // This isn't a coordinate we know about.
        if (iter == coords.end())
        {
            GeometryGraph::Node new_node(graph.size(), coord);
            LOG4CPLUS_TRACE(s_logger,
                            "Adding new coordinate to graph: " << coord.toString() << " with id "
                                                               << new_node.id);
            graph.push_back(new_node);

            auto result = coords.emplace(new_node.coord, new_node.id);
            if (result.second)
            {
                // This is a node we haven't seen before.
            }
            iter = result.first;
        }

        return graph.at(iter->second);
    }

    std::vector<GeometryGraph::Node> build_graph(const geos::geom::Geometry& geometry)
    {
        // Each node owns its own adjacency list.
        std::vector<GeometryGraph::Node> graph;
        graph.reserve(geometry.getNumPoints());

        // Need to look up a node's ID by it's coordinates, if it exists.
        std::map<geos::geom::Coordinate, std::size_t, geos::geom::CoordinateLessThen> nodes;

        for (const auto& geom : geom2graph::GeometryFlattener(geometry))
        {
            // LOG4CPLUS_TRACE(s_logger, "Adding " << geom.toString() << " to graph");

            // This pointer is the owner of the coordinates, so we have to copy from it into the
            // created Node
            const auto coords = geom.getCoordinates();
            for (std::size_t i = 0, j = 1; j < coords->getSize(); i = j++)
            {
                const auto& curr = coords->getAt(i);
                const auto& next = coords->getAt(j);

                LOG4CPLUS_TRACE(s_logger,
                                "Adding edge " << curr.toString() << " -> " << next.toString());

                // Add, or lookup the nodes in the graph.
                auto& curr_node = find_or_insert(graph, nodes, curr);
                auto& next_node = find_or_insert(graph, nodes, next);

                // Add each node to eachother's adjacency list.
                curr_node.adjacencies.emplace(next_node.id);
                next_node.adjacencies.emplace(curr_node.id);
            }
        }

        return graph;
    }
}  // namespace

GeometryGraph::GeometryGraph(const geos::geom::Geometry& multilinestring) :
    m_geometry(multilinestring)
{
    m_graph = build_graph(m_geometry);
}
}  // namespace geom2graph::noding
