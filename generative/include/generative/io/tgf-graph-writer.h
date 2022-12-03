#pragma once
#include "generative/io/graph-writer.h"

namespace generative::io {
class TGFGraphWriter : public GraphWriter
{
public:
    TGFGraphWriter(std::ostream& out) : GraphWriter(out) {}

private:
    void handle_node(const generative::noding::GeometryGraph::Node& node) override;
    void end_nodes() override;

    void handle_edge(const generative::noding::GeometryGraph::Node& src,
                     const generative::noding::GeometryGraph::Node& dst) override;
};
}  // namespace generative::io
