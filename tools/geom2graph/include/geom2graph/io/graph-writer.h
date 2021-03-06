#pragma once
#include "geom2graph/noding/geometry-graph.h"

#include <geos/io/WKTWriter.h>

#include <ostream>

namespace geom2graph::io {
class GraphWriter
{
public:
    explicit GraphWriter(std::ostream& output) : m_out(output)
    {
        m_writer.setTrim(true);
        m_writer.setOutputDimension(3);
    }
    virtual ~GraphWriter() = default;

    //! @brief Write the given graph to the provided ostream.
    virtual void write(const geom2graph::noding::GeometryGraph& graph)
    {
        const auto& nodes = graph.get_nodes();
        this->start_nodes();
        for (const auto& node : nodes)
        {
            this->handle_node(node);
        }
        this->end_nodes();

        this->start_edges();
        const auto edges = graph.get_edge_pairs();
        for (const auto& edge : edges)
        {
            this->handle_edge(edge.first, edge.second);
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
        return m_writer.write(node.point.get());
    }

private:
    std::ostream& m_out;
    geos::io::WKTWriter m_writer;
};
}  // namespace geom2graph::io
