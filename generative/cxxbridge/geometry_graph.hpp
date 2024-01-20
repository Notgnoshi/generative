#pragma once
#include "generative/generative/cxxbridge/coord_ffi.rs.h"
#include <generative/noding/geometry-graph.h>

#include <rust/cxx.h>

#include <vector>

class GeometryGraphShim
{
public:
    explicit GeometryGraphShim(generative::noding::GeometryGraph&& graph) :
        m_inner(std::move(graph))
    {
    }

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
