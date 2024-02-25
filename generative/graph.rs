use geo::Point;

pub type NodeData = Point;
pub type EdgeWeight = ();
pub type NodeIndex = usize;

pub type GeometryGraph<Direction = petgraph::Undirected> =
    petgraph::Graph<NodeData, EdgeWeight, Direction, NodeIndex>;

#[cfg(feature = "cxx-bindings")]
mod wrapper {
    use super::*;
    use crate::cxxbridge::{GeometryGraphShim, GraphEdge};

    impl<Direction: petgraph::EdgeType> From<&GeometryGraphShim> for GeometryGraph<Direction> {
        fn from(ffi_graph: &GeometryGraphShim) -> GeometryGraph<Direction> {
            let nodes = ffi_graph.nodes();
            // The edges are indices into the nodes array.
            let edges = ffi_graph.edges();

            let mut graph = GeometryGraph::default();
            graph.reserve_exact_nodes(nodes.len());
            graph.reserve_exact_edges(edges.len());

            for (_cxx_node_index, node) in nodes.iter().enumerate() {
                let point = Point::new(node.x, node.y);
                // We rely on the implementation detail of petgraph::Graph that when you insert nodes in
                // order, the node indices are generated in the same order.
                let _node_index = graph.add_node(point);
                debug_assert_eq!(_node_index.index(), _cxx_node_index);
            }
            for edge in &edges {
                let GraphEdge { src, dst } = edge;
                let src = petgraph::graph::NodeIndex::new(*src);
                let dst = petgraph::graph::NodeIndex::new(*dst);
                let _edge_index = graph.add_edge(src, dst, ());
            }

            graph
        }
    }
}
