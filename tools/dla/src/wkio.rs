use crate::dla;
use std::io::{BufWriter, Write};

pub fn format_tgf(writer: &mut BufWriter<Box<dyn Write>>, graph: dla::GraphType) {
    // let (nodes, edges) = graph.into_nodes_edges();
    for idx in graph.node_indices() {
        let particle = graph
            .node_weight(idx)
            .expect("Got index to nonexistent node.");
        let label = idx.index();
        writeln!(
            writer,
            "{}\tPOINT({} {})",
            label, particle.coordinates[0], particle.coordinates[1]
        )
        .expect("Failed to write node label");
    }
    writeln!(writer, "#").expect("Failed to write node/edge separator");
    for edge_index in graph.edge_indices() {
        let edge = graph.edge_endpoints(edge_index);
        let (source, target) = edge.expect("Failed to get source or target from edge");
        writeln!(writer, "{}\t{}", source.index(), target.index()).expect("Failed to write edge");
    }
}

pub fn format_wkt(writer: &mut BufWriter<Box<dyn Write>>, graph: dla::GraphType) {
    for idx in graph.node_indices() {
        let particle = graph
            .node_weight(idx)
            .expect("Got index to nonexistent node.");
        writeln!(
            writer,
            "POINT ({} {})",
            particle.coordinates[0], particle.coordinates[1]
        )
        .expect("Failed to write node WKT.");
    }
}
