#pragma once
#include <geos/geom/Coordinate.h>
#include <geos/geom/GeometryFactory.h>

#include <map>
#include <memory>
#include <unordered_set>
#include <vector>

namespace geos::geom {
class Geometry;
class LineString;
class Point;
}  // namespace geos::geom

namespace geom2graph::noding {
class GeometryGraph
{
public:
    struct Node
    {
        //! @brief The index into the GeometryGraph::get_nodes() array.
        const std::size_t index;
        //! @brief the point at which this node is located.
        std::unique_ptr<geos::geom::Point> point;
        //! @brief All of the Node is's adjacent to this node.
        std::unordered_set<std::size_t> adjacencies;

        Node(std::size_t _id, std::unique_ptr<geos::geom::Point> _point) :
            index(_id), point(std::move(_point))
        {
        }

        const geos::geom::Coordinate* coord() const
        {
            if (point)
            {
                return point->getCoordinate();
            }
            return nullptr;
        }
    };

    //! @brief Create an empty graph.
    //! @see GeometryGraph::build() to generate the graph from a geometry.
    //! @see GeometryGraph::set_nodes() and GeometryGraph::add_edge() to build a graph yourself.
    explicit GeometryGraph(const geos::geom::GeometryFactory& factory) : m_factory(factory) {}

    //! @brief Create and build the graph from the given geometry.
    //! @note The geometry must be fully noded.
    //! @param multilinestring - A fully noded collection of linestrings to build the graph from.
    explicit GeometryGraph(const geos::geom::Geometry& multilinestring);

    //! @brief Create a graph known a priori using the given factory.
    GeometryGraph(std::vector<Node>&& nodes, const geos::geom::GeometryFactory& factory);

    //! @brief Build the graph from the given geoemtry.
    //! @note The geometry must be fully noded.
    void build(const geos::geom::Geometry& geometry);

    //! @brief Get the constructed graph.
    [[nodiscard]] const std::vector<Node>& get_nodes() const { return m_nodes; }
    void set_nodes(std::vector<Node>&& nodes) { m_nodes = std::move(nodes); }

    //! @brief Add the given edge to the graph.
    //! @note The nodes at the indices @p src and @p dst must exist.
    void add_edge(std::size_t src, std::size_t dst);

    [[nodiscard]] std::vector<std::pair<const Node&, const Node&>> get_edge_pairs() const;

    //! @brief Get the graph edges.
    //! @note this is more expensive than get_nodes() because we have to create a new Linestring for
    //! every edge. We can't just return a reference to an existing data structure.
    [[nodiscard]] std::vector<std::unique_ptr<geos::geom::LineString>> get_edges() const;

private:
    struct Coordinate3DSafeLessThan
    {
        bool operator()(const geos::geom::Coordinate& lhs, const geos::geom::Coordinate& rhs) const
        {
            if (lhs.x < rhs.x)
            {
                return true;
            }
            if (lhs.x > rhs.x)
            {
                return false;
            }
            if (lhs.y < rhs.y)
            {
                return true;
            }
            if (lhs.y > rhs.y)
            {
                return false;
            }

            // For 2D coordinates, the z value is std::numeric_limits<double>::quiet_NaN()
            const auto l_z = std::isnan(lhs.z) ? 0 : lhs.z;
            const auto r_z = std::isnan(rhs.z) ? 0 : rhs.z;
            return l_z < r_z;
        }
    };
    using Nodes_t = std::map<geos::geom::Coordinate, std::size_t, Coordinate3DSafeLessThan>;

    // Used in get_edges and find_or_insert
    const geos::geom::GeometryFactory& m_factory;
    std::vector<Node> m_nodes;

    //! @brief Find the node for the given coordinate, or create one if one doesn't exist.
    //! @todo Find the right way to make this public. It's pretty limited to have to know all the
    //! nodes up front. Perhaps add_node(const Coordinate&) and add_node(std::unique_ptr<Point>)?
    //! Will require making a Nodes_t m_inserted_coords
    Node& find_or_insert(Nodes_t& inserted_coords, const geos::geom::Coordinate& coord);
};
}  // namespace geom2graph::noding
