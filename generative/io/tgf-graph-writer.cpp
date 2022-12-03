#include "generative/io/tgf-graph-writer.h"

namespace generative::io {
void TGFGraphWriter::handle_node(const generative::noding::GeometryGraph::Node& node)
{
    out() << node.index << "\t" << wkt(node) << "\n";
}

void TGFGraphWriter::end_nodes()
{
    out() << "#\n";
}

void TGFGraphWriter::handle_edge(const generative::noding::GeometryGraph::Node& src,
                                 const generative::noding::GeometryGraph::Node& dst)
{
    out() << src.index << "\t" << dst.index << "\n";
}
}  // namespace generative::io
