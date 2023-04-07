#include "generative/noding/geometry-graph.h"

#include "generative/geometry-flattener.h"
#include "generative/io/wkt.h"

#include <geos/geom/CoordinateSequence.h>
#include <geos/geom/Geometry.h>
#include <geos/geom/LineString.h>
#include <log4cplus/logger.h>
#include <log4cplus/loggingmacros.h>

#include <array>
#include <map>

static auto s_logger = log4cplus::Logger::getInstance("generative.noding.geometrygraph");

using generative::io::operator<<;
namespace generative::noding {

GeometryGraph::GeometryGraph(const geos::geom::Geometry& multilinestring) :
    m_factory(*multilinestring.getFactory())
{
    build(multilinestring);
}

GeometryGraph::GeometryGraph(std::vector<GeometryGraph::Node>&& nodes,
                             const geos::geom::GeometryFactory& factory) :
    m_factory(factory), m_nodes(std::move(nodes))
{
}

void GeometryGraph::add_edge(std::size_t src, std::size_t dst)
{
    m_nodes[src].adjacencies.emplace(dst);
    m_nodes[dst].adjacencies.emplace(src);
}

std::vector<std::pair<const GeometryGraph::Node&, const GeometryGraph::Node&>>
GeometryGraph::get_edge_pairs() const
{
    std::vector<std::pair<const GeometryGraph::Node&, const GeometryGraph::Node&>> pairs;
    // A heuristic.
    pairs.reserve(m_nodes.size() * 2);
    for (const auto& node : m_nodes)
    {
        for (const auto adj : node.adjacencies)
        {
            // Only print each edge once.
            if (node.index < adj)
            {
                pairs.emplace_back(node, m_nodes[adj]);
            }
        }
    }
    return pairs;
}

std::vector<std::unique_ptr<geos::geom::LineString>> GeometryGraph::get_edges() const
{
    const auto pairs = this->get_edge_pairs();
    std::vector<std::unique_ptr<geos::geom::LineString>> edges;
    edges.reserve(pairs.size());

    for (const auto& pair : pairs)
    {
        auto coords = std::make_unique<geos::geom::CoordinateSequence>(0, false, false, false);
        coords->add(pair.first.coord());
        coords->add(pair.second.coord());
        auto edge = m_factory.createLineString(std::move(coords));
        edges.push_back(std::move(edge));
    }

    return edges;
}

GeometryGraph::Node&
GeometryGraph::find_or_insert(Nodes_t& inserted_coords, const geos::geom::Coordinate& coord)
{
    auto iter = inserted_coords.find(coord);

    // This isn't a coordinate we know about.
    if (iter == inserted_coords.end())
    {
        auto point = std::unique_ptr<geos::geom::Point>(m_factory.createPoint(coord));
        GeometryGraph::Node new_node(m_nodes.size(), std::move(point));
        LOG4CPLUS_TRACE(s_logger, "Adding new node " << new_node.index << "\t" << new_node.point);
        auto result = inserted_coords.emplace(coord, new_node.index);
        iter = result.first;
        m_nodes.push_back(std::move(new_node));
    }

    return m_nodes.at(iter->second);
}

void GeometryGraph::build(const geos::geom::Geometry& geometry)
{
    m_nodes.reserve(geometry.getNumPoints());

    // Need to look up a node's ID by it's coordinates, if it exists.
    GeometryGraph::Nodes_t inserted_coords;

    for (const auto& geom : generative::GeometryFlattener(geometry))
    {
        const auto coords = geom.getCoordinates();
        for (std::size_t i = 0, j = 1; j < coords->getSize(); i = j++)
        {
            const auto& curr = coords->getAt(i);
            const auto& next = coords->getAt(j);

            LOG4CPLUS_TRACE(s_logger, "new edge " << curr.toString() << " -> " << next.toString());

            // Add, or lookup the nodes in the graph.
            auto& curr_node = find_or_insert(inserted_coords, curr);
            auto& next_node = find_or_insert(inserted_coords, next);

            add_edge(curr_node.index, next_node.index);
        }
    }
}
}  // namespace generative::noding
