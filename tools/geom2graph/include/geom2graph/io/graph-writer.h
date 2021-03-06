#pragma once
#include "geom2graph/noding/geometry-graph.h"

#include <geos/geom/GeometryFactory.h>
#include <geos/io/WKTWriter.h>

#include <ostream>

namespace geom2graph::io {
class GraphWriter
{
public:
    //! @todo It feels weird that a graph writer needs a geometry factory.
    //! It's needed because you can't directly convert Coordinates to WKT.
    //! I think the solution is to store POINTs in the GeometryGraph::Node, which will be nice,
    //! because we already have to copy the Coordinate.
    //! @todo Investigate the size difference between a Coordinate and a Point.
    explicit GraphWriter(std::ostream& output, const geos::geom::GeometryFactory& factory) :
        m_out(output), m_factory(factory)
    {
        m_writer.setTrim(true);
        m_writer.setOutputDimension(3);
    }
    virtual ~GraphWriter() = default;

    //! @brief Write the given graph to the provided ostream.
    virtual void write(const geom2graph::noding::GeometryGraph& graph)
    {
        const auto& nodes = graph.get_graph();
        this->start_nodes();
        for (const auto& node : nodes)
        {
            this->handle_node(node);
        }
        this->end_nodes();

        this->start_edges();

        for (const auto& node : nodes)
        {
            for (const auto adj : node.adjacencies)
            {
                // Only print each edge once.
                if (node.id < adj)
                {
                    handle_edge(node, nodes[adj]);
                }
            }
        }
        this->end_edges();
    }

protected:
    virtual void start_nodes() {}
    virtual void handle_node(const geom2graph::noding::GeometryGraph::Node& node) = 0;
    virtual void end_nodes() {}

    virtual void start_edges() {}
    virtual void handle_edge(const geom2graph::noding::GeometryGraph::Node& src,
                             const geom2graph::noding::GeometryGraph::Node& dst) = 0;
    virtual void end_edges() {}

    [[nodiscard]] std::ostream& out() const { return m_out; }
    [[nodiscard]] std::string wkt(const geom2graph::noding::GeometryGraph::Node& node)
    {
        const auto point = std::unique_ptr<geos::geom::Point>(m_factory.createPoint(node.coord));
        return m_writer.write(point.get());
    }

private:
    std::ostream& m_out;
    const geos::geom::GeometryFactory& m_factory;
    geos::io::WKTWriter m_writer;
};
}  // namespace geom2graph::io
