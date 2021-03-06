#include "geom2graph/io/tgf-graph-writer.h"

namespace geom2graph::io {
void TGFGraphWriter::handle_node(const geom2graph::noding::GeometryGraph::Node& node)
{
    out() << node.index << "\t" << wkt(node) << "\n";
}

void TGFGraphWriter::end_nodes()
{
    out() << "#\n";
}

void TGFGraphWriter::handle_edge(const geom2graph::noding::GeometryGraph::Node& src,
                                 const geom2graph::noding::GeometryGraph::Node& dst)
{
    out() << src.index << "\t" << dst.index << "\n";
}
}  // namespace geom2graph::io
