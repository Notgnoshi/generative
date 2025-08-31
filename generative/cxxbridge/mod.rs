mod coord_ffi;
mod geometry_collection;
mod geometry_collection_ffi;
mod geometry_graph_ffi;
mod noder_ffi;

pub use coord_ffi::ffi::{CoordShim, GraphEdge, LineStringShim, PolygonShim, PolygonizationResult};
pub use geometry_collection::GeometryCollectionShim;
pub use geometry_graph_ffi::ffi::{GeometryGraphShim, from_nodes_edges};
pub use noder_ffi::ffi::{node, polygonize};

pub fn to_ffi_graph<Direction: petgraph::EdgeType>(
    graph: &crate::graph::GeometryGraph<Direction>,
) -> cxx::UniquePtr<GeometryGraphShim> {
    let nodes: Vec<_> = graph
        .node_weights()
        .map(|w| CoordShim { x: w.0.x, y: w.0.y })
        .collect();
    let mut edges = Vec::with_capacity(graph.edge_count());
    for i in 0..graph.edge_count() {
        let edge = graph
            .edge_endpoints(petgraph::graph::EdgeIndex::new(i))
            .unwrap();
        edges.push(GraphEdge {
            src: edge.0.index(),
            dst: edge.1.index(),
        });
    }

    from_nodes_edges(&nodes, &edges)
}
