#pragma once
#include "generative/generative/cxxbridge/coord_ffi.rs.h"
#include <generative/noding/geometry-graph.h>

#include <geos/geom/Coordinate.h>
#include <geos/geom/GeometryFactory.h>
#include <rust/cxx.h>

#include <vector>

class GeometryGraphShim
{
public:
    explicit GeometryGraphShim(generative::noding::GeometryGraph&& graph) :
        m_inner(std::move(graph))
    {
    }

    [[nodiscard]] const generative::noding::GeometryGraph& inner() const noexcept
    {
        return m_inner;
    }
    [[nodiscard]] generative::noding::GeometryGraph& inner() noexcept { return m_inner; }

    [[nodiscard]] rust::Vec<CoordShim> nodes() const noexcept
    {
        const auto& cxx_nodes = m_inner.get_nodes();
        rust::Vec<CoordShim> rust_nodes;
        rust_nodes.reserve(cxx_nodes.size());

        for (const auto& node : cxx_nodes)
        {
            auto rust_node = CoordShim{node.coord().x, node.coord().y};
            rust_nodes.emplace_back(rust_node);
        }

        return rust_nodes;
    }

    [[nodiscard]] rust::Vec<GraphEdge> edges() const noexcept
    {
        const auto& cxx_edges = m_inner.get_edge_pairs();
        rust::Vec<GraphEdge> rust_edges;
        rust_edges.reserve(cxx_edges.size());
        for (const auto& edge : cxx_edges)
        {
            auto rust_edge = GraphEdge{edge.first.index, edge.second.index};
            rust_edges.emplace_back(rust_edge);
        }

        return rust_edges;
    }

private:
    generative::noding::GeometryGraph m_inner;
};

[[nodiscard]] std::unique_ptr<GeometryGraphShim>
from_nodes_edges(rust::Slice<const CoordShim> nodes, rust::Slice<const GraphEdge> edges) noexcept
{
    auto factory = geos::geom::GeometryFactory::create();
    auto graph = generative::noding::GeometryGraph(*factory);

    for (const auto& node : nodes)
    {
        const auto coord = geos::geom::CoordinateXY{node.x, node.y};
        // Ignore the created node's index, because we're creating them in the order of the nodes
        // slice, which the edges slice requires in order to be valid.
        const auto index = graph.add_node(coord);
        (void)index;
    }

    for (const auto& edge : edges)
    {
        graph.add_edge(edge.src, edge.dst);
    }

    return std::make_unique<GeometryGraphShim>(std::move(graph));
}
