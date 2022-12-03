#pragma once
#include "generative/io/graph-reader.h"

#include <list>
#include <vector>

namespace generative::io {
class TGFGraphReader : public GraphReader
{
public:
    TGFGraphReader(std::istream& input, const geos::geom::GeometryFactory& factory) :
        GraphReader(input, factory)
    {
    }

    generative::noding::GeometryGraph read() noexcept override;
    void read(const std::string& line) noexcept override;

private:
    std::list<generative::noding::GeometryGraph::Node> m_nodes_list;
    std::vector<generative::noding::GeometryGraph::Node> m_nodes_vec;
    //! @brief Indicates whether the reader is reading the nodes or the edges.
    //! In the TGF format, there's a group of lines for the nodes, and a group of lines for the
    //! edges, separated by a line with a single '#'.
    bool m_reading_nodes = true;

    //! @brief Interpret the given line as a node.
    //! @details A node has the format
    //! @code
    //! <id>[<whitespace><label>]
    //! @endcode
    //! where the ID is a numeric index into the array of all nodes, and the label is a WKT POINT or
    //! POINT Z.
    //! @warning The label is optional for any given TGF graph, but for our purposes, if the label
    //! is missing, or cannot be parsed as a WKT POINT or POINT Z, the node will not be added.
    //! @todo If the node is skipped, we need to ignore that node when adding edges...
    void read_node(const std::string& line) noexcept;

    //! @brief Interpret the given line as an edge.
    //! @details An edge has the format
    //! @code
    //! <id><whitespace><id>[<whitespace><label>]
    //! @endcode
    //! For our purposes, the IDs are numeric indices into the array of nodes, and the label will be
    //! ignored.
    void read_edge(const std::string& line) noexcept;
};
}  // namespace generative::io
