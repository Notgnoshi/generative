#pragma once
#include <geos/geom/Coordinate.h>

#include <set>
#include <vector>

namespace geos::geom {
class Geometry;
}  // namespace geos::geom

namespace geom2graph::noding {
class GeometryGraph
{
public:
    struct Node
    {
        const std::size_t id;  //!< An index into the GeometryGraph::get_graph() array.
        const geos::geom::Coordinate coord;  //!< The coordinate for this node in the graph.
        std::set<std::size_t> adjacencies;   //!< All of the Node id's adjacent to this node.

        Node(std::size_t _id, const geos::geom::Coordinate& _coord) : id(_id), coord(_coord) {}
    };

    //! @brief Calculate the intersection graph for the given MULTILINESTRING.
    //! @param multilinestring - A fully noded collection of linestrings to build the graph from.
    GeometryGraph(const geos::geom::Geometry& multilinestring);

    //! @brief Get the constructed graph.
    [[nodiscard]] const std::vector<Node>& get_graph() const { return m_graph; }

private:
    const geos::geom::Geometry& m_geometry;
    std::vector<Node> m_graph;
};
}  // namespace geom2graph::noding
