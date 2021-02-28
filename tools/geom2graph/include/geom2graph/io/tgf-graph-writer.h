#pragma once
#include "geom2graph/io/graph-writer.h"

namespace geom2graph::io {
class TGFGraphWriter : public GraphWriter
{
public:
    TGFGraphWriter(std::ostream& out, const geos::geom::GeometryFactory& factory) :
        GraphWriter(out, factory)
    {
    }

private:
    void handle_node(const geom2graph::noding::GeometryGraph::Node& node) override;
    void end_nodes() override;

    void handle_edge(const geom2graph::noding::GeometryGraph::Node& src,
                     const geom2graph::noding::GeometryGraph::Node& dst) override;
};
}  // namespace geom2graph::io
